package com.mikerosoft.threadbear.kit

/**
 * A reply's markdown as blocks, enough for a phone (MarkdownBlocks.swift,
 * cut down): headings, paragraphs, lists, code, quotes, rules and tables.
 * Inline marks (bold, code, links) are the renderer's.
 */
sealed class MdBlock {
    data class Heading(val level: Int, val text: String) : MdBlock()
    data class Paragraph(val text: String) : MdBlock()
    data class ListItems(val ordered: Boolean, val items: List<Item>) : MdBlock()
    data class Code(val language: String?, val text: String) : MdBlock()
    data class Quote(val text: String) : MdBlock()
    data object Rule : MdBlock()
    data class Table(val header: List<String>, val rows: List<List<String>>) : MdBlock()

    /** `marker`: "1." for an ordered item, "" for a bullet; `depth` its indent. */
    data class Item(val marker: String, val text: String, val depth: Int, val checked: Boolean? = null)
}

object Markdown {
    private val heading = Regex("""^(#{1,6})\s+(.*?)\s*#*\s*$""")
    private val bullet = Regex("""^(\s*)[-*+]\s+(.*)$""")
    private val ordered = Regex("""^(\s*)(\d{1,9})[.)]\s+(.*)$""")
    private val fence = Regex("""^\s*(```+|~~~+)\s*([\w+#.-]*)\s*$""")
    private val rule = Regex("""^\s*([-*_])(\s*\1){2,}\s*$""")
    private val tableDivider = Regex("""^\s*\|?\s*:?-+:?\s*(\|\s*:?-+:?\s*)*\|?\s*$""")
    private val task = Regex("""^\[([ xX])]\s+(.*)$""")

    fun blocks(text: String): List<MdBlock> {
        val lines = text.replace("\r\n", "\n").split('\n')
        val out = mutableListOf<MdBlock>()
        val para = StringBuilder()
        fun flush() {
            if (para.isNotBlank()) out += MdBlock.Paragraph(para.toString().trim())
            para.clear()
        }
        var i = 0
        while (i < lines.size) {
            val line = lines[i]
            val f = fence.matchEntire(line)
            if (f != null) {
                flush()
                val close = f.groupValues[1]
                val body = mutableListOf<String>()
                i++
                while (i < lines.size && !lines[i].trim().startsWith(close)) { body += lines[i]; i++ }
                out += MdBlock.Code(f.groupValues[2].ifEmpty { null }, body.joinToString("\n"))
                i++
                continue
            }
            if (line.isBlank()) { flush(); i++; continue }
            val h = heading.matchEntire(line)
            if (h != null) {
                flush()
                out += MdBlock.Heading(h.groupValues[1].length, h.groupValues[2])
                i++
                continue
            }
            if (rule.matches(line) && para.isEmpty()) { out += MdBlock.Rule; i++; continue }
            if (line.trimStart().startsWith(">")) {
                flush()
                val quote = mutableListOf<String>()
                while (i < lines.size && lines[i].trimStart().startsWith(">")) {
                    quote += lines[i].trimStart().removePrefix(">").removePrefix(" "); i++
                }
                out += MdBlock.Quote(quote.joinToString("\n"))
                continue
            }
            if ('|' in line && i + 1 < lines.size && tableDivider.matches(lines[i + 1])) {
                flush()
                val header = cells(line)
                i += 2
                val rows = mutableListOf<List<String>>()
                while (i < lines.size && '|' in lines[i] && lines[i].isNotBlank()) { rows += cells(lines[i]); i++ }
                out += MdBlock.Table(header, rows)
                continue
            }
            if (bullet.matches(line) || ordered.matches(line)) {
                flush()
                val isOrdered = ordered.matches(line)
                val items = mutableListOf<MdBlock.Item>()
                while (i < lines.size) {
                    val l = lines[i]
                    val b = bullet.matchEntire(l)
                    val o = ordered.matchEntire(l)
                    // Bullets turning into numbers (or back) at the top level start a new list.
                    if ((b != null && isOrdered && b.groupValues[1].isEmpty()) || (o != null && !isOrdered && o.groupValues[1].isEmpty())) break
                    when {
                        b != null -> {
                            val t = task.matchEntire(b.groupValues[2])
                            items += MdBlock.Item("", t?.groupValues?.get(2) ?: b.groupValues[2], b.groupValues[1].length / 2,
                                t?.let { it.groupValues[1] != " " })
                        }
                        o != null -> items += MdBlock.Item("${o.groupValues[2]}.", o.groupValues[3], o.groupValues[1].length / 2)
                        l.isNotBlank() && l.startsWith("  ") && items.isNotEmpty() -> {
                            val last = items.removeAt(items.size - 1)
                            items += last.copy(text = last.text + " " + l.trim())
                        }
                        else -> break
                    }
                    i++
                }
                out += MdBlock.ListItems(isOrdered, items)
                continue
            }
            if (para.isNotEmpty()) para.append('\n')
            para.append(line)
            i++
        }
        flush()
        return out
    }

    private fun cells(line: String): List<String> =
        line.trim().removePrefix("|").removeSuffix("|").split('|').map { it.trim() }
}
