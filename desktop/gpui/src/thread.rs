// One thread: its transcript (rows come from the reducer in threadbear-core),
// its cards and the composer. Reads the newest page of events, then subscribes
// from the last of them, as the Mac app's ThreadSession does.
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
use std::time::Duration;
use threadbear_client::runner::Client;
use threadbear_core::inbox::ThreadSummary;
use threadbear_core::reducer::{
    ApprovalOption, ApprovalText, BackgroundTask, Event, EventsPage, FeedAction, Pending, PendingMessage, Row, RowContent, RowDebug, ThreadFeed, ToolPresentation, TranscriptChange,
    TranscriptModel, TurnOutcome,
};

/// Events asked for when a thread opens, as ThreadSession.eventsPage.
const EVENTS_PAGE: u64 = 500;
/// Streamed text arrives faster than it's worth drawing: flushed at most this
/// often, as ThreadSession does (30 Hz).
const FLUSH_EVERY: Duration = Duration::from_millis(33);

pub struct ThreadPane {
    client: Arc<Client>,
    summary: ThreadSummary,
    feed: ThreadFeed,
    loaded: bool,
    error: Option<String>,
    scroller: Entity<MessageScrollerState>,
    input: Entity<TextareaState>,
    /// Tool and thinking rows opened, by row id. Groups keep theirs in the model.
    open_rows: HashSet<String>,
    chosen: HashMap<String, HashMap<usize, Vec<String>>>,
    pending: Vec<PendingMessage>,
    next_pending: u64,
    flush_queued: bool,
    _subs: Vec<Subscription>,
}

impl ThreadPane {
    pub fn new(client: Arc<Client>, summary: ThreadSummary, draft: String, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| TextareaState::new(window, cx).placeholder("Message").auto_grow(2, 8).submit_on_enter(true).default_value(draft));
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
            feed: ThreadFeed::new(TranscriptModel::new(None)),
            loaded: false,
            error: None,
            scroller: cx.new(|cx| MessageScrollerState::new(0, cx)),
            input,
            open_rows: HashSet::new(),
            chosen: HashMap::new(),
            pending: vec![],
            next_pending: 0,
            flush_queued: false,
            _subs: vec![sub],
        };
        pane.load(cx);
        pane
    }

    pub fn thread_id(&self) -> &str {
        &self.summary.id
    }

    /// What's typed and not yet sent.
    pub fn draft(&self, cx: &App) -> String {
        self.input.read(cx).value().to_string()
    }

    #[cfg(feature = "automation")]
    pub fn composer(&self) -> Entity<TextareaState> {
        self.input.clone()
    }

    pub fn stop(&mut self, cx: &mut Context<Self>) {
        if self.summary.busy() {
            self.request("threads.interrupt", json!({ "threadId": self.summary.id }), cx);
        }
    }

    fn background_tasks(&self) -> Vec<BackgroundTask> {
        self.summary.background.iter().flatten().filter_map(BackgroundTask::decode).collect()
    }

    pub fn load(&mut self, cx: &mut Context<Self>) {
        let client = self.client.clone();
        let id = self.summary.id.clone();
        let root = self.summary.cwd.clone();
        let background = self.background_tasks();
        cx.spawn(async move |this, cx| {
            let page = match call(client.clone(), "threads.events", json!({ "threadId": id, "limit": EVENTS_PAGE })).await {
                Ok(v) => EventsPage::decode(&v).unwrap_or_default(),
                Err(e) => {
                    this.update(cx, |this, cx| {
                        this.error = Some(e.message);
                        cx.notify();
                    })
                    .ok();
                    return;
                }
            };
            let after_seq = this
                .update(cx, |this, cx| {
                    let mut model = TranscriptModel::new(Some(root));
                    model.load(page.events, &page.in_progress);
                    model.known_start = page.at_start;
                    let _ = model.set_background(background);
                    this.feed = ThreadFeed::new(model);
                    this.feed.begin_subscribe();
                    this.loaded = true;
                    this.scroller.update(cx, |s, cx| s.reset(this.feed.model().rows().len(), cx));
                    this.mark_seen();
                    cx.notify();
                    this.feed.accepted_seq()
                })
                .unwrap_or(0);
            let sub = call(client, "threads.subscribe", json!({ "threadId": id, "afterSeq": after_seq })).await;
            this.update(cx, |this, cx| match sub.ok().and_then(|v| EventsPage::decode(&v)) {
                Some(page) => {
                    let action = this.feed.finish_subscribe(page.events, &page.in_progress);
                    this.handle(action, cx);
                }
                None => this.feed.catch_up_failed(),
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
                let change = self.feed.set_background(self.background_tasks());
                self.apply(change, cx);
            }
            cx.notify();
        }
    }

    pub fn event(&mut self, params: &Value, cx: &mut Context<Self>) {
        if params["threadId"].as_str() != Some(self.summary.id.as_str()) {
            return;
        }
        if let Some(event) = Event::decode(&params["event"]) {
            let action = self.feed.receive(event);
            self.handle(action, cx);
        }
    }

    fn handle(&mut self, action: FeedAction, cx: &mut Context<Self>) {
        match action {
            FeedAction::None => {}
            FeedAction::ScheduleFlush => self.schedule_flush(cx),
            FeedAction::NeedsCatchUp { after_seq } => self.catch_up(after_seq, cx),
        }
    }

    fn schedule_flush(&mut self, cx: &mut Context<Self>) {
        if self.flush_queued {
            return;
        }
        self.flush_queued = true;
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(FLUSH_EVERY).await;
            this.update(cx, |this, cx| {
                this.flush_queued = false;
                let change = this.feed.flush();
                this.apply(change, cx);
            })
            .ok();
        })
        .detach();
    }

    fn catch_up(&mut self, after_seq: i64, cx: &mut Context<Self>) {
        let client = self.client.clone();
        let id = self.summary.id.clone();
        cx.spawn(async move |this, cx| {
            let page = call(client, "threads.events", json!({ "threadId": id, "afterSeq": after_seq })).await;
            this.update(cx, |this, cx| match page.ok().and_then(|v| EventsPage::decode(&v)) {
                Some(page) => {
                    let action = this.feed.catch_up(page.events);
                    this.handle(action, cx);
                }
                None => this.feed.catch_up_failed(),
            })
            .ok();
        })
        .detach();
    }

    /// Tells the list what the reducer changed, so it keeps its place: rows
    /// gone first, from the bottom up, then new ones from the top down.
    fn apply(&mut self, change: TranscriptChange, cx: &mut Context<Self>) {
        self.pending = self.feed.model().pending().to_vec();
        if change.is_empty() {
            return;
        }
        self.scroller.update(cx, |s, cx| {
            for &i in change.removed.iter().rev() {
                s.splice(i..i + 1, 0, cx);
            }
            for &i in change.inserted.iter() {
                s.splice(i..i, 1, cx);
            }
            for &i in change.updated.iter() {
                s.remeasure_items(i..i + 1, cx);
            }
        });
        cx.notify();
    }

    fn send(&mut self, text: String, answers: Option<Value>, cx: &mut Context<Self>) {
        // Shown straight away and put right when the runner echoes it back.
        self.next_pending += 1;
        let message = PendingMessage::new(&format!("local-{}", self.next_pending), &text, self.feed.accepted_seq());
        self.pending.push(message);
        let change = self.feed.set_pending(self.pending.clone());
        self.apply(change, cx);
        self.scroller.update(cx, |s, cx| s.scroll_to_end(cx));

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

    fn toggle_row(&mut self, row: &Row, cx: &mut Context<Self>) {
        let open = !self.row_open(row);
        self.set_open(row, open, cx);
    }

    fn set_open(&mut self, row: &Row, open: bool, cx: &mut Context<Self>) {
        if matches!(row.content, RowContent::Group(_)) {
            let change = self.feed.set_expanded(&row.id, open);
            self.apply(change, cx);
        } else {
            if open {
                self.open_rows.insert(row.id.clone());
            } else {
                self.open_rows.remove(&row.id);
            }
            if let Some(i) = self.feed.model().index_of(&row.id) {
                self.scroller.update(cx, |s, cx| {
                    s.remeasure_items(i..i + 1, cx);
                });
            }
        }
        cx.notify();
    }

    fn row_open(&self, row: &Row) -> bool {
        match &row.content {
            RowContent::Group(group) => group.is_expanded,
            _ => self.open_rows.contains(&row.id),
        }
    }

    fn render_item(&mut self, ix: usize, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let Some(row) = self.feed.model().rows().get(ix).cloned() else { return div().into_any_element() };
        let theme = cx.theme();
        let (muted_bg, muted, border, danger, warning, fg, green) = (theme.muted, theme.muted_foreground, theme.border, theme.danger, theme.warning, theme.foreground, theme.success);
        let column = |el: Div| h_flex().w_full().justify_center().child(el.w_full().max_w(px(760.)));
        let note = |text: String, color: Hsla| column(div().text_sm().text_center().text_color(color).child(text)).into_any_element();
        let el: AnyElement = match &row.content {
            RowContent::User(message) if message.shows_as_note() => note(message.text.clone(), muted),
            RowContent::User(message) => {
                let bubble = div().max_w(relative(0.8)).px_3p5().py_2().rounded_2xl().bg(muted_bg).text_color(fg).when(message.pending.is_some(), |s| s.opacity(0.6)).child(message.text.clone());
                let queued = matches!(message.pending, Some(Pending::Queued));
                column(h_flex().justify_end().when(queued, |s| s.child(div().text_xs().text_color(muted).mr_2().child("Queued"))).child(bubble)).into_any_element()
            }
            RowContent::Assistant(item) => column(div().child(TextView::markdown(ElementId::Name(format!("md-{}", item.item_id).into()), item.text.clone()).selectable(true))).into_any_element(),
            RowContent::Reasoning(item) => {
                let open = self.row_open(&row);
                let label = if item.is_streaming { "Thinking" } else { "Thought" };
                let mut col = v_flex().gap_1().child(self.chevron_line(ix, &row, label.to_string(), muted, cx));
                if open {
                    col = col.child(div().pl_3p5().text_sm().italic().text_color(muted).child(item.text.clone()));
                }
                column(col).into_any_element()
            }
            RowContent::Tool(tool) => {
                let open = self.row_open(&row);
                let title = if tool.failed() { format!("{} (failed)", tool.title()) } else { tool.title() };
                let mut col = v_flex().gap_1().child(self.chevron_line(ix, &row, title, if tool.failed() { danger } else { muted }, cx));
                if open {
                    col = col.child(div().pl_3p5().text_sm().font_family("monospace").text_color(muted).child(ToolPresentation::output_preview(tool, 40)));
                }
                column(col).into_any_element()
            }
            RowContent::Images { images, .. } => {
                let mut grid = h_flex().gap_2().flex_wrap();
                for image in images {
                    grid = grid.child(img(std::path::PathBuf::from(&image.path)).max_w(px(320.)).max_h(px(240.)).rounded_lg());
                }
                column(grid).into_any_element()
            }
            RowContent::Approval(item) => {
                match &item.resolution {
                    Some(res) => note(format!("{} ({})", ApprovalText::title(&item.request), ApprovalText::decision(res.decision)), muted),
                    None => {
                        let mut card = card(warning).child(div().font_semibold().child(ApprovalText::title(&item.request)));
                        let target = ApprovalText::target(&item.request);
                        if !target.is_empty() {
                            card = card.child(div().text_sm().font_family("monospace").text_color(fg).child(ApprovalText::shown(&target)));
                        }
                        let mut buttons = h_flex().gap_2();
                        for (j, option) in item.request.options.iter().enumerate() {
                            let (label, decision) = match option {
                                ApprovalOption::Allow => ("Allow", "allow"),
                                ApprovalOption::AllowAlways => ("Always allow", "allowAlways"),
                                ApprovalOption::Deny => ("Deny", "deny"),
                            };
                            let (id, thread) = (item.request.approval_id.clone(), self.summary.id.clone());
                            buttons = buttons.child(Button::new(("approve", ix * 10 + j)).small().label(label).when(matches!(option, ApprovalOption::Allow), |b| b.primary()).on_click(cx.listener(
                                move |this, _: &ClickEvent, _, cx| this.request("approvals.respond", json!({ "threadId": thread, "approvalId": id, "decision": decision }), cx),
                            )));
                        }
                        column(card.child(buttons)).into_any_element()
                    }
                }
            }
            RowContent::Question(item) => {
                let settled = item.resolution.is_some();
                let mut card = card(if settled { border } else { warning });
                let id = item.request.approval_id.clone();
                let chosen = self.chosen.get(&id).cloned().unwrap_or_default();
                for (qi, q) in item.request.questions.iter().enumerate() {
                    card = card.child(div().font_semibold().child(q.question.clone()));
                    if !settled {
                        let mut buttons = h_flex().gap_2().flex_wrap();
                        for (oi, option) in q.options.iter().enumerate() {
                            let picked = chosen.get(&qi).is_some_and(|c| c.contains(&option.label));
                            let (id, label, multi) = (id.clone(), option.label.clone(), q.multi_select);
                            buttons = buttons.child(Button::new(("pick", ix * 1000 + qi * 50 + oi)).small().label(option.label.clone()).when(picked, |b| b.primary()).on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
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
                            })));
                        }
                        card = card.child(buttons);
                    }
                }
                if settled {
                    card = card.child(div().text_sm().text_color(muted).child(item.resolution.as_ref().and_then(|r| r.message.clone()).unwrap_or_else(|| "Answered".into())));
                } else {
                    let (answers, line) = question_answers(item, &chosen);
                    let thread = self.summary.id.clone();
                    card = card.child(h_flex().child(Button::new(("answer", ix)).small().primary().label("Send answer").disabled(line.is_empty()).on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.request("approvals.respond", json!({ "threadId": thread, "approvalId": id, "decision": "answered", "answers": answers, "message": line }), cx)
                    }))));
                }
                column(card).into_any_element()
            }
            RowContent::Secret(item) => match item.resolved_text() {
                Some(text) => note(text, muted),
                None => {
                    let (id, thread) = (item.request.approval_id.clone(), self.summary.id.clone());
                    column(card(warning).child(div().font_semibold().child(item.request.label.clone())).child(div().text_sm().text_color(muted).child(item.request.reason.clone())).child(
                        h_flex().child(Button::new(("decline", ix)).small().label("Decline").on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                            this.request("secrets.answer", json!({ "threadId": thread, "approvalId": id, "decline": true }), cx)
                        }))),
                    ))
                    .into_any_element()
                }
            },
            RowContent::Review(review) => column(card(border).child(div().font_semibold().child(review.title.clone())).child(div().text_sm().text_color(muted).child("Options to compare"))).into_any_element(),
            RowContent::Plan(_) | RowContent::Note(_) | RowContent::Gap(_) | RowContent::Background(_) | RowContent::Working(_) => note(RowDebug::text(&row), muted),
            RowContent::PlanCard(_) => column(card(border).child(div().text_sm().child(RowDebug::text(&row)))).into_any_element(),
            RowContent::Footer(done) => {
                let color = if done.outcome == TurnOutcome::Completed { muted } else { danger };
                column(div().text_xs().text_color(color).child(RowDebug::text(&row))).into_any_element()
            }
            RowContent::Asks(asks) if asks.is_done() => {
                let mut col = v_flex().gap_1().child(div().font_semibold().child("Done"));
                for next in &asks.next {
                    col = col.child(div().text_sm().text_color(muted).child(next.clone()));
                }
                column(card(border).child(col)).into_any_element()
            }
            RowContent::Asks(asks) => {
                let mut card = card(if asks.is_open { warning } else { border });
                for (ai, ask) in asks.asks.iter().enumerate() {
                    let mut block = v_flex().gap_1().child(div().font_semibold().child(ask.action.clone()));
                    if let Some(why) = &ask.why {
                        block = block.child(div().text_sm().text_color(muted).child(TextView::markdown(("why", ix * 10 + ai), why.clone())));
                    }
                    if asks.is_open && !ask.options.is_empty() {
                        let mut buttons = h_flex().gap_2().flex_wrap().pt_1();
                        for (oi, option) in ask.options.iter().enumerate() {
                            let text = format!("{}\n{}", ask.action, option);
                            let answers = json!([{ "action": ask.action, "answer": option }]);
                            buttons = buttons.child(Button::new(("ask", ix * 100 + ai * 10 + oi)).small().label(option.clone()).when(ask.suggested == Some(oi), |b| b.primary()).on_click(cx.listener(
                                move |this, _: &ClickEvent, _, cx| this.send(text.clone(), Some(answers.clone()), cx),
                            )));
                        }
                        block = block.child(buttons);
                    }
                    card = card.child(block);
                }
                column(card).into_any_element()
            }
            RowContent::Error { message, .. } => note(message.clone(), danger),
            RowContent::Group(group) => {
                let color = if group.failed_count > 0 { danger } else { muted };
                column(v_flex().child(self.chevron_line(ix, &row, group.summary.clone(), color, cx))).into_any_element()
            }
            RowContent::Changes(changes) => {
                let mut card = card(border).child(div().font_semibold().text_sm().child(changes.title()));
                for file in &changes.files {
                    card = card.child(h_flex().gap_2().text_sm().child(div().flex_1().truncate().child(file.path.clone())).child(div().text_color(green).child(format!("+{}", file.added))).child(div().text_color(danger).child(format!("-{}", file.removed))));
                }
                column(card).into_any_element()
            }
        };
        // A row folded into a group sits indented under it.
        if row.in_group {
            div().pl_6().child(el).into_any_element()
        } else {
            el
        }
    }

    fn chevron_line(&self, ix: usize, row: &Row, title: String, color: Hsla, cx: &mut Context<Self>) -> Stateful<Div> {
        let open = self.row_open(row);
        let row = row.clone();
        h_flex()
            .id(("line", ix))
            .gap_1p5()
            .cursor_pointer()
            .text_sm()
            .text_color(color)
            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| this.toggle_row(&row, cx)))
            .child(if open { "▾" } else { "▸" })
            .child(div().truncate().child(title))
    }
}

/// approvals.respond's answers for a question card, as the Mac app makes
/// them: each question's picks joined with ", ", and the whole lot in one
/// line, in the order asked.
fn question_answers(item: &threadbear_core::reducer::QuestionItem, chosen: &HashMap<usize, Vec<String>>) -> (serde_json::Map<String, Value>, String) {
    let mut answers = serde_json::Map::new();
    let mut line = vec![];
    for (i, q) in item.request.questions.iter().enumerate() {
        let picks: Vec<&str> = q.options.iter().filter(|o| chosen.get(&i).is_some_and(|c| c.contains(&o.label))).map(|o| o.label.as_str()).collect();
        if !picks.is_empty() {
            let joined = picks.join(", ");
            answers.insert(q.question.clone(), Value::String(joined.clone()));
            line.push(joined);
        }
    }
    (answers, line.join("; "))
}

fn card(border: Hsla) -> Div {
    v_flex().gap_3().p_3p5().rounded_xl().border_1().border_color(border)
}

#[cfg(feature = "automation")]
impl ThreadPane {
    /// The rows as the Swift app's dump gives them.
    pub fn dump(&self) -> Value {
        let rows: Vec<Value> = self
            .feed
            .model()
            .rows()
            .iter()
            .map(|row| {
                let mut out = json!({ "id": row.id, "kind": row.kind().as_str(), "text": RowDebug::excerpt(row, 100), "inGroup": row.in_group, "seq": row.seq });
                if let Some(turn) = &row.turn_id {
                    out["turn"] = json!(turn);
                }
                if matches!(row.content, RowContent::Group(_) | RowContent::Tool(_) | RowContent::Reasoning(_)) {
                    out["expanded"] = json!(self.row_open(row));
                }
                out
            })
            .collect();
        json!({ "threadId": self.summary.id, "state": if self.loaded { "loaded" } else { "loading" }, "atBottom": true, "hasMoreAbove": self.feed.model().has_older_events(), "rows": rows })
    }

    pub fn set_row_open(&mut self, row: &str, open: bool, cx: &mut Context<Self>) -> Result<(), String> {
        let found = self.feed.model().rows().iter().find(|r| r.id.starts_with(row)).cloned();
        let Some(found) = found else { return Err(format!("no row starting {row:?}")) };
        self.set_open(&found, open, cx);
        Ok(())
    }

    /// Every event the thread has, for the long-thread check.
    pub fn load_all_events(&mut self, cx: &mut Context<Self>) {
        let client = self.client.clone();
        let id = self.summary.id.clone();
        let root = self.summary.cwd.clone();
        cx.spawn(async move |this, cx| {
            let page = call(client, "threads.events", json!({ "threadId": id, "limit": 1000000 })).await;
            this.update(cx, |this, cx| {
                if let Some(page) = page.ok().and_then(|v| EventsPage::decode(&v)) {
                    let mut model = TranscriptModel::new(Some(root));
                    model.load(page.events, &page.in_progress);
                    this.feed = ThreadFeed::new(model);
                    this.scroller.update(cx, |s, cx| s.reset(this.feed.model().rows().len(), cx));
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }
}

impl Render for ThreadPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (border, muted, danger, bg) = (theme.border, theme.muted_foreground, theme.danger, theme.background);
        let busy = self.summary.busy();
        let model = threadbear_core::inbox::model_label(&self.summary.provider, self.summary.model_in_use.as_deref().or(self.summary.model.as_deref()));
        let meta = format!("{} · {} · {}", threadbear_core::inbox::folder_name(&self.summary.cwd), model, self.summary.status);
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
                                .when(busy, |s| s.child(Button::new("stop").small().label("Stop").tooltip("Stop the turn").on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.stop(cx)))))
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
