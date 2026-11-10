package com.mikerosoft.threadbear.android.ui

import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.animateFloat
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
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
import androidx.compose.material3.SwipeToDismissBox
import androidx.compose.material3.SwipeToDismissBoxValue
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.rememberSwipeToDismissBoxState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.produceState
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.mikerosoft.threadbear.android.LocalModel
import com.mikerosoft.threadbear.android.LocalSnackbar
import com.mikerosoft.threadbear.kit.InboxSections
import com.mikerosoft.threadbear.kit.PhoneProject
import com.mikerosoft.threadbear.kit.PhoneSection
import com.mikerosoft.threadbear.kit.PhoneThread
import com.mikerosoft.threadbear.kit.ProjectMark
import com.mikerosoft.threadbear.kit.RowStatus
import com.mikerosoft.threadbear.kit.ThreadChange
import java.time.Instant
import kotlinx.coroutines.delay

/** Now, every few seconds: the rows' times and working clocks follow it. */
@Composable
fun rememberNow(everyMs: Long = 5000): Instant {
    val now by produceState(Instant.now()) {
        while (true) { delay(everyMs); value = Instant.now() }
    }
    return now
}

@OptIn(ExperimentalMaterial3Api::class, ExperimentalFoundationApi::class)
@Composable
fun InboxScreen(open: (String) -> Unit) {
    val model = LocalModel.current
    val inbox by model.inbox.collectAsStateWithLifecycle()
    val wakes by model.wakes.collectAsStateWithLifecycle()
    val counts by model.counts.collectAsStateWithLifecycle()
    val folds by model.folds.collectAsStateWithLifecycle()
    val away by model.puttingAway.collectAsStateWithLifecycle()
    val connected by model.connected.collectAsStateWithLifecycle()
    val projects by model.projects.collectAsStateWithLifecycle()
    val snoozed by model.putAway(PhoneSection.HIBERNATING).collectAsStateWithLifecycle()
    val den by model.putAway(PhoneSection.ARCHIVED).collectAsStateWithLifecycle()
    val now = rememberNow()
    val scroll = TopAppBarDefaults.pinnedScrollBehavior()
    var menu by remember { mutableStateOf(false) }

    val wakeMap = remember(wakes) { InboxSections.wakes(wakes) }
    val sections = remember(inbox, wakeMap, away, now) {
        InboxSections.build((inbox ?: emptyList()).filter { it.threadId !in away }, now, wakeMap)
    }
    val projectById = remember(projects) { projects.associateBy { it.id } }

    Scaffold(
        modifier = Modifier.nestedScroll(scroll.nestedScrollConnection),
        snackbarHost = { SnackbarHost(LocalSnackbar.current) },
        topBar = {
            TopAppBar(
                title = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text("Threadbear")
                        val n = sections.needsYouCount
                        if (n > 0) {
                            Spacer(Modifier.width(10.dp))
                            CountBadge(n)
                        }
                    }
                },
                actions = {
                    if (!connected && inbox != null) {
                        IconButton(onClick = { model.note("Not connected. Showing what came last; it catches up by itself.") }) {
                            Icon(Line.CloudOff, "Not connected", Modifier.size(20.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                    Box {
                        IconButton(onClick = { menu = true }) { Icon(Line.More, "More", Modifier.size(20.dp)) }
                        DropdownMenu(expanded = menu, onDismissRequest = { menu = false }) {
                            val server = model.pairing.value?.serverName ?: ""
                            DropdownMenuItem(
                                text = { Text(server, color = MaterialTheme.colorScheme.onSurfaceVariant) },
                                onClick = {}, enabled = false,
                            )
                            DropdownMenuItem(
                                text = { Text("Unpair") },
                                leadingIcon = { Icon(Line.LogOut, null, Modifier.size(18.dp)) },
                                onClick = { menu = false; model.unpair() },
                            )
                        }
                    }
                },
                scrollBehavior = scroll,
            )
        },
    ) { inner ->
        if (inbox == null) {
            Column(Modifier.padding(inner).fillMaxWidth()) {
                LinearProgressIndicator(Modifier.fillMaxWidth().height(1.dp), color = MaterialTheme.colorScheme.outline)
            }
            return@Scaffold
        }
        LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(top = inner.calculateTopPadding(), bottom = inner.calculateBottomPadding() + 24.dp)) {
            fun section(section: PhoneSection, rows: List<PhoneThread>, count: Int?, empty: (@Composable () -> Unit)? = null) {
                val folded = folds.isFolded(section)
                stickyHeader(key = "h:${section.name}") {
                    Heading(section, count ?: rows.size, folded) { model.toggle(section) }
                }
                if (folded) return
                if (rows.isEmpty() && empty != null) item(key = "e:${section.name}") { empty() }
                items(rows, key = { "${section.name}:${it.threadId}" }) { t ->
                    SwipeRow(t, section, now, projectById, open)
                }
            }
            if (sections.starred.isNotEmpty()) section(PhoneSection.STARRED, sections.starred, null)
            section(PhoneSection.INBOX, sections.inbox, null) { EmptyInbox() }
            if (sections.running.isNotEmpty() || !folds.isFolded(PhoneSection.RUNNING)) section(PhoneSection.RUNNING, sections.running, null)
            val hibernating = InboxSections.hibernating(sections.scheduled, snoozed ?: emptyList(), wakeMap)
            section(PhoneSection.HIBERNATING, hibernating, if (snoozed == null) (counts?.hibernating ?: 0) + sections.scheduled.size else hibernating.size)
            section(PhoneSection.ARCHIVED, den ?: emptyList(), counts?.den ?: den?.size)
        }
    }
}

@Composable
private fun CountBadge(n: Int) {
    Box(
        Modifier.clip(CircleShape).background(LocalSignals.current.needsYou.copy(alpha = 0.18f)).padding(horizontal = 8.dp, vertical = 1.dp),
    ) {
        Text("$n", style = MaterialTheme.typography.labelMedium, color = LocalSignals.current.needsYou)
    }
}

@Composable
private fun Heading(section: PhoneSection, count: Int, folded: Boolean, toggle: () -> Unit) {
    val turn by animateFloatAsState(if (folded) -90f else 0f, label = "chevron")
    Row(
        Modifier.fillMaxWidth().background(MaterialTheme.colorScheme.background)
            .clickable(enabled = section != PhoneSection.INBOX, onClick = toggle)
            .padding(start = 20.dp, end = 16.dp, top = 14.dp, bottom = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(section.title, style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
        if (count > 0 && section != PhoneSection.INBOX) {
            Spacer(Modifier.width(6.dp))
            Text("$count", style = MaterialTheme.typography.labelLarge, color = LocalSignals.current.faint)
        }
        Spacer(Modifier.weight(1f))
        if (section != PhoneSection.INBOX) {
            Icon(Line.ChevronDown, if (folded) "Open" else "Fold", Modifier.size(16.dp).rotate(turn), tint = LocalSignals.current.faint)
        }
    }
}

@Composable
private fun EmptyInbox() {
    Column(Modifier.fillMaxWidth().padding(vertical = 40.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        Icon(Line.Check, null, Modifier.size(28.dp), tint = LocalSignals.current.faint)
        Spacer(Modifier.height(8.dp))
        Text("Nothing needs you", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

/** A row that archives with a swipe, or brings a put-away one back. */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun SwipeRow(t: PhoneThread, section: PhoneSection, now: Instant, projects: Map<String, PhoneProject>, open: (String) -> Unit) {
    val model = LocalModel.current
    val change: ThreadChange = when (section) {
        PhoneSection.ARCHIVED -> ThreadChange.Unarchive
        PhoneSection.HIBERNATING -> ThreadChange.Wake
        else -> ThreadChange.Archive
    }
    val state = rememberSwipeToDismissBoxState()
    LaunchedEffect(state.currentValue) {
        if (state.currentValue == SwipeToDismissBoxValue.EndToStart) {
            model.change(t.threadId, change)
            if (change !is ThreadChange.Archive) state.reset()
        }
    }
    SwipeToDismissBox(
        state = state,
        enableDismissFromStartToEnd = false,
        backgroundContent = {
            val icon = when (change) {
                ThreadChange.Unarchive -> Line.Inbox
                ThreadChange.Wake -> Line.Sun
                else -> Line.Archive
            }
            Box(
                Modifier.fillMaxSize().background(MaterialTheme.colorScheme.surfaceContainerHigh).padding(end = 24.dp),
                contentAlignment = Alignment.CenterEnd,
            ) { Icon(icon, null, Modifier.size(22.dp), tint = MaterialTheme.colorScheme.onSurface) }
        },
    ) {
        ThreadRow(t, now, projects, Modifier.background(MaterialTheme.colorScheme.background).clickable { open(t.threadId) })
    }
}

@Composable
fun ThreadRow(t: PhoneThread, now: Instant, projects: Map<String, PhoneProject>, modifier: Modifier = Modifier) {
    val signals = LocalSignals.current
    val kind = RowStatus.kind(t)
    Row(
        modifier.fillMaxWidth().animateContentSize().padding(start = 6.dp, end = 16.dp, top = 10.dp, bottom = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        // The gutter: the unread dot.
        Box(Modifier.width(14.dp), contentAlignment = Alignment.Center) {
            if (t.unread || t.needsYou) Box(Modifier.size(7.dp).clip(CircleShape).background(signals.accent))
        }
        ProjectTile(t.projects?.firstOrNull()?.let { projects[it.id] }, t.projects?.firstOrNull()?.title ?: t.folderName, 38.dp)
        Spacer(Modifier.width(12.dp))
        Column(Modifier.weight(1f)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    t.title.ifEmpty { "Untitled" },
                    style = MaterialTheme.typography.bodyLarge,
                    fontWeight = if (t.unread) FontWeight.SemiBold else FontWeight.Normal,
                    maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f, fill = false),
                )
                if (t.starredAt != null) {
                    Spacer(Modifier.width(4.dp))
                    Icon(Line.StarFilled, "Starred", Modifier.size(12.dp), tint = signals.faint)
                }
            }
            Row(verticalAlignment = Alignment.CenterVertically) {
                t.startedBy?.let {
                    Icon(Line.startedBy(it.kind), null, Modifier.size(12.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                    Spacer(Modifier.width(4.dp))
                }
                val line = listOfNotNull(InboxSections.wakeLine(t, now), t.placeText).joinToString(" · ")
                Text(line, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
        Spacer(Modifier.width(10.dp))
        val (text, color) = when (kind) {
            RowStatus.Kind.NEEDS_YOU -> "Needs you" to signals.needsYou
            RowStatus.Kind.WORKING -> RowStatus.workingLabel(t.lastActivityAt, now) to signals.accent
            RowStatus.Kind.FAILED -> "Failed" to signals.failed
            RowStatus.Kind.WAITING_FOR_CONNECTION -> "Offline" to MaterialTheme.colorScheme.onSurfaceVariant
            RowStatus.Kind.STOPPED -> "Stopped" to MaterialTheme.colorScheme.onSurfaceVariant
            RowStatus.Kind.TIME -> RowStatus.text(t, now) to MaterialTheme.colorScheme.onSurfaceVariant
        }
        if (kind == RowStatus.Kind.WORKING) {
            WorkingDot()
            Spacer(Modifier.width(6.dp))
        }
        Text(text, style = MaterialTheme.typography.labelMedium, color = color)
    }
}

/** A project's tile: its first letter on a quiet square, as the Mac's sidebar. */
@Composable
fun ProjectTile(project: PhoneProject?, title: String, size: androidx.compose.ui.unit.Dp) {
    Box(
        Modifier.size(size).clip(RoundedCornerShape(size * 0.26f)).background(MaterialTheme.colorScheme.surfaceContainerHighest),
        contentAlignment = Alignment.Center,
    ) {
        if (project == null && title.isEmpty()) {
            Icon(Line.Folder, null, Modifier.size(size * 0.5f), tint = MaterialTheme.colorScheme.onSurfaceVariant)
        } else {
            Text(
                ProjectMark.letter(project?.title ?: title),
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.85f),
            )
        }
    }
}

/** A soft pulse in the accent, for a thread that's working. */
@Composable
fun WorkingDot(size: androidx.compose.ui.unit.Dp = 6.dp) {
    val pulse = androidx.compose.animation.core.rememberInfiniteTransition(label = "pulse")
    val alpha by pulse.animateFloat(
        initialValue = 0.35f, targetValue = 1f,
        animationSpec = androidx.compose.animation.core.infiniteRepeatable(
            androidx.compose.animation.core.tween(900), androidx.compose.animation.core.RepeatMode.Reverse,
        ),
        label = "alpha",
    )
    Box(Modifier.size(size).clip(CircleShape).background(LocalSignals.current.accent.copy(alpha = alpha)))
}

