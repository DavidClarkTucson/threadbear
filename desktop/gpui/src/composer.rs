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
use crate::composer_card::CardState;
use crate::composer_chips::ModelPickerView;
use crate::style;
use crate::thread::ThreadPane;
use gpui_kit::assets::IconName;
use gpui_kit::component::input::{Copy, Cut, Enter, Escape, IndentInline, InlineToken, InlineTokenContext, InlineTokenHoverEvent, InputEvent, MoveDown, MoveUp, Textarea, TextareaState};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _};
use gpui_kit::prelude::*;
use gpui_kit::*;
// The zoom-aware px (style.rs), in place of GPUI's.
use crate::style::px;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use threadbear_client::runner::Client;
use threadbear_core::composer::{self, BOX_MAX_LINES, BOX_MIN_LINES, PICTURE_MARK, PICTURE_MARK_STR};
use threadbear_core::reducer::ImageRef;
use threadbear_core::slash::{PaletteKey, PaletteState};
use std::cell::Cell;
use std::rc::Rc;
use crate::tips::Tip as _;

/// A picture in the box: the token that draws its chip, what it came from,
/// and what the runner made of it (`media.import`).
pub(crate) struct PictureChip {
    /// The file it came from, for the thumbnail meanwhile.
    pub(crate) source: PathBuf,
    /// None while the runner is still copying it in.
    pub(crate) image: Option<ImageRef>,
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
    pub(crate) pictures: HashMap<SharedString, PictureChip>,
    /// Its card and thread chips, by token id (composer_inline.rs).
    pub(crate) mentions: HashMap<SharedString, crate::composer_inline::MentionChip>,
    pub(crate) next_picture: u64,
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
    restore: Option<RestoreDraft>,
    /// The model picker's search and column, kept while it's open.
    pub(crate) model_picker: Option<Entity<ModelPickerView>>,
    /// What's picked on the open card, and how the card's showing.
    pub(crate) card: CardState,
    /// The card's text style, one per appearance (`card_reading_style`).
    pub(crate) card_style: Option<((bool, u32), gpui_kit::base::TextViewStyle)>,
    shown_placeholder: &'static str,
    /// The slash palette's own state (`PaletteState`), and whether this
    /// opening of it has asked for the thread's commands yet.
    pub(crate) palette: PaletteState,
    pub(crate) palette_asked: bool,
    /// The palette's list, kept so the arrows can scroll the pick into sight.
    pub(crate) palette_scroll: ScrollHandle,
    /// The box's width at the last paint, for the palette's columns.
    pub(crate) card_width: Rc<Cell<f32>>,
    /// One of Mike's messages being edited (`AppModel.editing`): sending
    /// replaces it and carries on from there.
    pub(crate) editing: Option<MessageEdit>,
    /// What the box is to hold at the next render (an edit sent puts back
    /// what was in it before; that needs the window).
    pending_box: Option<(String, Vec<ImageRef>)>,
    _subs: Vec<Subscription>,
}

/// An edit under way: the message's seq, what was in the box before, and
/// whether it's on its way.
pub(crate) struct MessageEdit {
    pub(crate) seq: i64,
    before: (String, Vec<ImageRef>),
    sending: bool,
}

/// What a refused send gives back: the words and pictures, and the picks that
/// went with them (AppModel.restoreDraft and the `failed` closure in `send`).
pub struct RestoreDraft {
    pub(crate) text: String,
    pub(crate) images: Vec<ImageRef>,
    pub(crate) card: Option<SavedPicks>,
}

#[derive(Clone)]
pub struct SavedPicks {
    pub(crate) seq: i64,
    pub(crate) picks: std::collections::BTreeMap<i64, i64>,
    pub(crate) notes: std::collections::BTreeMap<i64, threadbear_core::reducer::AskNote>,
    pub(crate) next_picks: std::collections::BTreeSet<i64>,
    pub(crate) explain: std::collections::BTreeSet<i64>,
}

impl Composer {
    pub fn new(thread_id: Option<String>, client: Arc<Client>, app: WeakEntity<Threadbear>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Two lines at least, ten at most, then it scrolls (ComposerChips.boxHeight).
        let input = cx.new(|cx| TextareaState::new(window, cx).placeholder("Message").auto_grow(BOX_MIN_LINES, BOX_MAX_LINES).submit_on_enter(true));
        let sub = cx.subscribe_in(&input, window, |this, _, ev: &InputEvent, window, cx| match ev {
            InputEvent::PressEnter { shift: false, .. } => this.return_pressed(window, cx),
            // The send button, the picture strip and the slash palette follow
            // what's in the box.
            InputEvent::Change => {
                this.palette_text_changed(cx);
                // Kept across a quit (drafts.rs).
                if let Some(app) = this.app.upgrade() {
                    app.update(cx, |app, cx| app.drafts_changed(cx));
                }
                cx.notify();
            }
            _ => {}
        });
        Composer {
            thread_id,
            client,
            app,
            pane: WeakEntity::new_invalid(),
            input,
            pictures: HashMap::new(),
            mentions: HashMap::new(),
            next_picture: 0,
            chip_numbers: HashMap::new(),
            hovered_chip: None,
            peek_shown: None,
            hover_serial: 0,
            stopping: false,
            restore: None,
            model_picker: None,
            card: CardState::default(),
            card_style: None,
            shown_placeholder: "Message",
            editing: None,
            pending_box: None,
            palette: PaletteState::default(),
            palette_asked: false,
            palette_scroll: ScrollHandle::new(),
            card_width: Rc::new(Cell::new(style::MAX_CONTENT_WIDTH)),
            _subs: vec![sub],
        }
    }

    pub fn attach(&mut self, pane: WeakEntity<ThreadPane>) {
        self.pane = pane;
    }

    /// A new thread is on its way from this box (`AppModel.creatingThread`).
    fn creating(&self, cx: &App) -> bool {
        self.thread_id.is_none() && self.app.upgrade().is_some_and(|a| a.read(cx).new_thread.creating)
    }

    /// The thread exists: what was sent leaves the box, if it's still what
    /// the box holds (it was locked while the runner made it).
    pub(crate) fn clear_sent_draft(&mut self, stored: &str, window: &mut Window, cx: &mut Context<Self>) {
        if self.input.read(cx).value().as_ref() == stored {
            self.input.update(cx, |s, cx| s.set_value("", window, cx));
            self.pictures.clear();
            self.mentions.clear();
        }
        cx.notify();
    }

    /// What Schedule… takes from the box: its words, without the pictures'
    /// marks (a schedule sends only text), and how many pictures there are.
    pub(crate) fn schedule_draft(&self, cx: &App) -> (String, usize) {
        let (text, images) = self.draft(cx);
        (text.replace(PICTURE_MARK, "").trim().to_owned(), images.len())
    }

    /// The schedule's made from the box's words: they leave it. Pictures are
    /// chips in the text, so a box with any is left as it is.
    pub(crate) fn clear_scheduled_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.draft(cx).1.is_empty() {
            self.input.update(cx, |s, cx| s.set_value("", window, cx));
            self.pictures.clear();
            self.mentions.clear();
            cx.notify();
        }
    }

    /// `LimitBox`: the thread's login is at its limit, or its last turn
    /// stopped on it. At the top of the box, over a line like the card of
    /// asks: what happened, Switch to (a pool thread's Pin to) the login with
    /// the most room, and whether it carries on by itself once the limit
    /// resets. Mike asked for it here on 10 Nov, in place of a bar across the
    /// top of the thread that read as an error and offered only the switch.
    fn render_limit_box(&mut self, cx: &mut Context<Self>) -> Option<AnyElement> {
        use gpui_kit::base::TestSupportExt as _;
        use gpui_kit::component::button::{Button, ButtonVariants as _};
        use gpui_kit::component::menu::{DropdownMenu as _, PopupMenu, PopupMenuItem};
        let thread_id = self.thread_id.clone()?;
        let app = self.app.upgrade()?;
        let (thread, advice) = {
            let t = app.read(cx);
            let thread = t.threads.get(&thread_id)?.clone();
            // While it moves to another Mac there's nothing to switch.
            if thread.moving.is_some() {
                return None;
            }
            let advice = t.limit_advice(&thread, cx)?;
            (thread, advice)
        };
        let now = threadbear_core::sidebar::local_now();
        let when = advice.carries_on_at.map(|at| threadbear_core::sidebar::carries_on_when(at, &now));
        // Waiting to carry on reads as a quiet clock, not the red gauge.
        let waiting = advice.carries_on_at.is_some() && thread.carry_on_at_reset;
        let text = match (&when, waiting) {
            (Some(when), true) => format!("{} Carries on by itself {when}.", advice.text),
            _ => advice.text.clone(),
        };
        let secondary = style::secondary_text(cx);
        let mark = if waiting { crate::icons::symbol("clock").with_size(px(13.)).text_color(secondary) } else { crate::icons::symbol("gauge.with.dots.needle.100percent").with_size(px(13.)).text_color(cx.theme().danger) };
        let mut column = v_flex()
            .w_full()
            .gap(px(8.))
            .px(px(style::COMPOSER_PADDING))
            .pt(px(10.))
            .pb(px(10.))
            .text_size(px(style::size::LABEL))
            .line_height(px(16.))
            .child(h_flex().items_start().gap(px(8.)).child(div().flex_none().pt(px(1.)).child(mark)).child(crate::highlights::tipped(div().id("limit-text").flex_1().min_w_0(), text.clone()).child(text)));
        if advice.target.is_some() || advice.stopped {
            let mut row = h_flex().gap(px(10.)).items_center();
            if let Some((account, title)) = advice.target.clone() {
                let title = crate::header::limit_button_title(title, advice.pool);
                let tip = "Carry on this thread on that login now, from where it is";
                let (app, id) = (self.app.clone(), thread.id.clone());
                row = row.child(
                    div().id("limit-switch").test_support().aria_label(tip).child(Button::new("limit-switch-button").label(title).small().secondary().tooltip(tip).on_click(move |_, _, cx| {
                        app.update(cx, |a, cx| a.switch_account(&id, account.clone(), cx)).ok();
                    })),
                );
            }
            if advice.stopped {
                let on = thread.carry_on_at_reset;
                let tip: String = if on {
                    format!("It carries on from where it stopped {}. No to leave it until you send a message", when.clone().unwrap_or_else(|| "once its limit resets".into()))
                } else {
                    "It waits for you to send a message. Yes to carry on from where it stopped once there's room".into()
                };
                let (app, id) = (self.app.clone(), thread.id.clone());
                let menu = Button::new("limit-carry-on-button").label(if on { "Yes" } else { "No" }).small().secondary().dropdown_caret(true).tooltip(tip.clone()).dropdown_menu(move |menu: PopupMenu, _, _| {
                    let mut menu = menu;
                    for (label, value) in [("Yes", true), ("No", false)] {
                        let (app, id) = (app.clone(), id.clone());
                        menu = menu.item(PopupMenuItem::new(label).checked(on == value).on_click(move |_, _, cx| {
                            app.update(cx, |a, cx| a.set_carry_on_at_reset(&id, value, cx)).ok();
                        }));
                    }
                    menu
                });
                row = row.child(h_flex().gap(px(6.)).items_center().child(div().text_color(secondary).child("Carry on by itself")).child(div().id("limit-carry-on").test_support().aria_label(tip).child(menu)));
            }
            column = column.child(row);
        }
        Some(v_flex().w_full().child(column).child(div().w_full().h(gpui_kit::px(1.)).bg(style::hairline(cx))).into_any_element())
    }

    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        self.input.update(cx, |s, cx| s.focus(window, cx));
    }

    /// What the box holds as a draft (ComposerTextView.draftNow): the words,
    /// a mark for each chip whose picture is in, and those pictures in order.
    /// A chip still being copied in isn't part of it yet, nor is a stray
    /// U+FFFC pasted as text.
    /// The words and pictures in the box, for drafts.rs to keep.
    pub(crate) fn draft_for_saving(&self, cx: &App) -> (String, Vec<ImageRef>) {
        self.draft(cx)
    }

    /// A card's or thread's chip is its link in the draft (composer_inline.rs).
    fn draft(&self, cx: &App) -> (String, Vec<ImageRef>) {
        let text = self.input.read(cx).value().to_string();
        let end = text.len();
        threadbear_core::inline_chips::draft_in(&text, &self.box_chips(cx), 0..end)
    }

    /// The box's text as the Swift dump gives it: chips as the markers they'll
    /// go out as (`InlinePictures.messageText`).
    #[cfg(feature = "automation")]
    pub fn draft_text(&self, cx: &App) -> String {
        composer::message_text(&self.draft(cx).0, 1)
    }

    /// How many pictures are in the box (the dump's `composer.images`).
    #[cfg(feature = "automation")]
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
        if self.input.read(cx).value().is_empty() && self.return_when_empty(window, cx) {
            return;
        }
        self.send(window, cx);
    }

    /// ComposerView.returnWhenEmpty: Return with nothing typed allows the
    /// sole waiting approval (the ⏎ its Allow button shows). Not while a
    /// message is being edited: the box is the edit then, an empty one
    /// sends nothing, and Return there is about that message, never a card
    /// that may be off screen (Daybreak, 11 Nov: pencil, clear, Return
    /// allowed a waiting tool). Pictures are chips in the text, so an edit
    /// with only pictures left isn't empty and goes to `send`.
    fn return_when_empty(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        if self.editing.is_some() {
            return true;
        }
        if self.return_with_picks(window, cx) {
            return true;
        }
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
        composer::can_send(&text, images.len()) || !self.card.picks.is_empty() || !self.card.next_picks.is_empty() || !self.card.explain.is_empty()
    }

    /// AppModel.send: the box empties at once and the message shows dimmed
    /// at the end of the transcript until the runner has it; it comes back
    /// if the runner doesn't take it.
    pub(crate) fn send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.thread_id.is_none() {
            return self.send_new_thread(window, cx);
        }
        if self.editing.is_some() {
            return self.send_edit(cx);
        }
        let (draft_text, images) = self.draft(cx);
        // Pictures from the asks' boxes come first, numbered as the answers
        // say, then the box's own chips, whose markers go after them.
        let boxes = self.note_images_len(cx);
        let words = composer::sent_text(&draft_text, boxes + 1).trim().to_string();
        let (text, images_to_send, answers) = match self.card_message(&words, cx) {
            Some((message, mut note_images, answers)) => {
                note_images.extend(images.clone());
                (message, note_images, answers)
            }
            None => (words, images.clone(), Value::Null),
        };
        if text.trim().is_empty() && images_to_send.is_empty() {
            return;
        }
        let Some(pane) = self.pane() else { return };
        let picks = self.saved_picks();
        self.input.update(cx, |s, cx| s.set_value("", window, cx));
        self.pictures.clear();
        self.mentions.clear();
        self.clear_picks(cx);
        let composer = cx.entity().downgrade();
        let restore = RestoreDraft { text: draft_text, images, card: picks };
        pane.update(cx, |p, cx| p.send_message(text, images_to_send, answers, Some(restore), composer, cx));
        cx.notify();
    }

    /// `AppModel.send(threadId: nil)`: the new-thread page's box. The words
    /// stay (greyed, locked) until the thread exists; a failure leaves them.
    fn send_new_thread(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (draft_text, images) = self.draft(cx);
        let text = composer::sent_text(&draft_text, 1).trim().to_string();
        if text.is_empty() && images.is_empty() {
            return;
        }
        let stored = self.input.read(cx).value().to_string();
        if let Some(app) = self.app.upgrade() {
            app.update(cx, |app, cx| app.send_new_thread(text, images, stored, window, cx));
        }
    }

    fn saved_picks(&self) -> Option<SavedPicks> {
        self.card.seq.map(|seq| SavedPicks { seq, picks: self.card.picks.clone(), notes: self.card.notes.clone(), next_picks: self.card.next_picks.clone(), explain: self.card.explain.clone() })
    }

    fn note_images_len(&self, cx: &App) -> usize {
        self.open_card(cx).filter(|c| !c.is_done()).map_or(0, |c| composer::note_images(&c.asks, &self.card.picks, &self.card.notes).len())
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
    /// The pencil by one of Mike's messages (`startEditing`): its words and
    /// pictures go in the box, chips where their markers were, and what was
    /// in the box comes back if the edit's left.
    pub(crate) fn start_editing(&mut self, seq: i64, text: &str, images: &[ImageRef], window: &mut Window, cx: &mut Context<Self>) {
        let before = match self.editing.take() {
            Some(edit) => edit.before,
            None => self.draft(cx),
        };
        let (words, order) = composer::edit_draft(text, images.len());
        let pictures = order.iter().filter_map(|i| images.get(*i).cloned()).collect();
        self.set_box(words, pictures, window, cx);
        self.editing = Some(MessageEdit { seq, before, sending: false });
        self.focus(window, cx);
        cx.notify();
    }

    /// Leaves the edit (`cancelEditing`): the box gets back what was in it.
    pub(crate) fn cancel_editing(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(edit) = self.editing.take() else { return };
        let (text, images) = edit.before;
        self.set_box(text, images, window, cx);
        cx.notify();
    }

    /// The box holding exactly `text`, its marks as chips of `images`.
    fn set_box(&mut self, text: String, images: Vec<ImageRef>, window: &mut Window, cx: &mut Context<Self>) {
        self.input.update(cx, |s, cx| s.set_value("", window, cx));
        self.pictures.clear();
        self.mentions.clear();
        self.apply_restore(RestoreDraft { text, images, card: None }, window, cx);
    }

    /// Sends the edited message (`sendEdit`): the box keeps it, its banner
    /// saying it's on its way, until the runner has started the new turn;
    /// an error leaves it there to try again. Going back would stop work the
    /// agent left running, so that's asked first.
    fn send_edit(&mut self, cx: &mut Context<Self>) {
        let Some(seq) = self.editing.as_ref().filter(|e| !e.sending).map(|e| e.seq) else { return };
        let Some(thread_id) = self.thread_id.clone() else { return };
        let (draft_text, images) = self.draft(cx);
        let text = composer::sent_text(&draft_text, 1).trim().to_string();
        if text.is_empty() && images.is_empty() {
            return;
        }
        if let Some(edit) = self.editing.as_mut() {
            edit.sending = true;
        }
        crate::activity::sent(Some(&thread_id), images.len(), 0, true);
        cx.notify();
        let client = self.client.clone();
        let images_json: Vec<Value> = images.iter().map(image_ref_json).collect();
        cx.spawn(async move |this, cx| {
            let mut stop_background = false;
            loop {
                let mut params = json!({ "threadId": thread_id, "seq": seq, "text": text, "stopBackground": stop_background });
                if !images_json.is_empty() {
                    params["images"] = json!(images_json);
                }
                match crate::call(client.clone(), "threads.edit", params).await {
                    Ok(_) => {
                        this.update(cx, |this, cx| {
                            // Done with: the box gets back what was in it before,
                            // at the next render, which has the window.
                            if let Some(edit) = this.editing.take() {
                                this.pending_box = Some(edit.before);
                            }
                            cx.notify();
                        })
                        .ok();
                    }
                    Err(e) if e.code == -32050 && !stop_background => {
                        // An upload was lost that way (4 Nov): asked first.
                        let detail = format!("{}\n\nCancel keeps it running; you can send your change as a new message instead.", e.message);
                        if crate::alert::confirm("Stop the background work?", &detail, "Stop It and Go Back", "Cancel") {
                            stop_background = true;
                            continue;
                        }
                        this.update(cx, |this, cx| {
                            if let Some(edit) = this.editing.as_mut() {
                                edit.sending = false;
                            }
                            cx.notify();
                        })
                        .ok();
                    }
                    Err(e) => {
                        this.update(cx, |this, cx| {
                            if let Some(edit) = this.editing.as_mut() {
                                edit.sending = false;
                            }
                            if let Some(pane) = this.pane() {
                                pane.update(cx, |p, cx| p.show_error(e.message, cx));
                            }
                            cx.notify();
                        })
                        .ok();
                    }
                }
                return;
            }
        })
        .detach();
    }

    /// `EditingBanner`: what sending will do, and Cancel (Esc); while it's
    /// on its way, that it's cutting back, with a spinner.
    fn render_editing_banner(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let edit = self.editing.as_ref()?;
        let secondary = style::secondary_text(cx);
        let text = if edit.sending { "Cutting the thread back to this message…" } else { "Editing a message. Sending cuts the thread back to it and undoes the agent's file changes since." };
        let size = px(style::size::SIDEBAR_META + 1.);
        let end: AnyElement = if edit.sending {
            Spinner::new().color(secondary).with_size(px(12.)).into_any_element()
        } else {
            crate::highlights::tipped(div().id("edit-cancel").flex_none().cursor_pointer().text_size(size).text_color(style::accent(cx)), "Stop editing and put back what was in the box (Esc)".to_owned())
                .on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.cancel_editing(window, cx)))
                .child("Cancel")
                .into_any_element()
        };
        Some(
            h_flex()
                .w_full()
                .gap(px(6.))
                .items_start()
                .child(div().flex_none().pt(px(2.)).child(crate::icons::symbol("pencil").with_size(px(style::size::SIDEBAR_META)).text_color(secondary)))
                .child(div().flex_1().min_w_0().text_size(size).line_height(px(style::natural_line(style::size::SIDEBAR_META + 1.))).text_color(secondary).child(text))
                .child(end)
                .into_any_element(),
        )
    }

    pub fn restore_draft(&mut self, draft: RestoreDraft, cx: &mut Context<Self>) {
        self.restore = Some(draft);
        cx.notify();
    }

    fn apply_restore(&mut self, restore: RestoreDraft, window: &mut Window, cx: &mut Context<Self>) {
        let RestoreDraft { text: draft, images, card } = restore;
        if let Some(saved) = card {
            if self.card.seq == Some(saved.seq) {
                self.card.picks = saved.picks;
                self.card.notes = saved.notes;
                self.card.next_picks = saved.next_picks;
                self.card.explain = saved.explain;
                self.push_card_state(cx);
            }
        }
        if !self.input.read(cx).value().is_empty() {
            return;
        }
        // Chips again where the marks are, each picture back in place, and
        // the cards and threads linked in the words as theirs.
        let content = self.content_for(&draft, &images);
        self.input.update(cx, |s, cx| s.set_value(content, window, cx));
    }

    pub(crate) fn fresh_picture_id(&mut self) -> SharedString {
        self.next_picture += 1;
        format!("picture-{}", self.next_picture).into()
    }

    // ---- Pictures ----

    /// Paste: pictures and files first, as ComposerTextView.takeImages takes
    /// them. Returns whether it took the paste; words paste as words.
    pub(crate) fn take_clipboard(&mut self, item: &ClipboardItem, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let target = self.input.clone();
        self.take_clipboard_in(&target, item, window, cx)
    }

    /// A paste into `target`: the box, or one of the card's Something else
    /// boxes, which takes pictures as chips just the same (AskAnswerBox).
    pub(crate) fn take_clipboard_in(&mut self, target: &Entity<TextareaState>, item: &ClipboardItem, window: &mut Window, cx: &mut Context<Self>) -> bool {
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
            self.take_paths_in(target, paths, window, cx);
            return true;
        }
        // A copy of the box's own chips with its pictures, a thread's id, a
        // card's link: chips (composer_inline.rs). Other words are words.
        if has_text && self.take_clipboard_chips(target, item, window, cx) {
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
        self.insert_pictures_in(target, saved, window, cx);
        true
    }

    /// Files pasted or dropped: pictures become chips where the cursor is, and
    /// anything else goes in as its full path in backticks, for the agent to
    /// open where it is (ComposerTextView.takeImages).
    pub(crate) fn take_paths(&mut self, paths: Vec<PathBuf>, window: &mut Window, cx: &mut Context<Self>) {
        let target = self.input.clone();
        self.take_paths_in(&target, paths, window, cx);
    }

    pub(crate) fn take_paths_in(&mut self, target: &Entity<TextareaState>, paths: Vec<PathBuf>, window: &mut Window, cx: &mut Context<Self>) {
        let (images, others): (Vec<_>, Vec<_>) = paths.into_iter().partition(|p| is_picture(p));
        if !others.is_empty() {
            Self::insert_paths(target, others, window, cx);
        }
        if !images.is_empty() {
            self.insert_pictures_in(target, images, window, cx);
        }
    }

    /// ComposerTextView.insertPaths: the paths with a space either side.
    fn insert_paths(target: &Entity<TextareaState>, paths: Vec<PathBuf>, window: &mut Window, cx: &mut Context<Self>) {
        let (text, selection) = Self::text_and_selection(target, cx);
        let mut piece = composer::path_text(&paths.iter().map(|p| p.to_string_lossy().into_owned()).collect::<Vec<_>>());
        if text[..selection.start].chars().next_back().is_some_and(|c| !c.is_whitespace()) {
            piece.insert(0, ' ');
        }
        if !text[selection.end..].chars().next().is_some_and(char::is_whitespace) {
            piece.push(' ');
        }
        target.update(cx, |s, cx| {
            s.replace(piece, window, cx);
            s.focus(window, cx);
        });
    }

    fn text_and_selection(target: &Entity<TextareaState>, cx: &App) -> (String, Range<usize>) {
        let state = target.read(cx);
        (state.value().to_string(), state.selected_range())
    }

    /// ComposerTextView.insertPictures: chips where the cursor is in
    /// `target` (the box, or a Something else box), with a space either
    /// side, each copied in through the runner as it shows, dimmed until it's
    /// in; one that can't be read goes again.
    pub(crate) fn insert_pictures_in(&mut self, target: &Entity<TextareaState>, paths: Vec<PathBuf>, window: &mut Window, cx: &mut Context<Self>) {
        let ids: Vec<SharedString> = paths.iter().map(|_| self.fresh_picture_id()).collect();
        let tokens: Vec<InlineToken> = ids.iter().map(picture_token).collect();
        let (text, selection) = Self::text_and_selection(target, cx);
        let prev_is_space = text[..selection.start].chars().next_back().map_or(true, char::is_whitespace);
        let next_is_space = text[selection.end..].chars().next().is_some_and(char::is_whitespace);
        target.update(cx, |s, cx| {
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
            self.import_picture(id, path, target.downgrade(), window, cx);
        }
        cx.notify();
    }

    fn import_picture(&mut self, id: SharedString, path: PathBuf, target: WeakEntity<TextareaState>, window: &mut Window, cx: &mut Context<Self>) {
        let client = self.client.clone();
        let params = json!({ "path": path.to_string_lossy() });
        cx.spawn_in(window, async move |this, cx| {
            let result = crate::call(client, "media.import", params).await;
            // The runner keeps its own copy (or said no): a pasted picture's
            // staged file goes now, whatever happens to the chip.
            unstage(&path);
            this.update_in(cx, |this, window, cx| this.settle_picture(id, result, target, window, cx)).ok();
        })
        .detach();
    }

    /// ComposerTextView.settle: a chip's picture is in, or couldn't be read.
    /// Gone from the words by then (undone, deleted, its box gone), it's
    /// dropped.
    fn settle_picture(&mut self, id: SharedString, result: Result<Value, threadbear_client::runner::RpcError>, target: WeakEntity<TextareaState>, window: &mut Window, cx: &mut Context<Self>) {
        let Some(target) = target.upgrade() else {
            self.pictures.remove(&id);
            return;
        };
        let Some(range) = Self::chip_range(&target, &id, cx) else {
            self.pictures.remove(&id);
            return;
        };
        match result {
            Ok(v) => {
                let image = ImageRef::decode(&v["image"]);
                if let (Some(chip), Some(image)) = (self.pictures.get_mut(&id), image) {
                    chip.image = Some(image);
                }
                // A Something else box's answer takes it now.
                if target != self.input {
                    self.answer_box_changed(&target, cx);
                }
            }
            Err(e) => {
                // Taken out, with the space that came with it.
                Self::remove_chip_range(&target, range, window, cx);
                self.pictures.remove(&id);
                if let Some(pane) = self.pane() {
                    pane.update(cx, |p, cx| p.show_error(format!("Couldn't add that picture: {}", e.message), cx));
                }
            }
        }
        cx.notify();
    }

    /// The token ranges of the picture chips in the box, in order: a card's
    /// or thread's chip isn't one of them.
    fn chips(&self, cx: &App) -> Vec<(Range<usize>, SharedString)> {
        Self::chips_in(&self.input, cx)
    }

    pub(crate) fn chips_in(target: &Entity<TextareaState>, cx: &App) -> Vec<(Range<usize>, SharedString)> {
        target.read(cx).tokens().iter().filter(|span| !Self::is_mention_token(span.token().id())).map(|span| (span.range(), span.token().id().clone())).collect()
    }

    fn chip_range(target: &Entity<TextareaState>, id: &SharedString, cx: &App) -> Option<Range<usize>> {
        Self::chips_in(target, cx).into_iter().find(|(_, chip)| chip == id).map(|(r, _)| r)
    }

    /// Takes a chip out with one space beside it (ComposerTextView.withSpace),
    /// keeping the cursor where it was as far as it can.
    fn remove_chip_range(target: &Entity<TextareaState>, range: Range<usize>, window: &mut Window, cx: &mut Context<Self>) {
        let (text, selection) = Self::text_and_selection(target, cx);
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
        target.update(cx, |s, cx| {
            s.set_selected_range(gone.clone(), cx);
            s.replace("", window, cx);
            s.set_selected_range(caret.clone(), cx);
        });
    }

    /// The strip's × (ComposerView.removePicture): the chip goes too, and the
    /// cursor stays where it was.
    fn remove_picture(&mut self, id: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        let target = self.input.clone();
        self.remove_picture_in(&target, id, window, cx);
    }

    pub(crate) fn remove_picture_in(&mut self, target: &Entity<TextareaState>, id: SharedString, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(range) = Self::chip_range(target, &id, cx) {
            Self::remove_chip_range(target, range, window, cx);
        }
        self.pictures.remove(&id);
        cx.notify();
    }

    fn renumber_chips(&mut self, cx: &App) {
        self.chip_numbers = self.chips(cx).into_iter().enumerate().map(|(i, (_, id))| (id, i + 1)).collect();
        // A Something else box's chips count from 1 in each box.
        for input in self.card.answer_inputs() {
            for (i, (_, id)) in Self::chips_in(&input, cx).into_iter().enumerate() {
                self.chip_numbers.insert(id, i + 1);
            }
        }
    }

    /// The hover card over a chip, after a beat, as ComposerTextView.hoverChip
    /// does (0.35 s).
    fn hover_chip(&mut self, event: &InlineTokenHoverEvent, cx: &mut Context<Self>) {
        // A card's or thread's chip has no picture to show.
        if Self::is_mention_token(event.token().id()) {
            return;
        }
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
    pub(crate) fn render_chip(&self, ctx: &InlineTokenContext, cx: &App) -> AnyElement {
        let id = ctx.token().id();
        if Self::is_mention_token(id) {
            return self.render_mention_chip(ctx, cx);
        }
        let chip = self.pictures.get(id);
        let number = self.chip_numbers.get(id).copied().unwrap_or(1);
        let dimmed = chip.is_none_or(|c| c.image.is_none());
        let path = self.picture_path(id);
        let selected = ctx.is_selected();
        let fill = style::chip_fill(cx);
        let edge = if selected { style::accent(cx) } else { style::hairline(cx) };
        let label_color = if dimmed { style::faint_text(cx) } else { style::quiet_text(cx) };
        h_flex()
            .h_full()
            .items_center()
            .child(
                h_flex()
                    .h(px(style::size::CHIP_HEIGHT))
                    // PictureChip.draw strokes half a point inside its 3 and
                    // 7; here the edge takes room, so the padding gives it back.
                    .pl(px(2.5))
                    .pr(px(6.5))
                    .gap(px(5.))
                    .rounded(px(6.))
                    .bg(fill)
                    .border(gpui_kit::px(0.5))
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
                .tip("Remove")
                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| this.remove_picture(remove_id.clone(), window, cx)));
            row = row.child(div().id(("strip-tile", index)).group("tile").relative().child(tile).child(div().invisible().group_hover("tile", |s| s.visible()).child(remove)));
        }
        Some(row.into_any_element())
    }

    fn render_send_button(&self, running: bool, creating: bool, cx: &mut Context<Self>) -> AnyElement {
        // Each state named as SendButton's are, for VoiceOver and the
        // scripts that right-click it (composer_send.rs).
        use gpui_kit::base::TestSupportExt as _;
        let can_send = self.can_send(cx);
        let side = px(style::size::SEND_BUTTON);
        let circle = || div().flex().items_center().justify_center().size(side).rounded_full();
        // The runner is making the thread: a spinner in the button's place
        // until it has, however long a busy Mac makes that (SendButton).
        if creating {
            return circle()
                .id("composer-starting")
                .test_support()
                .aria_label("Starting the thread")
                .bg(style::send_faint_fill(cx))
                .child(Spinner::new().color(style::quiet_text(cx)).with_size(px(14.)))
                .tip("Starting the thread")
                .into_any_element();
        }
        if running && !can_send && self.stopping {
            return circle()
                .id("composer-stopping")
                .test_support()
                .aria_label("Stopping")
                .bg(style::send_faint_fill(cx))
                .child(Spinner::new().color(style::quiet_text(cx)).with_size(px(14.)))
                .tip("Stopping")
                .into_any_element();
        }
        if running && !can_send {
            let (fg, bg) = (cx.theme().background, cx.theme().foreground);
            return circle()
                .id("composer-stop")
                .test_support()
                .aria_label("Stop")
                .cursor_pointer()
                .bg(bg)
                .child(Icon::from(IconName::Square).with_size(px(10.)).text_color(fg))
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.stop(cx)))
                .tip(if cfg!(target_os = "macos") { "Stop (⌘.)" } else { "Stop (Ctrl+.)" })
                // Its key to the left while ⌘ is held (key_hints.rs).
                .relative()
                .children(crate::key_hints::hint(self.app.upgrade().and_then(|a| a.read(cx).shortcut_display(threadbear_core::shortcuts::ShortcutCommand::Stop)), crate::key_hints::Placement::Before))
                .into_any_element();
        }
        let tip = if running { "Send (into this turn)" } else { "Send (Return)" };
        let (fg, bg) = if can_send { (cx.theme().background, cx.theme().foreground) } else { (style::faint_text(cx), style::send_faint_fill(cx)) };
        circle()
            .id("composer-send")
            .test_support()
            .aria_label("Send")
            .when(can_send, |s| s.cursor_pointer())
            .bg(bg)
            .child(Icon::from(IconName::ArrowUp).with_size(px(14.)).text_color(fg))
            .when(can_send, |s| s.on_click(cx.listener(|this, _: &ClickEvent, window, cx| this.send(window, cx))))
            .tip(tip)
            .into_any_element()
    }

    /// The text field, at the transcript's body size and line height so a
    /// message keeps its rhythm when it's sent (ComposerTextField).
    fn render_text_field(&self, creating: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let this = cx.entity().downgrade();
        let paste_target = this.clone();
        let hover_target = this.clone();
        let token_owner = this.clone();
        let thread_drop = this.clone();
        let file_drop = this.clone();
        let drop_target = this;
        div()
            .w_full()
            // Greyed and locked while a new thread is being made from it.
            .when(creating, |s| s.opacity(0.5))
            .capture_action(cx.listener(|this, action: &Enter, window, cx| {
                if action.shift && this.shift_return(window, cx) {
                    cx.stop_propagation();
                } else if !action.shift && this.palette_key(PaletteKey::Enter, window, cx) {
                    cx.stop_propagation();
                }
            }))
            // The palette's keys, while it's open (ComposerView.paletteKey);
            // otherwise the box has them as usual.
            .capture_action(cx.listener(|this, _: &MoveUp, window, cx| {
                if this.palette_key(PaletteKey::Up, window, cx) {
                    cx.stop_propagation();
                }
            }))
            .capture_action(cx.listener(|this, _: &MoveDown, window, cx| {
                if this.palette_key(PaletteKey::Down, window, cx) {
                    cx.stop_propagation();
                }
            }))
            .capture_action(cx.listener(|this, _: &IndentInline, window, cx| {
                if this.palette_key(PaletteKey::Tab, window, cx) {
                    cx.stop_propagation();
                }
            }))
            // Copying or cutting chips keeps what they stand for
            // (composer_inline.rs); words alone are the box's to copy.
            .capture_action(cx.listener(|this, _: &Copy, window, cx| {
                if this.copy_chips(false, window, cx) {
                    cx.stop_propagation();
                }
            }))
            .capture_action(cx.listener(|this, _: &Cut, window, cx| {
                if this.copy_chips(true, window, cx) {
                    cx.stop_propagation();
                }
            }))
            .capture_action(cx.listener(|this, _: &Escape, window, cx| {
                if this.palette_key(PaletteKey::Escape, window, cx) {
                    cx.stop_propagation();
                } else if this.editing.as_ref().is_some_and(|e| !e.sending) {
                    // Esc while editing a message leaves the edit.
                    this.cancel_editing(window, cx);
                    cx.stop_propagation();
                }
            }))
            .on_drop(move |paths: &ExternalPaths, window, cx| {
                if let Some(this) = drop_target.upgrade() {
                    this.update(cx, |this, cx| this.take_paths(paths.paths().to_vec(), window, cx));
                }
            })
            .on_drop(move |drag: &crate::composer_inline::DraggedThread, window, cx| {
                if let Some(this) = thread_drop.upgrade() {
                    this.update(cx, |this, cx| this.take_thread_drop(drag, window, cx));
                }
            })
            // A file or picture dragged from inside the app (file_drag.rs).
            .on_drop(move |file: &crate::file_drag::DraggedFile, window, cx| {
                if let (Some(this), Some(real)) = (file_drop.upgrade(), file.still_grabbed()) {
                    this.update(cx, |this, cx| this.take_paths(vec![real], window, cx));
                }
            })
            .child(
                bare_textarea(Textarea::new(&self.input))
                    .disabled(creating)
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
        if let Some(restore) = self.restore.take() {
            self.apply_restore(restore, window, cx);
        }
        if let Some((text, images)) = self.pending_box.take() {
            self.set_box(text, images, window, cx);
        }
        self.renumber_chips(cx);
        // With answers picked the box is for anything else; under asks with
        // none picked it's the way to answer in other words.
        let placeholder = if self.editing.is_some() { "Your message…" } else { self.card_placeholder(cx) };
        if self.shown_placeholder != placeholder {
            self.shown_placeholder = placeholder;
            self.input.update(cx, |s, cx| s.set_placeholder(placeholder, window, cx));
        }
        let running = self.pane().is_some_and(|p| p.read(cx).summary.busy());
        if !running {
            self.stopping = false;
        }
        let strip = self.render_strip(cx);
        let creating = self.creating(cx);
        let send_button = self.render_send_button(running, creating, cx);
        // Right-click it to send the message later (composer_send.rs).
        let send_button = self.with_send_menu(send_button, cx);
        let controls = self.render_controls(send_button, window, cx);
        // The box: the limit box and the open card, each over a line, then
        // the message.
        let limit = self.render_limit_box(cx);
        // An edit is a message of its own: the open card waits for after it.
        let card = if self.editing.is_some() { None } else { self.render_card(window, cx) };
        let editing_banner = self.render_editing_banner(cx);
        // With the card folded, its picks as chips over the words, so they
        // don't go with no sign of them (composer_answers.rs).
        let answer_chips = self.render_answer_chips(cx);
        let needs_you = self.open_card(cx).is_some_and(|c| !c.is_done());
        let palette = self.render_palette(window, cx);
        let width = self.card_width.clone();
        // The folder's actions, on shelves on the box's top edge, in from its
        // rounded corner (ActionChips), as wide as the box leaves them.
        let shelves = self.pane().and_then(|pane| {
            let summary = pane.read(cx).summary.clone();
            let icons = self.app.upgrade().map(|a| a.read(cx).sidebar.project_icons.clone()).unwrap_or_default();
            let room = (self.card_width.get() - style::zoomed(style::COMPOSER_RADIUS) * 2.).max(0.);
            crate::viewer::action_chips(&summary, room, &icons, window, cx)
        });
        let body = v_flex().w_full().gap(px(style::COMPOSER_GAP)).p(px(style::COMPOSER_PADDING)).children(editing_banner).children(answer_chips).when_some(strip, |s, strip| s.child(strip)).child(self.render_text_field(creating, cx)).child(controls);
        // The transcript's reading column: 780 wide at most, centred, with
        // the same side insets once the window is narrower.
        h_flex()
            .w_full()
            .justify_center()
            .px(px(style::SIDE_INSET))
            .pt(px(4.))
            .pb(px(20.))
            .child(
                v_flex()
                    .w_full()
                    .max_w(px(style::MAX_CONTENT_WIDTH))
                    // Overlapping the box's border, so the two read as one piece.
                    .when_some(shelves, |s, shelves| s.child(div().px(px(style::COMPOSER_RADIUS)).mb(gpui_kit::px(-0.5)).child(shelves)))
                    .child(
                div()
                    .relative()
                    .w_full()
                    .rounded(px(style::COMPOSER_RADIUS))
                    .bg(style::composer_fill(cx))
                    .when(!cx.theme().is_dark(), |s| s.shadow_md())
                    // The edge, drawn over the box rather than taking room in
                    // it, as the Swift card's strokeBorder is: what's inside
                    // sits its 14 in from the box's own edge, and the needs-you
                    // edge (1 pt in the accent, otherwise a 0.5 hairline)
                    // moves nothing. First, so the rest draws over it.
                    .child(div().absolute().inset_0().rounded(px(style::COMPOSER_RADIUS)).border(px(if needs_you { 1. } else { 0.5 })).border_color(if needs_you { style::accent(cx) } else { style::hairline(cx) }))
                    // Clicking the card's padding puts the cursor in the text.
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, _: &MouseDownEvent, window, cx| this.focus(window, cx)))
                    .children(limit)
                    .children(card)
                    .child(body)
                    // Measures the box for the palette, which sits on its top
                    // edge, and for the shelves, which get as many chips as fit.
                    .child(canvas(move |bounds, _, _| width.set(f32::from(bounds.size.width)), |_, _, _, _| {}).absolute().size_full())
                    .children(palette),
                    ),
            )
            .children(self.render_peek(cx))
    }
}

/// A text box with no frame and none of the kit's padding inside it, so its
/// text starts where the box does, as an NSTextView with no inset and no
/// line fragment padding does (ComposerTextField, the Something else box).
/// The kit's smallest size still pads 4 either side (its own 4, whatever the
/// text size), so the box reaches 4 out past each side to put the text back
/// on the edge.
pub(crate) fn bare_textarea(textarea: Textarea) -> Textarea {
    textarea.appearance(false).bordered(false).xsmall().mx(gpui_kit::px(-4.))
}

/// The token a chip draws from. The token's text is the mark the draft holds
/// (InlinePictures.mark); its label is only for accessibility and measuring.
pub(crate) fn picture_token(id: &SharedString) -> InlineToken {
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
    stage_private(&paste_base(), &format!("pasted-{:x}.{ext}", image.id), &image.bytes)
}

/// Where pasted pictures wait for the runner: the session's own runtime
/// folder on Linux (`XDG_RUNTIME_DIR`, which only this user can open), else
/// the temp folder (already this user's own on a Mac and Windows; `/tmp` on
/// a Linux without one, which everyone can list).
fn paste_base() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).filter(|d| d.is_absolute() && d.is_dir()).unwrap_or_else(std::env::temp_dir)
}

/// Writes `bytes` as `name` in a folder of its own under `base`: made fresh
/// with a random name and 0700, the file new with 0600, so on a shared
/// `/tmp` nobody else can read it, list it, or make the folder first and
/// have it written into theirs (Daybreak, 10 Nov 2026, finding 63). The
/// folder goes once the runner has its own copy (`import_picture`).
fn stage_private(base: &std::path::Path, name: &str, bytes: &[u8]) -> Option<PathBuf> {
    use std::io::Write as _;
    let mut builder = tempfile::Builder::new();
    builder.prefix("threadbear-paste-");
    // Without a mode the folder would take the umask's (0755 under 022).
    // Windows has none: a folder there takes the user's temp folder's access.
    #[cfg(unix)]
    builder.permissions(std::os::unix::fs::PermissionsExt::from_mode(0o700));
    let folder = builder.tempdir_in(base).ok()?.keep();
    let path = folder.join(name);
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let written = options.open(&path).and_then(|mut f| f.write_all(bytes));
    if written.is_err() {
        let _ = std::fs::remove_dir_all(&folder);
        return None;
    }
    STAGED.lock().unwrap_or_else(|e| e.into_inner()).push(folder);
    Some(path)
}

/// The folders `stage_private` made and hasn't removed: only these are
/// ever deleted, never a folder that just has the same kind of name (Codex
/// review, 10 Nov 2026: a picture dropped from `/tmp/threadbear-paste-x/`
/// would have taken that whole folder with it).
static STAGED: std::sync::Mutex<Vec<PathBuf>> = std::sync::Mutex::new(Vec::new());

/// A staged picture's folder, once the runner has copied it (or couldn't).
/// A dropped or pasted file's own folder is never touched.
fn unstage(path: &std::path::Path) {
    let Some(folder) = path.parent() else { return };
    let mut staged = STAGED.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(at) = staged.iter().position(|f| f == folder) {
        staged.remove(at);
        drop(staged);
        let _ = std::fs::remove_dir_all(folder);
    }
}

#[cfg(test)]
mod paste_tests {
    use super::{stage_private, unstage};
    use std::fs;

    /// Finding 63: under umask 022 the old folder was 0755 and the file
    /// 0644, in a fixed `/tmp/threadbear-paste` anyone could list or make
    /// first. Each paste now gets a fresh folder only its owner can open.
    #[cfg(unix)]
    #[test]
    fn a_pasted_picture_is_kept_private() {
        use std::os::unix::fs::PermissionsExt;
        let base = tempfile::tempdir().unwrap();
        let path = stage_private(base.path(), "pasted-1.png", b"png bytes").unwrap();
        let folder = path.parent().unwrap();
        assert_eq!(fs::metadata(folder).unwrap().permissions().mode() & 0o777, 0o700);
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
        assert_eq!(fs::read(&path).unwrap(), b"png bytes");
        // Not a name anyone can guess: the same picture twice goes to two folders.
        let again = stage_private(base.path(), "pasted-1.png", b"png bytes").unwrap();
        assert_ne!(again.parent(), path.parent());
        assert!(folder.file_name().unwrap().to_string_lossy().len() > "threadbear-paste-".len() + 4);
    }

    /// A folder someone else made first, under the old fixed name, is never
    /// written into.
    #[test]
    fn a_folder_made_first_isnt_used() {
        let base = tempfile::tempdir().unwrap();
        let planted = base.path().join("threadbear-paste");
        fs::create_dir(&planted).unwrap();
        let path = stage_private(base.path(), "pasted-2.png", b"x").unwrap();
        assert!(!path.starts_with(&planted));
        assert_eq!(fs::read_dir(&planted).unwrap().count(), 0);
    }

    /// Once the runner has its copy, the staged folder goes, and only a
    /// folder this made: a dropped file's own folder stays, even one with
    /// the same kind of name.
    #[test]
    fn the_staged_copy_goes_after_the_import() {
        let base = tempfile::tempdir().unwrap();
        let path = stage_private(base.path(), "pasted-3.png", b"x").unwrap();
        let folder = path.parent().unwrap().to_owned();
        unstage(&path);
        assert!(!folder.exists());
        for name in ["Desktop", "threadbear-paste-project"] {
            let dropped_dir = base.path().join(name);
            fs::create_dir(&dropped_dir).unwrap();
            fs::write(dropped_dir.join("other.txt"), b"keep").unwrap();
            let dropped = dropped_dir.join("shot.png");
            fs::write(&dropped, b"x").unwrap();
            unstage(&dropped);
            assert!(dropped.exists() && dropped_dir.join("other.txt").exists(), "{name}");
        }
    }
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
