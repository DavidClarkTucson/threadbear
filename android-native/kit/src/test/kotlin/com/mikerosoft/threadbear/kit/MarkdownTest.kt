package com.mikerosoft.threadbear.kit

import kotlin.test.Test
import kotlin.test.assertEquals

class MarkdownTest {
    @Test fun blocksOfAReply() {
        val text = """
            ## Done

            Fixed the **cache**, and
            the tests pass.

            - one
            - [x] two
              wrapped
            1. first
            2) second

            ```kotlin
            val x = 1

            val y = 2
            ```
            > quoted
            > more
            ---
            | A | B |
            |---|:-:|
            | 1 | 2 |
        """.trimIndent()
        val b = Markdown.blocks(text)
        assertEquals(MdBlock.Heading(2, "Done"), b[0])
        assertEquals(MdBlock.Paragraph("Fixed the **cache**, and\nthe tests pass."), b[1])
        assertEquals(MdBlock.ListItems(false, listOf(MdBlock.Item("", "one", 0), MdBlock.Item("", "two wrapped", 0, true))), b[2])
        assertEquals(MdBlock.ListItems(true, listOf(MdBlock.Item("1.", "first", 0), MdBlock.Item("2.", "second", 0))), b[3])
        assertEquals(MdBlock.Code("kotlin", "val x = 1\n\nval y = 2"), b[4])
        assertEquals(MdBlock.Quote("quoted\nmore"), b[5])
        assertEquals(MdBlock.Rule, b[6])
        assertEquals(MdBlock.Table(listOf("A", "B"), listOf(listOf("1", "2"))), b[7])
        assertEquals(8, b.size)
    }

    @Test fun anUnclosedFenceRunsToTheEnd() {
        assertEquals(listOf(MdBlock.Paragraph("Here:"), MdBlock.Code(null, "npm test")), Markdown.blocks("Here:\n```\nnpm test"))
    }

    @Test fun aDashedLineUnderWordsIsNotARule() {
        assertEquals(1, Markdown.blocks("Words\n---").size)
    }
}
