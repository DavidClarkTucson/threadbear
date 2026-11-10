package com.mikerosoft.threadbear.android

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.mikerosoft.threadbear.android.ui.InboxScreen
import com.mikerosoft.threadbear.android.ui.PairScreen
import com.mikerosoft.threadbear.android.ui.ThreadScreen
import com.mikerosoft.threadbear.android.ui.ThreadbearTheme
import kotlinx.coroutines.flow.MutableStateFlow

val LocalModel = staticCompositionLocalOf<AppModel> { error("No model") }
val LocalSnackbar = staticCompositionLocalOf { SnackbarHostState() }

class MainActivity : ComponentActivity() {
    private val model get() = (application as ThreadbearApplication).model
    /** A thread to open, from a notification or a debug launch. */
    private val openThread = MutableStateFlow<String?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
        if (savedInstanceState == null) handle(intent)
        setContent {
            ThreadbearTheme {
                val snackbar = remember { SnackbarHostState() }
                CompositionLocalProvider(LocalModel provides model, LocalSnackbar provides snackbar) {
                    Root(model, openThread)
                }
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handle(intent)
    }

    private fun handle(intent: Intent?) {
        val data = intent?.dataString
        if (data != null && data.startsWith("threadbear://pair")) {
            if (!model.ask(data)) model.note("That isn't a pairing link Threadbear can use.")
        }
        intent?.getStringExtra("threadId")?.let { openThread.value = it }
    }
}

@Composable
private fun Root(model: AppModel, openThread: MutableStateFlow<String?>) {
    val pairing by model.pairing.collectAsStateWithLifecycle()
    val pending by model.pendingPair.collectAsStateWithLifecycle()
    val snackbar = LocalSnackbar.current
    LaunchedEffect(Unit) { model.notes.collect { snackbar.showSnackbar(it) } }

    if (pairing == null) {
        PairScreen()
    } else {
        val nav = rememberNavController()
        val open by openThread.collectAsStateWithLifecycle()
        LaunchedEffect(open) {
            open?.let { nav.navigate("thread/$it") { launchSingleTop = true }; openThread.value = null }
        }
        NavHost(nav, startDestination = "inbox") {
            composable("inbox") { InboxScreen(open = { nav.navigate("thread/$it") }) }
            composable("thread/{id}") { entry ->
                ThreadScreen(threadId = entry.arguments?.getString("id") ?: "", back = { nav.popBackStack() },
                    open = { nav.navigate("thread/$it") })
            }
        }
    }

    pending?.let { request ->
        AlertDialog(
            onDismissRequest = { model.cancelPair() },
            title = { Text(request.title) },
            text = { Text(request.message) },
            confirmButton = { TextButton(onClick = { model.acceptPair(request) }) { Text("Pair") } },
            dismissButton = { TextButton(onClick = { model.cancelPair() }) { Text("Cancel") } },
        )
    }
}
