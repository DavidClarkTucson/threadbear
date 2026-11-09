// The message box under a thread: a port of the Swift app's ComposerView.swift
// and PictureChips.swift, and the send button from ComposerControls.swift. One
// Composer per thread lives on the app (Threadbear.composers), so what's typed,
// its undo and its pictures stay with the thread while others are open.
//
// Pictures are real inline tokens in gpui-kit's text input (InlineToken): one
// U+FFFC mark per chip, the chip drawn by the renderer below, and the input
// keeps them atomic through edits and undo. That mark is exactly what the
// Swift draft holds (InlinePictures), so the two apps' drafts read the same.
use crate::app::Threadbear;
use crate::style;
use crate::thread::ThreadPane;
use gpui_kit::assets::IconName;
use gpui_kit::component::input::{Enter, InlineToken, InlineTokenContext, InlineTokenHoverEvent, InputContent, InputEvent, Textarea, TextareaState};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _};
use gpui_kit::prelude::*;
use gpui_kit::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use threadbear_client::runner::Client;
use threadbear_core::composer::{self, BOX_MAX_LINES, BOX_MIN_LINES, PICTURE_MARK, PICTURE_MARK_STR};
use threadbear_core::reducer::ImageRef;

/// A picture in the box: the token that draws its chip, what it came from,
/// and what the runner made of it (`media.import`).
struct PictureChip {
    /// The file it came from, for the thumbnail meanwhile.
    source: PathBuf,
    /// None while the runner is still copying it in.
    image: Option<ImageRef>,
}

pub struct Composer {
    /// None for the new-thread page's box.
    pub(crate) thread_id: Option<String>,
    pub(crate) client: Arc<Client>,
    pub(crate) app: WeakEntity<Threadbear>,
    /// The open thread's pane, once it has one; only ever read, or asked to
    /// send, from here (the pane never updates this composer back from those
    /// same calls, so no double lease).
    pub(crate) pane: WeakEntity<ThreadPane>,
    pub(crate) input: Entity<TextareaState>,
    /// The chips in the box, by token id, in no particular order: the box's
    /// tokens say the order (`chips`).
    pictures: HashMap<SharedString, PictureChip>,
    next_picture: u64,
    /// Each chip's place among the box's pictures, worked out each render so
    /// the renderer (which can't edit the input) can read it.
    chip_numbers: HashMap<SharedString, usize>,
    /// The chip the pointer is over, and the card showing it after a beat.
    hovered_chip: Option<(SharedString, Bounds<Pixels>)>,
    peek_shown: Option<(SharedString, Bounds<Pixels>)>,
    hover_serial: u64,
    /// Stop was pressed and the turn hasn't ended yet: a spinner in its place.
    stopping: bool,
    /// A refused send's draft, to put back at the next render.
    restore: Option<(String, Vec<ImageRef>)>,
    _subs: Vec<Subscription>,
}

impl Composer {
    pub fn new(thread_id: Option<String>, client: Arc<Client>, app: WeakEntity<Threadbear>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Two lines at least, ten at most, then it scrolls (ComposerChips.boxHeight).
        let input = cx.new(|cx| TextareaState::new(window, cx).placeholder("Message").auto_grow(BOX_MIN_LINES, BOX_MAX_LINES).submit_on_enter(true));
        let sub = cx.subscribe_in(&input, window, |this, _, ev: &InputEvent, window, cx| match ev {
            InputEvent::PressEnter { shift: false, .. } => this.return_pressed(window, cx),
            // The send button and the picture strip follow what's in the box.
            InputEvent::Change => cx.notify(),
            _ => {}
        });
        Composer {
            thread_id,
            client,
            app,
            pane: WeakEntity::new_invalid(),
            input,
            pictures: HashMap::new(),
            next_picture: 0,
            chip_numbers: HashMap::new(),
            hovered_chip: None,
            peek_shown: None,
            hover_serial: 0,
            stopping: false,
            restore: None,
            _subs: vec![sub],
        }
    }

    pub fn attach(&mut self, pane: WeakEntity<ThreadPane>) {
        self.pane = pane;
    }

    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        self.input.update(cx, |s, cx| s.focus(window, cx));
    }

    /// What the box holds as a draft (ComposerTextView.draftNow): the words,
    /// a mark for each chip whose picture is in, and those pictures in order.
    /// A chip still being copied in isn't part of it yet, nor is a stray
    /// U+FFFC pasted as text.
    fn draft(&self, cx: &App) -> (String, Vec<ImageRef>) {
        let content = self.input.read(cx).content();
        let text = content.text().to_string();
        let mut out = String::new();
        let mut images = Vec::new();
        let mut last = 0;
        for span in content.tokens() {
            let range = span.range();
            out.push_str(&composer::draft_words(&text[last..range.start]));
            if let Some(PictureChip { image: Some(image), .. }) = self.pictures.get(span.token().id()) {
                out.push(PICTURE_MARK);
                images.push(image.clone());
            }
            last = range.end;
        }
        out.push_str(&composer::draft_words(&text[last..]));
        (out, images)
    }

    /// The box's text as the Swift dump gives it: chips as the markers they'll
    /// go out as (`InlinePictures.messageText`).
    pub fn draft_text(&self, cx: &App) -> String {
        composer::message_text(&self.draft(cx).0, 1)
    }

    /// How many pictures are in the box (the dump's `composer.images`).
    pub fn draft_images(&self, cx: &App) -> usize {
        self.draft(cx).1.len()
    }

    fn pane(&self) -> Option<Entity<ThreadPane>> {
        self.pane.upgrade()
    }

    // ---- Keys ----

    /// Return in the box (ComposerTextView.keyDown): with nothing typed it
    /// goes to `return_when_empty`, otherwise it sends.
    fn return_pressed(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.read(cx).value().is_empty() && self.return_when_empty(cx) {
            return;
        }
        self.send(window, cx);
    }

    /// ComposerView.returnWhenEmpty: Return with nothing typed allows the
    /// sole waiting approval (the ⏎ its Allow button shows).
    fn return_when_empty(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(pane) = self.pane() else { return false };
        pane.update(cx, |p, cx| p.allow_sole_approval(cx))
    }

    /// Shift-Return, before the input takes it: a markdown list carries on
    /// (ComposerTextView.insertLineContinuingList), through one edit so it's
    /// one undo.
    fn shift_return(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let (text, selection) = {
            let state = self.input.read(cx);
            (state.value().to_string(), state.selected_range())
        };
        let Some(edit) = composer::list_newline(&text, selection.clone()) else { return false };
        self.input.update(cx, |s, cx| {
            if edit.range != selection {
                // An item with nothing in it: the marker comes off.
                s.set_selected_range(edit.range.clone(), cx);
            }
            s.replace(edit.text.clone(), window, cx);
        });
        true
    }

    // ---- Sending ----

    /// Whether there's anything to send (ComposerView.canSend).
    fn can_send(&self, cx: &App) -> bool {
        let (text, images) = self.draft(cx);
        composer::can_send(&text, images.len())
    }

    /// AppModel.send: the box empties at once and the message shows dimmed
    /// at the end of the transcript until the runner has it; it comes back
    /// if the runner doesn't take it.
    pub(crate) fn send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (draft_text, images) = self.draft(cx);
        let text = composer::sent_text(&draft_text, 1);
        if text.trim().is_empty() && images.is_empty() {
            return;
        }
        let Some(pane) = self.pane() else { return };
        self.input.update(cx, |s, cx| s.set_value("", window, cx));
        self.pictures.clear();
        let composer = cx.entity().downgrade();
        pane.update(cx, |p, cx| p.send_message(text.trim().to_string(), images.clone(), Value::Null, Some((draft_text.clone(), images.clone())), composer, cx));
        cx.notify();
    }

    /// The Stop button (⌘. goes through ThreadPane::stop too).
    fn stop(&mut self, cx: &mut Context<Self>) {
        self.stopping = true;
        if let Some(pane) = self.pane() {
            pane.update(cx, |p, cx| p.stop(cx));
        }
        cx.notify();
    }

    /// A send the runner refused: what was typed goes back in the box
    /// (AppModel.restoreDraft), unless something new has been typed since.
    /// At the next render, where there's a window to do it with.
    pub fn restore_draft(&mut self, draft: String, images: Vec<ImageRef>, cx: &mut Context<Self>) {
        self.restore = Some((draft, images));
        cx.notify();
    }

    fn apply_restore(&mut self, draft: String, images: Vec<ImageRef>, window: &mut Window, cx: &mut Context<Self>) {
        if !self.input.read(cx).value().is_empty() {
            return;
        }
        // Chips again where the marks are, each picture back in place.
        let mut content = InputContent::new(draft.clone());
        let mut images = images.into_iter();
        for (start, _) in draft.match_indices(PICTURE_MARK) {
            let Some(image) = images.next() else { break };
            let id = self.fresh_picture_id();
            let token = picture_token(&id);
            content = match content.clone().with_token(start..start + PICTURE_MARK.len_utf8(), token) {
                Ok(content) => content,
                Err(_) => content,
            };
            self.pictures.insert(id, PictureChip { source: PathBuf::from(&image.path), image: Some(image) });
        }
        self.input.update(cx, |s, cx| s.set_value(content, window, cx));
    }

    fn fresh_picture_id(&mut self) -> SharedString {
        self.next_picture += 1;
        format!("picture-{}", self.next_picture).into()
    }

    // ---- Pictures ----

    /// Paste: pictures and files first, as ComposerTextView.takeImages takes
    /// them. Returns whether it took the paste; words paste as words.
    pub(crate) fn take_clipboard(&mut self, item: &ClipboardItem, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let mut paths = Vec::new();
        let mut images = Vec::new();
        let mut has_text = false;
        for entry in item.entries() {
            match entry {
                ClipboardEntry::ExternalPaths(p) => paths.extend(p.paths().iter().cloned()),
                ClipboardEntry::Image(image) => images.push(image.clone()),
                ClipboardEntry::String(_) => has_text = true,
            }
        }
        if !paths.is_empty() {
            self.take_paths(paths, window, cx);
            return true;
        }
        // Text with an image flavour (rich text from a browser) stays text.
        if has_text || images.is_empty() {
            return false;
        }
        let saved: Vec<PathBuf> = images.iter().filter_map(save_pasted_image).collect();
        if saved.is_empty() {
            return false;
        }
        self.insert_pictures(saved, window, cx);
        true
    }

    /// Files pasted or dropped: pictures become chips where the cursor is, and
    /// anything else goes in as its full path in backticks, for the agent to
    /// open where it is (ComposerTextView.takeImages).
    pub(crate) fn take_paths(&mut self, paths: Vec<PathBuf>, window: &mut Window, cx: &mut Context<Self>) {
        let (images, others): (Vec<_>, Vec<_>) = paths.into_iter().partition(|p| is_picture(p));
        if !others.is_empty() {
            self.insert_paths(others, window, cx);
        }
        if !images.is_empty() {
            self.insert_pictures(images, window, cx);
        }
    }

    /// ComposerTextView.insertPaths: the paths with a space either side.
    fn insert_paths(&mut self, paths: Vec<PathBuf>, window: &mut Window, cx: &mut Context<Self>) {
        let (text, selection) = self.text_and_selection(cx);
        let mut piece = composer::path_text(&paths.iter().map(|p| p.to_string_lossy().into_owned()).collect::<Vec<_>>());
        if text[..selection.start].chars().next_back().is_some_and(|c| !c.is_whitespace()) {
            piece.insert(0, ' ');
        }
        if !text[selection.end..].chars().next().is_some_and(char::is_whitespace) {
            piece.push(' ');
        }
        self.input.update(cx, |s, cx| {
            s.replace(piece, window, cx);
            s.focus(window, cx);
        });
    }

    fn text_and_selection(&self, cx: &App) -> (String, Range<usize>) {
        let state = self.input.read(cx);
        (state.value().to_string(), state.selected_range())
    }

    /// ComposerTextView.insertPictures: chips where the cursor is, with a
    /// space either side, each copied in through the runner as it shows,
    /// dimmed until it's in; one that can't be read goes again.
    pub(crate) fn insert_pictures(&mut self, paths: Vec<PathBuf>, window: &mut Window, cx: &mut Context<Self>) {
        let ids: Vec<SharedString> = paths.iter().map(|_| self.fresh_picture_id()).collect();
        let tokens: Vec<InlineToken> = ids.iter().map(picture_token).collect();
        let (text, selection) = self.text_and_selection(cx);
        let prev_is_space = text[..selection.start].chars().next_back().map_or(true, char::is_whitespace);
        let next_is_space = text[selection.end..].chars().next().is_some_and(char::is_whitespace);
        self.input.update(cx, |s, cx| {
            if !selection.is_empty() {
                s.replace("", window, cx);
            }
            if !prev_is_space {
                s.insert(" ", window, cx);
            }
            for (i, token) in tokens.iter().enumerate() {
                if i > 0 {
                    s.insert(" ", window, cx);
                }
                let _ = s.replace_with_token(token.clone(), window, cx);
            }
            if !next_is_space {
                s.insert(" ", window, cx);
            }
            s.focus(window, cx);
        });
        for (id, path) in ids.into_iter().zip(paths) {
            self.pictures.insert(id.clone(), PictureChip { source: path.clone(), image: None });
            self.import_picture(id, path, window, cx);
        }
        cx.notify();
    }

    fn import_picture(&mut self, id: SharedString, path: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let client = self.client.clone();
        let params = json!({ "path": path.to_string_lossy() });
        cx.spawn_in(window, async move |this, cx| {
            let result = crate::call(client, "media.import", params).await;
            this.update_in(cx, |this, window, cx| this.settle_picture(id, result, window, cx)).ok();
        })
        .detach();
    }

    /// ComposerTextView.settle: a chip's picture is in, or couldn't be read.
    /// Gone from the words by then (undone, deleted), it's dropped.
    fn settle_picture(&mut self, id: SharedString, result: Result<Value, threadbear_client::runner::RpcError>, window: &mut Window, cx: &mut Context<Self>) {
        let Some(range) = self.chip_range(&id, cx) else {
            self.pictures.remove(&id);
            return;
        };
        match result {
            Ok(v) => {
                let image = ImageRef::decode(&v["image"]);
                if let (Some(chip), Some(image)) = (self.pictures.get_mut(&id), image) {
                    chip.image = Some(image);
                }
            }
            Err(e) => {
                // Taken out, with the space that came with it.
                self.remove_chip_range(range, window, cx);
                self.pictures.remove(&id);
                if let Some(pane) = self.pane() {
                    pane.update(cx, |p, cx| p.show_error(format!("Couldn't add that picture: {}", e.message), cx));
                }
            }
        }
        cx.notify();
    }

    /// The token ranges of the chips in the box, in order.
    fn chips(&self, cx: &App) -> Vec<(Range<usize>, SharedString)> {
        self.input.read(cx).tokens().iter().map(|span| (span.range(), span.token().id().clone())).collect()
    }

    fn chip_range(&self, id: &SharedString, cx: &App) -> Option<Range<usize>> {
        self.chips(cx).into_iter().find(|(_, chip)| chip == id).map(|(r, _)| r)
    }

    /// Takes a chip out with one space beside it (ComposerTextView.withSpace),
    /// keeping the cursor where it was as far as it can.
    fn remove_chip_range(&mut self, range: Range<usize>, window: &mut Window, cx: &mut Context<Self>) {
        let (text, selection) = self.text_and_selection(cx);
        let bytes = text.as_bytes();
        let gone = if bytes.get(range.end) == Some(&b' ') {
            range.start..range.end + 1
        } else if range.start > 0 && bytes[range.start - 1] == b' ' {
            range.start - 1..range.end
        } else {
            range.clone()
        };
        let shift = |at: usize| if at <= gone.start { at } else { at.saturating_sub(gone.len()).max(gone.start) };
        let caret = shift(selection.start)..shift(selection.end);
        self.input.update(cx, |s, cx| {
            s.set_selected_range(gone.clone(), cx);
            s.replace("", window, cx);
            s.set_selected_range(caret.clone(), cx);
        });
    }

    /// The strip's × (ComposerView.removePicture): the chip goes too, and the
    /// cursor stays where it was.
    fn remove_picture(&mut self, id: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(range) = self.chip_range(&id, cx) {
            self.remove_chip_range(range, window, cx);
        }
        self.pictures.remove(&id);
        cx.notify();
    }

    fn renumber_chips(&mut self, cx: &App) {
        self.chip_numbers = self.chips(cx).into_iter().enumerate().map(|(i, (_, id))| (id, i + 1)).collect();
    }

    /// The hover card over a chip, after a beat, as ComposerTextView.hoverChip
    /// does (0.35 s).
    fn hover_chip(&mut self, event: &InlineTokenHoverEvent, cx: &mut Context<Self>) {
        self.hover_serial += 1;
        let serial = self.hover_serial;
        self.peek_shown = None;
        self.hovered_chip = event.is_hovered().then(|| (event.token().id().clone(), event.bounds()));
        cx.notify();
        if self.hovered_chip.is_some() {
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(std::time::Duration::from_millis(350)).await;
                this.update(cx, |this, cx| {
                    if this.hover_serial == serial {
                        this.peek_shown = this.hovered_chip.clone();
                        cx.notify();
                    }
                })
                .ok();
            })
            .detach();
        }
    }

    /// Shows the chip numbered `number`'s picture, as the automation's
    /// `composer-peek` does (ComposerTextView.debugHoverChip).
    #[cfg(feature = "automation")]
    pub fn debug_peek(&mut self, number: Option<usize>, cx: &mut Context<Self>) -> Result<(), String> {
        self.hovered_chip = None;
        match number {
            None => self.peek_shown = None,
            Some(number) => {
                let chips = self.chips(cx);
                let Some((_, id)) = chips.get(number.saturating_sub(1)) else { return Err(format!("no chip {number}")) };
                self.peek_shown = Some((id.clone(), Bounds::default()));
            }
        }
        cx.notify();
        Ok(())
    }

    // ---- Drawing ----

    fn picture_path(&self, id: &SharedString) -> Option<PathBuf> {
        self.pictures.get(id).map(|chip| chip.image.as_ref().map_or_else(|| chip.source.clone(), |i| PathBuf::from(&i.path)))
    }

    /// A picture's chip among the words: a small rounded tag with the
    /// thumbnail and "Picture N" (PictureChip.draw), its sizes from the font
    /// of the words around it. Dimmed until the picture's in.
    fn render_chip(&self, ctx: &InlineTokenContext, cx: &App) -> AnyElement {
        let id = ctx.token().id();
        let chip = self.pictures.get(id);
        let number = self.chip_numbers.get(id).copied().unwrap_or(1);
        let dimmed = chip.is_none_or(|c| c.image.is_none());
        let path = self.picture_path(id);
        let selected = ctx.is_selected();
        let fill = style::chip_fill(cx);
        let edge = if selected { cx.theme().accent } else { style::hairline(cx) };
        let label_color = if dimmed { style::faint_text(cx) } else { style::quiet_text(cx) };
        h_flex()
            .h_full()
            .items_center()
            .child(
                h_flex()
                    .h(px(style::size::CHIP_HEIGHT))
                    .pl(px(3.))
                    .pr(px(7.))
                    .gap(px(5.))
                    .rounded(px(6.))
                    .bg(fill)
                    .border_1()
                    .border_color(edge)
                    .when_some(path, |s, path| s.child(img(path).size(px(14.)).rounded(px(3.)).object_fit(ObjectFit::Cover).when(dimmed, |s| s.opacity(0.5))))
                    .child(div().text_size(px(13.)).text_color(label_color).child(format!("Picture {number}"))),
            )
            .into_any_element()
    }

    /// The strip over the words (AttachmentStrip): a thumbnail for each
    /// picture that's in, with × on hover, and a placeholder for each still
    /// being copied in, so the strip doesn't jump when it arrives.
    fn render_strip(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let chips = self.chips(cx);
        if chips.is_empty() {
            return None;
        }
        let side = px(style::size::THUMBNAIL);
        let radius = px(8.);
        let edge = style::hairline(cx);
        let mut row = h_flex().gap(px(8.));
        for (index, (_, id)) in chips.iter().enumerate() {
            let chip = self.pictures.get(id);
            let image = chip.and_then(|c| c.image.as_ref());
            let tile = div().relative().size(side).rounded(radius).border_1().border_color(edge).overflow_hidden().bg(style::chip_fill(cx));
            let tile = match image {
                Some(image) => tile.child(img(PathBuf::from(&image.path)).size(side).rounded(radius).object_fit(ObjectFit::Cover)),
                None => tile.flex().items_center().justify_center().child(Spinner::new().color(style::quiet_text(cx)).with_size(px(14.))),
            };
            let remove_id = id.clone();
            let remove = div()
                .id(("remove-picture", index))
                .absolute()
                .top(px(4.))
                .right(px(4.))
                .size(px(18.))
                .flex()
                .items_center()
                .justify_center()
                .rounded_full()
                .bg(hsla(0., 0., 0., 0.72))
                .cursor_pointer()
                .child(Icon::from(IconName::X).with_size(px(9.)).text_color(gpui_kit::white()))
                .tooltip(|window, cx| Tooltip::new("Remove").build(window, cx))
                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| this.remove_picture(remove_id.clone(), window, cx)));
            row = row.child(div().id(("strip-tile", index)).group("tile").relative().child(tile).child(div().invisible().group_hover("tile", |s| s.visible()).child(remove)));
        }
        Some(row.into_any_element())
    }

    fn render_send_button(&self, running: bool, cx: &mut Context<Self>) -> AnyElement {
        let can_send = self.can_send(cx);
        let side = px(style::size::SEND_BUTTON);
        let circle = || div().flex().items_center().justify_center().size(side).rounded_full();
        if running && !can_send && self.stopping {
            return circle()
                .id("composer-stopping")
                .bg(style::send_faint_fill(cx))
                .child(Spinner::new().color(style::quiet_text(cx)).with_size(px(14.)))
                .tooltip(|window, cx| Tooltip::new("Stopping").build(window, cx))
                .into_any_element();
        }
        if running && !can_send {
            let (fg, bg) = (cx.theme().background, cx.theme().foreground);
            return circle()
                .id("composer-stop")
                .cursor_pointer()
                .bg(bg)
                .child(Icon::from(IconName::Square).with_size(px(10.)).text_color(fg))
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.stop(cx)))
                .tooltip(|window, cx| Tooltip::new(if cfg!(target_os = "macos") { "Stop (⌘.)" } else { "Stop (Ctrl+.)" }).build(window, cx))
                .into_any_element();
        }
        let tip = if running { "Send (into this turn)" } else { "Send (Return)" };
        let (fg, bg) = if can_send { (cx.theme().background, cx.theme().foreground) } else { (style::faint_text(cx), style::send_faint_fill(cx)) };
        circle()
            .id("composer-send")
            .when(can_send, |s| s.cursor_pointer())
            .bg(bg)
            .child(Icon::from(IconName::ArrowUp).with_size(px(14.)).text_color(fg))
            .when(can_send, |s| s.on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.send(window, cx))))
            .tooltip(move |window, cx| Tooltip::new(tip).build(window, cx))
            .into_any_element()
    }

    /// The text field, at the transcript's body size and line height so a
    /// message keeps its rhythm when it's sent (ComposerTextField).
    fn render_text_field(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity().downgrade();
        let paste_target = this.clone();
        let hover_target = this.clone();
        let token_owner = this.clone();
        let drop_target = this;
        div()
            .w_full()
            .capture_action(cx.listener(|this, action: &Enter, window, cx| {
                if action.shift && this.shift_return(window, cx) {
                    cx.stop_propagation();
                }
            }))
            .on_drop(move |paths: &ExternalPaths, window, cx| {
                if let Some(this) = drop_target.upgrade() {
                    this.update(cx, |this, cx| this.take_paths(paths.paths().to_vec(), window, cx));
                }
            })
            .child(
                Textarea::new(&self.input)
                    .appearance(false)
                    .bordered(false)
                    .text_size(px(style::size::COMPOSER))
                    .line_height(px(style::size::BODY_LINE))
                    .token(move |ctx, _window, cx| match token_owner.upgrade() {
                        Some(this) => this.read(cx).render_chip(ctx, cx),
                        None => div().into_any_element(),
                    })
                    .on_paste(move |item, window, cx| paste_target.upgrade().is_some_and(|this| this.update(cx, |this, cx| this.take_clipboard(item, window, cx))))
                    .on_token_hover(move |event, _window, cx| {
                        if let Some(this) = hover_target.upgrade() {
                            this.update(cx, |this, cx| this.hover_chip(event, cx));
                        }
                    }),
            )
    }

    /// The card over a chip while the pointer rests on it: the picture, to
    /// tell which is which (LinkPeekCard with pictureOnly).
    fn render_peek(&self, cx: &App) -> Option<AnyElement> {
        let (id, bounds) = self.peek_shown.as_ref()?;
        let path = self.picture_path(id)?;
        Some(
            deferred(
                anchored().position(bounds.origin).anchor(Anchor::BottomLeft).snap_to_window().offset(point(px(0.), px(-8.))).child(
                    div().p(px(4.)).rounded(px(10.)).bg(cx.theme().popover).border_1().border_color(style::hairline(cx)).shadow_lg().child(img(path).max_w(px(320.)).max_h(px(240.)).rounded(px(6.))),
                ),
            )
            .with_priority(1)
            .into_any_element(),
        )
    }
}

impl Render for Composer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some((draft, images)) = self.restore.take() {
            self.apply_restore(draft, images, window, cx);
        }
        self.renumber_chips(cx);
        let running = self.pane().is_some_and(|p| p.read(cx).summary.busy());
        if !running {
            self.stopping = false;
        }
        let strip = self.render_strip(cx);
        let body = v_flex()
            .w_full()
            .gap(px(style::COMPOSER_GAP))
            .p(px(style::COMPOSER_PADDING))
            .when_some(strip, |s, strip| s.child(strip))
            .child(self.render_text_field(cx))
            .child(h_flex().w_full().h(px(style::size::SEND_BUTTON)).justify_end().child(self.render_send_button(running, cx)));
        // The transcript's reading column: 780 wide at most, centred, with
        // the same side insets once the window is narrower.
        h_flex()
            .w_full()
            .justify_center()
            .px(px(style::SIDE_INSET))
            .pt(px(4.))
            .pb(px(20.))
            .child(
                div()
                    .w_full()
                    .max_w(px(style::MAX_CONTENT_WIDTH))
                    .rounded(px(style::COMPOSER_RADIUS))
                    .bg(style::composer_fill(cx))
                    .border_1()
                    .border_color(style::hairline(cx))
                    .when(!cx.theme().is_dark(), |s| s.shadow_md())
                    // Clicking the card's padding puts the cursor in the text.
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _: &MouseDownEvent, window, cx| this.focus(window, cx)))
                    .child(body),
            )
            .children(self.render_peek(cx))
    }
}

/// The token a chip draws from. The token's text is the mark the draft holds
/// (InlinePictures.mark); its label is only for accessibility and measuring.
fn picture_token(id: &SharedString) -> InlineToken {
    InlineToken::new(id.clone(), PICTURE_MARK_STR).with_label("Picture")
}

/// Whether a dropped or pasted file is a picture, by its name
/// (ComposerTextView.isImage asks the file's type; a name is what every
/// system can tell).
pub(crate) fn is_picture(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "heic" | "heif" | "tif" | "tiff" | "bmp")
    )
}

/// A picture pasted as raw image data (a screenshot on the clipboard, say),
/// saved to a file the runner can import, as ComposerTextView.takeImages does.
/// PNG and JPEG go as they are; the runner turns TIFF and BMP into PNG
/// (`media.import`). An SVG stays out: the runner doesn't take them.
fn save_pasted_image(image: &gpui_kit::Image) -> Option<PathBuf> {
    let ext = match image.format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpeg",
        ImageFormat::Webp => "webp",
        ImageFormat::Gif => "gif",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Tiff => "tiff",
        _ => return None,
    };
    let folder = std::env::temp_dir().join("threadbear-paste");
    std::fs::create_dir_all(&folder).ok()?;
    let path = folder.join(format!("pasted-{:x}.{ext}", image.id));
    std::fs::write(&path, &image.bytes).ok()?;
    Some(path)
}

/// The JSON a picture goes as (`ImageRef` in docs/protocol.md).
pub fn image_ref_json(image: &ImageRef) -> Value {
    let mut out = json!({ "path": image.path, "mediaType": image.media_type });
    if let Some(w) = image.width {
        out["width"] = json!(w);
    }
    if let Some(h) = image.height {
        out["height"] = json!(h);
    }
    out
}
