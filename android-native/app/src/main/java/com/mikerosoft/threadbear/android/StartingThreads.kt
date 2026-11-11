package com.mikerosoft.threadbear.android

import com.mikerosoft.threadbear.kit.CommandState
import com.mikerosoft.threadbear.kit.CommandStatus
import com.mikerosoft.threadbear.kit.TransportError
import java.time.Instant
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.retryWhen
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/** What the new thread screen had in it when Start was tapped. */
class NewThreadDraft(
    val text: String,
    val photos: List<PickedPhoto>,
    /** Null when the backend picks (only one runner). */
    val runnerId: String? = null,
    val modelId: String? = null,
    val effort: String? = null,
)

/** A new thread on its way: what was asked, until the runner has made it, or why it couldn't. */
data class StartingThread(
    /** The create command's id. */
    val id: String,
    /** What went to the runner, photos as "[Picture 1]" markers. */
    val message: String,
    /** The screen as it was, for trying again if it fails. */
    val draft: NewThreadDraft,
    val startedAt: Instant,
    val failed: String? = null,
)

/**
 * New threads asked for from this phone that their runner hasn't made yet,
 * newest first (the iPhone's AppModel.starting). The new thread screen
 * closes as soon as the backend has the request, and these stand in for
 * them at the top of the inbox: the runner reports a new thread only once
 * its row is in Convex, which waits behind any sync it's in the middle of,
 * and on 7 Nov the iPhone's spinner sat there 22 seconds after the M5's
 * runner had restarted.
 */
class StartingThreads(private val model: AppModel, private val scope: CoroutineScope) {
    private val _rows = MutableStateFlow<List<StartingThread>>(emptyList())
    val rows: StateFlow<List<StartingThread>> = _rows

    /** A failed start's screen, to open again as it was ([retry]). */
    val retryDraft = MutableStateFlow<NewThreadDraft?>(null)

    private val watchers = HashMap<String, Job>()
    private val openers = HashMap<String, (String) -> Unit>()
    /** Only the newest start opens by itself: an earlier one landing while you wait on the next mustn't take its place. */
    private var latest: String? = null

    init {
        // Paired with another Mac, or unpaired: these were the old one's.
        scope.launch { model.backend.drop(1).collect { clear() } }
    }

    /**
     * `open` gets the thread's id if it starts within a minute, as the
     * iPhone's sheet used to wait to do; one that took longer (its Mac was
     * asleep) just turns up in the list, rather than pulling Mike out of
     * what he's on.
     */
    fun started(commandId: String, message: String, draft: NewThreadDraft, open: (String) -> Unit) {
        _rows.update { listOf(StartingThread(commandId, message, draft, Instant.now())) + it }
        latest = commandId
        openers[commandId] = open
        watch(commandId)
    }

    /** Gone from the list: a failed one dismissed, or tapped to try again. */
    fun drop(id: String) {
        watchers.remove(id)?.cancel()
        openers.remove(id)
        _rows.update { list -> list.filterNot { it.id == id } }
    }

    /**
     * A failed one back in the new thread screen as it was: words, photos,
     * runner, model and effort. The photos go up again, as the backend
     * deleted its copies with the failed command.
     */
    fun retry(row: StartingThread) {
        drop(row.id)
        retryDraft.value = row.draft
    }

    fun takeRetry(): NewThreadDraft? = retryDraft.value.also { retryDraft.value = null }

    private fun clear() {
        watchers.values.forEach { it.cancel() }
        watchers.clear()
        openers.clear()
        _rows.value = emptyList()
        retryDraft.value = null
    }

    private fun watch(id: String) {
        val b = model.backend.value ?: return
        watchers.remove(id)?.cancel()
        watchers[id] = scope.launch {
            b.command(id)
                .retryWhen { cause, _ -> cause !is TransportError.Unauthorized && run { delay(2000); true } }
                .collect { changed(id, it) }
            // The flow ends once the command's gone from the backend: as changed(null).
            changed(id, null)
        }
    }

    private fun changed(id: String, status: CommandStatus?) {
        val row = _rows.value.firstOrNull { it.id == id } ?: return
        val threadId = status?.threadId
        when {
            threadId != null -> {
                val open = openers[id]
                drop(id)
                val soon = Instant.now().toEpochMilli() - row.startedAt.toEpochMilli() < 60_000
                // The nav it navigates with can be gone (the activity recreated): nothing to do then.
                if (id == latest && soon) open?.let { runCatching { it(threadId) } }
            }
            status?.state == CommandState.FAILED -> {
                _rows.update { list -> list.map { if (it.id == id) it.copy(failed = status.note ?: "The runner couldn't start it.") else it } }
                row.draft.photos.forEach { it.attach(null) }
                watchers.remove(id)?.cancel()
            }
            // The command's gone from the backend (pruned long after), or
            // done without saying which thread: nothing left to wait for,
            // and the thread, if any, is in the list.
            status == null || status.state == CommandState.DONE -> drop(id)
        }
    }
}
