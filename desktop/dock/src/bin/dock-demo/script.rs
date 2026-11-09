// Scripted control of the demo, for screenshots and for testing the real
// gestures without touching the mouse, as Swift's ScriptedEvents.swift is:
// `BEARDOCK_EVENTS="down 1062 490;drag 640 420;esc;up;shot /tmp/x.png"`.
// Only built with the `automation` feature. Points are window-local, top-left
// origin. Steps: down x y | drag x y (in ten moves) | move x y | up | right x y
// | middle x y | esc | wait ms | shot path | quit. Add `BEARDOCK_FRESH=1` to
// start from the default layout without saving and `BEARDOCK_QUIT=1` to quit
// at the end.
use gpui_kit::test::{TestEventExt as _, TestWindowExt as _};
use gpui_kit::*;
use std::time::Duration;

pub fn run(window: AnyWindowHandle, cx: &mut App) {
    let Ok(script) = std::env::var("BEARDOCK_EVENTS") else { return };
    let quit = std::env::var_os("BEARDOCK_QUIT").is_some();
    cx.spawn(async move |cx: &mut AsyncApp| {
        cx.background_executor().timer(Duration::from_millis(700)).await;
        let mut last = point(px(0.), px(0.));
        for step in script.split(';') {
            let parts: Vec<&str> = step.split_whitespace().collect();
            let Some(&cmd) = parts.first() else { continue };
            let arg = |i: usize| parts.get(i).and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.);
            match cmd {
                "down" => {
                    let p = point(px(arg(1)), px(arg(2)));
                    last = p;
                    let _ = window.update(cx, |_, window, cx| {
                        window.pointer_move(p, None, cx);
                        window.pointer_down(p, MouseButton::Left, cx);
                    });
                }
                "drag" => {
                    let to = point(px(arg(1)), px(arg(2)));
                    for i in 1..=10 {
                        let t = i as f32 / 10.;
                        let p = point(last.x + (to.x - last.x) * t, last.y + (to.y - last.y) * t);
                        let _ = window.update(cx, |_, window, cx| window.pointer_move(p, Some(MouseButton::Left), cx));
                        cx.background_executor().timer(Duration::from_millis(16)).await;
                    }
                    last = to;
                }
                "up" => {
                    let _ = window.update(cx, |_, window, cx| window.pointer_up(last, MouseButton::Left, cx));
                }
                "move" => {
                    last = point(px(arg(1)), px(arg(2)));
                    let p = last;
                    let _ = window.update(cx, |_, window, cx| window.pointer_move(p, None, cx));
                }
                "right" => {
                    let p = point(px(arg(1)), px(arg(2)));
                    last = p;
                    let _ = window.update(cx, |_, window, cx| {
                        window.pointer_move(p, None, cx);
                        window.pointer_down(p, MouseButton::Right, cx);
                        window.pointer_up(p, MouseButton::Right, cx);
                    });
                }
                "middle" => {
                    let p = point(px(arg(1)), px(arg(2)));
                    last = p;
                    let _ = window.update(cx, |_, window, cx| {
                        window.pointer_move(p, None, cx);
                        window.pointer_down(p, MouseButton::Middle, cx);
                        window.pointer_up(p, MouseButton::Middle, cx);
                    });
                }
                "esc" => {
                    let _ = window.update(cx, |_, window, cx| window.key_down("escape", false, cx));
                }
                "wait" => {
                    cx.background_executor().timer(Duration::from_millis(arg(1) as u64)).await;
                }
                "shot" => {
                    let path = parts.get(1).copied().unwrap_or("/tmp/beardock.png").to_string();
                    let _ = window.update(cx, |_, window, cx| {
                        window.render_frame(cx);
                        match window.render_to_image() {
                            Ok(image) => {
                                if let Err(e) = image.save(&path) {
                                    eprintln!("shot failed: {e}");
                                }
                            }
                            Err(e) => eprintln!("shot failed: {e}"),
                        }
                    });
                }
                "quit" => {
                    let _ = cx.update(|cx| cx.quit());
                }
                other => eprintln!("unknown step {other}"),
            }
            cx.background_executor().timer(Duration::from_millis(30)).await;
        }
        if quit {
            let _ = cx.update(|cx| cx.quit());
        }
    })
    .detach();
}
