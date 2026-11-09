// One thread: its transcript (transcript.rs folds the events), its cards
// and the composer. Reads the newest 500 events, then subscribes from the
// last of them, as the Mac app does.
use crate::inbox::ThreadSummary;
use crate::transcript::{duration, question_answers, Item, Step, ThreadEvents};
use crate::{call, RT};
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit::component::message_scroller::{MessageScroller, MessageScrollerState};
use gpui_kit::component::text::TextView;
use gpui_kit::component::{h_flex, v_flex, ActiveTheme as _, Disableable as _, Sizable as _, StyledExt as _};
use gpui_kit::prelude::*;
use gpui_kit::*;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use threadbear_client::runner::Client;

pub struct ThreadPane {
    client: Arc<Client>,
    summary: ThreadSummary,
    events: ThreadEvents,
    items: Vec<Item>,
    loaded: bool,
    error: Option<String>,
    scroller: Entity<MessageScrollerState>,
    input: Entity<TextareaState>,
    open_steps: HashSet<String>,
    chosen: HashMap<String, HashMap<usize, Vec<String>>>,
    _subs: Vec<Subscription>,
}

impl ThreadPane {
    pub fn new(client: Arc<Client>, summary: ThreadSummary, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextareaState::new(window, cx).placeholder("Message").auto_grow(2, 8).submit_on_enter(true));
        let sub = cx.subscribe_in(&input, window, |this, input, ev: &InputEvent, window, cx| {
            if let InputEvent::PressEnter { shift: false, .. } = ev {
                let text = input.read(cx).value().trim().to_string();
                if !text.is_empty() {
                    input.update(cx, |s, cx| s.set_value("", window, cx));
                    this.send(text, None, cx);
                }
            }
        });
        let mut pane = ThreadPane {
            client,
            summary,
            events: ThreadEvents::default(),
            items: vec![],
            loaded: false,
            error: None,
            scroller: cx.new(|cx| MessageScrollerState::new(0, cx)),
            input,
            open_steps: HashSet::new(),
            chosen: HashMap::new(),
            _subs: vec![sub],
        };
        pane.load(cx);
        pane
    }

    pub fn load(&mut self, cx: &mut Context<Self>) {
        let client = self.client.clone();
        let id = self.summary.id.clone();
        cx.spawn(async move |this, cx| {
            let page = call(client.clone(), "threads.events", json!({ "threadId": id, "limit": 500 })).await;
            let page = match page {
                Ok(p) => p,
                Err(e) => {
                    this.update(cx, |this, cx| {
                        this.error = Some(e.message);
                        cx.notify();
                    })
                    .ok();
                    return;
                }
            };
            let sub = call(client.clone(), "threads.subscribe", json!({ "threadId": id, "afterSeq": page["lastSeq"] })).await;
            this.update(cx, |this, cx| {
                let mut events = ThreadEvents::default();
                events.add(page["events"].as_array().map(Vec::as_slice).unwrap_or(&[]), &[]);
                if let Ok(sub) = &sub {
                    events.add(sub["events"].as_array().map(Vec::as_slice).unwrap_or(&[]), sub["inProgress"].as_array().map(Vec::as_slice).unwrap_or(&[]));
                }
                this.events = events;
                this.loaded = true;
                this.refold(cx);
                this.mark_seen();
            })
            .ok();
        })
        .detach();
    }

    // Opening it reads it, as on the Mac.
    fn mark_seen(&self) {
        if self.summary.unread() {
            let client = self.client.clone();
            let params = json!({ "threadId": self.summary.id, "seq": self.summary.last_seq });
            RT.spawn(async move { client.call("threads.markSeen", params).await });
        }
    }

    pub fn unsubscribe(&self) {
        let client = self.client.clone();
        let params = json!({ "threadId": self.summary.id });
        RT.spawn(async move { client.call("threads.unsubscribe", params).await });
    }

    pub fn summary_changed(&mut self, t: &ThreadSummary, cx: &mut Context<Self>) {
        if t.id == self.summary.id {
            self.summary = t.clone();
            if self.loaded {
                self.mark_seen();
            }
            cx.notify();
        }
    }

    pub fn event(&mut self, params: &Value, cx: &mut Context<Self>) {
        if params["threadId"].as_str() == Some(self.summary.id.as_str()) {
            self.events.add(std::slice::from_ref(&params["event"]), &[]);
            self.refold(cx);
        }
    }

    fn refold(&mut self, cx: &mut Context<Self>) {
        let old = self.items.len();
        self.items = self.events.fold();
        let n = self.items.len();
        self.scroller.update(cx, |s, cx| {
            if n == old {
                // Text streaming into a row already there: heights are cached.
                s.remeasure(cx);
            } else {
                s.reset(n, cx);
            }
        });
        cx.notify();
    }

    fn send(&mut self, text: String, answers: Option<Value>, cx: &mut Context<Self>) {
        let mut params = json!({ "threadId": self.summary.id, "text": text });
        if let Some(a) = answers {
            params["answers"] = a;
        }
        self.request("threads.send", params, cx);
    }

    fn request(&mut self, method: &'static str, params: Value, cx: &mut Context<Self>) {
        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let res = call(client, method, params).await;
            this.update(cx, |this, cx| {
                this.error = res.err().map(|e| e.message);
                this.scroller.update(cx, |s, cx| s.scroll_to_end(cx));
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn render_item(&mut self, ix: usize, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(item) = self.items.get(ix).cloned() else { return div().into_any_element() };
        let theme = cx.theme();
        let (muted_bg, muted, border, danger, warning, fg) = (theme.muted, theme.muted_foreground, theme.border, theme.danger, theme.warning, theme.foreground);
        let column = |el: Div| h_flex().w_full().justify_center().child(el.w_full().max_w(px(760.)));
        let el: AnyElement = match item {
            Item::User { text, .. } => column(h_flex().justify_end().child(div().max_w(relative(0.8)).px_3p5().py_2().rounded_2xl().bg(muted_bg).text_color(fg).child(text))).into_any_element(),
            Item::Text { item_id, text, streaming } => column(div().when(streaming, |s| s.opacity(0.85)).child(TextView::markdown(ElementId::Name(format!("md-{item_id}").into()), text))).into_any_element(),
            Item::Reasoning { text, .. } => column(div().text_sm().italic().text_color(muted).child(text)).into_any_element(),
            Item::Steps(steps) => column(self.render_steps(ix, steps, muted, danger, cx)).into_any_element(),
            Item::Note { text, error } => column(div().text_sm().text_center().text_color(if error { danger } else { muted }).child(text)).into_any_element(),
            Item::Approval { approval_id, title, options, decision } => {
                let mut card = card(border).child(div().font_semibold().child(format!("Allow {title}?")));
                match decision {
                    Some(d) => card = card.child(div().text_sm().text_color(muted).child(if d == "deny" { "Denied" } else if d == "expired" { "Expired" } else { "Allowed" })),
                    None => {
                        let mut row = h_flex().gap_2();
                        for (j, o) in options.into_iter().enumerate() {
                            let label = match o.as_str() {
                                "allow" => "Allow",
                                "allowAlways" => "Allow for this thread",
                                _ => "Deny",
                            };
                            let (id, thread) = (approval_id.clone(), self.summary.id.clone());
                            row = row.child(Button::new(("approve", ix * 10 + j)).small().label(label).when(o == "allow", |b| b.primary()).on_click(cx.listener(
                                move |this, _: &ClickEvent, _, cx| this.request("approvals.respond", json!({ "threadId": thread, "approvalId": id, "decision": o }), cx),
                            )));
                        }
                        card = card.border_color(warning).child(row);
                    }
                }
                column(card).into_any_element()
            }
            Item::Question { approval_id, questions, decision, message } => {
                let mut card = card(if decision.is_some() { border } else { warning });
                let chosen = self.chosen.get(&approval_id).cloned().unwrap_or_default();
                for (qi, q) in questions.iter().enumerate() {
                    card = card.child(div().font_semibold().child(q.question.clone()));
                    if decision.is_none() {
                        let mut row = h_flex().gap_2().flex_wrap();
                        for (oi, o) in q.options.iter().enumerate() {
                            let picked = chosen.get(&qi).is_some_and(|c| c.contains(o));
                            let (id, label, multi) = (approval_id.clone(), o.clone(), q.multi);
                            row = row.child(Button::new(("pick", ix * 1000 + qi * 50 + oi)).small().label(o.clone()).when(picked, |b| b.primary()).on_click(cx.listener(
                                move |this, _: &ClickEvent, _, cx| {
                                    let picks = this.chosen.entry(id.clone()).or_default().entry(qi).or_default();
                                    if multi {
                                        if let Some(at) = picks.iter().position(|p| *p == label) {
                                            picks.remove(at);
                                        } else {
                                            picks.push(label.clone());
                                        }
                                    } else {
                                        *picks = vec![label.clone()];
                                    }
                                    cx.notify();
                                },
                            )));
                        }
                        card = card.child(row);
                    }
                }
                if decision.is_some() {
                    card = card.child(div().text_sm().text_color(muted).child(message.unwrap_or_else(|| "Answered".into())));
                } else {
                    let (answers, line) = question_answers(&questions, &chosen);
                    let (id, thread) = (approval_id.clone(), self.summary.id.clone());
                    card = card.child(h_flex().child(Button::new(("answer", ix)).small().primary().label("Answer").disabled(line.is_empty()).on_click(cx.listener(
                        move |this, _: &ClickEvent, _, cx| {
                            this.request("approvals.respond", json!({ "threadId": thread, "approvalId": id, "decision": "answered", "answers": answers, "message": line }), cx)
                        },
                    ))));
                }
                column(card).into_any_element()
            }
            Item::TurnEnd { seq, outcome, duration_ms, asks, error } => {
                let open = self.summary.asks_seq == Some(seq);
                let mut col = v_flex().gap_1p5();
                if !asks.is_empty() {
                    let mut card = card(if open { warning } else { border });
                    for (ai, ask) in asks.into_iter().enumerate() {
                        let mut block = v_flex().gap_1().child(div().font_semibold().child(ask.action.clone()));
                        if let Some(why) = &ask.why {
                            block = block.child(div().text_sm().text_color(muted).child(TextView::markdown(("why", ix * 10 + ai), why.clone())));
                        }
                        if open && !ask.options.is_empty() {
                            let mut row = h_flex().gap_2().flex_wrap().pt_1();
                            for (oi, o) in ask.options.iter().enumerate() {
                                let text = format!("{}\n{}", ask.action, o);
                                let answers = json!([{ "action": ask.action, "answer": o }]);
                                row = row.child(Button::new(("ask", ix * 100 + ai * 10 + oi)).small().label(o.clone()).when(ask.suggested == Some(oi), |b| b.primary()).on_click(cx.listener(
                                    move |this, _: &ClickEvent, _, cx| this.send(text.clone(), Some(answers.clone()), cx),
                                )));
                            }
                            block = block.child(row);
                        }
                        card = card.child(block);
                    }
                    col = col.child(card);
                }
                let word = match outcome.as_str() {
                    "completed" => "Done",
                    "interrupted" => "Stopped",
                    _ => "Failed",
                };
                col = col.child(
                    h_flex()
                        .gap_1()
                        .text_xs()
                        .text_color(muted)
                        .child(format!("{word} in {}", duration(duration_ms)))
                        .when_some(error, |s, e| s.child(div().text_color(danger).child(format!("· {e}")))),
                );
                column(col).into_any_element()
            }
        };
        el
    }

    fn render_steps(&mut self, ix: usize, steps: Vec<Step>, muted: Hsla, danger: Hsla, cx: &mut Context<Self>) -> Div {
        let key = steps.first().map(|s| s.item_id.clone()).unwrap_or_default();
        let open = self.open_steps.contains(&key);
        let running = steps.iter().any(|s| s.ok.is_none());
        let failed = steps.iter().filter(|s| s.ok == Some(false)).count();
        let title = if steps.len() == 1 { steps[0].title.clone() } else { format!("{} steps", steps.len()) };
        let head = h_flex()
            .id(("steps", ix))
            .gap_1p5()
            .cursor_pointer()
            .text_sm()
            .text_color(muted)
            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                if !this.open_steps.remove(&key) {
                    this.open_steps.insert(key.clone());
                }
                this.scroller.update(cx, |s, cx| s.remeasure(cx));
                cx.notify();
            }))
            .child(if open { "▾" } else { "▸" })
            .child(div().truncate().child(title))
            .when(running, |s| s.child(div().text_xs().px_1p5().rounded_sm().bg(muted.opacity(0.15)).child("running")))
            .when(failed > 0, |s| s.child(div().text_xs().text_color(danger).child(format!("{failed} failed"))));
        let mut col = v_flex().gap_1().child(head);
        if open {
            let mut list = v_flex().gap_0p5().pl_3p5().border_l_2().border_color(muted.opacity(0.3)).text_sm();
            for s in steps {
                list = list.child(
                    h_flex()
                        .gap_2()
                        .text_color(if s.ok == Some(false) { danger } else { muted })
                        .child(div().w(px(60.)).flex_none().child(s.kind))
                        .child(div().truncate().child(s.title)),
                );
            }
            col = col.child(list);
        }
        col
    }
}

fn card(border: Hsla) -> Div {
    v_flex().gap_3().p_3p5().rounded_xl().border_1().border_color(border)
}

impl Render for ThreadPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (border, muted, danger, bg) = (theme.border, theme.muted_foreground, theme.danger, theme.background);
        let busy = self.summary.busy();
        let model = crate::inbox::model_label(&self.summary.provider, self.summary.model_in_use.as_deref().or(self.summary.model.as_deref()));
        let meta = format!("{} · {} · {}", crate::inbox::folder_name(&self.summary.cwd), model, self.summary.status);
        let thread = self.summary.id.clone();
        v_flex()
            .size_full()
            .child(
                v_flex()
                    .px_7()
                    .pt_3p5()
                    .pb_2p5()
                    .border_b_1()
                    .border_color(border)
                    .child(div().font_semibold().truncate().child(self.summary.title.clone()))
                    .child(div().text_xs().text_color(muted).child(meta)),
            )
            .when_some(self.error.clone(), |s, e| s.child(div().px_7().py_1().text_sm().text_color(danger).child(e)))
            .when(!self.loaded && self.error.is_none(), |s| s.child(div().p_4().text_sm().text_center().text_color(muted).child("Loading…")))
            .child(
                div().flex_1().min_h_0().child(
                    MessageScroller::new("transcript", self.scroller.clone(), cx.processor(|this, ix: usize, window, cx| this.render_item(ix, window, cx)))
                        .with_bottom_fade(bg)
                        .size_full(),
                ),
            )
            .when(busy, |s| s.child(h_flex().justify_center().child(div().w_full().max_w(px(760.)).text_sm().text_color(muted).child("Working…"))))
            .child(
                h_flex().justify_center().px_7().pt_3().pb_4().border_t_1().border_color(border).child(
                    v_flex()
                        .w_full()
                        .max_w(px(760.))
                        .gap_2()
                        .child(Textarea::new(&self.input).w_full())
                        .child(
                            h_flex()
                                .justify_end()
                                .gap_2()
                                .when(busy, |s| {
                                    s.child(Button::new("stop").small().label("Stop").tooltip("Stop the turn").on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                        this.request("threads.interrupt", json!({ "threadId": thread }), cx)
                                    })))
                                })
                                .child(Button::new("send").small().primary().label("Send").tooltip("Send (Enter)").on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                                    let text = this.input.read(cx).value().trim().to_string();
                                    if !text.is_empty() {
                                        this.input.update(cx, |s, cx| s.set_value("", window, cx));
                                        this.send(text, None, cx);
                                    }
                                }))),
                        ),
                ),
            )
    }
}
