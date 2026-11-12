//! Threadbear's change: popup menus drawn as macOS draws its own.
//!
//! The Swift app's menus are real macOS menus, and the GPUI app is replacing
//! it, so its menus have to look the same: a tick column, then the icon, then
//! the title, at the system's sizes. The numbers are measured off real menus
//! (NSMenu and SwiftUI's) captured from the screen on macOS 26, 12 Nov 2026
//! (features/tools/menu-shot.ts, docs/gpui-port/PARITY.md), in points.
//! Everything here is what the kit's own look had in its place: its rows
//! were 26 tall with a 2 point gap, its icon took the tick's place, and its
//! highlight was a grey.

use crate::ActiveTheme as _;
use gpui::{App, BoxShadow, Hsla, Pixels, hsla, point, px, rgb};

/// A line's height: rows follow each other with no gap.
pub(crate) const ROW_HEIGHT: Pixels = px(24.);
/// The title's size, the system font's at its menu size.
pub(crate) const TEXT_SIZE: Pixels = px(13.);
/// The menu's corners, at its outer edge.
pub(crate) const RADIUS: Pixels = px(13.);
/// The highlight's corners.
pub(crate) const ROW_RADIUS: Pixels = px(7.);
/// From the inside of the menu's 1 point edge to the rows, on every side:
/// the highlight stands 6 points in from the outer edge.
pub(crate) const PADDING: Pixels = px(5.);
/// A separator's band, its line in the middle, 11 points in from each end of
/// a row (17 from the menu's outer edge).
pub(crate) const SEPARATOR_HEIGHT: Pixels = px(11.);
pub(crate) const SEPARATOR_INSET: Pixels = px(11.);

/// Where a line's pieces start, from the row's left, as drawn ink: the tick,
/// the icon and the title. A menu with no ticked line has no tick column and
/// one with no icons no icon column, so the title moves left as macOS's does.
#[derive(Clone, Copy)]
pub(crate) struct Columns {
    pub check: Option<f32>,
    pub icon: Option<f32>,
    pub title: f32,
}

pub(crate) fn columns(ticks: bool, icons: bool) -> Columns {
    match (ticks, icons) {
        (false, false) => Columns { check: None, icon: None, title: 12. },
        (true, false) => Columns { check: Some(5.), icon: None, title: 19. },
        (false, true) => Columns { check: None, icon: Some(13.), title: 30. },
        (true, true) => Columns { check: Some(4.), icon: Some(20.), title: 37. },
    }
}

/// The title's ink to the row's right end, and to the right of a key or the
/// submenu's chevron: 18 points to the menu's outer edge.
pub(crate) const TRAILING: f32 = 12.;
/// Between a title and its key or chevron, at the least.
pub(crate) const KEY_GAP: f32 = 18.;

/// How far the ink of what's drawn starts inside its box, so the columns
/// above can be given as ink. The title's first letter has about a point of
/// side bearing at 13; the kit's Lucide icons are drawn 2 points in from
/// their box at 12, and the tick below about 1.
pub(crate) const TITLE_BEARING: f32 = 1.;
pub(crate) const ICON_BEARING: f32 = 2.;
pub(crate) const CHECK_BEARING: f32 = 1.;
/// Icons and the tick at 12 points: about the ink SF Symbols have at 13.
pub(crate) const ICON_SIZE: Pixels = px(12.);
/// The submenu's chevron: 6 by 10 of ink, its right end where a title's is.
pub(crate) const CHEVRON_SIZE: Pixels = px(17.);
pub(crate) const CHEVRON_BEARING: f32 = 5.67;

/// SF's `checkmark` at a menu's weight: a short stroke and a long one rising
/// well above it, 9 by 9 at 12 points. Lucide's check is wide and low.
pub(crate) const CHECKMARK: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="M4 13.5l5 5.5L20 3.5"/></svg>"#;

/// The menu's colours in the current look.
#[derive(Clone, Copy)]
pub(crate) struct Colors {
    pub background: Hsla,
    /// The 1 point line round the outside.
    pub edge: Hsla,
    /// The light line just inside it: 1 point on dark, 2 on light.
    pub rim: Hsla,
    pub rim_width: Pixels,
    pub label: Hsla,
    pub disabled: Hsla,
    /// Keys and other second things on a line (a submenu's chevron is the title's colour).
    pub secondary: Hsla,
    pub separator: Hsla,
    pub highlight: Hsla,
    pub highlight_label: Hsla,
}

pub(crate) fn colors(cx: &App) -> Colors {
    let white = |a: f32| hsla(0., 0., 1., a);
    let black = |a: f32| hsla(0., 0., 0., a);
    if cx.theme().is_dark() {
        Colors {
            background: rgb(0x232323).into(),
            edge: black(0.9),
            rim: white(0.33),
            rim_width: px(1.),
            label: white(0.85),
            disabled: white(0.25),
            secondary: white(0.25),
            separator: white(0.14),
            // The system's blue as a menu line's highlight, which is darker
            // than the accent itself on dark.
            highlight: rgb(0x084db0).into(),
            highlight_label: white(1.),
        }
    } else {
        Colors {
            background: rgb(0xfafafa).into(),
            edge: black(0.2),
            rim: white(0.8),
            rim_width: px(2.),
            label: black(0.85),
            disabled: black(0.25),
            secondary: black(0.25),
            separator: black(0.1),
            highlight: rgb(0x409afa).into(),
            highlight_label: white(1.),
        }
    }
}

/// The soft shadow under it: wide on light, where it's what shows the edge,
/// and darker on dark.
pub(crate) fn shadow(cx: &App) -> Vec<BoxShadow> {
    let alpha = if cx.theme().is_dark() { 0.45 } else { 0.18 };
    vec![BoxShadow { color: hsla(0., 0., 0., alpha), offset: point(px(0.), px(6.)), blur_radius: px(20.), spread_radius: px(0.), inset: false }]
}
