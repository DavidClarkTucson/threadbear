// Videos and PDFs in the Mac's own views (native/media_views.m): AVKit's
// player and PDFKit's view, over a GPUI layout slot, as the Swift app's pane
// (FilePreviewView, Quick Look) and review window (ReviewVideo) show them.
// GPUI has no video player, and the web runtime's Chromium has no H.264, so
// on a Mac the system's own views do it; on Windows and Linux a video or a
// PDF still opens in its own app, through the open gate.
//
// A view is made the first time its slot is drawn, through the open gate
// (`open_gate::show_media`, the only way one is made), and then placed by its
// window once a frame after everything's laid out (`sweeper`), as the web
// panels' pages are (web_panels.rs): where its slot was drawn that frame, and
// hidden while a dialog, a sheet or one of the app's menus could be over it,
// as GPUI can't draw over a native view. A slot that isn't drawn in a frame
// has gone (the preview closed, another thread or shot is showing, the pane
// was put away), so its view stops and goes then, as a SwiftUI view's
// dismantle stops the Swift app's.
//
// Kept in thread-locals rather than an entity: the views are the main
// thread's, and a slot drawn while an entity is being updated never has to
// read that entity back (a double lease).
use gpui_kit::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ffi::c_void;
use std::rc::Rc;

/// How a view is used, as native/media_views.m numbers them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub(crate) enum Kind {
    /// A film in the pane: the player's floating controls, its sound on,
    /// paused on its first frame, sized to its own shape.
    PaneVideo = 0,
    /// A clip on the review window's stage: controls and sound, looping from
    /// the start, at the mute and volume the clip before it was left at.
    StageVideo = 1,
    /// A clip among them all in the review window: muted, no controls,
    /// looping, and a click goes to the cell under it.
    CellVideo = 2,
    /// A PDF in the pane, in place of Quick Look.
    PaneDocument = 3,
}

/// What a view is showing, for the scripted app's dump.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct MediaState {
    /// The film's shape once its track has said (0 until then), or a PDF's
    /// first page, in points.
    pub width: f64,
    pub height: f64,
    pub playing: bool,
    pub muted: bool,
    pub controls: bool,
    pub looping: bool,
    pub volume: f32,
    pub pages: i32,
}

/// native/media_views.m's TBMediaState, field for field.
#[repr(C)]
#[derive(Default)]
struct NativeState {
    width: f64,
    height: f64,
    playing: bool,
    muted: bool,
    controls: bool,
    looping: bool,
    volume: f32,
    pages: i32,
}

#[cfg(target_os = "macos")]
extern "C" {
    fn tb_media_place(view: *mut c_void, x: f64, y: f64, width: f64, height: f64);
    fn tb_media_set_hidden(view: *mut c_void, hidden: bool);
    fn tb_media_close(view: *mut c_void);
    fn tb_media_state(view: *mut c_void, out: *mut NativeState);
    #[cfg(feature = "automation")]
    fn tb_media_play(view: *mut c_void) -> bool;
}

/// One of those views, owned: dropped, it stops and leaves its window.
pub(crate) struct Media {
    view: *mut c_void,
    kind: Kind,
    hidden: Cell<bool>,
    placed: Cell<Option<Bounds<Pixels>>>,
}

impl Media {
    /// The view `tb_media_open` handed back, owned from here. Only the open
    /// gate makes them (`open_gate::show_media`).
    ///
    /// # Safety
    /// `view` is null or a view from `tb_media_open` that nothing else owns.
    pub(crate) unsafe fn from_native(view: *mut c_void, kind: Kind) -> Option<Media> {
        (!view.is_null()).then(|| Media { view, kind, hidden: Cell::new(true), placed: Cell::new(None) })
    }

    fn place(&self, bounds: Bounds<Pixels>) {
        if self.placed.get() == Some(bounds) {
            return;
        }
        self.placed.set(Some(bounds));
        #[cfg(target_os = "macos")]
        unsafe {
            tb_media_place(self.view, f64::from(bounds.origin.x), f64::from(bounds.origin.y), f64::from(bounds.size.width), f64::from(bounds.size.height))
        };
    }

    fn set_hidden(&self, hidden: bool) {
        if self.hidden.replace(hidden) == hidden {
            return;
        }
        #[cfg(target_os = "macos")]
        unsafe {
            tb_media_set_hidden(self.view, hidden)
        };
    }

    pub(crate) fn state(&self) -> MediaState {
        let mut out = NativeState::default();
        #[cfg(target_os = "macos")]
        unsafe {
            tb_media_state(self.view, &mut out)
        };
        MediaState { width: out.width, height: out.height, playing: out.playing, muted: out.muted, controls: out.controls, looping: out.looping, volume: out.volume, pages: out.pages }
    }
}

impl Drop for Media {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        unsafe {
            tb_media_close(self.view)
        };
    }
}

// ---------------------------------------------------------------- slots

struct Slot {
    key: String,
    window: WindowId,
    /// The view, or why it couldn't be made (said in the slot's place, and
    /// not tried again every frame).
    media: Result<Media, String>,
    /// The frame it was last drawn in, and where.
    drawn: Rc<Cell<(u64, Bounds<Pixels>)>>,
}

thread_local! {
    static SLOTS: RefCell<Vec<Slot>> = const { RefCell::new(Vec::new()) };
    /// Each window's frames, counted as it starts each (`sweeper`).
    static FRAMES: RefCell<HashMap<WindowId, u64>> = RefCell::new(HashMap::new());
    /// The mute and volume each review window's stage clip was left at, for
    /// the next: clips compared by ear play at the same level (ReviewVideo).
    static STAGE_AUDIO: RefCell<HashMap<WindowId, (bool, f32)>> = RefCell::new(HashMap::new());
}

fn frame_of(window: WindowId) -> u64 {
    FRAMES.with(|f| f.borrow().get(&window).copied().unwrap_or(0))
}

/// What a slot's view is given when it's made: the window's own view, and
/// the review stage's mute and volume to carry on with.
pub(crate) struct Opening {
    pub parent: *mut c_void,
    pub audio: Option<(bool, f32)>,
}

/// The view for `key`, placed where this element's drawn in `window`: made
/// now the first time, by `open` (which goes through the open gate), or why
/// it couldn't be, for the caller to say instead.
pub(crate) fn slot(key: &str, window: &Window, open: impl FnOnce(Opening) -> Result<Media, String>) -> Result<AnyElement, String> {
    let id = window.window_handle().window_id();
    let found = SLOTS.with(|s| {
        let slots = s.borrow();
        let slot = slots.iter().find(|s| s.key == key && s.window == id)?;
        // One that couldn't be made has no element to say it's still
        // wanted, so asking for it does.
        if slot.media.is_err() {
            slot.drawn.set((frame_of(id), Bounds::default()));
        }
        Some(slot.media.as_ref().map(|_| slot.drawn.clone()).map_err(Clone::clone))
    });
    let drawn = match found {
        Some(found) => found?,
        None => {
            let media = match parent_view(window) {
                Some(parent) => open(Opening { parent, audio: STAGE_AUDIO.with(|a| a.borrow().get(&id).copied()) }),
                None => Err("This window has no view to show it in.".to_owned()),
            };
            let drawn = Rc::new(Cell::new((if media.is_err() { frame_of(id) } else { 0 }, Bounds::default())));
            let made = media.as_ref().map(|_| drawn.clone()).map_err(Clone::clone);
            SLOTS.with(|s| s.borrow_mut().push(Slot { key: key.to_owned(), window: id, media, drawn }));
            made?
        }
    };
    Ok(canvas(move |bounds, window, _| drawn.set((frame_of(window.window_handle().window_id()), bounds)), |_, _, _, _| {}).size_full().into_any_element())
}

/// Whether `key`'s view is up in some window, and what it's showing.
pub(crate) fn state_of(key: &str) -> Option<MediaState> {
    SLOTS.with(|s| s.borrow().iter().find(|s| s.key == key).and_then(|s| s.media.as_ref().ok().map(Media::state)))
}

/// The play button of `key`'s film, for the scripted app: false when there's
/// no film up by that key.
#[cfg(feature = "automation")]
pub(crate) fn play(key: &str) -> bool {
    SLOTS.with(|s| {
        s.borrow().iter().find(|s| s.key == key).and_then(|s| s.media.as_ref().ok()).is_some_and(|m| {
            #[cfg(target_os = "macos")]
            return unsafe { tb_media_play(m.view) };
            #[cfg(not(target_os = "macos"))]
            {
                let _ = m;
                false
            }
        })
    })
}

/// Why `key`'s view couldn't be made, if it couldn't.
pub(crate) fn failure_of(key: &str) -> Option<String> {
    SLOTS.with(|s| s.borrow().iter().find(|s| s.key == key).and_then(|s| s.media.as_ref().err().cloned()))
}

/// What could be over a window's views this frame.
#[derive(Default)]
pub(crate) struct Cover {
    /// A dialog, a sheet, a popover or the picture viewer: over everything.
    pub everything: bool,
    /// Where the app's own menus are open: one reaching a view's column
    /// covers it.
    pub menus: Vec<Point<Pixels>>,
}

/// Made as a window's frame starts (its root view's render) and drawn after
/// everything else in it: counts the frame, then once everything's laid out
/// places the views drawn in it and closes the ones that weren't.
pub(crate) fn sweeper(window: &Window, cover: Cover) -> AnyElement {
    let id = window.window_handle().window_id();
    let frame = FRAMES.with(|f| {
        let mut f = f.borrow_mut();
        let n = f.entry(id).or_insert(0);
        *n += 1;
        *n
    });
    canvas(move |_, window, cx| sweep(id, frame, &cover, window, cx), |_, _, _, _| {}).absolute().size_0().into_any_element()
}

fn sweep(id: WindowId, frame: u64, cover: &Cover, window: &mut Window, cx: &mut App) {
    use gpui_kit::component::WindowExt as _;
    let modal = cover.everything || window.has_active_dialog(cx) || window.has_active_sheet(cx);
    let gone = SLOTS.with(|s| {
        let mut slots = s.borrow_mut();
        let (keep, gone): (Vec<Slot>, Vec<Slot>) = std::mem::take(&mut *slots).into_iter().partition(|slot| slot.window != id || slot.drawn.get().0 == frame);
        *slots = keep;
        for slot in slots.iter().filter(|s| s.window == id) {
            let Ok(media) = &slot.media else { continue };
            let (_, b) = slot.drawn.get();
            // A menu opens down and to the right of where it was asked for,
            // about this wide (web_panels.rs's rule).
            let menu_over = cover.menus.iter().any(|at| at.x + gpui_kit::px(300.) > b.origin.x && at.x < b.origin.x + b.size.width);
            if !modal && !menu_over && f32::from(b.size.width) >= 1. && f32::from(b.size.height) >= 1. {
                media.place(b);
                media.set_hidden(false);
            } else {
                media.set_hidden(true);
            }
        }
        gone
    });
    // Dropped outside the borrow: each stops and leaves its window as it goes.
    for slot in gone {
        remember_audio(&slot);
    }
}

/// A stage clip going: its mute and volume are where the next one starts.
fn remember_audio(slot: &Slot) {
    if let Some(media) = slot.media.as_ref().ok().filter(|m| m.kind == Kind::StageVideo) {
        let state = media.state();
        STAGE_AUDIO.with(|a| a.borrow_mut().insert(slot.window, (state.muted, state.volume)));
    }
}

/// A window's going: its views stop and go with it.
pub(crate) fn close_window(id: WindowId) {
    let gone: Vec<Slot> = SLOTS.with(|s| {
        let mut slots = s.borrow_mut();
        let (gone, keep) = std::mem::take(&mut *slots).into_iter().partition(|slot| slot.window == id);
        *slots = keep;
        gone
    });
    drop(gone);
    FRAMES.with(|f| f.borrow_mut().remove(&id));
    STAGE_AUDIO.with(|a| a.borrow_mut().remove(&id));
}

/// The window's own view, a media view's parent.
fn parent_view(window: &Window) -> Option<*mut c_void> {
    #[cfg(target_os = "macos")]
    {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let handle = HasWindowHandle::window_handle(window).ok()?;
        match handle.as_raw() {
            RawWindowHandle::AppKit(h) => Some(h.ns_view.as_ptr()),
            _ => None,
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window;
        None
    }
}

/// Whether this system has the views at all: on Windows and Linux a video or
/// a PDF opens in its own app.
pub(crate) const SHOWN_HERE: bool = cfg!(target_os = "macos");

/// The views up now, for the dump.
#[cfg(feature = "automation")]
pub(crate) fn dump() -> serde_json::Value {
    SLOTS.with(|s| {
        serde_json::Value::Array(
            s.borrow()
                .iter()
                .map(|slot| match &slot.media {
                    Ok(media) => {
                        let st = media.state();
                        serde_json::json!({ "key": slot.key, "kind": format!("{:?}", media.kind), "hidden": media.hidden.get(), "width": st.width, "height": st.height, "playing": st.playing, "muted": st.muted, "controls": st.controls, "looping": st.looping, "pages": st.pages })
                    }
                    Err(why) => serde_json::json!({ "key": slot.key, "failed": why }),
                })
                .collect(),
        )
    })
}
