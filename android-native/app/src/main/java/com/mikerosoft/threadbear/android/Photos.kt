package com.mikerosoft.threadbear.android

import android.content.Context
import android.content.res.AssetFileDescriptor
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.ImageDecoder
import android.net.Uri
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.text.input.InputTransformation
import androidx.compose.foundation.text.input.OutputTransformation
import androidx.compose.foundation.text.input.TextFieldBuffer
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextRange
import com.mikerosoft.threadbear.kit.BackendApi
import com.mikerosoft.threadbear.kit.PhotoMarks
import com.mikerosoft.threadbear.kit.PhotoPrep
import com.mikerosoft.threadbear.kit.PhotoUpload
import com.mikerosoft.threadbear.kit.TransportError
import java.io.ByteArrayOutputStream
import java.io.File
import java.util.UUID
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * Uploads outlive the screen that started them (a photo taken out after
 * its screen's gone still has its copy taken back), so they run here
 * rather than in a composable's scope.
 */
private val photoScope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)

/**
 * A photo waiting in a composer, already prepared off the main thread
 * (PhotoPicking.swift's PickedPhoto). `upload` is its copy in file storage,
 * which goes back if this is dropped before it's sent, and which trying a
 * failed send again reuses rather than leaving a second one in Convex.
 */
@Stable
class PickedPhoto(val jpeg: ByteArray, val thumbnail: ImageBitmap) {
    val id: String = UUID.randomUUID().toString()
    var upload: PhotoUpload? = null
        private set

    /** The upload's state and progress, as Compose state for the tile and Send. */
    var state by mutableStateOf(PhotoUpload.State.GOING)
        private set
    var progress by mutableFloatStateOf(0f)
        private set
    private var watching: Job? = null

    val isUp get() = state == PhotoUpload.State.UP

    fun attach(next: PhotoUpload?) {
        watching?.cancel()
        upload = next
        state = PhotoUpload.State.GOING
        progress = 0f
        if (next == null) return
        watching = photoScope.launch {
            launch { next.state.collect { state = it } }
            next.progress.collect { progress = it }
        }
    }
}

/** Whether this backend takes photos (backend API 18). Older ones refuse the field, so they're not offered. */
val AppModel.canSendPhotos: Boolean get() = apiVersion >= BackendApi.PHOTOS

/**
 * Starts a photo on its way to file storage, if it isn't already. Done as
 * soon as it's picked, so that Send usually has nothing to wait for.
 */
fun AppModel.startUpload(photo: PickedPhoto) {
    val b = backend.value ?: return
    if (photo.upload != null) return
    val jpeg = photo.jpeg
    photo.attach(PhotoUpload(photoScope, { progress -> b.uploadPhoto(jpeg, progress = progress) }, { runCatching { b.discardPhoto(it) } }))
}

/** A failed tile tapped: one more go. */
fun AppModel.retryUpload(photo: PickedPhoto) {
    photo.attach(null)
    startUpload(photo)
}

/** Waits for the photos to be up and returns the storage ids, in order. */
suspend fun AppModel.upload(photos: List<PickedPhoto>): List<String> {
    if (photos.isEmpty()) return emptyList()
    if (backend.value == null) throw TransportError.Server("Not connected.")
    // All of them at once, whatever's left to go.
    photos.forEach(::startUpload)
    return photos.map { photo ->
        val upload = photo.upload ?: throw TransportError.Server("Not connected.")
        try {
            upload.storageId()
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            // It may have failed back when it was picked, with the signal
            // since come back, so a send gets one fresh try of its own.
            retryUpload(photo)
            photo.upload?.storageId() ?: throw e
        }
    }
}

/** They've gone with a message, so they're the backend's to delete now. */
fun sent(photos: List<PickedPhoto>) = photos.forEach { it.upload?.keep() }

/**
 * Reading and preparing photos with Android's ImageDecoder (PhotoPrep.swift's
 * pixel half): a JPEG no longer than PhotoPrep.MAX_PIXELS on its long edge,
 * the right way up (ImageDecoder applies the EXIF orientation), drawn afresh
 * so no metadata (a location) goes with it.
 */
object PhotoDecoding {
    class Prepared(val jpeg: ByteArray, val thumbnail: ImageBitmap)

    private class TooBig : RuntimeException("Too big a picture")

    /** Two at a time across the app, off the main thread. Null when it isn't a picture this can read, or it's too big. */
    suspend fun prepare(source: () -> ImageDecoder.Source, bytes: Long? = null): Prepared? = PhotoPrep.inTurn {
        withContext(Dispatchers.Default) {
            try {
                prepared(source(), bytes)
            } catch (e: CancellationException) {
                throw e
            } catch (_: Throwable) {
                // Not a picture, too big, or out of memory on a huge one: refused, not a crash.
                null
            }
        }
    }

    fun fromUri(context: Context, uri: Uri): suspend () -> Prepared? = {
        val resolver = context.contentResolver
        // Its file's size, before reading it, where the provider says.
        val bytes = withContext(Dispatchers.IO) {
            runCatching { resolver.openAssetFileDescriptor(uri, "r")?.use { it.length } }.getOrNull()
                ?.takeIf { it != AssetFileDescriptor.UNKNOWN_LENGTH && it >= 0 }
        }
        prepare({ ImageDecoder.createSource(resolver, uri) }, bytes)
    }

    fun fromFile(file: File): suspend () -> Prepared? = {
        try {
            prepare({ ImageDecoder.createSource(file) }, file.length())
        } finally {
            file.delete()
        }
    }

    private fun prepared(source: ImageDecoder.Source, bytes: Long?): Prepared {
        if (bytes != null && bytes > PhotoPrep.MAX_SOURCE_BYTES) throw TooBig()
        val decoded = ImageDecoder.decodeBitmap(source) { decoder, info, _ ->
            // Refused from its header, before any of it's decoded: a small
            // file can unpack to a huge picture (Daybreak, 7 Nov 2026).
            if (!PhotoPrep.fits(info.size.width, info.size.height)) throw TooBig()
            val (w, h) = PhotoPrep.fitted(info.size.width, info.size.height)
            decoder.setTargetSize(w, h)
            // Drawn onto a canvas below, which a hardware bitmap can't be.
            decoder.allocator = ImageDecoder.ALLOCATOR_SOFTWARE
        }
        // On white, as JPEG has no transparency and a clear PNG would go black.
        val flat = Bitmap.createBitmap(decoded.width, decoded.height, Bitmap.Config.ARGB_8888)
        Canvas(flat).apply {
            drawColor(android.graphics.Color.WHITE)
            drawBitmap(decoded, 0f, 0f, null)
        }
        decoded.recycle()
        val out = ByteArrayOutputStream()
        flat.compress(Bitmap.CompressFormat.JPEG, PhotoPrep.JPEG_QUALITY, out)
        val (tw, th) = PhotoPrep.fitted(flat.width, flat.height, PhotoPrep.THUMBNAIL_PIXELS)
        val thumbnail = Bitmap.createScaledBitmap(flat, tw, th, true)
        if (thumbnail !== flat) flat.recycle()
        return Prepared(out.toByteArray(), thumbnail.asImageBitmap())
    }
}

/**
 * A composer's words and photos together (PhotoPicking.swift's PhotoWords):
 * each photo a chip in the words, a mark there (PhotoMarks), and the photos
 * in the marks' order. The text field draws each mark as its chip
 * ([chips]) and a chip deleted takes its photo with it ([keepMarks]).
 */
@Stable
class PhotoBox(val text: TextFieldState) {
    val photos = mutableStateListOf<PickedPhoto>()

    /** Photos being read and prepared, which the strip shows as spinners. */
    var adding by mutableIntStateOf(0)

    /** How many more it can take, counting the ones on their way. */
    val room: Int get() = maxOf(0, PhotoPrep.MAX_PHOTOS - photos.size - adding)

    private val words: String get() = PhotoMarks.normalized(text.text.toString(), photos.size)

    /** What's sent: the words with each chip as a [Picture N] marker, or none when there are only photos. */
    val message: String get() = PhotoMarks.sentText(words).trim()

    val isEmpty: Boolean get() = message.isEmpty() && photos.isEmpty() && adding == 0

    /** Every photo's in and up: Send and Start wait for this, so none is left behind. */
    val photosUp: Boolean get() = adding == 0 && photos.all { it.isUp }

    /**
     * Reads and prepares each photo off the main thread, shows it as soon as
     * it's ready and starts uploading it. All at once, so one slow to read
     * doesn't hold up the rest, but they go in in the order given.
     */
    fun add(loads: List<suspend () -> PhotoDecoding.Prepared?>, model: AppModel, scope: CoroutineScope) {
        val take = loads.take(room)
        if (take.isEmpty()) return
        adding += take.size
        val reading = take.map { load -> scope.async { load() } }
        scope.launch {
            for (task in reading) {
                val prepared = try { task.await() } catch (e: CancellationException) { throw e } catch (_: Exception) { null }
                adding -= 1
                if (prepared != null) {
                    val photo = PickedPhoto(prepared.jpeg, prepared.thumbnail)
                    model.startUpload(photo)
                    insert(listOf(photo))
                } else {
                    model.note("Couldn't read that photo.")
                }
            }
        }
    }

    /** New photos as chips where the cursor is (the end, before it's been in the box), as many as still fit. */
    fun insert(new: List<PickedPhoto>) {
        val have = photos.size
        val take = new.take(maxOf(0, PhotoPrep.MAX_PHOTOS - have))
        if (take.isEmpty()) return
        val words = words
        val selection = text.selection
        val result = PhotoMarks.inserting(take.size, words, selection.min, selection.max)
        photos.addAll(minOf(result.firstIndex, photos.size), take)
        // A programmatic edit: it doesn't go through keepMarks.
        text.edit {
            replace(0, length, result.text)
            this.selection = TextRange(result.cursor)
        }
    }

    /** The strip's remove button: the photo and its chip. */
    fun remove(photo: PickedPhoto) {
        val index = photos.indexOfFirst { it.id == photo.id }
        if (index < 0) return
        val words = words
        photos.removeAt(index)
        photo.upload?.drop()
        val out = PhotoMarks.removingMark(index, words)
        text.edit { replace(0, length, out) }
    }

    /** Emptied after a send: the photos have gone with it. */
    fun clear() {
        photos.clear()
        text.clearText()
    }

    /** Back as they were, after a send that didn't go. */
    fun restore(typed: String, back: List<PickedPhoto>) {
        photos.clear()
        photos.addAll(back)
        text.edit {
            replace(0, length, typed)
            selection = TextRange(length)
        }
    }

    /** The screen's gone: anything not sent is taken back. */
    fun dropAll() {
        photos.forEach { it.upload?.drop() }
    }

    /**
     * Mike's own edits (programmatic ones skip this): a chip deleted or cut
     * takes its photo out too, and a mark typed or pasted as text is no photo.
     */
    @OptIn(ExperimentalFoundationApi::class)
    val keepMarks = InputTransformation {
        val changes = changes
        if (changes.changeCount == 0) return@InputTransformation
        val replaced = (0 until changes.changeCount).map { i -> changes.getOriginalRange(i).let { it.min until it.max } }
        val removed = PhotoMarks.removedMarks(originalText, replaced)
        val inserted = (0 until changes.changeCount).map { changes.getRange(it) }.sortedByDescending { it.min }
        for (range in inserted) {
            for (j in range.max - 1 downTo range.min) if (j < length && charAt(j) == PhotoMarks.MARK) replace(j, j + 1, "")
        }
        if (removed.isNotEmpty()) {
            for (i in removed.sortedDescending()) if (i < photos.size) photos.removeAt(i).upload?.drop()
        }
    }

    /** Each mark drawn as "Picture N" in a chip's colours; a spare mark (no photo) isn't drawn. */
    @OptIn(ExperimentalFoundationApi::class)
    fun chips(style: SpanStyle) = OutputTransformation { drawChips(this, style) }

    @OptIn(ExperimentalFoundationApi::class)
    private fun drawChips(buffer: TextFieldBuffer, style: SpanStyle) = with(buffer) {
        val count = photos.size
        var i = 0
        var n = 0
        while (i < length) {
            if (charAt(i) != PhotoMarks.MARK) { i++; continue }
            n++
            if (n > count) { replace(i, i + 1, ""); continue }
            // Thin spaces either side stand in for the chip's padding.
            val label = " Picture $n "
            replace(i, i + 1, label)
            addStyle(style, i, i + label.length)
            i += label.length
        }
    }
}
