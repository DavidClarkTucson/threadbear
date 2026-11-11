// A sealed page drawn with no window (Chromium's windowless rendering), for
// the transcript's diagrams and pages: laid out at the size asked for, asked
// things and pictured through DevTools, which needs no window and no open
// port (the same in-process channel Ghostex uses). As the Swift app's
// offscreen WKWebViews in RichRenderer, with sealed.rs's rules.
use super::sealed;
use cef::*;
use futures::channel::oneshot;
use serde_json::{json, Value as Json};
use std::collections::HashMap;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Mutex};

/// An answer still to come from Chromium.
pub type Reply<T> = oneshot::Receiver<Result<T, String>>;

/// What the page's handlers share with it. They're called on Chromium's own
/// threads, so it's all behind locks, and none is held while calling CEF.
struct Shared {
    size: Mutex<(i32, i32)>,
    scale: Mutex<f32>,
    /// The one address the page may show; anything else is cancelled.
    allowed: Mutex<String>,
    calls: Mutex<HashMap<i32, oneshot::Sender<Result<Json, String>>>>,
    load: Mutex<Option<oneshot::Sender<Result<(), String>>>>,
    gone: AtomicBool,
}

impl Shared {
    fn fail_all(&self, why: &str) {
        self.gone.store(true, Ordering::Release);
        let calls: Vec<_> = self.calls.lock().map(|mut c| c.drain().map(|(_, tx)| tx).collect()).unwrap_or_default();
        for tx in calls {
            let _ = tx.send(Err(why.to_owned()));
        }
        if let Some(tx) = self.load.lock().ok().and_then(|mut l| l.take()) {
            let _ = tx.send(Err(why.to_owned()));
        }
    }

    fn finish_load(&self, result: Result<(), String>) {
        if let Some(tx) = self.load.lock().ok().and_then(|mut l| l.take()) {
            let _ = tx.send(result);
        }
    }
}

/// How a page is made.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// Off for an agent's html, as the Swift app has `allowsContentJavaScript`
    /// false; on for the app's own diagram page, which runs Pele.
    pub javascript: bool,
    pub width: i32,
    pub height: i32,
    /// Pixels a CSS pixel, as the screen it's for (2 on Retina).
    pub scale: f32,
}

pub struct Offscreen {
    browser: Browser,
    shared: Arc<Shared>,
    next: AtomicI32,
    _observer: Option<Registration>,
}

impl Offscreen {
    /// Opens a sealed page at `url` (a data: address the app made). The
    /// receiver says when it has loaded, or why it didn't.
    pub fn open(url: &str, options: Options) -> Result<(Offscreen, Reply<()>), String> {
        if super::host::status() != super::host::Status::Ready {
            return Err("the web runtime isn't running".into());
        }
        let mut context = sealed::context().ok_or("the sealed pages' context isn't there")?;
        let (loaded, load) = oneshot::channel();
        let shared = Arc::new(Shared {
            size: Mutex::new((options.width.max(1), options.height.max(1))),
            scale: Mutex::new(options.scale.max(1.)),
            allowed: Mutex::new(url.to_owned()),
            calls: Mutex::new(HashMap::new()),
            load: Mutex::new(Some(loaded)),
            gone: AtomicBool::new(false),
        });
        let window_info = WindowInfo::default().set_as_windowless(std::ptr::null_mut());
        let off = if options.javascript { State::ENABLED } else { State::DISABLED };
        let settings = BrowserSettings {
            // Enough to settle a layout quickly; nothing here animates.
            windowless_frame_rate: 30,
            javascript: off,
            // The rest a page could use to keep or pass on anything.
            javascript_access_clipboard: State::DISABLED,
            javascript_dom_paste: State::DISABLED,
            javascript_close_windows: State::DISABLED,
            local_storage: State::DISABLED,
            databases_deprecated: State::DISABLED,
            webgl: State::DISABLED,
            // Transparent, so a diagram sits on the box's own fill.
            background_color: 0,
            ..Default::default()
        };
        let mut client = OffscreenClient::new(
            OffscreenRender::new(shared.clone()),
            SealedRequests::new(shared.clone()),
            Loads::new(shared.clone()),
            NoPopups::new(),
            QuietConsole::new(),
        );
        let browser = browser_host_create_browser_sync(Some(&window_info), Some(&mut client), Some(&CefString::from(url)), Some(&settings), None, Some(&mut context))
            .ok_or("Chromium didn't make the page")?;
        let host = browser.host().ok_or("the page has no host")?;
        let mut observer = Answers::new(shared.clone());
        let registration = host.add_dev_tools_message_observer(Some(&mut observer));
        Ok((Offscreen { browser, shared, next: AtomicI32::new(1), _observer: registration }, load))
    }

    /// A DevTools method on the page (chromedevtools.github.io/devtools-protocol),
    /// its result when it comes.
    pub fn devtools(&self, method: &str, params: Json) -> Reply<Json> {
        let (tx, rx) = oneshot::channel();
        if self.shared.gone.load(Ordering::Acquire) {
            let _ = tx.send(Err("its page stopped".into()));
            return rx;
        }
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut calls) = self.shared.calls.lock() {
            calls.insert(id, tx);
        }
        let message = json!({ "id": id, "method": method, "params": params }).to_string();
        let sent = self.browser.host().map(|h| h.send_dev_tools_message(Some(message.as_bytes()))).unwrap_or(0);
        if sent != 1 {
            if let Some(tx) = self.shared.calls.lock().ok().and_then(|mut c| c.remove(&id)) {
                let _ = tx.send(Err(format!("DevTools didn't take {method}")));
            }
        }
        rx
    }

    /// Lays the page out at a new size.
    pub fn resize(&self, width: i32, height: i32) {
        if let Ok(mut size) = self.shared.size.lock() {
            *size = (width.max(1), height.max(1));
        }
        if let Some(host) = self.browser.host() {
            host.was_resized();
        }
    }

    /// Shows another of the app's own pages in it.
    pub fn load(&self, url: &str) -> Reply<()> {
        let (tx, rx) = oneshot::channel();
        if let Ok(mut allowed) = self.shared.allowed.lock() {
            *allowed = url.to_owned();
        }
        if let Ok(mut load) = self.shared.load.lock() {
            *load = Some(tx);
        }
        if let Some(frame) = self.browser.main_frame() {
            frame.load_url(Some(&CefString::from(url)));
        }
        rx
    }

    pub fn stopped(&self) -> bool {
        self.shared.gone.load(Ordering::Acquire)
    }
}

impl Drop for Offscreen {
    fn drop(&mut self) {
        self.shared.gone.store(true, Ordering::Release);
        if let Some(host) = self.browser.host() {
            host.close_browser(1);
        }
    }
}

/// A PNG of the top-left `width` by `height` CSS pixels, at the page's
/// scale, taken from what Chromium drew.
pub async fn picture(page: &Offscreen, width: f64, height: f64) -> Result<Vec<u8>, String> {
    let shot = page
        .devtools("Page.captureScreenshot", json!({ "format": "png", "fromSurface": true, "captureBeyondViewport": true, "clip": { "x": 0, "y": 0, "width": width, "height": height, "scale": 1 } }))
        .await
        .map_err(|_| "the page went away".to_owned())??;
    let data = shot["data"].as_str().ok_or("the picture came back empty")?;
    base64(data).ok_or_else(|| "the picture couldn't be read".to_owned())
}

/// A PNG's size, from its header.
pub fn png_size(png: &[u8]) -> Option<(u32, u32)> {
    if png.len() < 24 || &png[..8] != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let width = u32::from_be_bytes(png[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(png[20..24].try_into().ok()?);
    Some((width, height))
}

/// Standard base64, as DevTools sends pictures.
fn base64(text: &str) -> Option<Vec<u8>> {
    fn value(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a' + 26) as u32),
            b'0'..=b'9' => Some((c - b'0' + 52) as u32),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let bytes: Vec<u8> = text.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    let trimmed = bytes.iter().rposition(|b| *b != b'=').map_or(&bytes[..0], |end| &bytes[..=end]);
    let mut out = Vec::with_capacity(trimmed.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for &c in trimmed {
        acc = (acc << 6) | value(c)?;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}

wrap_client! {
    struct OffscreenClient {
        render: RenderHandler,
        requests: RequestHandler,
        loads: LoadHandler,
        popups: LifeSpanHandler,
        console: DisplayHandler,
    }

    impl Client {
        fn render_handler(&self) -> Option<RenderHandler> {
            Some(self.render.clone())
        }
        fn request_handler(&self) -> Option<RequestHandler> {
            Some(self.requests.clone())
        }
        fn load_handler(&self) -> Option<LoadHandler> {
            Some(self.loads.clone())
        }
        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(self.popups.clone())
        }
        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(self.console.clone())
        }
    }
}

wrap_render_handler! {
    struct OffscreenRender {
        shared: Arc<Shared>,
    }

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
            let (width, height) = self.shared.size.lock().map(|s| *s).unwrap_or((1, 1));
            if let Some(rect) = rect {
                *rect = Rect { x: 0, y: 0, width, height };
            }
        }

        fn screen_info(&self, _browser: Option<&mut Browser>, screen_info: Option<&mut ScreenInfo>) -> c_int {
            let Some(info) = screen_info else { return 0 };
            let (width, height) = self.shared.size.lock().map(|s| *s).unwrap_or((1, 1));
            info.device_scale_factor = self.shared.scale.lock().map(|s| *s).unwrap_or(2.);
            info.rect = Rect { x: 0, y: 0, width, height };
            info.available_rect = info.rect.clone();
            1
        }

        // The pictures come from DevTools; nothing to do with each frame.
        fn on_paint(&self, _browser: Option<&mut Browser>, _type_: PaintElementType, _dirty_rects: Option<&[Rect]>, _buffer: *const u8, _width: c_int, _height: c_int) {}
    }
}

wrap_request_handler! {
    struct SealedRequests {
        shared: Arc<Shared>,
    }

    impl RequestHandler {
        // The page the app gave, in the main frame; in a frame only what the
        // page carries in itself (its policy has frame-src 'none' anyway).
        // Never a link, a form, a refresh or a script's navigation.
        fn on_before_browse(&self, _browser: Option<&mut Browser>, frame: Option<&mut Frame>, request: Option<&mut Request>, _user_gesture: c_int, _is_redirect: c_int) -> c_int {
            let url = request.map(|r| CefString::from(&r.url()).to_string()).unwrap_or_default();
            let main = frame.is_some_and(|f| f.is_main() == 1);
            let allowed = if main { self.shared.allowed.lock().is_ok_and(|a| *a == url) || url == "about:blank" } else { sealed::may_load(&url) };
            if allowed {
                0
            } else {
                1
            }
        }

        fn resource_request_handler(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _request: Option<&mut Request>,
            _is_navigation: c_int,
            _is_download: c_int,
            _request_initiator: Option<&CefString>,
            _disable_default_handling: Option<&mut c_int>,
        ) -> Option<ResourceRequestHandler> {
            Some(sealed::Blocker::new())
        }

        fn on_render_process_terminated(&self, _browser: Option<&mut Browser>, _status: TerminationStatus, _error_code: c_int, _error_string: Option<&CefString>) {
            self.shared.fail_all("its page stopped");
        }
    }
}

wrap_load_handler! {
    struct Loads {
        shared: Arc<Shared>,
    }

    impl LoadHandler {
        fn on_load_end(&self, _browser: Option<&mut Browser>, frame: Option<&mut Frame>, _http_status_code: c_int) {
            if frame.is_some_and(|f| f.is_main() == 1) {
                self.shared.finish_load(Ok(()));
            }
        }

        fn on_load_error(&self, _browser: Option<&mut Browser>, frame: Option<&mut Frame>, _error_code: Errorcode, error_text: Option<&CefString>, _failed_url: Option<&CefString>) {
            if frame.is_some_and(|f| f.is_main() == 1) {
                let why = error_text.map(|t| t.to_string()).filter(|t| !t.is_empty()).unwrap_or_else(|| "it didn't load".into());
                self.shared.finish_load(Err(why));
            }
        }
    }
}

wrap_life_span_handler! {
    struct NoPopups;

    impl LifeSpanHandler {
        // Nothing opens a window: a sealed page has nowhere to send one.
        fn on_before_popup(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _popup_id: c_int,
            _target_url: Option<&CefString>,
            _target_frame_name: Option<&CefString>,
            _target_disposition: WindowOpenDisposition,
            _user_gesture: c_int,
            _popup_features: Option<&PopupFeatures>,
            _window_info: Option<&mut WindowInfo>,
            _client: Option<&mut Option<Client>>,
            _settings: Option<&mut BrowserSettings>,
            _extra_info: Option<&mut Option<DictionaryValue>>,
            _no_javascript_access: Option<&mut c_int>,
        ) -> c_int {
            1
        }
    }
}

wrap_display_handler! {
    struct QuietConsole;

    impl DisplayHandler {
        // What a page says to its console stays out of Chromium's log: it can
        // carry the agent's words.
        fn on_console_message(&self, _browser: Option<&mut Browser>, _level: LogSeverity, _message: Option<&CefString>, _source: Option<&CefString>, _line: c_int) -> c_int {
            1
        }
    }
}

wrap_dev_tools_message_observer! {
    struct Answers {
        shared: Arc<Shared>,
    }

    impl DevToolsMessageObserver {
        fn on_dev_tools_method_result(&self, _browser: Option<&mut Browser>, message_id: c_int, success: c_int, result: Option<&[u8]>) {
            let Some(tx) = self.shared.calls.lock().ok().and_then(|mut c| c.remove(&message_id)) else { return };
            let parsed = result.and_then(|bytes| serde_json::from_slice::<Json>(bytes).ok()).unwrap_or(Json::Null);
            let answer = if success == 1 { Ok(parsed) } else { Err(parsed["message"].as_str().unwrap_or("DevTools said no").to_owned()) };
            let _ = tx.send(answer);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_base64_and_png_sizes() {
        assert_eq!(base64("aGVsbG8=").unwrap(), b"hello");
        assert_eq!(base64("aGVsbG8gd29ybGQ=").unwrap(), b"hello world");
        assert_eq!(base64("YQ==").unwrap(), b"a");
        assert!(base64("a$b").is_none());
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend_from_slice(&800u32.to_be_bytes());
        png.extend_from_slice(&400u32.to_be_bytes());
        assert_eq!(png_size(&png), Some((800, 400)));
        assert_eq!(png_size(b"nope"), None);
    }
}
