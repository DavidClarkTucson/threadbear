// Threadbear as a native GPUI app: the runner connection (../client), drawn
// with GPUI and gpui-kit's components. GPUI has no tokio, so the
// connection runs on a runtime of its own and hands its events across on a
// channel; requests run there too and their results come back as tasks.
#[cfg(feature = "automation")]
mod automation;
mod thread;

use gpui_kit::component::{h_flex, v_flex, ActiveTheme as _, Sizable as _, StyledExt as _, Theme};
use gpui_kit::component::input::{Input, InputEvent, InputState, TextareaState};
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::prelude::*;
use gpui_kit::*;
use threadbear_core::inbox::{self, ThreadSummary};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};
use thread::ThreadPane;
use threadbear_client::config::RunnerConfig;
use threadbear_client::runner::{Client, RpcError, Sink, Status};
use tokio::sync::mpsc;

pub static RT: LazyLock<tokio::runtime::Runtime> =
    LazyLock::new(|| tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().expect("a tokio runtime"));

pub enum Seen {
    Status(Status),
    Note(String, Value),
}

struct ChannelSink(mpsc::UnboundedSender<Seen>);

impl Sink for ChannelSink {
    fn status(&self, status: &Status) {
        let _ = self.0.send(Seen::Status(status.clone()));
    }
    fn notification(&self, method: &str, params: &Value) {
        let _ = self.0.send(Seen::Note(method.to_string(), params.clone()));
    }
}

actions!(threadbear, [StopTurn, ArchiveThread]);

/// A request on the connection's runtime, awaited from GPUI's.
pub async fn call(client: Arc<Client>, method: impl Into<String>, params: Value) -> Result<Value, RpcError> {
    let method = method.into();
    RT.spawn(async move { client.call(&method, params).await })
        .await
        .unwrap_or_else(|e| Err(RpcError { code: -1, message: e.to_string(), data: None }))
}

use threadbear_core::inbox::iso_time;

pub fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

pub struct Threadbear {
    client: Arc<Client>,
    status: Status,
    threads: HashMap<String, ThreadSummary>,
    list_error: Option<String>,
    selected: Option<String>,
    pane: Option<Entity<ThreadPane>>,
    search: Entity<InputState>,
    /// The sections folded away under their headings, by key.
    folded: HashSet<&'static str>,
    /// What's typed for each thread that isn't open.
    drafts: HashMap<String, String>,
    _events: Task<()>,
    _subs: Vec<Subscription>,
}

impl Threadbear {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // gpui-kit starts in its light theme whatever the system says; follow
        // the system's light or dark, and keep following it when it changes.
        Theme::sync_system_appearance(Some(window), cx);
        let appearance = window.observe_window_appearance(|window, cx| Theme::sync_system_appearance(Some(window), cx));
        let (tx, mut rx) = mpsc::unbounded_channel();
        let client = Client::new(RunnerConfig::from_env(), ChannelSink(tx));
        RT.spawn(client.clone().run());
        let events = cx.spawn(async move |this, cx| {
            while let Some(seen) = rx.recv().await {
                if this.update(cx, |this, cx| this.on_seen(seen, cx)).is_err() {
                    break;
                }
            }
        });
        let status = client.status();
        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search threads"));
        // The filter is worked out at render, so a change only needs a redraw.
        let searching = cx.subscribe(&search, |_, _, ev: &InputEvent, cx| {
            if matches!(ev, InputEvent::Change) {
                cx.notify();
            }
        });
        #[cfg(feature = "automation")]
        automation::serve(cx.entity().downgrade(), window.window_handle(), cx);
        Threadbear {
            client,
            status,
            threads: HashMap::new(),
            list_error: None,
            selected: None,
            pane: None,
            search,
            folded: HashSet::from(["hibernating", "archived"]),
            drafts: HashMap::new(),
            _events: events,
            _subs: vec![appearance, searching],
        }
    }

    fn on_seen(&mut self, seen: Seen, cx: &mut Context<Self>) {
        match seen {
            Seen::Status(status) => {
                let connected = matches!(status, Status::Connected { .. });
                self.status = status;
                // Each time it's connected, the whole list again, and the
                // open thread read again: thread.updated only says what changed.
                if connected {
                    self.load_list(cx);
                    if let Some(pane) = &self.pane {
                        pane.update(cx, |p, cx| p.load(cx));
                    }
                }
            }
            Seen::Note(method, params) => match method.as_str() {
                "thread.updated" => {
                    if let Ok(t) = serde_json::from_value::<ThreadSummary>(params["thread"].clone()) {
                        if let Some(pane) = &self.pane {
                            pane.update(cx, |p, cx| p.summary_changed(&t, cx));
                        }
                        self.threads.insert(t.id.clone(), t);
                    }
                }
                "thread.deleted" => {
                    self.threads.remove(params["threadId"].as_str().unwrap_or(""));
                }
                "thread.event" => {
                    if let Some(pane) = &self.pane {
                        pane.update(cx, |p, cx| p.event(&params, cx));
                    }
                }
                _ => {}
            },
        }
        cx.notify();
    }

    fn load_list(&mut self, cx: &mut Context<Self>) {
        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let res = call(client, "threads.list", json!({ "include": ["inbox", "snoozed", "archived"] })).await;
            this.update(cx, |this, cx| {
                match res {
                    Ok(v) => {
                        let list: Vec<ThreadSummary> = serde_json::from_value(v["threads"].clone()).unwrap_or_default();
                        this.threads = list.into_iter().map(|t| (t.id.clone(), t)).collect();
                        this.list_error = None;
                    }
                    Err(e) => this.list_error = Some(e.message),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn open(&mut self, id: String, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected.as_deref() == Some(id.as_str()) {
            return;
        }
        let Some(summary) = self.threads.get(&id).cloned() else { return };
        if let Some(old) = &self.pane {
            old.read(cx).unsubscribe();
            let (thread, text) = (old.read(cx).thread_id().to_string(), old.read(cx).draft(cx));
            if text.is_empty() {
                self.drafts.remove(&thread);
            } else {
                self.drafts.insert(thread, text);
            }
        }
        let client = self.client.clone();
        let draft = self.drafts.get(&id).cloned().unwrap_or_default();
        self.selected = Some(id);
        self.pane = Some(cx.new(|cx| ThreadPane::new(client, summary, draft, window, cx)));
        cx.notify();
    }

    /// `threads.update` for the open thread, with the summary changed here
    /// first: anything clicked shows at once, and the runner's answer (or its
    /// thread.updated) puts it right.
    fn update_open(&mut self, change: impl FnOnce(&mut ThreadSummary), params: Value, cx: &mut Context<Self>) -> Result<(), String> {
        let Some(id) = self.selected.clone() else { return Err("no thread open".into()) };
        let before = self.threads.get(&id).cloned();
        if let Some(t) = self.threads.get_mut(&id) {
            change(t);
        }
        let mut params = params;
        params["threadId"] = json!(id);
        let client = self.client.clone();
        cx.spawn(async move |this, cx| {
            let res = call(client, "threads.update", params).await;
            this.update(cx, |this, cx| {
                if let Err(e) = res {
                    this.list_error = Some(e.message);
                    if let Some(before) = before {
                        this.threads.insert(before.id.clone(), before);
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
        Ok(())
    }

    fn archive_open(&mut self, cx: &mut Context<Self>) -> Result<(), String> {
        let now = iso_time(now_ms());
        self.update_open(move |t| t.archived_at = Some(now), json!({ "archived": true, "archivedBy": "mike" }), cx)
    }

    fn snooze_open(&mut self, seconds: i64, cx: &mut Context<Self>) -> Result<(), String> {
        let until = iso_time(now_ms() + seconds * 1000);
        let local = until.clone();
        self.update_open(move |t| t.snoozed_until = Some(local), json!({ "snoozedUntil": until }), cx)
    }

    fn stop_open(&mut self, cx: &mut Context<Self>) {
        if let Some(pane) = &self.pane {
            pane.update(cx, |p, cx| p.stop(cx));
        }
    }

    #[cfg(feature = "automation")]
    fn composer(&self, cx: &App) -> Option<Entity<TextareaState>> {
        self.pane.as_ref().map(|p| p.read(cx).composer())
    }

    #[cfg(feature = "automation")]
    fn set_row_open(&mut self, row: &str, open: bool, cx: &mut Context<Self>) -> Result<(), String> {
        let Some(pane) = &self.pane else { return Err("no thread open".into()) };
        pane.update(cx, |p, cx| p.set_row_open(row, open, cx))
    }

    #[cfg(feature = "automation")]
    fn load_all_events(&mut self, cx: &mut Context<Self>) {
        if let Some(pane) = &self.pane {
            pane.update(cx, |p, cx| p.load_all_events(cx));
        }
    }

    /// A thread named by the start of its id, as the Swift app's automation has it.
    #[cfg(feature = "automation")]
    fn thread_id_from(&self, start: &str) -> Result<String, String> {
        let mut found = self.threads.keys().filter(|id| id.starts_with(start));
        match (found.next(), found.next()) {
            (Some(id), None) => Ok(id.clone()),
            (None, _) => Err(format!("no thread starting {start:?}")),
            _ => Err(format!("more than one thread starts {start:?}")),
        }
    }

    /// The window as data, in the Swift app's shape (app/Sources/Threadbear/AppDump.swift).
    #[cfg(feature = "automation")]
    fn dump(&self, cx: &App) -> Value {
        let row = |t: &ThreadSummary| json!({ "id": t.id, "title": t.title, "status": t.status, "needsYou": t.needs_you, "unread": t.unread() });
        let now = now_ms();
        let shown = self.shown_sections(cx, now);
        let sections: Vec<Value> = shown.iter().map(|s| json!({ "name": s.title, "collapsed": self.folded.contains(s.key), "threads": s.rows.iter().map(row).collect::<Vec<_>>() })).collect();
        let snoozed = shown.iter().find(|s| s.key == "hibernating").map_or(0, |s| s.rows.len());
        let archived = shown.iter().find(|s| s.key == "archived").map_or(0, |s| s.rows.len());
        let connection = match self.status {
            Status::Connected { .. } => "connected",
            Status::Connecting { .. } => "connecting",
            Status::Disconnected { .. } => "lost",
        };
        let selected = self.selected.as_ref().and_then(|id| self.threads.get(id));
        let pane = self.pane.as_ref().map(|p| p.read(cx));
        json!({
            "connection": connection,
            "selection": selected.map(row),
            "sidebar": { "mode": "inbox", "sections": sections, "snoozed": snoozed, "archived": archived },
            "composer": { "draft": self.selected.clone().unwrap_or_else(|| "new".into()), "text": pane.map(|p| p.draft(cx)).unwrap_or_default() },
            "transcript": pane.map(|p| p.dump()).unwrap_or_else(|| json!({ "rows": [] })),
            "banner": Value::Null,
        })
    }

    /// The sections with the search applied, as they're drawn.
    fn shown_sections(&self, cx: &App, now: i64) -> Vec<inbox::Section> {
        let query = self.search.read(cx).value().trim().to_lowercase();
        inbox::sections(self.threads.values().filter(|t| query.is_empty() || inbox::matches(t, &query)), now)
    }

    fn connection_note(&self) -> Option<String> {
        match &self.status {
            Status::Connected { .. } => None,
            Status::Connecting { .. } => Some("Connecting to the runner…".into()),
            Status::Disconnected { reason, retry_in_ms, .. } => Some(format!("Can't reach the runner: {reason}. Trying again in {}s.", retry_in_ms.div_ceil(1000))),
        }
    }

    fn render_sidebar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (side, border, muted, accent_bg, fg) = (theme.sidebar, theme.sidebar_border, theme.muted_foreground, theme.sidebar_accent, theme.sidebar_foreground);
        let (orange, blue, green, red) = (theme.warning, theme.blue, theme.success, theme.danger);
        let now = now_ms();
        let sections = self.shown_sections(cx, now);
        let conn = match self.status {
            Status::Connected { .. } => green,
            Status::Connecting { .. } => muted,
            Status::Disconnected { .. } => red,
        };
        let mut ix = 0usize;
        let mut list = v_flex().gap_0p5().px_2().pb_4();
        for section in sections {
            let key = section.key;
            let folded = self.folded.contains(key);
            list = list.child(
                h_flex()
                    .id(key)
                    .gap_1()
                    .px_2()
                    .pt_3()
                    .pb_1()
                    .text_xs()
                    .font_semibold()
                    .text_color(muted)
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        if !this.folded.remove(key) {
                            this.folded.insert(key);
                        }
                        cx.notify();
                    }))
                    .child(section.title)
                    .child(div().font_normal().child(section.rows.len().to_string())),
            );
            if folded {
                continue;
            }
            if section.rows.is_empty() {
                list = list.child(div().px_2().text_sm().text_color(muted).child("Nothing here"));
            }
            for row in section.rows {
                ix += 1;
                let selected = self.selected.as_deref() == Some(row.id.as_str());
                let dot = if row.busy() { Some(blue.opacity(0.5)) } else if row.wants_you() { Some(orange) } else if row.unread() { Some(blue) } else { None };
                let id = row.id.clone();
                list = list.child(
                    h_flex()
                        .id(("row", ix))
                        .items_start()
                        .gap_2()
                        .px_2()
                        .py_1p5()
                        .rounded_md()
                        .cursor_pointer()
                        .hover(|s| s.bg(accent_bg.opacity(0.6)))
                        .when(selected, |s| s.bg(accent_bg))
                        .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| this.open(id.clone(), window, cx)))
                        .child(div().mt(px(7.)).size(px(7.)).flex_none().rounded_full().when_some(dot, |s, c| s.bg(c)))
                        .child(
                            v_flex()
                                .min_w_0()
                                .flex_1()
                                .child(div().text_sm().font_medium().text_color(fg).truncate().child(row.title.clone()))
                                .child(div().text_xs().text_color(muted).truncate().child(format!("{} · {}", row.place(), inbox::ago(row.activity_ms(), now)))),
                        ),
                );
            }
        }
        v_flex()
            .w(px(300.))
            .h_full()
            .flex_none()
            .bg(side)
            .border_r_1()
            .border_color(border)
            .child(
                h_flex()
                    .justify_between()
                    .px_4()
                    .pt_3()
                    .pb_2()
                    .child(div().font_semibold().text_color(fg).child("Threadbear (GPUI)"))
                    .child(div().size(px(8.)).rounded_full().bg(conn)),
            )
            .child(div().px_3().pb_2().child(Input::new(&self.search).small().cleanable(true)))
            .when_some(self.connection_note(), |s, note| s.child(div().mx_3().mb_2().p_2().rounded_md().bg(theme_muted(cx)).text_sm().text_color(muted).child(note)))
            .when_some(self.list_error.clone(), |s, e| s.child(div().mx_3().mb_2().text_sm().text_color(red).child(e)))
            .child(div().flex_1().min_h_0().child(list.size_full().overflow_y_scrollbar()))
    }
}

fn theme_muted(cx: &App) -> Hsla {
    cx.theme().muted
}

impl Render for Threadbear {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let main = match &self.pane {
            Some(pane) => div().size_full().child(pane.clone()),
            None => div().size_full().flex().items_center().justify_center().text_color(muted).child(if matches!(self.status, Status::Connected { .. }) {
                "No thread open".to_string()
            } else {
                self.connection_note().unwrap_or_default()
            }),
        };
        h_flex()
            .key_context("Threadbear")
            .on_action(cx.listener(|this, _: &StopTurn, _, cx| this.stop_open(cx)))
            .on_action(cx.listener(|this, _: &ArchiveThread, _, cx| {
                let _ = this.archive_open(cx);
            }))
            .size_full()
            .items_start()
            .child(self.render_sidebar(cx))
            .child(div().flex_1().min_w_0().h_full().child(main))
    }
}

fn main() {
    gpui_kit::application().with_assets(gpui_kit::assets::Assets).run(|cx| {
        gpui_kit::init(cx);
        // ⌘ on the Mac, Control elsewhere. KeyBinding::new doesn't take GPUI's
        // "secondary-", which needs to know the platform.
        let (stop, archive) = if cfg!(target_os = "macos") { ("cmd-.", "cmd-e") } else { ("ctrl-.", "ctrl-e") };
        cx.bind_keys([KeyBinding::new(stop, StopTurn, Some("Threadbear")), KeyBinding::new(archive, ArchiveThread, Some("Threadbear"))]);
        let bounds = Bounds::centered(None, size(px(1200.), px(800.)), cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions { title: Some("Threadbear (GPUI)".into()), ..Default::default() }),
                window_min_size: Some(size(px(720.), px(480.))),
                app_id: Some("threadbear".into()),
                ..Default::default()
            },
            cx,
            |window, cx| cx.new(|cx| Threadbear::new(window, cx)),
        )
        .expect("failed to open the window");
    });
}
