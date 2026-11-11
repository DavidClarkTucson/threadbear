package com.mikerosoft.threadbear.kit

import java.time.Instant
import kotlin.math.roundToLong

/**
 * The recap box: what a thread that has sat for a while is for and where it
 * stands. The same rule as the Mac's Recap.swift, the iPhone's, the
 * website's view-logic.ts and the runner's recapShowing (docs/protocol.md,
 * "The recap box"), so a change goes in all of them.
 */
object Recap {
    /** 12 hours: AWAY_MS in runner.ts. */
    const val QUIET_AFTER_MS = 12 * 3600 * 1000L

    /**
     * `dismissedAt`: the thread's lastActivityAt, in epoch seconds, when ×
     * was tapped on this phone (as the iPhone keeps it), so it stays away
     * for this quiet spell only.
     */
    fun isDue(thread: PhoneThread, dismissedAt: Double?, now: Instant = Instant.now()): Boolean {
        val recap = thread.recap
        if (recap.isNullOrEmpty() || thread.status != ThreadStatus.IDLE || thread.archivedAt != null) return false
        if (now.toEpochMilli() - thread.lastActivityAt.toEpochMilli() < QUIET_AFTER_MS) return false
        return dismissedAt != seconds(thread.lastActivityAt)
    }

    /** The thread's lastActivityAt as × keeps it. */
    fun seconds(at: Instant): Double = at.toEpochMilli() / 1000.0

    /** The sentence on what it's for, if the note has one, and the bullets. */
    data class Parts(val purpose: String?, val points: List<String>)

    fun parts(recap: String): Parts {
        var purpose: String? = null
        val points = mutableListOf<String>()
        for (raw in recap.split("\n")) {
            if (raw.isEmpty()) continue
            val line = raw.trimSpaces()
            when {
                line.startsWith("- ") -> points += line.substring(2)
                purpose == null && points.isEmpty() && line.isNotEmpty() -> purpose = line
                line.isNotEmpty() -> points += line
            }
        }
        return Parts(purpose, points)
    }

    fun lastWorkedOn(date: Instant, now: Instant = Instant.now()): String {
        val hours = ((now.toEpochMilli() - date.toEpochMilli()) / 3_600_000.0).roundToLong()
        return "Last worked on " + if (hours < 48) "$hours hours ago" else "${(hours / 24.0).roundToLong()} days ago"
    }
}
