// The message box under a thread: a port of the Swift app's ComposerView.swift
// and the send button from ComposerControls.swift. One Composer per thread
// lives on the app (Threadbear.composers), so what's typed, its undo and its
// pictures stay with the thread while others are open.
use crate::app::Threadbear;
use crate::style;
use crate::thread::ThreadPane;
use gpui_kit::assets::IconName;
use gpui_kit::component::input::{Enter, InputEvent, Textarea, TextareaState};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{h_flex, v_flex, ActiveTheme as _, Icon, Sizable as _};
use gpui_kit::prelude::*;
use gpui_kit::*;
use serde_json::{json, Value};
use std::sync::Arc;
use threadbear_client::runner::Client;
use threadbear_core::composer::{self, BOX_MAX_LINES, BOX_MIN_LINES};
use threadbear_core::reducer::ImageRef;

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
    /// Stop was pressed and the turn hasn't ended yet: a spinner in its place.
    stopping: bool,
    /// A refused send's draft, to put back at the next render.
    restore: Option<String>,
    _subs: Vec<Subscription>,
}

impl Composer {
    pub fn new(thread_id: Option<String>, client: Arc<Client>, app: WeakEntity<Threadbear>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Two lines at least, ten at most, then it scrolls (ComposerChips.boxHeight).
        let input = cx.new(|cx| TextareaState::new(window, cx).placeholder("Message").auto_grow(BOX_MIN_LINES, BOX_MAX_LINES).submit_on_enter(true));
        let sub = cx.subscribe_in(&input, window, |this, _, ev: &InputEvent, window, cx| match ev {
            InputEvent::PressEnter { shift: false, .. } => this.return_pressed(window, cx),
            // The send button follows what's typed.
            InputEvent::Change => cx.notify(),
            _ => {}
        });
        Composer { thread_id, client, app, pane: WeakEntity::new_invalid(), input, stopping: false, restore: None, _subs: vec![sub] }
    }

    pub fn attach(&mut self, pane: WeakEntity<ThreadPane>) {
        self.pane = pane;
    }

    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        self.input.update(cx, |s, cx| s.focus(window, cx));
    }

    /// What the box holds, with each picture's chip as the marker it will go
    /// out as (the Swift dump's `composer.text`).
    pub fn draft_text(&self, cx: &App) -> String {
        composer::message_text(&self.input.read(cx).value(), 1)
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
        composer::can_send(&self.input.read(cx).value(), 0)
    }

    /// AppModel.send: the box empties at once and the message shows dimmed
    /// at the end of the transcript until the runner has it; it comes back
    /// if the runner doesn't take it.
    pub(crate) fn send(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let raw = self.input.read(cx).value().to_string();
        let text = composer::sent_text(&raw, 1);
        if text.trim().is_empty() {
            return;
        }
        let Some(pane) = self.pane() else { return };
        self.input.update(cx, |s, cx| s.set_value("", window, cx));
        let composer = cx.entity().downgrade();
        pane.update(cx, |p, cx| p.send_message(text.trim().to_string(), Vec::<ImageRef>::new(), Value::Null, raw, composer, cx));
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
    pub fn restore_draft(&mut self, draft: String, cx: &mut Context<Self>) {
        self.restore = Some(draft);
        cx.notify();
    }

    // ---- Drawing ----

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
        div()
            .w_full()
            .capture_action(cx.listener(|this, action: &Enter, window, cx| {
                if action.shift && this.shift_return(window, cx) {
                    cx.stop_propagation();
                }
            }))
            .child(Textarea::new(&self.input).appearance(false).bordered(false).text_size(px(style::size::COMPOSER)).line_height(px(style::size::BODY_LINE)))
    }
}

impl Render for Composer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(draft) = self.restore.take() {
            if self.input.read(cx).value().is_empty() {
                self.input.update(cx, |s, cx| s.set_value(draft, window, cx));
            }
        }
        let running = self.pane().is_some_and(|p| p.read(cx).summary.busy());
        if !running {
            self.stopping = false;
        }
        let padding = px(style::COMPOSER_PADDING);
        let body = v_flex().w_full().gap(px(style::COMPOSER_GAP)).p(padding).child(self.render_text_field(cx)).child(h_flex().w_full().h(px(style::size::SEND_BUTTON)).justify_end().child(self.render_send_button(running, cx)));
        // The transcript's reading column: 780 wide at most, centred, with
        // the same side insets once the window is narrower.
        h_flex().w_full().justify_center().px(px(style::SIDE_INSET)).pt(px(4.)).pb(px(20.)).child(
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
