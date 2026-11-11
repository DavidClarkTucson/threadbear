package com.mikerosoft.threadbear.android.ui

import android.content.Context
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.mikerosoft.threadbear.kit.PhoneLink
import com.mikerosoft.threadbear.kit.PhoneThread
import com.mikerosoft.threadbear.kit.Recap

/**
 * What a thread that has sat for a while is for and where it stands, above
 * the message box (docs/protocol.md, "The recap box"). × is remembered on
 * this phone, for this quiet spell.
 */
@Composable
fun RecapBox(thread: PhoneThread) {
    val context = LocalContext.current
    val prefs = remember(context) { context.getSharedPreferences("recaps", Context.MODE_PRIVATE) }
    val key = "recapAway.${thread.threadId}"
    // Seconds, as the iPhone keeps it; as text, so the Double comes back exactly.
    var dismissedAt by remember(thread.threadId) { mutableStateOf(prefs.getString(key, null)?.toDoubleOrNull()) }
    val recap = thread.recap ?: return
    if (!Recap.isDue(thread, dismissedAt)) return
    val parts = remember(recap) { Recap.parts(recap) }
    val colors = MaterialTheme.colorScheme
    Box(Modifier.fillMaxWidth().padding(start = 12.dp, end = 12.dp, bottom = 8.dp).clip(RoundedCornerShape(16.dp)).background(colors.surfaceContainer)) {
        Column(Modifier.padding(start = 14.dp, end = 40.dp, top = 11.dp, bottom = 11.dp), verticalArrangement = Arrangement.spacedBy(5.dp)) {
            parts.purpose?.let { Text(it, style = MaterialTheme.typography.bodyMedium) }
            for (point in parts.points) Row {
                Text("•", style = MaterialTheme.typography.bodyMedium)
                Spacer(Modifier.width(7.dp))
                Text(point, style = MaterialTheme.typography.bodyMedium)
            }
            Text(Recap.lastWorkedOn(thread.lastActivityAt), style = MaterialTheme.typography.labelSmall, color = colors.onSurfaceVariant)
        }
        Box(
            Modifier.align(Alignment.TopEnd).size(44.dp).clip(CircleShape).clickable(onClickLabel = "Put this away") {
                val at = Recap.seconds(thread.lastActivityAt)
                prefs.edit().putString(key, at.toString()).apply()
                dismissedAt = at
            },
            contentAlignment = Alignment.Center,
        ) { Icon(Line.Close, "Put this away", Modifier.size(14.dp), tint = colors.onSurfaceVariant) }
    }
}

/**
 * A thread's patch link buttons: each opens its page in the browser here,
 * with nothing sent to the Mac. A row that scrolls sideways when there are
 * more than fit.
 */
@Composable
fun LinkChips(links: List<PhoneLink>, open: (String) -> Unit) {
    val pages = links.mapNotNull { l -> l.page?.let { l.title to it } }
    if (pages.isEmpty()) return
    val colors = MaterialTheme.colorScheme
    Row(
        Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = 12.dp, vertical = 2.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        for ((title, page) in pages) {
            Row(
                Modifier.clip(CircleShape).background(colors.surfaceContainerHigh).clickable(onClickLabel = "Opens it in the browser") { open(page) }
                    .padding(horizontal = 10.dp, vertical = 6.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(Line.External, null, Modifier.size(13.dp), tint = colors.onSurfaceVariant)
                Spacer(Modifier.width(5.dp))
                Text(title, style = MaterialTheme.typography.labelLarge, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
    }
}
