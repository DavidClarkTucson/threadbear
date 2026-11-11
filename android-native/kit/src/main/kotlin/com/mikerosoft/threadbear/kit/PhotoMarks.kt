package com.mikerosoft.threadbear.kit

/**
 * Photos among a composer's words, as on the Mac (docs/protocol.md,
 * "Pictures in a message"): the composer half of InlinePictures.swift. The
 * message half, reading `[Picture N]` back out, is InlinePictures in
 * WaitingSends.kt.
 *
 * In the composer each photo is a mark in the text, U+FFFC, the k-th mark
 * standing for the k-th photo; the box draws each as a "Picture N" chip. A
 * message carries `[Picture N]` markers instead, and the runner puts
 * picture N straight after its marker when it goes to the agent. Offsets
 * are UTF-16, as a Kotlin String's (and NSString's) are.
 */
object PhotoMarks {
    const val MARK = '￼'
    const val MARK_STRING = "￼"

    fun marker(number: Int) = "[Picture $number]"

    fun markCount(text: CharSequence): Int = text.count { it == MARK }

    /** Words put right for `count` photos: one mark each. Spare marks go; a photo with no mark gets one at the end. */
    fun normalized(text: String, count: Int): String {
        val marks = markCount(text)
        if (marks == count) return text
        if (marks > count) {
            var kept = 0
            val out = StringBuilder()
            for (c in text) {
                if (c == MARK && ++kept > count) continue
                out.append(c)
            }
            return out.toString()
        }
        var out = text
        repeat(count - marks) { out = appending(MARK_STRING, out) }
        return out
    }

    internal fun appending(piece: String, text: String): String =
        if (text.isEmpty() || text.last().isWhitespace()) text + piece else "$text $piece"

    /** The words as they're sent: the k-th mark as `[Picture first+k]`, or "" when there's nothing but photos, which go as they always did. */
    fun sentText(text: String, first: Int = 1): String {
        if (MARK !in text) return text
        if (text.all { it == MARK || it.isWhitespace() }) return ""
        return messageText(text, first)
    }

    /** The words with the k-th mark as `[Picture first+k]`. */
    fun messageText(text: String, first: Int = 1): String {
        if (MARK !in text) return text
        val out = StringBuilder()
        var number = first
        for (c in text) if (c == MARK) out.append(marker(number++)) else out.append(c)
        return out.toString()
    }

    /** Where `count` marks went in: the words, which photo the first is (how many marks come before it), and where the cursor goes. */
    data class Inserted(val text: String, val firstIndex: Int, val cursor: Int)

    /** `count` marks in place of `start until end` (clamped), with a space either side, as the Mac's composer puts them. */
    fun inserting(count: Int, text: String, start: Int, end: Int = start): Inserted {
        val location = start.coerceIn(0, text.length)
        val stop = end.coerceIn(location, text.length)
        fun isSpace(i: Int) = i in text.indices && text[i].isWhitespace()
        var piece = if (location > 0 && !isSpace(location - 1)) " " else ""
        piece += List(count) { MARK_STRING }.joinToString(" ")
        if (!isSpace(stop)) piece += " "
        val out = text.substring(0, location) + piece + text.substring(stop)
        return Inserted(out, markCount(text.substring(0, location)), location + piece.length)
    }

    /** The words without the `index`-th mark and one space beside it. */
    fun removingMark(index: Int, text: String): String {
        var seen = 0
        val at = text.indices.firstOrNull { text[it] == MARK && seen++ == index } ?: return text
        val chars = StringBuilder(text).deleteCharAt(at)
        if (at < chars.length && chars[at] == ' ') chars.deleteCharAt(at)
        else if (at > 0 && chars[at - 1] == ' ') chars.deleteCharAt(at - 1)
        return chars.toString()
    }

    /** The words without their photos' marks. */
    fun words(text: String): String = text.filterNot { it == MARK }

    /**
     * Which photos an edit took out: the places, among `original`'s marks,
     * of those inside the ranges it replaced (`start until end` each, in
     * `original`'s offsets). A chip deleted or cut takes its photo with it.
     */
    fun removedMarks(original: CharSequence, replaced: List<IntRange>): List<Int> {
        if (replaced.isEmpty()) return emptyList()
        val out = mutableListOf<Int>()
        var place = 0
        for (i in original.indices) {
            if (original[i] != MARK) continue
            if (replaced.any { i in it }) out += place
            place++
        }
        return out
    }
}
