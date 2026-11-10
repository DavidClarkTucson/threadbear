package com.mikerosoft.threadbear.android.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextLinkStyles
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.mikerosoft.threadbear.kit.Markdown
import com.mikerosoft.threadbear.kit.MdBlock

/** A reply's markdown, drawn quietly: links in the accent, code in mono. */
@Composable
fun MarkdownText(text: String, modifier: Modifier = Modifier, style: TextStyle = MaterialTheme.typography.bodyLarge) {
    val blocks = remember(text) { Markdown.blocks(text) }
    val colors = MaterialTheme.colorScheme
    val accent = LocalSignals.current.accent
    val codeBg = colors.surfaceContainerHigh
    val inline = remember(accent, codeBg) { InlineStyle(accent, codeBg) }
    SelectionContainer(modifier) {
        Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
            for (block in blocks) when (block) {
                is MdBlock.Heading -> Text(
                    inline.parse(block.text),
                    style = style.copy(fontWeight = FontWeight.SemiBold, fontSize = if (block.level <= 2) 18.sp else style.fontSize),
                )
                is MdBlock.Paragraph -> Text(inline.parse(block.text), style = style)
                is MdBlock.ListItems -> Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    for (item in block.items) Row(Modifier.padding(start = (item.depth * 16).dp)) {
                        val mark = when {
                            item.checked == true -> "☑"
                            item.checked == false -> "☐"
                            item.marker.isNotEmpty() -> item.marker
                            else -> "•"
                        }
                        Text(mark, style = style, color = colors.onSurfaceVariant, modifier = Modifier.widthIn(min = 18.dp))
                        Spacer(Modifier.width(4.dp))
                        Text(inline.parse(item.text), style = style)
                    }
                }
                is MdBlock.Code -> Box(
                    Modifier.fillMaxWidth().clip(RoundedCornerShape(10.dp)).background(codeBg)
                        .horizontalScroll(rememberScrollState()).padding(horizontal = 12.dp, vertical = 10.dp),
                ) {
                    Text(block.text, style = style.copy(fontFamily = Mono, fontSize = 13.sp, lineHeight = 18.sp))
                }
                is MdBlock.Quote -> Row(Modifier.height(IntrinsicSize.Min)) {
                    Box(Modifier.width(2.dp).fillMaxHeight().background(colors.outline))
                    Spacer(Modifier.width(10.dp))
                    Text(inline.parse(block.text), style = style, color = colors.onSurfaceVariant)
                }
                MdBlock.Rule -> HorizontalDivider(color = colors.outlineVariant)
                is MdBlock.Table -> Table(block, inline, style)
            }
        }
    }
}

@Composable
private fun Table(table: MdBlock.Table, inline: InlineStyle, style: TextStyle) {
    val colors = MaterialTheme.colorScheme
    val small = style.copy(fontSize = 14.sp, lineHeight = 19.sp)
    Column(
        Modifier.clip(RoundedCornerShape(10.dp)).background(colors.surfaceContainerLow).horizontalScroll(rememberScrollState()),
    ) {
        val columns = maxOf(table.header.size, table.rows.maxOfOrNull { it.size } ?: 0)
        @Composable
        fun line(cells: List<String>, header: Boolean) = Row(Modifier.padding(horizontal = 4.dp)) {
            for (i in 0 until columns) {
                Text(
                    inline.parse(cells.getOrElse(i) { "" }),
                    style = if (header) small.copy(fontWeight = FontWeight.SemiBold) else small,
                    modifier = Modifier.width(140.dp).padding(horizontal = 8.dp, vertical = 6.dp),
                )
            }
        }
        line(table.header, true)
        HorizontalDivider(color = colors.outlineVariant)
        table.rows.forEach { line(it, false) }
    }
}

/** Inline marks: `code`, **bold**, *italic*, [links](url) and bare web addresses. */
class InlineStyle(private val accent: Color, private val codeBg: Color) {
    private val token = Regex(
        """`([^`]+)`|\*\*(.+?)\*\*|__(.+?)__|(?<![\w*])\*(?!\s)(.+?)(?<!\s)\*(?!\w)|(?<!\w)_(?!\s)(.+?)(?<!\s)_(?!\w)|\[([^\]]+)]\(([^)\s]+)\)|(https?://[^\s<>()]+[^\s<>().,;:!?'"])""",
    )

    fun parse(text: String): AnnotatedString = buildAnnotatedString { append(text, this) }

    private fun append(text: String, b: AnnotatedString.Builder) {
        var last = 0
        for (m in token.findAll(text)) {
            if (m.range.first > last) b.append(text.substring(last, m.range.first))
            val g = m.groups
            when {
                g[1] != null -> b.withStyle(SpanStyle(fontFamily = Mono, fontSize = 0.9.em(), background = codeBg)) { append(" ${g[1]!!.value} ") }
                g[2] != null -> b.withStyle(SpanStyle(fontWeight = FontWeight.SemiBold)) { append(g[2]!!.value, b) }
                g[3] != null -> b.withStyle(SpanStyle(fontWeight = FontWeight.SemiBold)) { append(g[3]!!.value, b) }
                g[4] != null -> b.withStyle(SpanStyle(fontStyle = FontStyle.Italic)) { append(g[4]!!.value, b) }
                g[5] != null -> b.withStyle(SpanStyle(fontStyle = FontStyle.Italic)) { append(g[5]!!.value, b) }
                g[6] != null -> link(b, g[6]!!.value, g[7]!!.value)
                g[8] != null -> link(b, g[8]!!.value, g[8]!!.value)
            }
            last = m.range.last + 1
        }
        if (last < text.length) b.append(text.substring(last))
    }

    private fun link(b: AnnotatedString.Builder, words: String, url: String) {
        val web = url.startsWith("https://") || url.startsWith("http://")
        if (!web) {
            // A file on the Mac, a work: or thread: link: just its words for now.
            b.withStyle(SpanStyle(color = accent)) { append(words) }
            return
        }
        b.withLink(LinkAnnotation.Url(url, TextLinkStyles(SpanStyle(color = accent)))) { append(words) }
    }

    private fun Double.em() = androidx.compose.ui.unit.TextUnit(this.toFloat(), androidx.compose.ui.unit.TextUnitType.Em)
}
