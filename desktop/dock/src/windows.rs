// The panels taken out of the dock into windows of their own, kept in step
// with the layout's `floating` list. Ported from
// dock/Sources/BearDock/DockWindows.swift. Each window draws its panel's bar
// over the panel, and dragging the bar moves the window with the drop targets
// showing over the main window (see platform.rs for what that takes).
use crate::content::{DockOptions, SharedContent};
use crate::model::DockModel;
use crate::platform;
use crate::theme::DockTheme;
use gpui_kit::component::h_flex;
use gpui_kit::prelude::*;
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenu, PopupMenuItem};
use gpui_kit::*;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};
use threadbear_dock_layout as dl;
use threadbear_dock_layout::{DockFloat, PanelId};

/// The windows of panels taken out of the dock.
#[derive(Default)]
pub struct FloatingWindows {
    windows: HashMap<PanelId, WindowHandle<FloatingPanelView>>,
    /// A middle-click pressed on this panel's tab, closing it on release.
    pub(crate) middle_down: Option<PanelId>,
}

impl FloatingWindows {
    /// Opens a window for each floating panel without one, closes those
    /// whose panel went back into the dock or was closed. A panel with
    /// nothing to show right now (a thread's patch panel while another
    /// thread is on show) keeps its window, hidden, until it has something
    /// again (Swift's `sync`; hiding needs the raw window, so for now a
    /// window stays showing while its panel is unavailable).
    pub fn sync(&mut self, model: Entity<DockModel>, content: SharedContent, options: Rc<DockOptions>, cx: &mut App) {
        let floating = model.read(cx).controller.layout.floating.clone();
        let gone: Vec<PanelId> = self.windows.keys().filter(|p| !floating.iter().any(|f| &f.panel == *p)).cloned().collect();
        for panel in gone {
            if let Some(handle) = self.windows.remove(&panel) {
                let _ = handle.update(cx, |_, window, _| window.remove_window());
            }
        }
        let focus = model.read(cx).controller.focus_next.clone();
        for float in &floating {
            if self.windows.contains_key(&float.panel) {
                continue;
            }
            let handle = open_floating_window(float, focus.as_deref() == Some(float.panel.as_str()), model.clone(), content.clone(), options.clone(), cx);
            if let Some(handle) = handle {
                self.windows.insert(float.panel.clone(), handle);
            }
        }
        if focus.is_some_and(|p| floating.iter().any(|f| f.panel == p)) {
            model.update(cx, |m, _| m.controller.focus_next = None);
        }
    }
}

/// A frame saved on a display that's since gone comes back on the main one.
fn on_screen(frame: dl::Rect, cx: &App) -> dl::Rect {
    let displays = cx.displays();
    let visible = displays.iter().any(|d| {
        let b = d.bounds();
        let bx = b.origin.x.as_f32() as f64;
        let by = b.origin.y.as_f32() as f64;
        frame.x < bx + b.size.width.as_f32() as f64 && frame.max_x() > bx && frame.y < by + b.size.height.as_f32() as f64 && frame.max_y() > by
    });
    if visible || displays.is_empty() {
        return frame;
    }
    let Some(display) = cx.primary_display() else { return frame };
    let b = display.bounds();
    let cx_ = (b.origin.x.as_f32() as f64) + b.size.width.as_f32() as f64 / 2.;
    let cy = (b.origin.y.as_f32() as f64) + b.size.height.as_f32() as f64 / 2.;
    dl::Rect::new(cx_ - frame.width / 2., cy - frame.height / 2., frame.width, frame.height)
}

fn open_floating_window(float: &DockFloat, focus: bool, model: Entity<DockModel>, content: SharedContent, options: Rc<DockOptions>, cx: &mut App) -> Option<WindowHandle<FloatingPanelView>> {
    let frame = on_screen(float.frame, cx);
    let bounds = Bounds::new(point(px(frame.x as f32), px(frame.y as f32)), size(px(frame.width as f32), px(frame.height as f32)));
    let panel = float.panel.clone();
    let title: SharedString = content.title(&panel, cx);
    let opts = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        // The panel's own bar is the top of the window, as in the main one.
        titlebar: Some(TitlebarOptions { title: Some(title), appears_transparent: true, traffic_light_position: None }),
        focus,
        show: true,
        window_min_size: Some(size(px(240.), px(160.))),
        app_id: Some("threadbear".into()),
        ..Default::default()
    };
    let result = cx.open_window(opts, |window, cx| {
        if let Some(prepare) = &options.prepare_window {
            prepare(window, cx);
        }
        let view = cx.new(|cx| FloatingPanelView::new(panel, model, content, options, cx));
        // The red button closes the panel, as its × would, or puts back one
        // the app always keeps (Threadbear's thread).
        let on_close = view.clone();
        window.on_window_should_close(cx, move |_, cx| {
            let (panel, model, closable) = on_close.read_with(cx, |v, cx| (v.panel.clone(), v.model.clone(), v.content.closable(&v.panel, cx)));
            model.update(cx, |m, cx| if closable { m.close(&panel, cx) } else { m.put_back(&panel, cx) });
            // The sync closes the window.
            false
        });
        cx.new(|cx| gpui_kit::base::Root::new(view, window, cx))
    });
    match result {
        Ok(handle) => handle.downcast::<FloatingPanelView>(),
        Err(_) => None,
    }
}

/// A panel in a window of its own: its bar (title, buttons, menu) over it, the
/// bar dragging it back into the dock.
pub struct FloatingPanelView {
    pub panel: PanelId,
    pub model: Entity<DockModel>,
    pub content: SharedContent,
    options: Rc<DockOptions>,
    press: Option<FloatPress>,
    _subscriptions: Vec<Subscription>,
}

struct FloatPress {
    /// Escape, whatever has focus (see DockView::watch_escape).
    escape_watch: Option<Subscription>,
    start: Point<Pixels>,
    pressed_at: Instant,
    /// From the title (5 pt, no hold) or the bar's empty part (a hold).
    needs_hold: bool,
    mode: FloatMode,
    frame_at_start: dl::Rect,
    mouse_at_start: dl::Point,
    _timer: Option<Task<()>>,
}

#[derive(Clone, Copy, PartialEq)]
enum FloatMode {
    Waiting,
    /// The OS is moving it; the release ends it, and there's nothing to track.
    WindowMove,
    /// Moving it by hand, with the drop targets showing.
    Panel,
}

impl FloatingPanelView {
    fn new(panel: PanelId, model: Entity<DockModel>, content: SharedContent, options: Rc<DockOptions>, cx: &mut Context<Self>) -> Self {
        let observed = cx.observe(&model, |_, _, cx| cx.notify());
        Self { panel, model, content, options, press: None, _subscriptions: vec![observed] }
    }

    /// Where the window is, and where the pointer is on screen.
    fn window_frame(window: &Window) -> dl::Rect {
        let b = window.bounds();
        dl::Rect::new(b.origin.x.as_f32() as f64, b.origin.y.as_f32() as f64, b.size.width.as_f32() as f64, b.size.height.as_f32() as f64)
    }

    fn mouse_on_screen(position: Point<Pixels>, window: &Window) -> dl::Point {
        let b = window.bounds();
        // These windows have full-size content: the frame's origin is the content's.
        dl::Point::new((b.origin.x + position.x).as_f32() as f64, (b.origin.y + position.y).as_f32() as f64)
    }

    fn begin_press(&mut self, needs_hold: bool, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.press.is_some() {
            return;
        }
        let timer = needs_hold.then(|| {
            cx.spawn_in(window, async move |this, cx| {
                cx.background_executor().timer(Duration::from_secs_f64(dl::HOLD_SECONDS)).await;
                let _ = this.update_in(cx, |this, window, cx| this.hold_fired(window, cx));
            })
        });
        self.press = Some(FloatPress {
            escape_watch: None,
            start: event.position,
            pressed_at: Instant::now(),
            needs_hold,
            mode: FloatMode::Waiting,
            frame_at_start: Self::window_frame(window),
            mouse_at_start: Self::mouse_on_screen(event.position, window),
            _timer: timer,
        });
        cx.notify();
    }

    fn hold_fired(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.press.as_ref().is_some_and(|p| p.mode == FloatMode::Waiting) {
            self.lift(window, cx);
        }
    }

    /// Becomes a panel drag: the window moves with the pointer from here on.
    fn lift(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let weak = cx.entity().downgrade();
        let watch = cx.intercept_keystrokes(move |event, window, cx| {
            if event.keystroke.key == "escape" && weak.update(cx, |v, cx| v.escape(window, cx)).unwrap_or(false) {
                cx.stop_propagation();
            }
        });
        let Some(p) = &mut self.press else { return };
        p.mode = FloatMode::Panel;
        p._timer = None;
        p.escape_watch = Some(watch);
        let (frame, mouse) = (p.frame_at_start, p.mouse_at_start);
        let panel = self.panel.clone();
        let update = self.model.update(cx, |m, cx| m.update_controller(cx, |c| c.window_drag_changed(&panel, frame, mouse)));
        if let Some(u) = update {
            apply_window_update(window, u);
        }
    }

    fn pointer_moved(&mut self, event: &MouseMoveEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(p) = &mut self.press else { return };
        let dx = (event.position.x - p.start.x).as_f32() as f64;
        let dy = (event.position.y - p.start.y).as_f32() as f64;
        let moved = (dx * dx + dy * dy).sqrt() >= dl::DRAG_DISTANCE;
        if p.mode == FloatMode::Waiting && moved {
            let held = !p.needs_hold || p.pressed_at.elapsed().as_secs_f64() >= dl::HOLD_SECONDS;
            if held && platform::can_move_windows_by_hand() {
                self.lift(window, cx);
            } else if let Some(p) = &mut self.press {
                // Moving first (or where the window can't be moved by
                // hand): the OS's own window drag. No targets, but the
                // window still goes where it's dragged.
                p.mode = FloatMode::WindowMove;
                p._timer = None;
                window.start_window_move();
            }
        }
        let Some(p) = &self.press else { return };
        if p.mode == FloatMode::Panel {
            let mouse = Self::mouse_on_screen(event.position, window);
            let frame = p.frame_at_start;
            let panel = self.panel.clone();
            let update = self.model.update(cx, |m, cx| m.update_controller(cx, |c| c.window_drag_changed(&panel, frame, mouse)));
            if let Some(u) = update {
                apply_window_update(window, u);
            }
        }
    }

    fn pointer_up(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(p) = self.press.take() else { return };
        cx.notify();
        if p.mode == FloatMode::Panel {
            platform::set_window_alpha(window, 1.);
            let frame = Self::window_frame(window);
            let panel = self.panel.clone();
            self.model.update(cx, |m, cx| {
                m.update_controller(cx, |c| {
                    c.drag_ended();
                    // It stayed out of the dock: remember where it was left.
                    c.remember_window_frame(&panel, frame);
                })
            });
        } else if p.mode == FloatMode::Waiting {
            let frame = Self::window_frame(window);
            let panel = self.panel.clone();
            self.model.update(cx, |m, cx| m.update_controller(cx, |c| c.remember_window_frame(&panel, frame)));
        }
    }

    fn escape(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(p) = &self.press else { return false };
        if p.mode != FloatMode::Panel {
            return false;
        }
        let frame = Self::window_frame(window);
        let back = self.model.update(cx, |m, cx| m.update_controller(cx, |c| c.cancel_drag(Some(frame))));
        platform::set_window_alpha(window, 1.);
        if let Some(from) = back {
            platform::set_window_origin(window, point(px(from.x as f32), px(from.y as f32)));
        }
        true
    }
}

fn apply_window_update(window: &Window, update: dl::WindowDragUpdate) {
    platform::set_window_origin(window, point(px(update.origin.x as f32), px(update.origin.y as f32)));
    platform::set_window_alpha(window, update.alpha as f64);
}

impl Render for FloatingPanelView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = DockTheme::new(cx);
        let panel = self.panel.clone();
        let content = self.content.clone();
        let title = content.title(&panel, cx);
        let icon = content.icon(&panel, window, cx);
        let accessory = content.accessory(&panel, window, cx);
        let body = content.view(&panel, false, window, cx);
        let body_menu = self.menu();
        let bar_menu = self.menu();
        let tracking = self.press.is_some();
        let dragging = self.press.as_ref().is_some_and(|p| p.mode == FloatMode::Panel);
        let this = cx.entity().downgrade();

        let lights = 78.;
        let title_el = div()
            .id("dock-window-title")
            .px(px(8.))
            .flex_shrink_1()
            .min_w(px(0.))
            .text_size(px(self.options.title_size))
            .font_weight(FontWeight::SEMIBOLD)
            .truncate()
            .child(title.clone())
            .tooltip(|window, cx| gpui_kit::component::tooltip::Tooltip::new("Drag to move, or onto the main window to dock it").build(window, cx))
            .on_mouse_down(MouseButton::Left, cx.listener(|this, e: &MouseDownEvent, window, cx| this.begin_press(false, e, window, cx)))
            .context_menu(self.menu());

        let bar = h_flex()
            .flex_none()
            .w_full()
            .h(px(self.options.header_height))
            .pl(px(lights))
            .pr(px(6.))
            .gap(px(2.))
            .items_center()
            .border_b_1()
            .border_color(theme.bar_line)
            .child(title_el)
            .when_some(icon, |d, i| d.child(i))
            .child(
                div()
                    .id("dock-window-bar-space")
                    .flex_1()
                    .h_full()
                    .min_w(px(0.))
                    .on_mouse_down(MouseButton::Left, cx.listener(|this, e: &MouseDownEvent, window, cx| this.begin_press(true, e, window, cx)))
                    .context_menu(bar_menu),
            )
            .when_some(accessory, |d, a| d.child(a));

        div()
            .id("dock-window")
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.panel)
            .text_color(theme.header_text)
            .child(bar)
            .child(
                div()
                    .id("dock-window-body")
                    .flex_1()
                    .min_h(px(0.))
                    .overflow_hidden()
                    .relative()
                    .child(body)
                    .context_menu(body_menu),
            )
            .when(tracking, |d| d.child(float_listeners(this, dragging)))
    }
}

impl FloatingPanelView {
    fn menu(&self) -> impl Fn(PopupMenu, &mut Window, &mut Context<PopupMenu>) -> PopupMenu + 'static {
        let model = self.model.clone();
        let content = self.content.clone();
        let panel = self.panel.clone();
        move |menu, _, cx| {
            let hidden = model.read(cx).controller.layout.hidden_titles.contains(&panel);
            let (m1, p1) = (model.clone(), panel.clone());
            let mut menu = menu.item(PopupMenuItem::new(if hidden { "Show Title" } else { "Hide Title" }).on_click(move |_, _, cx| m1.update(cx, |m, cx| m.set_title_hidden(&p1, !hidden, cx))));
            let (m2, p2) = (model.clone(), panel.clone());
            menu = menu.item(PopupMenuItem::new("Put Back in Main Window").on_click(move |_, _, cx| m2.update(cx, |m, cx| m.put_back(&p2, cx))));
            if content.closable(&panel, cx) {
                let (m3, p3) = (model.clone(), panel.clone());
                let title = content.title(&panel, cx);
                menu = menu.separator().item(PopupMenuItem::new(format!("Close {title}")).on_click(move |_, _, cx| m3.update(cx, |m, cx| m.close(&p3, cx))));
            }
            menu
        }
    }
}

fn float_listeners(view: WeakEntity<FloatingPanelView>, dragging: bool) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |_, _, window, _| {
            if dragging {
                window.set_window_cursor_style(CursorStyle::ClosedHand);
            }
            let moved = view.clone();
            window.on_mouse_event(move |e: &MouseMoveEvent, phase, window, cx| {
                if phase == DispatchPhase::Capture {
                    let _ = moved.update(cx, |v, cx| v.pointer_moved(e, window, cx));
                }
            });
            let released = view.clone();
            window.on_mouse_event(move |_: &MouseUpEvent, phase, window, cx| {
                if phase == DispatchPhase::Capture {
                    let _ = released.update(cx, |v, cx| v.pointer_up(window, cx));
                }
            });
        },
    )
    .absolute()
    .size_full()
}
