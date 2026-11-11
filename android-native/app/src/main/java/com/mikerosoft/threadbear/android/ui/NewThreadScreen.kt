package com.mikerosoft.threadbear.android.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

/**
 * A stand-in so the inbox's New thread button has somewhere to go. The New
 * thread screen itself is built on another branch: on merge, take that
 * branch's version of this file.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NewThreadScreen(close: () -> Unit, opened: (String) -> Unit) {
    Scaffold(
        topBar = {
            TopAppBar(
                navigationIcon = { IconButton(onClick = close) { Icon(Line.Back, "Back", Modifier.size(22.dp)) } },
                title = { Text("New thread") },
            )
        },
    ) { inner ->
        Box(Modifier.padding(inner).fillMaxSize(), contentAlignment = Alignment.Center) {
            Text("New thread", color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}
