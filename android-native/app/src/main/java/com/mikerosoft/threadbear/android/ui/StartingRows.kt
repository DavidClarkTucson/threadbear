package com.mikerosoft.threadbear.android.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.mikerosoft.threadbear.android.LocalModel
import com.mikerosoft.threadbear.android.StartingThread
import com.mikerosoft.threadbear.kit.NewThreadChoices
import java.time.Instant

/**
 * New threads on their way, at the top of the inbox (the iPhone's
 * StartingRow), until the runner has made each and its own row takes over.
 * One item, so the inbox needs one line for them.
 */
fun LazyListScope.startingRows() {
    item(key = "starting") { StartingRows() }
}

@Composable
private fun StartingRows() {
    val model = LocalModel.current
    val rows by model.starts.rows.collectAsStateWithLifecycle()
    if (rows.isEmpty()) return
    val now = rememberNow(5000)
    Column {
        for (row in rows) StartingRow(row, now)
    }
}

/**
 * A thread row's look for a new thread on its way: what was asked, and
 * "Starting" with a spinner where a working row's time goes. A failed one
 * opens the new thread screen again with what was typed, or goes with its X.
 */
@Composable
private fun StartingRow(row: StartingThread, now: Instant) {
    val model = LocalModel.current
    val runners by model.runners.collectAsStateWithLifecycle()
    val signals = LocalSignals.current
    val colors = MaterialTheme.colorScheme
    val failed = row.failed
    val title = row.message.lineSequence().firstOrNull()?.trim().orEmpty().ifEmpty {
        val n = row.draft.photos.size
        if (n == 1) "A photo" else "$n photos"
    }
    val line2 = if (failed != null) "Tap to try again · $failed" else {
        // Its Mac asleep: it starts when it's back, maybe much later.
        // No runner picked: the backend's, which with one runner is that one.
        val picked = row.draft.runnerId?.let { id -> runners.firstOrNull { it.runnerId == id } }
        val runner = picked ?: runners.singleOrNull()
        when {
            runner == null -> NewThreadChoices.offlineText(runners, now) ?: "On its way"
            !runner.isOnline(now) -> "${runner.name} offline, starts when it's back"
            else -> runner.name
        }
    }
    Row(
        Modifier.fillMaxWidth().background(colors.background)
            .clickable(enabled = failed != null) { model.starts.retry(row) }
            .padding(start = 6.dp, end = if (failed != null) 4.dp else 16.dp, top = 10.dp, bottom = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Spacer(Modifier.width(14.dp))
        ProjectTile(null, "", 38.dp)
        Spacer(Modifier.width(12.dp))
        Column(Modifier.weight(1f)) {
            Text(
                title, style = MaterialTheme.typography.bodyLarge, maxLines = 1, overflow = TextOverflow.Ellipsis,
                color = if (failed == null) colors.onSurfaceVariant else colors.onSurface,
            )
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(line2, style = MaterialTheme.typography.bodyMedium, color = colors.onSurfaceVariant, maxLines = 1,
                    overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
                Spacer(Modifier.width(8.dp))
                if (failed != null) {
                    Text("Failed", style = MaterialTheme.typography.labelMedium, fontWeight = FontWeight.Medium, color = signals.failed)
                } else {
                    CircularProgressIndicator(Modifier.size(10.dp), strokeWidth = 1.5.dp, color = signals.accent)
                    Spacer(Modifier.width(5.dp))
                    Text("Starting", style = MaterialTheme.typography.labelMedium, color = signals.accent)
                }
            }
        }
        if (failed != null) {
            Box(Modifier.padding(start = 4.dp)) {
                IconButton(onClick = { model.starts.drop(row.id) }, modifier = Modifier.size(36.dp)) {
                    Icon(Line.Close, "Remove", Modifier.size(16.dp), tint = colors.onSurfaceVariant)
                }
            }
        }
    }
}
