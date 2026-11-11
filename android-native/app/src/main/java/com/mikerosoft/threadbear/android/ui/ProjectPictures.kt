package com.mikerosoft.threadbear.android.ui

import android.graphics.BitmapFactory
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import com.mikerosoft.threadbear.android.BuildConfig
import java.io.ByteArrayOutputStream
import java.net.HttpURLConnection
import java.net.URL
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * Project pictures (the iPhone's ProjectPictures): fetched once per URL and
 * kept for the app's life. A changed picture is a new file in storage, so a
 * new URL, and nothing here goes stale. Reading [image] from a composable
 * redraws it when the picture arrives, as the map is Compose state.
 */
object ProjectPictures {
    private val images = mutableStateMapOf<String, ImageBitmap>()
    private val loading = HashSet<String>()
    private val failed = HashSet<String>()
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Main)

    /** Capped like the iPhone's: the address comes from the mirror. */
    private const val LIMIT = 5 * 1024 * 1024
    /** An icon is never drawn bigger than a tile. */
    private const val MAX_SIDE = 512

    fun image(url: String): ImageBitmap? {
        images[url]?.let { return it }
        if (url in loading || url in failed) return null
        // Only https from the backend's storage (http for a local backend in debug).
        if (!url.startsWith("https://") && !(BuildConfig.ALLOW_LOCAL && url.startsWith("http://"))) {
            failed += url
            return null
        }
        loading += url
        scope.launch {
            val bitmap = withContext(Dispatchers.IO) { runCatching { fetch(url) }.getOrNull() }
            loading -= url
            if (bitmap == null) failed += url else images[url] = bitmap
        }
        return null
    }

    private fun fetch(url: String): ImageBitmap? {
        val conn = URL(url).openConnection() as HttpURLConnection
        conn.connectTimeout = 15_000
        conn.readTimeout = 15_000
        try {
            if (conn.responseCode != 200) return null
            if (conn.contentLengthLong > LIMIT) return null
            val out = ByteArrayOutputStream()
            conn.inputStream.use { input ->
                val buf = ByteArray(16 * 1024)
                while (true) {
                    val n = input.read(buf)
                    if (n < 0) break
                    out.write(buf, 0, n)
                    if (out.size() > LIMIT) return null
                }
            }
            val bytes = out.toByteArray()
            val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
            BitmapFactory.decodeByteArray(bytes, 0, bytes.size, bounds)
            if (bounds.outWidth <= 0 || bounds.outHeight <= 0) return null
            var sample = 1
            while (maxOf(bounds.outWidth, bounds.outHeight) / (sample * 2) >= MAX_SIDE) sample *= 2
            val opts = BitmapFactory.Options().apply { inSampleSize = sample }
            return BitmapFactory.decodeByteArray(bytes, 0, bytes.size, opts)?.asImageBitmap()
        } finally {
            conn.disconnect()
        }
    }
}
