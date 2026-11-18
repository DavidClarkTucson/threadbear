#![cfg(feature = "automation")]

// Up on a text box's first line and Down on its last: the kit leaves the
// caret where it is; with ArrowsReachEnds on (the Mac default, Settings >
// General > Typing) they go to the start and the end, as Mac text views do.

use gpui_kit::base::input::ArrowsReachEnds;
use gpui_kit::component::input::{MoveDown, MoveUp, Textarea, TextareaState};
use gpui_kit::prelude::*;
use gpui_kit::test::TestWindowExt as _;
use gpui_kit::{div, AnyWindowHandle, Entity, Focusable as _, TestAppContext, Window, WindowOptions};

struct Boxed(Entity<TextareaState>);

impl Render for Boxed {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(gpui_kit::px(600.)).child(Textarea::new(&self.0))
    }
}

fn text_box(cx: &mut TestAppContext) -> (AnyWindowHandle, Entity<TextareaState>) {
    cx.update(gpui_kit::init);
    let (handle, root) = cx.update(|cx| gpui_kit::open_window(WindowOptions::default(), cx, |window, cx| cx.new(|cx| Boxed(cx.new(|cx| TextareaState::new(window, cx))))).unwrap());
    let input = cx.update(|cx| root.read(cx).0.clone());
    (handle, input)
}

/// Puts the caret at `at` in "first line\nsecond line", presses `key` and
/// says where the caret went.
fn press(cx: &mut TestAppContext, on: Option<bool>, at: usize, key: &dyn gpui_kit::Action) -> usize {
    let (handle, input) = text_box(cx);
    if let Some(on) = on {
        cx.update(|cx| cx.set_global(ArrowsReachEnds(on)));
    }
    cx.update_window(handle, |_, window, cx| {
        input.update(cx, |s, cx| {
            s.set_value("first line\nsecond line", window, cx);
            s.set_selected_range(at..at, cx);
            s.focus(window, cx);
        });
        window.render_frame(cx);
        input.read(cx).focus_handle(cx).dispatch_action(key, window, cx);
        input.read(cx).selected_range().start
    })
    .unwrap()
}

#[gpui_kit::test]
fn up_on_the_first_line_goes_to_the_start_when_on(cx: &mut TestAppContext) {
    assert_eq!(press(cx, Some(true), 5, &MoveUp), 0);
}

#[gpui_kit::test]
fn down_on_the_last_line_goes_to_the_end_when_on(cx: &mut TestAppContext) {
    let len = "first line\nsecond line".len();
    assert_eq!(press(cx, Some(true), 14, &MoveDown), len);
}

#[gpui_kit::test]
fn up_from_the_second_line_still_keeps_its_column_when_on(cx: &mut TestAppContext) {
    // "second line", caret after "sec": up lands after "fir".
    assert_eq!(press(cx, Some(true), 14, &MoveUp), 3);
}

#[gpui_kit::test]
fn up_on_the_first_line_stays_put_when_off(cx: &mut TestAppContext) {
    assert_eq!(press(cx, Some(false), 5, &MoveUp), 5);
}

#[gpui_kit::test]
fn the_kit_on_its_own_stays_put(cx: &mut TestAppContext) {
    assert_eq!(press(cx, None, 5, &MoveUp), 5);
}
