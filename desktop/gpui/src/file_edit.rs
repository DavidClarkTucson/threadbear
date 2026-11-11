// A text file's own text in the pane, a port of the editing half of the Swift
// app's TextFileView: the raw text with line numbers, which a Markdown file's
// Markdown view shows (MarkdownViewMenu). What's in the file and the bars
// over it come from threadbear_core::text_file::EditSession, which reads the
// file on this machine: only a thread on this runner has the view, as another
// Mac's file would only be a copy here.
use crate::highlights::{icon, palette, SidePane};
use crate::style::{self, px};
use gpui_kit::component::input::{Editor, EditorState};
use gpui_kit::component::{h_flex, v_flex, ActiveTheme as _};
use gpui_kit::prelude::*;
use gpui_kit::*;
use threadbear_core::text_file::EditSession;

/// The file's text in an editor of its own, and the document it came from.
pub(crate) struct FileEdit {
    pub(crate) session: EditSession,
    pub(crate) editor: Entity<EditorState>,
}

impl FileEdit {
    /// Reads `path` here and puts its text in a new editor: coloured as code
    /// when it's code (`language`), with line numbers down its side.
    pub(crate) fn open(path: &str, language: Option<&'static str>, window: &mut Window, cx: &mut Context<SidePane>) -> Self {
        let session = EditSession::open(path, true);
        let text = session.view_text(session.document.saved());
        let editor = cx.new(|cx| {
            // As AppKit's text view behaves: a tab is a tab, Return starts a
            // plain new line, and nothing is folded, guided or closed for you.
            let state = EditorState::new(window, cx)
                .soft_wrap(true)
                .line_number(true)
                .folding(false)
                .indent_guides(false)
                .auto_close(false)
                .smart_indent(false)
                .searchable(false)
                .tab_size(gpui_kit::component::input::TabSize { tab_size: 4, hard_tabs: true });
            match language {
                Some(language) => state.language(language),
                None => state,
            }
        });
        editor.update(cx, |s, cx| s.set_value(text, window, cx));
        Self { session, editor }
    }
}

impl SidePane {
    /// The raw text with what's over it: the grey note (cut off, can't be
    /// edited, gone), then the find bar.
    pub(crate) fn render_edit(&self, thread_id: &str, edit: &FileEdit, cx: &mut Context<Self>) -> AnyElement {
        let p = palette(cx);
        let note = edit.session.note.clone().map(|text| {
            // `NoteBar`: an ⓘ and the words, grey, over a hairline.
            h_flex()
                .flex_none()
                .w_full()
                .px(px(14.))
                .py(px(7.))
                .gap(px(8.))
                .items_center()
                .border_b_1()
                .border_color(p.hairline)
                .text_size(px(style::size::LABEL))
                .line_height(px(style::natural_line(style::size::LABEL)))
                .text_color(p.secondary)
                .child(icon("info.circle", style::size::LABEL, p.secondary).flex_none())
                .child(div().flex_1().min_w_0().line_clamp(2).child(text))
        });
        let find = self.render_find_bar(thread_id, cx);
        let editor = Editor::new(&edit.editor)
            // Typing comes with saving (the pencil and Done, autosave and ⌘S).
            .readonly(true)
            .h_full()
            .appearance(false)
            .bordered(false)
            .text_size(px(style::size::DETAIL))
            .line_height(px(crate::files::CODE_LINE))
            .font_family(cx.theme().mono_font_family.clone());
        // The text starts where the Swift app's does, past its 44 point
        // gutter of line numbers.
        v_flex().size_full().children(note).children(find).child(div().flex_1().min_h_0().pl(px(14.)).child(editor)).into_any_element()
    }
}
