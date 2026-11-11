// A file or page open in the pane, in place of its section: a port of the Swift
// app's PreviewBody, QuickLookView/FilePreviewView and TextFileView. What the
// file holds always comes from the runner (`files.read`), so a thread on
// another Mac works the same. A Markdown file's own text (its Markdown view)
// is the one thing read here, from a thread on this runner (file_edit.rs).
//
// A picture is drawn here with Copy Image; Markdown drawn as a page, or its
// text with the bar's Preview/Markdown menu; code and plain text in the kit's
// read-only editor, coloured by its highlighter; any text with the find bar
// over it (file_find.rs). PDFs and everything else say they open in their
// app, as this first pass has no viewer for them (`file.quicklook`), and web
// pages go to the browser (`file.htmlpreview`, `panel.web` are later work).
use crate::file_edit::FileEdit;
use crate::file_find::{FindBar, FindNext, FindPrevious};
use crate::files_store::FilePrefs;
use crate::highlights::{palette, Palette, PanePreview, SidePane, Target, FILES};
use crate::{call, icons, style};
use gpui_kit::base::{TextViewState, TextViewStyle};
use gpui_kit::component::input::{Editor, EditorState};
use gpui_kit::component::menu::{DropdownMenu, PopupMenu, PopupMenuItem};
use gpui_kit::component::{h_flex, spinner::Spinner, v_flex, ActiveTheme as _, Selectable, Sizable as _};
use gpui_kit::prelude::*;
use gpui_kit::*;
// Shadows gpui's px, so sizes here follow the app's zoom (style.rs).
use crate::style::px;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use threadbear_core::folders::{picture_size, FileRead};
use threadbear_core::highlights::{self as core, Viewer};
use threadbear_core::inbox::ThreadSummary;
use threadbear_core::text_file::{MarkdownView, LIMIT};

/// Whether the system has "Finder", for the menu's words.
#[cfg(target_os = "macos")]
pub(crate) const REVEAL_TITLE: &str = "Reveal in Finder";
#[cfg(not(target_os = "macos"))]
pub(crate) const REVEAL_TITLE: &str = "Show in folder";

pub(crate) const OPEN_IN_BROWSER: &str = "Open in Browser";

/// The bytes of a file come from the runner; this is how much a preview asks for
/// (`files.read`'s own default is 16 MB, its limit 48).
const MAX_BYTES: u64 = 32 * 1024 * 1024;

/// Past this much text, Markdown and code are shown plain: highlighting
/// a huge file line by line would stall (`TextFileController.highlightLimit`).
const HIGHLIGHT_LIMIT: usize = 400_000;

/// A line of code in the pane: the Swift text view's for its 12 point
/// monospaced font (measured, 15 apart).
pub(crate) const CODE_LINE: f32 = 15.;

/// What the pane's reading: one open at a time per thread.
pub(crate) struct OpenFile {
    pub path: String,
    pub opened: u64,
    pub body: FileBody,
    /// Only the start of the file came (`truncated`).
    pub truncated: bool,
    /// The whole file's size, as the runner said: past `LIMIT`, it's never
    /// shown as its raw text to work on (`TextFileView.canEdit`).
    pub size: i64,
    /// The find bar, while it's up (⌘F).
    pub find: Option<FindBar>,
    /// The file's own text, while the bar's menu has it showing (Markdown).
    pub edit: Option<FileEdit>,
    /// The drawn Markdown page's scroll, which the find bar moves to a match.
    pub page_scroll: ScrollHandle,
    /// Hears the drawn page change (its parse landing) for the find bar.
    _page_watch: Option<Subscription>,
}

impl OpenFile {
    fn new(path: String, opened: u64, body: FileBody) -> Self {
        OpenFile { path, opened, body, truncated: false, size: 0, find: None, edit: None, page_scroll: ScrollHandle::new(), _page_watch: None }
    }

    /// The raw text's editor, while it's showing.
    pub(crate) fn raw_editor(&self) -> Option<Entity<EditorState>> {
        self.edit.as_ref().map(|e| e.editor.clone())
    }

    /// Text this pane shows that can be worked on as text: Markdown or code
    /// that came whole and isn't too big (`TextFileView.shows`, `canEdit`).
    fn is_whole_text(&self) -> bool {
        matches!(self.body, FileBody::Markdown { .. } | FileBody::Text { .. }) && !self.truncated && self.size < LIMIT as i64
    }
}

pub(crate) struct FilesState {
    /// The file open in the pane, per thread.
    pub open: HashMap<String, OpenFile>,
    /// Paths the runner has said are folders, so a click goes to the browser.
    known_folders: HashSet<String>,
    /// The pane's own text style, built once per appearance.
    /// Keyed by appearance and zoom, as `rows`' is.
    reading_style: Option<((bool, u32), TextViewStyle)>,
    /// How files and folders show, remembered across launches.
    pub(crate) prefs: FilePrefs,
    /// What was last looked for, which the next ⌘F starts with, as AppKit's
    /// find pasteboard has it.
    pub(crate) last_query: String,
    /// The file's own focus, so ⌘F reaches the pane with nothing in it picked.
    pub(crate) focus: Option<FocusHandle>,
}

impl Default for FilesState {
    fn default() -> Self {
        FilesState { open: HashMap::new(), known_folders: HashSet::new(), reading_style: None, prefs: crate::files_store::load(), last_query: String::new(), focus: None }
    }
}

impl FilesState {
    pub(crate) fn is_known_folder(&self, path: &str) -> bool {
        self.known_folders.contains(path)
    }
}

/// What's drawn for a file.
pub(crate) enum FileBody {
    /// A spinner would only flash for a normal file; a beat later there's a note.
    Loading,
    Failed { message: String },
    Markdown { doc: Entity<TextViewState>, text: Arc<str> },
    /// Code or plain text: a read-only editor, coloured for code. The editor
    /// is made at the next frame, as it needs a window.
    Text { editor: Option<Entity<EditorState>>, text: Arc<str>, language: Option<&'static str> },
    Picture { image: Arc<Image>, size: Option<(u32, u32)> },
    /// Something only its own app can show.
    Opaque { note: String },
}

/// An image's format from its name, for the kit's decoder.
pub(crate) fn image_format(path: &str) -> Option<ImageFormat> {
    let ext = path.rsplit('.').next()?.to_lowercase();
    Some(match ext.as_str() {
        "png" => ImageFormat::Png,
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        "gif" => ImageFormat::Gif,
        "webp" => ImageFormat::Webp,
        "bmp" => ImageFormat::Bmp,
        "tif" | "tiff" => ImageFormat::Tiff,
        "svg" => ImageFormat::Svg,
        _ => return None,
    })
}

/// `%20` and the like, in a file link.
pub(crate) fn percent_decode(path: &str) -> String {
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() + 1 {
            if let Some(v) = bytes.get(i + 1..i + 3).and_then(|h| std::str::from_utf8(h).ok()).and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(crate) fn home_dir() -> String {
    // Windows sets USERPROFILE; HOME only exists there under Git Bash.
    std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_default()
}

/// A web page in the browser (the panel page is later work). Only http and
/// https: anything else is refused by the open gate, said so.
pub(crate) fn open_web(url: &str, cx: &mut App) {
    crate::open_gate::open_web(url, cx);
}

/// A file on this runner in its own app, through the open gate: a document
/// opens, a program asks first (OpenGate's rule).
pub(crate) fn open_file_in_app(path: &str, origin: crate::open_gate::Origin, cx: &mut App) {
    crate::open_gate::open_file(path, origin, cx);
}

pub(crate) fn reveal(path: &str, origin: crate::open_gate::Origin, cx: &mut App) {
    crate::open_gate::reveal(path, origin, cx);
}

/// A quiet line in the middle of the pane (`PaneNote`).
pub(crate) fn pane_note(text: &str, p: Palette) -> impl IntoElement {
    v_flex().size_full().items_center().justify_center().p(px(24.)).child(div().text_center().text_size(px(style::size::LABEL)).text_color(p.secondary).child(text.to_owned()))
}

/// An ask's words without their markdown (`plainAction`).
pub(crate) fn plain_markdown(text: &str) -> String {
    threadbear_core::sidebar::preview_plain(text)
}

/// The label of the bar's Preview/Markdown menu (`MarkdownViewMenu`): its
/// pick and a small chevron, grey until the pointer's on it, then a rounded
/// fill behind.
#[derive(IntoElement)]
pub(crate) struct ModeLabel {
    inner: Stateful<Div>,
}

impl ModeLabel {
    fn new(id: &'static str, title: &str, p: Palette) -> Self {
        let (hover, fg) = (p.hover, p.fg);
        let inner = div()
            .id(id)
            .flex_none()
            .h(px(26.))
            .px(px(7.))
            .flex()
            .items_center()
            .gap(px(3.))
            .rounded(px(6.))
            .cursor_pointer()
            .text_size(px(style::size::SMALL))
            .font_weight(FontWeight::MEDIUM)
            .text_color(p.secondary)
            .hover(move |s| s.bg(hover).text_color(fg))
            .child(title.to_owned())
            .child(icons::symbol("chevron.down").with_size(px(style::size::SMALL - 2.)));
        ModeLabel { inner }
    }
}

impl RenderOnce for ModeLabel {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        self.inner
    }
}

impl Styled for ModeLabel {
    fn style(&mut self) -> &mut StyleRefinement {
        self.inner.style()
    }
}

impl InteractiveElement for ModeLabel {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.inner.interactivity()
    }
}

impl Selectable for ModeLabel {
    fn selected(self, _: bool) -> Self {
        self
    }
    fn is_selected(&self) -> bool {
        false
    }
}

impl DropdownMenu for ModeLabel {}

impl SidePane {
    /// Reads the file the pane's preview names, if it's one that shows here.
    pub(crate) fn begin_preview(&mut self, thread: &ThreadSummary, cx: &mut Context<Self>) {
        let Some(preview) = self.previews.get(&thread.id).cloned() else { return };
        match &preview.target {
            Target::File(path) => match core::viewer_for_path(path) {
                // A page in the system browser for now, and the preview
                // closes. Another Mac's file has no copy here to open.
                Viewer::Web if thread.is_remote() => {
                    let note = "Web pages from another Mac can't be opened from here yet.".to_owned();
                    self.files.open.insert(thread.id.clone(), OpenFile::new(path.clone(), preview.opened, FileBody::Opaque { note }));
                    cx.notify();
                }
                // The real file, after its symlinks: a page named .html that's
                // really something else asks first, as any file does.
                Viewer::Web => {
                    open_file_in_app(path, crate::open_gate::Origin::ThisMac, cx);
                    self.previews.remove(&thread.id);
                    cx.notify();
                }
                _ => self.load_file(thread, path.clone(), preview.opened, cx),
            },
            Target::Page(url) => {
                open_web(url, cx);
                self.previews.remove(&thread.id);
                cx.notify();
            }
            Target::Folder(_) => self.load_folder_for_preview(thread, cx),
        }
    }

    fn load_file(&mut self, thread: &ThreadSummary, path: String, opened: u64, cx: &mut Context<Self>) {
        // The same open again (a repaint, a re-select) reads nothing new;
        // opening it again from a row does (`opened` moved).
        if self.files.open.get(&thread.id).is_some_and(|f| f.path == path && f.opened == opened) {
            return;
        }
        let keep_previous = self.files.open.get(&thread.id).is_some_and(|f| f.path == path);
        if !keep_previous {
            self.files.open.insert(thread.id.clone(), OpenFile::new(path.clone(), opened, FileBody::Loading));
            cx.notify();
        }
        let client = self.client.clone();
        let thread_id = thread.id.clone();
        let mut params = json!({ "path": path, "maxBytes": MAX_BYTES });
        if let Some(runner) = &thread.runner_id {
            params["runnerId"] = json!(runner);
        }
        cx.spawn(async move |this, cx| {
            let res = call(client, "files.read", params).await;
            this.update(cx, |this, cx| {
                // A newer file opened while this one was on its way.
                let current = this.previews.get(&thread_id).and_then(|p| p.file_path().map(str::to_owned));
                if current.as_deref() != Some(path.as_str()) {
                    return;
                }
                match res {
                    Ok(v) => match FileRead::decode(&v) {
                        Some(read) => this.show_read(&thread_id, &path, opened, read, cx),
                        None => this.fail_file(&thread_id, &path, opened, "Couldn't read that answer from the runner.".into(), cx),
                    },
                    Err(e) if e.message.starts_with("Not a file") => {
                        // A folder: into the browser, as a click on one is.
                        this.files.known_folders.insert(path.clone());
                        this.files.open.remove(&thread_id);
                        if let Some(p) = this.previews.get_mut(&thread_id) {
                            p.target = Target::Folder(path.clone());
                        }
                        if let Some(thread) = this.thread(cx) {
                            this.begin_preview(&thread, cx);
                        }
                        cx.notify();
                    }
                    Err(e) => this.fail_file(&thread_id, &path, opened, e.message, cx),
                }
            })
            .ok();
        })
        .detach();
    }

    fn fail_file(&mut self, thread_id: &str, path: &str, opened: u64, message: String, cx: &mut Context<Self>) {
        self.files.open.insert(thread_id.to_owned(), OpenFile::new(path.to_owned(), opened, FileBody::Failed { message }));
        cx.notify();
    }

    fn show_read(&mut self, thread_id: &str, path: &str, opened: u64, read: FileRead, cx: &mut Context<Self>) {
        let viewer = core::viewer_for_path(path);
        let is_picture = viewer == Viewer::Picture && (core::looks_like_picture(path) || image_format(path).is_some());
        let body = if is_picture {
            match image_format(path) {
                Some(format) => {
                    FileBody::Picture { size: picture_size(&read.data), image: Arc::new(Image::from_bytes(format, read.data)) }
                }
                None => FileBody::Opaque { note: "This kind of picture opens in its own app.".into() },
            }
        } else {
            let text_ok = viewer == Viewer::Text || core::looks_like_text(&read.data);
            let opaque = viewer != Viewer::Text && !core::looks_like_text(&read.data);
            if opaque || !text_ok {
                FileBody::Opaque { note: "This file opens in its own app.".into() }
            } else {
                match String::from_utf8(read.data) {
                    Err(_) => FileBody::Opaque { note: "This file isn't text this app can read.".into() },
                    Ok(text) => self.text_body(path, text, cx),
                }
            }
        };
        let page_watch = match &body {
            // The find bar searches what the page draws, which lands after
            // its parse: it searches again when the page changes.
            FileBody::Markdown { doc, .. } => {
                let id = thread_id.to_owned();
                Some(cx.observe(doc, move |this, _, cx| this.page_changed(&id, cx)))
            }
            _ => None,
        };
        // A file read again (opened again from a row) keeps its find bar, as
        // the Swift app's view does. Its own text is read again too, when
        // it's next drawn.
        let mut open = OpenFile::new(path.to_owned(), opened, body);
        if let Some(previous) = self.files.open.remove(thread_id).filter(|f| f.path == path) {
            open.find = previous.find;
            open.page_scroll = previous.page_scroll;
        }
        open.truncated = read.truncated;
        open.size = read.size;
        open._page_watch = page_watch;
        self.files.open.insert(thread_id.to_owned(), open);
        // The new text under an open find bar.
        if self.files.open.get(thread_id).is_some_and(|o| o.find.is_some()) {
            self.find_changed(thread_id, cx);
        }
        cx.notify();
    }

    /// Markdown drawn, code coloured, text plain, all of it selectable.
    fn text_body(&mut self, path: &str, text: String, cx: &mut Context<Self>) -> FileBody {
        let text: Arc<str> = text.into();
        if core::is_markdown(path) {
            let source = text.to_string();
            let doc = cx.new(|cx| TextViewState::markdown(&source, cx));
            return FileBody::Markdown { doc, text };
        }
        let language = core::code_language(path).filter(|_| text.len() < HIGHLIGHT_LIMIT);
        // The editor is made at the next frame, with a window (render_file).
        FileBody::Text { editor: None, text, language }
    }

    /// A previewed Markdown file, its links going through the open gate:
    /// a file it links to (from the file's own folder) previews here too,
    /// read through the runner, so a repo's page can't hand the system one
    /// of its programs; a web page opens; anything else asks.
    fn preview_text(&self, doc: &Entity<TextViewState>, thread: &ThreadSummary, path: &str, cx: &mut Context<Self>) -> gpui_kit::base::TextView {
        use threadbear_core::open_policy::{self as policy, LinkKind};
        let origin = if thread.is_remote() { crate::open_gate::Origin::OtherMac } else { crate::open_gate::Origin::ThisMac };
        let folder = path.rsplit_once(['/', '\\']).map_or(String::new(), |(dir, _)| dir.to_owned());
        let (side, thread) = (cx.entity().downgrade(), thread.clone());
        let base = folder.clone();
        crate::open_gate::text_view(doc, origin, &folder).on_link_click(move |url: &SharedString, _: &ClickEvent, _: &mut Window, cx: &mut App| match policy::link_kind(url) {
            LinkKind::File(target) => {
                if let Some(target) = policy::absolute(&target, &base, &home_dir()) {
                    let _ = side.update(cx, |side, cx| side.preview_path(&target, None, &thread, cx));
                }
            }
            _ => crate::open_gate::open_link(url, origin, &base, cx),
        })
    }

    /// Whether the bar offers the file's own text (the Markdown menu): a
    /// whole text file on this runner's machine. Another Mac's would only be
    /// a copy here (`PreviewButtons`, `preview.away == nil`).
    fn offers_text(&self, preview: &PanePreview, thread: &ThreadSummary) -> bool {
        let Some(path) = preview.file_path() else { return false };
        !thread.is_remote() && self.files.open.get(&thread.id).is_some_and(|f| f.path == path && f.is_whole_text())
    }

    /// The bar's buttons over a preview: the Markdown menu, Reveal, Open
    /// (`PreviewButtons`).
    pub(crate) fn preview_buttons(&mut self, preview: &PanePreview, thread: &ThreadSummary, p: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let mut row = h_flex().flex_none().gap(px(0.));
        if let Target::Folder(path) = &preview.target {
            row = row.child(self.folder_mode_button(p, cx));
            // Another Mac's folder has no copy here to hand over.
            if !thread.is_remote() {
                let open_path = path.clone();
                row = row.child(self.header_button("hl-open", "arrow.up.right", "Open in Finder".to_owned(), p, cx, move |this, cx| this.open_externally(&open_path.clone(), cx)));
            }
        }
        if let Some(path) = preview.file_path().filter(|path| core::is_markdown(path) && self.offers_text(preview, thread)) {
            let _ = path;
            row = row.child(self.markdown_menu(preview.editing, p, cx));
        }
        if let Some(path) = preview.file_path().filter(|_| !thread.is_remote()).map(str::to_owned) {
            let reveal_path = path.clone();
            row = row.child(self.header_button("hl-reveal", "folder", REVEAL_TITLE.to_owned(), p, cx, move |_, cx| reveal(&reveal_path.clone(), crate::open_gate::Origin::ThisMac, cx)));
            let open_path = path.clone();
            row = row.child(self.header_button("hl-open", "arrow.up.right", "Open in its app".to_owned(), p, cx, move |this, cx| this.open_externally(&open_path.clone(), cx)));
        }
        row
    }

    /// `MarkdownViewMenu`: Preview or Markdown, the pick remembered for the
    /// next Markdown file.
    fn markdown_menu(&self, editing: bool, p: Palette, cx: &mut Context<Self>) -> AnyElement {
        let showing = if editing { MarkdownView::Markdown } else { MarkdownView::Preview };
        // The text doesn't take typing yet (`file.editor`), so its tip
        // doesn't say it's ready for it.
        let tip = match showing {
            MarkdownView::Markdown => "Showing the Markdown text. Pick Preview to read it as a page.",
            MarkdownView::Preview => showing.tip(),
        };
        let side = cx.entity().downgrade();
        let holder = crate::highlights::tipped(div().id("hl-markdown-view").flex_none(), tip);
        let menu = ModeLabel::new("hl-markdown-view-label", showing.title(), p).dropdown_menu(move |menu: PopupMenu, _, _| {
            let mut menu = menu;
            for view in MarkdownView::ALL {
                let side = side.clone();
                let item = PopupMenuItem::new(view.title()).checked(view == showing).icon(icons::symbol(view.symbol())).on_click(move |_, window, cx| {
                    side.update(cx, |this, cx| this.pick_markdown_view(view, window, cx)).ok();
                });
                menu = menu.item(item);
            }
            menu
        });
        holder.child(menu).into_any_element()
    }

    /// A pick from the Markdown menu: this file shows that way, and so does
    /// the next one opened (`MarkdownView.current`).
    pub(crate) fn pick_markdown_view(&mut self, view: MarkdownView, window: &mut Window, cx: &mut Context<Self>) {
        self.files.prefs.markdown_view = view;
        crate::files_store::save(&self.files.prefs);
        let Some(thread) = self.thread(cx) else { return };
        self.set_preview_editing(&thread, view == MarkdownView::Markdown, window, cx);
    }

    /// The file's own text, shown or put away (the Markdown menu's pick).
    pub(crate) fn set_preview_editing(&mut self, thread: &ThreadSummary, editing: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(preview) = self.previews.get_mut(&thread.id) else { return };
        if preview.file_path().is_none() || preview.editing == editing {
            return;
        }
        preview.editing = editing;
        let had_find = self.files.open.get(&thread.id).is_some_and(|o| o.find.is_some());
        if let Some(open) = self.files.open.get_mut(&thread.id) {
            if !editing {
                open.edit = None;
            }
        }
        self.ensure_edit(thread, window, cx);
        // The find bar stays, searching what shows now.
        if had_find {
            self.find_changed(&thread.id, cx);
        }
        cx.notify();
    }

    /// The file's own text for a preview that's showing it, made once there's
    /// a window to make its editor with.
    fn ensure_edit(&mut self, thread: &ThreadSummary, window: &mut Window, cx: &mut Context<Self>) {
        let Some(preview) = self.previews.get(&thread.id).cloned() else { return };
        if !preview.editing || !self.offers_text(&preview, thread) {
            return;
        }
        let Some(open) = self.files.open.get(&thread.id) else { return };
        if open.edit.is_some() {
            return;
        }
        let path = open.path.clone();
        let language = core::code_language(&path).filter(|_| open.size < HIGHLIGHT_LIMIT as i64);
        let edit = FileEdit::open(&path, language, window, cx);
        if let Some(open) = self.files.open.get_mut(&thread.id) {
            open.edit = Some(edit);
        }
    }

    pub(crate) fn render_preview(&mut self, thread: &ThreadSummary, preview: &PanePreview, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let p = palette(cx);
        match &preview.target {
            Target::File(path) => self.render_file(thread, path, preview.editing, window, cx),
            Target::Page(url) => pane_note(&format!("{url} opened in your browser."), p).into_any_element(),
            Target::Folder(path) => self.render_folder(thread, path, preview, window, cx),
        }
    }

    fn render_file(&mut self, thread: &ThreadSummary, path: &str, editing: bool, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let p = palette(cx);
        // A file's text that's waiting for its editor gets one now.
        if let Some(open) = self.files.open.get_mut(&thread.id).filter(|f| f.path == path) {
            if let FileBody::Text { editor, text, language } = &mut open.body {
                if editor.is_none() {
                    let (value, language) = (text.to_string(), *language);
                    let made = cx.new(|cx| {
                        // Reading, as the Swift app's text view is until its
                        // pencil: no line numbers, folds or guides, and ⌘F
                        // left to the pane's own find bar.
                        let state = EditorState::new(window, cx).soft_wrap(true).line_number(false).folding(false).indent_guides(false).searchable(false);
                        match language {
                            Some(language) => state.language(language),
                            None => state,
                        }
                    });
                    made.update(cx, |state, cx| state.set_value(value, window, cx));
                    *editor = Some(made);
                }
            }
        }
        if editing {
            self.ensure_edit(thread, window, cx);
        }
        let focus = self.files.focus.get_or_insert_with(|| cx.focus_handle()).clone();
        let Some(open) = self.files.open.get(&thread.id).filter(|f| f.path == path) else {
            return div().size_full().into_any_element();
        };
        let thread_id = thread.id.clone();
        if let Some(edit) = open.edit.as_ref().filter(|_| editing) {
            let body = self.render_edit(&thread_id, edit, cx);
            return self.file_keys(div().id("hl-file").size_full().track_focus(&focus).child(body), &thread_id, cx).into_any_element();
        }
        let cut = open.truncated.then(|| div().flex_none().w_full().py(px(6.)).text_center().text_size(px(style::size::SMALL)).text_color(p.secondary).child("Only the start of this file came from the runner."));
        let body: AnyElement = match &open.body {
            // A spinner so a click shows something at once; the header already
            // says which file it's getting.
            FileBody::Loading => div().size_full().flex().items_center().justify_center().child(Spinner::new().with_size(px(14.)).color(p.secondary)).into_any_element(),
            FileBody::Failed { message } => pane_note(&format!("Couldn't read it: {message}"), p).into_any_element(),
            FileBody::Opaque { note } => pane_note(note, p).into_any_element(),
            FileBody::Picture { image, .. } => {
                let source = ImageSource::Image(image.clone());
                let (path2, entity) = (path.to_owned(), cx.entity().downgrade());
                div()
                    .id("hl-picture")
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .p(px(0.))
                    .on_mouse_down(MouseButton::Right, move |event, window, cx| {
                        // Copy Image, the picture's own right-click (`FilePreviewView.menu`).
                        let position = event.position;
                        let path3 = path2.clone();
                        entity
                            .update(cx, |this, cx| this.open_picture_menu(path3, position, window, cx))
                            .ok();
                    })
                    .child(img(source).size_full().object_fit(ObjectFit::Contain))
                    .into_any_element()
            }
            FileBody::Markdown { doc, .. } => {
                let key = (cx.theme().is_dark(), crate::style::zoom_percent());
                let style = match &self.files.reading_style {
                    Some((was, style)) if *was == key => style.clone(),
                    _ => {
                        let built = crate::rows::reading_style(cx);
                        self.files.reading_style = Some((key, built.clone()));
                        built
                    }
                };
                let scroll = open.page_scroll.clone();
                let reveal = scroll.clone();
                // A page, with margins and lines no longer than the Swift
                // app's pageWidth however wide the panel is.
                div()
                    .id("hl-markdown")
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .child(
                        div()
                            .max_w(px(720.))
                            .mx_auto()
                            .px(px(24.))
                            .py(px(18.))
                            .text_size(px(style::size::BODY))
                            .line_height(px(style::size::BODY_LINE))
                            .child(self.preview_text(doc, thread, path, cx).selectable(true).style(style).on_reveal(move |line, _, _| {
                                // The find bar's match, scrolled into the pane.
                                let viewport = reveal.bounds();
                                let mut offset = reveal.offset();
                                if line.bottom() > viewport.bottom() {
                                    offset.y -= line.bottom() - viewport.bottom();
                                } else if line.top() < viewport.top() {
                                    offset.y += viewport.top() - line.top();
                                }
                                reveal.set_offset(offset);
                            })),
                    )
                    .into_any_element()
            }
            FileBody::Text { editor: Some(editor), .. } => div()
                .size_full()
                .child(
                    Editor::new(editor)
                        .readonly(true)
                        .h_full()
                        .appearance(false)
                        .bordered(false)
                        .text_size(px(style::size::DETAIL))
                        // AppKit's line for the 12 point monospaced font, not the kit's 18.
                        .line_height(px(CODE_LINE))
                        .font_family(cx.theme().mono_font_family.clone()),
                )
                .into_any_element(),
            FileBody::Text { .. } => div().size_full().into_any_element(),
        };
        let find = self.render_find_bar(&thread_id, cx);
        let column = v_flex().size_full().when_some(cut, |s, c| s.child(c)).children(find).child(div().flex_1().min_h_0().child(body));
        self.file_keys(div().id("hl-file").size_full().track_focus(&focus).child(column), &thread_id, cx).into_any_element()
    }

    /// The keys a file in the pane takes: ⌘F, ⌘G and ⇧⌘G for the find bar.
    fn file_keys(&self, el: Stateful<Div>, thread_id: &str, cx: &mut Context<Self>) -> Stateful<Div> {
        let (find, next, previous) = (thread_id.to_owned(), thread_id.to_owned(), thread_id.to_owned());
        el.key_context(crate::file_find::CONTEXT)
            .on_action(cx.listener(move |this, _: &gpui_kit::component::input::Search, window, cx| this.open_find(&find, window, cx)))
            .on_action(cx.listener(move |this, _: &FindNext, window, cx| this.find_again(&next, false, window, cx)))
            .on_action(cx.listener(move |this, _: &FindPrevious, window, cx| this.find_again(&previous, true, window, cx)))
    }

    /// `Copy Image`: the file's own bytes, not a path, so it pastes as the picture.
    fn open_picture_menu(&mut self, _path: String, position: Point<Pixels>, window: &mut Window, cx: &mut Context<Self>) {
        let items = vec![crate::highlights::MenuItem::picture_copy()];
        self.open_menu(items, position, window, cx);
    }

    /// Puts the picture on the clipboard as the file's own bytes (a PNG stays a
    /// PNG), which every app that takes a pasted picture reads (`copyImage`).
    pub(crate) fn copy_open_picture(&mut self, cx: &mut Context<Self>) -> Option<String> {
        let thread = self.thread(cx)?;
        let open = self.files.open.get(&thread.id)?;
        let FileBody::Picture { image, .. } = &open.body else { return None };
        let format = match image.format() {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpeg",
            ImageFormat::Gif => "gif",
            ImageFormat::Webp => "webp",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Tiff => "tiff",
            ImageFormat::Svg => "svg",
            _ => "image",
        };
        cx.write_to_clipboard(ClipboardItem::new_image(image));
        Some(format.to_owned())
    }

    /// `dump`'s preview line (`Panels.preview`).
    #[cfg(feature = "automation")]
    pub(crate) fn preview_line(&self, preview: &PanePreview) -> String {
        match &preview.target {
            Target::File(path) => format!("file {path}{}", if preview.editing { " (editing)" } else { "" }),
            Target::Folder(path) => format!("folder {path}"),
            Target::Page(url) => format!("page {url}"),
        }
    }

    #[cfg(feature = "automation")]
    pub(crate) fn fill_preview_dump(&self, pane: &mut serde_json::Map<String, Value>, preview: &PanePreview, cx: &App) {
        match &preview.target {
            Target::Page(url) => {
                pane.insert("previewKind".into(), json!("page"));
                pane.insert("previewText".into(), json!(url));
            }
            Target::Folder(path) => {
                pane.insert("previewKind".into(), json!("folder"));
                pane.insert("previewText".into(), json!(path));
            }
            Target::File(path) => {
                let Some(thread) = self.thread(cx) else { return };
                let Some(open) = self.files.open.get(&thread.id).filter(|f| &f.path == path) else { return };
                if let Some(find) = self.find_line(&thread.id, cx) {
                    pane.insert("previewFind".into(), json!(find));
                }
                // What the bar offers: the Markdown menu (PreviewButtons).
                if core::is_markdown(path) && self.offers_text(preview, &thread) {
                    pane.insert("previewMode".into(), json!(if preview.editing { "markdown" } else { "preview" }));
                }
                if let Some(edit) = open.edit.as_ref().filter(|_| preview.editing) {
                    pane.insert("previewKind".into(), json!(if core::is_markdown(path) || core::code_language(path).is_none() { "text" } else { "code" }));
                    pane.insert("previewText".into(), json!(edit.editor.read(cx).value().chars().take(4000).collect::<String>()));
                    if let Some(note) = &edit.session.note {
                        pane.insert("previewNote".into(), json!(note));
                    }
                    return;
                }
                match &open.body {
                    FileBody::Loading => {}
                    FileBody::Failed { message } => {
                        pane.insert("previewKind".into(), json!("note"));
                        pane.insert("previewText".into(), json!(format!("Couldn't read it: {message}")));
                    }
                    FileBody::Opaque { note } => {
                        pane.insert("previewKind".into(), json!("note"));
                        pane.insert("previewText".into(), json!(note));
                    }
                    FileBody::Picture { size, .. } => {
                        pane.insert("previewKind".into(), json!("picture"));
                        pane.insert("previewText".into(), json!(size.map(|(w, h)| format!("{w}x{h}"))));
                    }
                    FileBody::Markdown { text, .. } => {
                        pane.insert("previewKind".into(), json!("markdown"));
                        pane.insert("previewText".into(), json!(text.chars().take(4000).collect::<String>()));
                    }
                    FileBody::Text { text, language, .. } => {
                        pane.insert("previewKind".into(), json!(if language.is_some() { "code" } else { "text" }));
                        pane.insert("previewText".into(), json!(text.chars().take(4000).collect::<String>()));
                    }
                }
            }
        }
    }
}

/// The `preview` automation command (`Automation.swift`). Over a text file,
/// "find" is ⌘F with those words typed in the find bar, "findNext" and
/// "findPrevious" are ⌘G and ⇧⌘G, "findDone" the bar's Done, each saying what
/// the bar has found; "markdownView" is a pick from a Markdown file's menu.
#[cfg(feature = "automation")]
pub(crate) fn preview_command(side: &Entity<SidePane>, thread: &ThreadSummary, cmd: &Value, window: &mut Window, cx: &mut App) -> Result<Value, String> {
    if cmd["copyImage"].as_bool() == Some(true) {
        let format = side.update(cx, |pane, cx| pane.copy_open_picture(cx)).ok_or("no picture is showing")?;
        return Ok(json!(format!("ok copied {format}")));
    }
    if ["find", "findNext", "findPrevious", "findDone"].iter().any(|k| !cmd[*k].is_null()) {
        let id = thread.id.clone();
        let showing = side.read(cx).files.open.get(&id).is_some_and(|o| matches!(o.body, FileBody::Markdown { .. } | FileBody::Text { .. }) || o.edit.is_some());
        if !showing {
            return Err("no text file is showing".into());
        }
        side.update(cx, |pane, cx| {
            if let Some(query) = cmd["find"].as_str() {
                pane.find_typed(&id, query, window, cx);
            }
            if cmd["findNext"].as_bool() == Some(true) {
                pane.find_again(&id, false, window, cx);
            }
            if cmd["findPrevious"].as_bool() == Some(true) {
                pane.find_again(&id, true, window, cx);
            }
            if cmd["findDone"].as_bool() == Some(true) {
                pane.close_find(&id, window, cx);
            }
        });
        let line = side.read(cx).find_line(&id, cx).unwrap_or_else(|| "no find bar".to_owned());
        return Ok(json!(format!("ok {line}")));
    }
    if let Some(name) = cmd["markdownView"].as_str() {
        let view = MarkdownView::from_name(name).ok_or("markdownView is preview or markdown")?;
        let is_markdown = side.read(cx).previews.get(&thread.id).and_then(|p| p.file_path().map(core::is_markdown)).unwrap_or(false);
        if !is_markdown {
            return Err("no Markdown file is open in the pane".into());
        }
        side.update(cx, |pane, cx| pane.pick_markdown_view(view, window, cx));
    } else if cmd["close"].as_bool() == Some(true) {
        side.update(cx, |pane, cx| pane.close_preview(&thread.id, cx));
    } else if cmd["back"].as_bool() == Some(true) {
        side.update(cx, |pane, cx| pane.preview_back(&thread.id, cx));
    } else if let Some(path) = cmd["path"].as_str() {
        side.update(cx, |pane, cx| pane.preview_path(path, None, thread, cx));
    } else if let Some(url) = cmd["url"].as_str() {
        side.update(cx, |pane, cx| {
            let section = pane.section_for(thread);
            pane.show_preview(PanePreview::new(Target::Page(url.to_owned()), core::text::address(url), section), &thread.id, cx);
        });
    } else {
        return Err("preview needs path, url or close".into());
    }
    let shown = side.read(cx).previews.get(&thread.id).map_or_else(|| "no preview".to_owned(), |p| format!("showing {}", p.title));
    Ok(json!(format!("ok {shown}")))
}

/// The Files panel has no preview stored: its section shows the thread's folder.
pub(crate) fn files_section_preview(thread: &ThreadSummary) -> PanePreview {
    PanePreview::new(Target::Folder(thread.cwd.clone()), "Files".into(), FILES.to_owned())
}
