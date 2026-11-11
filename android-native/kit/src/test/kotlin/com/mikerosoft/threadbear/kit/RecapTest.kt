package com.mikerosoft.threadbear.kit

import java.time.Instant
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/** RecapTests.swift. */
class RecapTest {
    private val now = Instant.ofEpochSecond(2_000_000_000)

    private fun thread(hoursAgo: Double, status: ThreadStatus = ThreadStatus.IDLE) = PhoneThread(
        threadId = "t", title = "T", status = status, preview = "",
        lastActivityAt = now.minusMillis((hoursAgo * 3_600_000).toLong()), recap = "For this.\n- Done",
    )

    @Test fun showsOnAQuietIdleThreadUntilPutAway() {
        val t = thread(12.0)
        assertTrue(Recap.isDue(t, null, now))
        assertFalse(Recap.isDue(thread(11.9), null, now))
        assertFalse(Recap.isDue(thread(72.0, ThreadStatus.RUNNING), null, now))
        assertFalse(Recap.isDue(t, Recap.seconds(t.lastActivityAt), now))
    }

    @Test fun splitsThePurposeFromTheBullets() {
        val parts = Recap.parts("For this.\n- One\n- Two")
        assertEquals("For this.", parts.purpose)
        assertEquals(listOf("One", "Two"), parts.points)
    }

    @Test fun saysHowLongInHoursThenDays() {
        assertEquals("Last worked on 3 days ago", Recap.lastWorkedOn(now.minusSeconds(72 * 3600), now))
    }
}
