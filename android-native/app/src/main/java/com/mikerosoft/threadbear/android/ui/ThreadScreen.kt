package com.mikerosoft.threadbear.android.ui

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import com.mikerosoft.threadbear.android.LocalModel
import com.mikerosoft.threadbear.android.LocalSend
import com.mikerosoft.threadbear.android.LocalSnackbar
import com.mikerosoft.threadbear.android.Loaded
import com.mikerosoft.threadbear.android.ThreadViewModel
import com.mikerosoft.threadbear.kit.BackendApi
import com.mikerosoft.threadbear.kit.Hibernate
import com.mikerosoft.threadbear.kit.PendingApproval
import com.mikerosoft.threadbear.kit.PhoneMessage
import com.mikerosoft.threadbear.kit.PhoneThread
import com.mikerosoft.threadbear.kit.Role
import com.mikerosoft.threadbear.kit.RowStatus
import com.mikerosoft.threadbear.kit.ThreadChange
import com.mikerosoft.threadbear.kit.ThreadItem
import com.mikerosoft.threadbear.kit.ThreadStatus
import com.mikerosoft.threadbear.kit.Turns
import com.mikerosoft.threadbear.kit.WaitingSend
import com.mikerosoft.threadbear.kit.WaitingSends
import java.time.Instant

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ThreadScreen(threadId: String, back: () -> Unit, open: (String) -> Unit) {
    val model = LocalModel.current
    val vm: ThreadViewModel = viewModel(key = threadId) { ThreadViewModel(model, threadId) }
    val loaded by vm.loaded.collectAsStateWithLifecycle()
    // The reply's words and photos (Composer.kt).
    val box = rememberPhotoBox()
    val thread = (loaded as? Loaded.Here)?.detail?.thread
    val stopping by model.stopping.collectAsStateWithLifecycle()

    LaunchedEffect(thread?.isBusy) { if (thread?.isBusy == false) model.stopped(threadId) }

    Scaffold(
        snackbarHost = { SnackbarHost(LocalSnackbar.current) },
        topBar = { ThreadBar(thread, back) },
        bottomBar = {
            if (thread != null) {
                val offline = model.offlineRunner(thread)
                Composer(
                    box = box,
                    placeholder = when {
                        offline != null -> "$offline is offline, this will wait"
                        thread.isBusy -> "Steer the agent"
                        else -> "Reply"
                    },
                    running = thread.isBusy,
                    canStop = model.apiVersion == 0 || model.apiVersion >= BackendApi.STOP,
                    stopping = threadId in stopping,
                    onSend = { message, photos ->
                        // Cleared at once, as the message shows below as it goes;
                        // a send that fails puts it all back if nothing's been typed since.
                        val typed = box.text.text.toString()
                        box.clear()
                        vm.send(message, photos = photos) { if (box.isEmpty) box.restore(typed, photos) else photos.forEach { it.upload?.drop() } }
                    },
                    onStop = { model.stop(threadId) },
                )
            }
        },
    ) { inner ->
        when (val l = loaded) {
            Loaded.Loading -> Box(Modifier.padding(inner).fillMaxWidth()) {
                LinearProgressIndicator(Modifier.fillMaxWidth().height(1.dp), color = MaterialTheme.colorScheme.outline)
            }
            Loaded.Gone -> Box(Modifier.padding(inner).fillMaxSize(), contentAlignment = Alignment.Center) {
                Text("This thread's gone", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            is Loaded.Here -> Transcript(vm, l.detail.thread, l.detail.messages, inner, back) { box.text.setTextAndPlaceCursorAtEnd(it) }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ThreadBar(thread: PhoneThread?, back: () -> Unit) {
    val model = LocalModel.current
    val context = LocalContext.current
    var menu by remember { mutableStateOf(false) }
    var hibernate by remember { mutableStateOf(false) }
    val projects by model.projects.collectAsStateWithLifecycle()
    TopAppBar(
        navigationIcon = { IconButton(onClick = back) { Icon(Line.Back, "Back", Modifier.size(22.dp)) } },
        title = {
            if (thread != null) Row(verticalAlignment = Alignment.CenterVertically) {
                val ref = thread.projects?.firstOrNull()
                ProjectTile(ref?.let { r -> projects.firstOrNull { it.id == r.id } }, ref?.title ?: thread.folderName, 26.dp)
                Spacer(Modifier.width(10.dp))
                Text(thread.title.ifEmpty { "Untitled" }, style = MaterialTheme.typography.titleMedium, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        },
        actions = {
            if (thread != null) Box {
                IconButton(onClick = { menu = true }) { Icon(Line.More, "More", Modifier.size(20.dp)) }
                DropdownMenu(expanded = menu, onDismissRequest = { menu = false }) {
                    if (thread.archivedAt != null) {
                        MenuItem("Back to the inbox", Line.Inbox) { menu = false; model.change(thread.threadId, ThreadChange.Unarchive) }
                    } else {
                        val asleep = thread.snoozedUntil != null || thread.snoozedUntilRestart == true
                        if (asleep) {
                            MenuItem("Wake", Line.Sun) { menu = false; model.change(thread.threadId, ThreadChange.Wake) }
                        } else {
                            MenuItem("Hibernate", Line.Moon) { menu = false; hibernate = true }
                        }
                        val starred = thread.starredAt != null
                        if (model.apiVersion == 0 || model.apiVersion >= BackendApi.STARS) {
                            MenuItem(if (starred) "Remove star" else "Star", if (starred) Line.StarFilled else Line.Star) {
                                menu = false; model.change(thread.threadId, ThreadChange.Star(!starred))
                            }
                        }
                        MenuItem("Archive", Line.Archive) { menu = false; model.change(thread.threadId, ThreadChange.Archive); back() }
                    }
                    HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                    MenuItem("Copy thread ID", Line.Copy) {
                        menu = false
                        val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                        clipboard.setPrimaryClip(ClipData.newPlainText("Thread ID", thread.threadId))
                    }
                    DropdownMenuItem(
                        text = {
                            Text(
                                listOfNotNull(thread.cwd, listOfNotNull(thread.accountId, thread.model).joinToString(" · "),
                                    thread.costUsd?.takeIf { it > 0 }?.let { "$%.2f so far".format(it) }).joinToString("\n"),
                                style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        },
                        onClick = {}, enabled = false,
                    )
                }
                DropdownMenu(expanded = hibernate, onDismissRequest = { hibernate = false }) {
                    for (p in Hibernate.presets(Instant.now())) {
                        MenuItem(p.title, Line.Moon) { hibernate = false; model.change(thread.threadId, ThreadChange.Snooze(p.date)); back() }
                    }
                    MenuItem("Until the runner restarts", Line.Moon) {
                        hibernate = false; model.change(thread.threadId, ThreadChange.SnoozeUntilRestart); back()
                    }
                }
            }
        },
        colors = TopAppBarDefaults.topAppBarColors(containerColor = MaterialTheme.colorScheme.background),
    )
}

@Composable
private fun MenuItem(text: String, icon: androidx.compose.ui.graphics.vector.ImageVector, onClick: () -> Unit) {
    DropdownMenuItem(text = { Text(text) }, leadingIcon = { Icon(icon, null, Modifier.size(18.dp)) }, onClick = onClick)
}

/** One row of the transcript, bottom first (the list is drawn upside down so it sits on the newest). */
private sealed class Row_ {
    abstract val key: String
    data class Item(val item: ThreadItem) : Row_() { override val key get() = "i:${item.id}" }
    data class Waiting(val send: WaitingSend, val says: Boolean) : Row_() { override val key get() = "w:${send.commandId}" }
    data class Local(val send: LocalSend) : Row_() { override val key get() = "l:${send.key}" }
    data object Card : Row_() { override val key get() = "card" }
    data object Earlier : Row_() { override val key get() = "earlier" }
}

@Composable
private fun Transcript(vm: ThreadViewModel, thread: PhoneThread, threadMessages: List<PhoneMessage>, inner: PaddingValues, leave: () -> Unit, giveBack: (String) -> Unit) {
    val model = LocalModel.current
    val page by vm.page.collectAsStateWithLifecycle()
    val older by vm.older.collectAsStateWithLifecycle()
    val olderHasMore by vm.olderHasMore.collectAsStateWithLifecycle()
    val loadingOlder by vm.loadingOlder.collectAsStateWithLifecycle()
    val waiting by vm.waiting.collectAsStateWithLifecycle()
    val locals by vm.locals.collectAsStateWithLifecycle()
    val cancelling by vm.cancelling.collectAsStateWithLifecycle()
    val runners by model.runners.collectAsStateWithLifecycle()
    val now = rememberNow(1000)

    // From API 2 the live page (tool lines too), with older chunks before it.
    val live = page?.messages
    val messages = remember(live, older, threadMessages) {
        if (live == null) threadMessages
        else {
            val first = live.firstOrNull()?.seq ?: Int.MAX_VALUE
            older.filter { it.seq < first } + live
        }
    }
    val items = remember(messages, thread.isBusy) { Turns.fold(messages, thread.isBusy) }
    val offline = remember(runners, thread.runnerId, now) { WaitingSends.offlineRunner(thread.runnerId, runners, now) }
    val shown = remember(waiting, messages) { WaitingSends.shown(waiting, messages) }
    val listed = shown.map { it.commandId }.toSet()
    val pendingLocals = locals.filter { it.commandId == null || it.commandId !in listed }
    val canLoadMore = live != null && (if (older.isEmpty()) page?.hasMore == true else olderHasMore)

    val rows = remember(items, shown, pendingLocals, canLoadMore) {
        buildList {
            add(Row_.Card)
            pendingLocals.reversed().forEach { add(Row_.Local(it)) }
            shown.reversed().forEachIndexed { i, s -> add(Row_.Waiting(s, says = i == 0)) }
            items.reversed().forEach { add(Row_.Item(it)) }
            if (canLoadMore) add(Row_.Earlier)
        }
    }
    val state = rememberLazyListState()
    val lastSeq = messages.lastOrNull()?.seq
    LaunchedEffect(lastSeq, pendingLocals.size) {
        if (state.firstVisibleItemIndex <= 2) state.animateScrollToItem(0)
    }

    LazyColumn(
        state = state,
        reverseLayout = true,
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = inner.calculateTopPadding() + 8.dp, bottom = inner.calculateBottomPadding() + 12.dp),
        verticalArrangement = Arrangement.spacedBy(14.dp, Alignment.Bottom),
    ) {
        items(rows, key = { it.key }) { row ->
            when (row) {
                Row_.Card -> StatusCard(vm, thread, messages, pendingLocals.isEmpty() && shown.isEmpty(), now, leave)
                is Row_.Local -> PendingBubble(row.send.text, "Sending…", cancel = null, cancelling = false)
                is Row_.Waiting -> PendingBubble(
                    WaitingSends.mirrored(row.send),
                    if (row.says) WaitingSends.note(row.send, offline) else null,
                    cancel = if (row.send.canCancel) ({ vm.cancel(row.send, giveBack) }) else null,
                    cancelling = row.send.commandId in cancelling,
                )
                is Row_.Item -> ItemView(row.item)
                Row_.Earlier -> Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.Center) {
                    if (loadingOlder) CircularProgressIndicator(Modifier.size(18.dp), strokeWidth = 1.5.dp)
                    else TextButton(onClick = { messages.firstOrNull()?.seq?.let(vm::loadOlder) }) {
                        Text("Earlier messages", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }
    }
}

@Composable
private fun ItemView(item: ThreadItem, nested: Boolean = false) {
    when (item) {
        is ThreadItem.Message -> MessageView(item.message, nested)
        is ThreadItem.Tools -> ToolRun(item.items)
        is ThreadItem.Worked -> Worked(item)
        is ThreadItem.Gap -> GapLine(item.gap.label)
    }
}

@Composable
private fun MessageView(m: PhoneMessage, nested: Boolean) {
    val colors = MaterialTheme.colorScheme
    when (m.role) {
        Role.USER -> {
            val mine = m.origin == null || m.origin == "phone"
            if (mine) {
                Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.CenterEnd) {
                    Text(
                        m.text,
                        style = MaterialTheme.typography.bodyLarge,
                        modifier = Modifier.widthIn(max = 320.dp).clip(RoundedCornerShape(20.dp, 20.dp, 6.dp, 20.dp))
                            .background(colors.surfaceContainerHigh).padding(horizontal = 14.dp, vertical = 10.dp),
                    )
                }
            } else {
                // A schedule's, the runner's or another agent's message: told apart from Mike's.
                val icon = when (m.origin) { "schedule" -> Line.Clock; "runner" -> Line.Bell; else -> Line.Paw }
                Row(
                    Modifier.fillMaxWidth().clip(RoundedCornerShape(14.dp)).border(1.dp, colors.outlineVariant, RoundedCornerShape(14.dp))
                        .padding(horizontal = 12.dp, vertical = 10.dp),
                ) {
                    Icon(icon, null, Modifier.size(16.dp).padding(top = 2.dp), tint = colors.onSurfaceVariant)
                    Spacer(Modifier.width(8.dp))
                    Text(m.text, style = MaterialTheme.typography.bodyMedium, color = colors.onSurfaceVariant, maxLines = 8, overflow = TextOverflow.Ellipsis)
                }
            }
        }
        Role.ASSISTANT -> MarkdownText(
            m.text,
            Modifier.fillMaxWidth(),
            style = if (nested) MaterialTheme.typography.bodyMedium.copy(color = colors.onSurfaceVariant) else MaterialTheme.typography.bodyLarge,
        )
        Role.TOOL -> ToolRun(listOf(m))
    }
}

@Composable
private fun ToolRun(lines: List<PhoneMessage>) {
    var open by remember { mutableStateOf(false) }
    val faint = MaterialTheme.colorScheme.onSurfaceVariant
    Column(Modifier.fillMaxWidth().animateContentSize().clickable(enabled = lines.size > 1) { open = !open }) {
        val shown = if (open) lines else listOf(lines.last())
        for (m in shown) Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(vertical = 2.dp)) {
            Icon(Line.tool(m.toolKind), null, Modifier.size(14.dp), tint = faint)
            Spacer(Modifier.width(8.dp))
            Text(m.text, style = MaterialTheme.typography.bodySmall.copy(fontFamily = Mono), color = faint, maxLines = if (open) 3 else 1,
                overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f, fill = false))
            if (!open && lines.size > 1) {
                Spacer(Modifier.width(6.dp))
                Text("+${lines.size - 1}", style = MaterialTheme.typography.labelSmall, color = LocalSignals.current.faint)
            }
        }
    }
}

@Composable
private fun Worked(item: ThreadItem.Worked) {
    var open by remember { mutableStateOf(false) }
    val turn by animateFloatAsState(if (open) 90f else 0f, label = "chevron")
    val faint = MaterialTheme.colorScheme.onSurfaceVariant
    Column(Modifier.fillMaxWidth()) {
        Row(
            Modifier.clip(RoundedCornerShape(8.dp)).clickable { open = !open }.padding(vertical = 4.dp, horizontal = 2.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(item.worked.title, style = MaterialTheme.typography.labelLarge, color = faint)
            Spacer(Modifier.width(4.dp))
            Icon(Line.ChevronRight, if (open) "Hide" else "Show", Modifier.size(14.dp).rotate(turn), tint = faint)
        }
        AnimatedVisibility(open, enter = expandVertically() + fadeIn(), exit = shrinkVertically() + fadeOut()) {
            Row(Modifier.height(IntrinsicSize.Min).padding(top = 6.dp)) {
                Box(Modifier.width(1.dp).fillMaxHeight().background(MaterialTheme.colorScheme.outlineVariant))
                Spacer(Modifier.width(12.dp))
                Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    item.worked.items.forEach { ItemView(it, nested = true) }
                }
            }
        }
    }
}

@Composable
private fun GapLine(label: String) {
    Row(Modifier.fillMaxWidth().padding(vertical = 4.dp), verticalAlignment = Alignment.CenterVertically) {
        HorizontalDivider(Modifier.weight(1f), color = MaterialTheme.colorScheme.outlineVariant)
        Text(label, style = MaterialTheme.typography.labelSmall, color = LocalSignals.current.faint, modifier = Modifier.padding(horizontal = 10.dp))
        HorizontalDivider(Modifier.weight(1f), color = MaterialTheme.colorScheme.outlineVariant)
    }
}

/** A message on its way: dimmed, with why it's waiting and a Cancel while it can be. */
@Composable
private fun PendingBubble(text: String, note: String?, cancel: (() -> Unit)?, cancelling: Boolean) {
    Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.End) {
        Text(
            text,
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.6f),
            modifier = Modifier.widthIn(max = 320.dp).clip(RoundedCornerShape(20.dp, 20.dp, 6.dp, 20.dp))
                .background(MaterialTheme.colorScheme.surfaceContainer).padding(horizontal = 14.dp, vertical = 10.dp),
        )
        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.heightIn(min = 32.dp)) {
            if (note != null) Text(note, style = MaterialTheme.typography.labelSmall, color = LocalSignals.current.faint)
            if (cancel != null) {
                if (cancelling) {
                    Spacer(Modifier.width(10.dp))
                    CircularProgressIndicator(Modifier.size(14.dp), strokeWidth = 1.5.dp)
                } else {
                    TextButton(onClick = cancel, contentPadding = PaddingValues(horizontal = 8.dp)) {
                        Text("Cancel", style = MaterialTheme.typography.labelMedium)
                    }
                }
            }
        }
    }
}

/** Below the newest message: what the thread is doing, or the card waiting on Mike. */
@Composable
private fun StatusCard(vm: ThreadViewModel, thread: PhoneThread, messages: List<PhoneMessage>, nothingOnItsWay: Boolean, now: Instant, leave: () -> Unit) {
    val model = LocalModel.current
    val signals = LocalSignals.current
    val colors = MaterialTheme.colorScheme
    val answering by vm.answering.collectAsStateWithLifecycle()
    val answeredAsks by vm.answeredAsks.collectAsStateWithLifecycle()
    val answeredDone by vm.answeredDone.collectAsStateWithLifecycle()
    val pressing by vm.pressingRuns.collectAsStateWithLifecycle()
    val pending = thread.pendingApproval
    // The choice tapped on this card, spinning until it goes.
    fun tapped(id: String) = answering?.takeIf { it.first == id }?.second
    val lastSeq = messages.lastOrNull()?.seq
    val asks = thread.asks.orEmpty()
    val showsAsks = asks.isNotEmpty() && nothingOnItsWay && answeredAsks != thread.asksSeq && thread.waitingOnMike
    val done = thread.status == ThreadStatus.IDLE && !thread.needsYou && thread.archivedAt == null && thread.working == null &&
        Turns.lastShown(messages)?.role == Role.ASSISTANT && nothingOnItsWay && answeredDone != lastSeq
    var picks by remember(thread.asksSeq) { mutableStateOf(mapOf<Int, Int>()) }
    var ticked by remember(lastSeq) { mutableStateOf(setOf<Int>()) }
    when {
        pending is PendingApproval.Approval -> ApprovalCard(pending, tapped(pending.id)) { vm.respond(pending.id, it) }
        pending is PendingApproval.Questions -> QuestionCard(pending, tapped(pending.id)) { chosen, typed -> vm.answer(pending.id, pending.questions, chosen, typed) }
        pending is PendingApproval.Secret -> SecretCard(
            pending,
            com.mikerosoft.threadbear.kit.RunnerKeyPins.check(pending.key, model.runnerKey(thread.runnerId)),
            tapped(pending.id),
        ) { vm.secret(pending, thread.runnerId, it) }
        pending is PendingApproval.Other -> CardFrame(signals.needsYou) {
            Text("Waiting on you", style = MaterialTheme.typography.titleMedium)
            Text("Answer this one on the Mac.", style = MaterialTheme.typography.bodyMedium, color = colors.onSurfaceVariant)
        }
        thread.isBusy || thread.working != null -> Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(vertical = 4.dp)) {
            WorkingDot(7.dp)
            Spacer(Modifier.width(10.dp))
            Text(thread.working ?: "Working", style = MaterialTheme.typography.bodyMedium, color = colors.onSurfaceVariant,
                maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f, fill = false))
            if (thread.isBusy) {
                Spacer(Modifier.width(8.dp))
                Text(RowStatus.workingLabel(thread.lastActivityAt, now), style = MaterialTheme.typography.labelSmall, color = signals.faint)
            }
        }
        showsAsks -> AsksCard(
            asks, picks, busy = answeredAsks == thread.asksSeq, pressing = pressing,
            pick = { i, c -> picks = if (c == null) picks - i else picks + (i to c) },
            send = { vm.sendAsks(thread, picks) },
            run = { vm.run(it.buttonId) },
        )
        thread.status == ThreadStatus.ERROR -> Row(verticalAlignment = Alignment.CenterVertically) {
            Icon(Line.Alert, null, Modifier.size(16.dp), tint = signals.failed)
            Spacer(Modifier.width(8.dp))
            Text(
                if (thread.waitingForNetwork != null) "Lost the connection. It carries on by itself." else "The last turn failed. Send a message to try again.",
                style = MaterialTheme.typography.bodyMedium, color = colors.onSurfaceVariant,
            )
        }
        done -> DoneCard(
            thread.next.orEmpty(), ticked, busy = answeredDone == lastSeq,
            tick = { i -> ticked = if (i in ticked) ticked - i else ticked + i },
            send = { val next = thread.next.orEmpty(); vm.sendFollowUps(ticked.sorted().mapNotNull { next.getOrNull(it) }, lastSeq) },
            archive = { model.change(thread.threadId, ThreadChange.Archive); leave() },
        )
        else -> Spacer(Modifier.height(1.dp))
    }
}
