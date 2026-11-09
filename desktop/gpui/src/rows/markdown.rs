// Reply text: MarkdownRenderer.swift's look in gpui-kit's TextView. The kit
// does the blocks; this sets its sizes, spacing and colours to the Swift
// app's, and sends links where they should go.
use super::metrics::CODE_PAD;
use crate::style::{self, size};
use crate::thread::ThreadPane;
use gpui_kit::base::{TextViewState, TextViewStyle};
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::*;
use gpui_kit::*;

/// How far a list's text sits in from the text around it (listIndent).
const LIST_INDENT: f32 = 30.;
/// The gap between paragraphs (paragraphGap).
const PARAGRAPH_GAP: f32 = 10.;

/// The reading column's style. Built once per appearance and shared: a new
/// `heading` closure per frame would make the kit see a different style and
/// lay every reply out again.
pub(crate) fn reading_style(cx: &App) -> TextViewStyle {
    let (label, muted, link, code_bg, inline_bg, border, hairline, mono) = (
        style::label_text(cx),
        style::secondary_label(cx),
        style::link_text(cx),
        style::code_background(cx),
        style::inline_code_background(cx),
        style::card_border(cx),
        style::hairline(cx),
        cx.theme().mono_font_family.clone(),
    );
    let rem = f32::from(cx.theme().font_size);
    let code_block = StyleRefinement::default()
        .rounded(px(7.))
        .border_0()
        .p(px(CODE_PAD))
        .text_size(px(size::CODE))
        .line_height(px(size::CODE_LINE))
        .font_family(mono.clone());
    // The pill behind inline code, with a little room either side (pillPad);
    // the kit reads padding and radius from this.
    let inline_code = StyleRefinement::default().bg(inline_bg).rounded(px(4.)).px(px(2.5)).py(px(1.)).text_size(px(size::CODE)).font_family(mono.clone()).text_color(label);
    // A rule under the header and hairlines between rows, none around them
    // (renderTable). No cell backgrounds.
    let table = StyleRefinement::default().border_0().rounded(px(0.));
    let table_head = StyleRefinement::default().text_size(px(size::LABEL)).font_weight(FontWeight::SEMIBOLD).text_color(muted).bg(gpui_kit::transparent_black()).border_color(border);
    // Transparent: the kit draws a column divider in the cell border colour, and
    // the rows carry the rules (renderTable has no vertical lines).
    let table_cell = StyleRefinement::default().py(px(4.)).px(px(12.)).border_color(gpui_kit::transparent_black());
    TextViewStyle::default()
        .with_foreground(label)
        .with_muted_foreground(muted)
        .with_link(link)
        .with_selection(cx.theme().selection)
        .with_code_background(code_bg)
        .with_border(hairline)
        .with_paragraph_gap(rems(PARAGRAPH_GAP / rem))
        .with_heading(move |level| {
            // Sizes and the 6 above / 6 below from MarkdownRenderer (the
            // paragraph before leaves 10 of the 16 above a heading).
            let (font, line) = match level {
                1 => (18., 26.),
                2 => (16., 23.),
                _ => (15., 21.),
            };
            StyleRefinement::default().text_size(px(font)).line_height(px(line)).font_weight(FontWeight::SEMIBOLD).pt(px(6.)).pb(px(6.))
        })
        .with_code_block(code_block)
        .with_inline_code(HighlightStyle { background_color: Some(inline_bg), ..Default::default() })
        .with_inline_code_style(inline_code)
        .with_list_style(StyleRefinement::default().text_color(label).min_w(px(LIST_INDENT)).pr(px(LIST_INDENT - 22.)))
        .with_blockquote_style(StyleRefinement::default().border_l(px(0.)).pl(px(14.)).text_color(muted))
        .with_table(table)
        .with_table_head(table_head)
        .with_table_cell(table_cell)
        .with_dark(cx.theme().is_dark())
}

/// `WorkLinks.escapeDestinations` and the "↑" after a work link: a space ends a
/// link's destination for the parser, and agents write `[tests pass](work:npm
/// test)`. The arrow says the link goes up the thread, not out of it, and goes
/// in the link's own words so it's coloured and selected with them.
pub(crate) fn prepare(text: &str) -> String {
    if !text.contains("](work:") && !text.contains("](thread:") {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len() + 16);
    let mut rest = text;
    loop {
        let (start, is_work) = match (rest.find("](work:"), rest.find("](thread:")) {
            (Some(w), Some(t)) if w < t => (w, true),
            (_, Some(t)) => (t, false),
            (Some(w), None) => (w, true),
            (None, None) => {
                out.push_str(rest);
                return out;
            }
        };
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        // No `)` to end it: not a link, and the rest of its line is used up.
        let end = after.find([')', '\n']).unwrap_or(after.len());
        if after[end..].starts_with(')') {
            if is_work {
                out.push('\u{2009}');
                out.push('↑');
            }
            out.push_str("](");
            out.push_str(&after[..end].replace(' ', "%20"));
            out.push(')');
            rest = &after[end + 1..];
        } else {
            out.push_str("](");
            out.push_str(&after[..end]);
            rest = &after[end..];
        }
    }
}

/// `%20` and friends back to what the link's words were.
pub(crate) fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(a), Some(b)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push(a * 16 + b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

impl ThreadPane {
    /// The kit's document for a row, kept so scrolling a row away and back
    /// doesn't parse it again, and so a streaming reply only parses what's new
    /// (`set_text` appends when the text extends what's there).
    pub(crate) fn markdown_state(&mut self, key: &str, text: &str, cx: &mut Context<Self>) -> Entity<TextViewState> {
        let state = match self.rows.documents.get(key) {
            Some(state) => state.clone(),
            None => {
                let state = cx.new(|cx| TextViewState::markdown(text, cx));
                if self.rows.documents.len() >= DOCUMENTS_KEPT {
                    // Far more than a screen's worth; the oldest are the least
                    // likely to come back soon.
                    if let Some(oldest) = self.rows.document_order.first().cloned() {
                        self.rows.documents.remove(&oldest);
                        self.rows.document_order.remove(0);
                    }
                }
                self.rows.document_order.push(key.to_owned());
                self.rows.documents.insert(key.to_owned(), state.clone());
                state
            }
        };
        state.update(cx, |s, cx| s.set_text(text, cx));
        state
    }

    /// Where a link goes. A web address or file opens outside the app (the
    /// `OpenGate` trust rules come with file previews); `work:`, `thread:` and
    /// `review:` links act here.
    pub(crate) fn link_clicked(&mut self, url: &str, row_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let (scheme, rest) = match url.split_once(':') {
            Some((s, r)) if s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-')) && !s.is_empty() => (s.to_ascii_lowercase(), r),
            _ => {
                self.open_file_url(url, window, cx);
                return;
            }
        };
        match scheme.as_str() {
            "work" => self.open_work_link(&percent_decode(rest), row_id, window, cx),
            "thread" => self.open_thread_link(&percent_decode(rest), window, cx),
            "review" => {}
            "http" | "https" | "mailto" | "file" => cx.open_url(url),
            _ => {}
        }
    }

    fn open_file_url(&mut self, path: &str, _window: &mut Window, cx: &mut Context<Self>) {
        let decoded = percent_decode(path);
        let path = decoded.strip_prefix("~/").map(|p| format!("{}/{}", threadbear_core::reducer::text::home_directory(), p)).unwrap_or(decoded);
        let absolute = if path.starts_with('/') {
            path
        } else if !self.summary.cwd.is_empty() {
            format!("{}/{}", self.summary.cwd.trim_end_matches('/'), path)
        } else {
            return;
        };
        cx.open_url(&format!("file://{absolute}"));
    }
}

/// How many row documents a thread keeps parsed.
const DOCUMENTS_KEPT: usize = 600;
