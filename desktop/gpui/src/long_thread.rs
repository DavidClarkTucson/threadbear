// The long thread card above the message box (protocol "Long threads";
// Swift `LongThreadOffer`): Compact, Continue in a new thread and Not now
// once the runner says a thread's conversation is long. What it shows is
// threadbear_core::long_thread; this draws it and sends its clicks. In a file
// of its own so the feature stays in one place for a fork.
use crate::app::Threadbear;
use crate::call;
use crate::spinner::Spinner;
use crate::style::{self, px};
use gpui_kit::base::TestSupportExt as _;
use gpui_kit::component::{h_flex, Sizable as _};
use gpui_kit::prelude::*;
use gpui_kit::*;
use serde_json::json;
use threadbear_core::long_thread::LongThread;

/// `tipped!` in header.rs: the tooltip and the label scripts find it by.
macro_rules! tipped {
    ($el:expr, $tip:expr $(,)?) => {{
        let tip_text: SharedString = $tip.into();
        let shown = tip_text.clone();
        crate::tips::Tip::tip(crate::a11y::A11yRole::a11y_role($el.test_support(), gpui_kit::Role::Button).aria_label(tip_text), shown)
    }};
}

/// Which of the card's two buttons is waiting on the runner, by thread
/// (Swift `LongThreadBusy`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LongBusy {
    Compact,
    Continue,
}

impl Threadbear {
    /// The card the open thread shows, if any.
    pub(crate) fn long_thread_card(&self, thread_id: &str) -> Option<LongThread> {
        let thread = self.threads.get(thread_id).filter(|_| self.selected.as_deref() == Some(thread_id))?;
        LongThread::card(thread, self.new_thread.prefs.long_thread_dismissals.get(thread_id))
    }

    /// The card's button as it reads now, for the automation dump.
    pub(crate) fn long_thread_line(&self, thread_id: &str) -> Option<String> {
        self.long_thread_card(thread_id).map(|card| card.message())
    }

    /// The card, or None when the open thread isn't long.
    pub(crate) fn long_thread_offer(app: &Entity<Threadbear>, thread_id: &str, cx: &App) -> Option<AnyElement> {
        let t = app.read(cx);
        let card = t.long_thread_card(thread_id)?;
        let busy = t.long_busy.get(thread_id).copied();
        let tokens = t.threads.get(thread_id).and_then(|s| s.context_tokens);
        Some(render(app.downgrade(), thread_id.to_owned(), card, busy, tokens, cx))
    }

    /// Compact: sends `/compact`, the button spinning until the runner takes it.
    pub(crate) fn compact_long_thread(&mut self, thread: String, cx: &mut Context<Self>) {
        let runner_id = self.threads.get(&thread).and_then(|t| t.runner_id.clone());
        self.long_call(thread.clone(), LongBusy::Compact, "threads.send", json!({ "threadId": thread, "text": "/compact", "runnerId": runner_id }), cx);
    }

    /// Continue in a new thread (`threads.continueFresh`): the runner asks
    /// the agent for a handoff and starts the next thread with it.
    pub(crate) fn continue_long_thread(&mut self, thread: String, cx: &mut Context<Self>) {
        let runner_id = self.threads.get(&thread).and_then(|t| t.runner_id.clone());
        self.long_call(thread.clone(), LongBusy::Continue, "threads.continueFresh", json!({ "threadId": thread, "runnerId": runner_id }), cx);
    }

    fn long_call(&mut self, thread: String, busy: LongBusy, method: &'static str, params: serde_json::Value, cx: &mut Context<Self>) {
        if self.long_busy.contains_key(&thread) {
            return;
        }
        self.long_busy.insert(thread.clone(), busy);
        cx.notify();
        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let result = call(client, method, params).await;
            this.update(cx, |this, cx| {
                this.long_busy.remove(&thread);
                if let Err(e) = result {
                    this.show_error(e.message, cx);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    /// Not now: hides the card for this thread until it's longer still, or
    /// a compact brings it back down and it grows again. Kept on this Mac.
    pub(crate) fn dismiss_long_thread(&mut self, thread: String, card: &LongThread, tokens: Option<i64>, cx: &mut Context<Self>) {
        self.new_thread.prefs.long_thread_dismissals.dismiss(&thread, card.dismissal(tokens));
        self.new_thread.save();
        cx.notify();
    }
}

fn render(app: WeakEntity<Threadbear>, id: String, card: LongThread, busy: Option<LongBusy>, tokens: Option<i64>, cx: &App) -> AnyElement {
    let (hairline, fill, secondary) = (style::hairline(cx), style::card_background(cx), style::secondary_text(cx));
    let button = |key: &'static str, label: &'static str, tip: &'static str, idle: bool| {
        tipped!(div().id(key).flex_none().px(px(10.)).py(px(4.)).rounded(px(6.)).bg(style::primary_opacity(cx, 0.1)).when(!idle, |s| s.opacity(0.5)), tip).child(label)
    };
    let on = |app: &WeakEntity<Threadbear>, f: fn(&mut Threadbear, String, &mut Context<Threadbear>), id: &String| {
        let (app, id) = (app.clone(), id.clone());
        move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
            if let Some(app) = app.upgrade() {
                app.update(cx, |this, cx| f(this, id.clone(), cx));
            }
        }
    };
    let not_now = {
        let (app, id, card) = (app.clone(), id.clone(), card.clone());
        button("long-notnow", "Not now", "Hide this for this thread until it gets longer still", true).cursor_pointer().on_click(move |_: &ClickEvent, _, cx| {
            if let Some(app) = app.upgrade() {
                app.update(cx, |this, cx| this.dismiss_long_thread(id.clone(), &card, tokens, cx));
            }
        })
    };
    let buttons = h_flex().flex_none().gap(px(8.)).items_center();
    let buttons = match &card {
        LongThread::Suggest { .. } => {
            let idle = busy.is_none();
            let compact = button(
                "long-compact",
                if busy == Some(LongBusy::Compact) { "Compacting…" } else { "Compact" },
                "Send /compact: the agent sums up the conversation so far and carries on from the summary",
                idle,
            );
            let carry = button(
                "long-continue",
                if busy == Some(LongBusy::Continue) { "Starting…" } else { "Continue in a new thread" },
                "Ask the agent for a short handoff, then start a new thread with it in this folder, on the same model and pool",
                idle,
            );
            buttons
                .child(compact.when(idle, |s| s.cursor_pointer().on_click(on(&app, Threadbear::compact_long_thread, &id))))
                .child(carry.when(idle, |s| s.cursor_pointer().on_click(on(&app, Threadbear::continue_long_thread, &id))))
                .child(not_now)
        }
        LongThread::Writing => buttons.child(Spinner::new().icon(gpui_kit::assets::IconName::LoaderCircle).color(secondary).with_size(px(12.))),
        LongThread::Continued(next) => {
            let (app, next_id) = (app.clone(), next.thread_id.clone());
            buttons
                .child(button("long-open", "Open", "Go to the thread this one carried on in", true).cursor_pointer().on_click(move |_: &ClickEvent, window, cx| {
                    if let Some(app) = app.upgrade() {
                        app.update(cx, |this, cx| this.open(next_id.clone(), window, cx));
                    }
                }))
                .child(not_now)
        }
    };
    h_flex()
        .flex_none()
        .w_full()
        .flex_wrap()
        .mb(px(8.))
        .p(px(12.))
        .gap(px(12.))
        .items_center()
        .rounded(px(12.))
        .bg(fill)
        .border_1()
        .border_color(hairline)
        .text_size(px(style::size::LABEL))
        .child(div().flex_1().min_w(px(240.)).text_color(secondary).child(card.message()))
        .child(buttons)
        .into_any_element()
}
