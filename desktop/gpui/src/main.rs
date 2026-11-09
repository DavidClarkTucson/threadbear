// Threadbear as a native GPUI app: the runner connection (../client), drawn
// with GPUI and gpui-kit's components. GPUI has no tokio, so the
// connection runs on a runtime of its own and hands its events across on a
// channel; requests run there too and their results come back as tasks.
mod inbox;
mod thread;
mod transcript;

use gpui_kit::component::{h_flex, v_flex, ActiveTheme as _, StyledExt as _, Theme};
use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::prelude::*;
use gpui_kit::*;
use inbox::ThreadSummary;
use serde_json::{json, Value};
use std::collections::HashMap;
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

/// A request on the connection's runtime, awaited from GPUI's.
pub async fn call(client: Arc<Client>, method: &'static str, params: Value) -> Result<Value, RpcError> {
    RT.spawn(async move { client.call(method, params).await })
        .await
        .unwrap_or_else(|e| Err(RpcError { code: -1, message: e.to_string(), data: None }))
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

struct Threadbear {
    client: Arc<Client>,
    status: Status,
    threads: HashMap<String, ThreadSummary>,
    list_error: Option<String>,
    selected: Option<String>,
    pane: Option<Entity<ThreadPane>>,
    _events: Task<()>,
    _appearance: Subscription,
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
        Threadbear { client, status, threads: HashMap::new(), list_error: None, selected: None, pane: None, _events: events, _appearance: appearance }
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
            let res = call(client, "threads.list", json!({ "include": ["inbox", "snoozed"] })).await;
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
        }
        let client = self.client.clone();
        self.selected = Some(id);
        self.pane = Some(cx.new(|cx| ThreadPane::new(client, summary, window, cx)));
        cx.notify();
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
        let sections = inbox::sections(self.threads.values(), now);
        let conn = match self.status {
            Status::Connected { .. } => green,
            Status::Connecting { .. } => muted,
            Status::Disconnected { .. } => red,
        };
        let mut ix = 0usize;
        let mut list = v_flex().gap_0p5().px_2().pb_4();
        for section in sections {
            list = list.child(
                h_flex()
                    .gap_1()
                    .px_2()
                    .pt_3()
                    .pb_1()
                    .text_xs()
                    .font_semibold()
                    .text_color(muted)
                    .child(section.title)
                    .child(div().font_normal().child(section.rows.len().to_string())),
            );
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
        h_flex().size_full().items_start().child(self.render_sidebar(cx)).child(div().flex_1().min_w_0().h_full().child(main))
    }
}

fn main() {
    gpui_kit::application().with_assets(gpui_kit::assets::Assets).run(|cx| {
        gpui_kit::init(cx);
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
