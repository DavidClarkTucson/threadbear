package com.mikerosoft.threadbear.android

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
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
import com.mikerosoft.threadbear.android.ui.NewThreadScreen
import com.mikerosoft.threadbear.android.ui.PocketScreen
import com.mikerosoft.threadbear.android.ui.RunnerOrderScreen
import com.mikerosoft.threadbear.android.ui.SettingsScreen
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
        // A thread that started soon after it was asked for opens, as the iPhone's
        // sheet used to wait to do, unless you've gone into another meanwhile.
        val started by model.shell.justStarted.collectAsStateWithLifecycle()
        LaunchedEffect(started) {
            val id = started ?: return@LaunchedEffect
            model.shell.justStarted.value = null
            if (nav.currentDestination?.route == "inbox") nav.navigate("thread/$id")
        }
        NavHost(nav, startDestination = "inbox") {
            composable("inbox") {
                InboxScreen(
                    open = { nav.navigate("thread/$it") },
                    newThread = { nav.navigate("new") { launchSingleTop = true } },
                    settings = { nav.navigate("settings") { launchSingleTop = true } },
                    pocket = { nav.navigate("pocket") { launchSingleTop = true } },
                )
            }
            composable("new") {
                NewThreadScreen(close = { nav.popBackStack() }, opened = { id ->
                    nav.navigate("thread/$id") { popUpTo("inbox") }
                })
            }
            composable("settings") { SettingsScreen(back = { nav.popBackStack() }, runnerOrder = { nav.navigate("settings/runners") }) }
            composable("settings/runners") { RunnerOrderScreen(back = { nav.popBackStack() }) }
            composable("pocket") {
                // Only once the Pocket's gone, as the iPhone's cover: the thread
                // or New thread goes on the inbox, not over the Pocket.
                PocketScreen(
                    close = { nav.popBackStack() },
                    openThread = { id -> nav.navigate("thread/$id") { popUpTo("inbox") } },
                    add = {
                        model.shell.composeText.value = "Add to my pocket: "
                        nav.navigate("new") { popUpTo("inbox") }
                    },
                )
            }
            composable("thread/{id}") { entry ->
                ThreadScreen(threadId = entry.arguments?.getString("id") ?: "", back = { nav.popBackStack() },
                    open = { nav.navigate("thread/$it") })
            }
        }
    }

    pending?.let { request ->
        AlertDialog(
            onDismissRequest = { model.cancelPair() },
            title = { Text(breakable(request.title, request.host), style = MaterialTheme.typography.titleLarge) },
            text = { Text(breakable(request.message, request.host)) },
            confirmButton = { TextButton(onClick = { model.acceptPair(request) }) { Text("Pair") } },
            dismissButton = { TextButton(onClick = { model.cancelPair() }) { Text("Cancel") } },
        )
    }
}

/**
 * A Convex host is longer than a dialog line, and Android broke it
 * mid-word with the "?" alone on the last line: let it break only after
 * its dots (a zero-width space), and keep what follows it joined on.
 */
private fun breakable(text: String, host: String): String =
    text.replace(host, host.replace(".", ".\u200B") + "\u2060")
