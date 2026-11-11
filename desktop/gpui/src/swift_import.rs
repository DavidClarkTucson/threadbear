// Bringing the Swift app's settings and unsent drafts across, once, at this
// app's first launch after this came in (core's swift_import.rs says what
// and why), and a note saying what came. Only on the real data folder: a
// mock's or a dev runner's settings never get Mike's. THREADBEAR_SWIFT_DEFAULTS
// names a property list to read in place of the Swift app's own, for the
// specs.
use crate::app::Threadbear;
use crate::settings::Settings;
use crate::style::px;
use crate::tips::Tip as _;
use gpui_kit::component::Sizable as _;
use gpui_kit::prelude::*;
use gpui_kit::*;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;
use threadbear_core::swift_import::{self as rules, Plist};

/// From main(), before the window opens and anything reads the settings or
/// the drafts. The note that says what came across, if anything did.
pub(crate) fn run_once(settings: &mut Settings, cx: &App) -> Option<String> {
    if settings.saved.swift_import.is_some() {
        return None;
    }
    let stand_in = std::env::var_os("THREADBEAR_SWIFT_DEFAULTS");
    if stand_in.is_none() && !crate::installed::on_real_data() {
        return None;
    }
    let xml = match &stand_in {
        Some(file) => read_some(Path::new(file)).ok().flatten().map(|d| String::from_utf8_lossy(&d).into_owned()),
        None => export(),
    };
    let defaults = match xml.as_deref().and_then(rules::parse_plist) {
        Some(Plist::Dict(d)) => d,
        _ => Default::default(),
    };
    // AppKit's frames are measured up from the bottom of the main screen.
    let screen = cx.primary_display().map_or(900., |d| f64::from(d.bounds().size.height));
    let mut current = serde_json::to_value(&settings.saved).ok().and_then(|v| v.as_object().cloned()).unwrap_or_default();
    let mut filled = rules::fill_missing(&mut current, rules::settings(&defaults, screen));
    if let Ok(saved) = serde_json::from_value(Value::Object(current)) {
        settings.saved = saved;
    }
    let home = settings.path().parent().map(Path::to_path_buf);
    let mut drafts = 0;
    let mut failed = None;
    if let Some(home) = &home {
        match bring_drafts(home) {
            Ok(count) => drafts = count,
            Err(e) => failed = Some(e),
        }
        match bring_side_files(home, &defaults) {
            Ok(keys) => filled.extend(keys),
            Err(e) => failed = Some(e),
        }
    }
    // Once: done, so a setting changed here later stays. Not if something
    // couldn't be read or written, as that may go next time; trying again
    // only fills what's still missing, so it's safe to repeat.
    match failed {
        None => settings.saved.swift_import = Some(threadbear_core::inbox::iso_time(crate::now_ms())),
        Some(e) => eprintln!("threadbear: bringing the Swift app's settings across will try again next launch: {e}"),
    }
    settings.save();
    rules::note(&filled, drafts)
}

/// The most of a file or of the Swift app's settings that's read: anything
/// bigger isn't something either app wrote, so it's left where it is.
const MOST_BYTES: u64 = 8 * 1024 * 1024;

/// `path`'s bytes, None when there's none or it's too big to be real, and an
/// error when it's there but couldn't be read.
fn read_some(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match std::fs::metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("couldn't look at {}: {e}", path.display())),
        Ok(meta) if meta.len() > MOST_BYTES => Ok(None),
        Ok(_) => std::fs::read(path).map(Some).map_err(|e| format!("couldn't read {}: {e}", path.display())),
    }
}

/// The Swift app's settings as an XML property list, through cfprefsd, so
/// they're as the Swift app last left them. None where there's no Swift app.
#[cfg(target_os = "macos")]
fn export() -> Option<String> {
    use std::process::{Command, Stdio};
    let out = Command::new("/usr/bin/defaults").args(["export", "com.mikerosoft.threadbear", "-"]).stderr(Stdio::null()).output().ok()?;
    (out.status.success() && out.stdout.len() as u64 <= MOST_BYTES).then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(not(target_os = "macos"))]
fn export() -> Option<String> {
    None
}

/// The Swift app's `app-drafts.json` into this app's `desktop-drafts.json`,
/// where a box is empty here, written as drafts.rs writes it (core's
/// private_file.rs: whole, and only Mike's account can read it).
fn bring_drafts(home: &Path) -> Result<usize, String> {
    // A file that isn't JSON won't be by next time, so it's nothing to bring.
    let Some(swift) = read_some(&home.join("app-drafts.json"))?.and_then(|d| serde_json::from_slice::<Value>(&d).ok()) else { return Ok(0) };
    let path = home.join("desktop-drafts.json");
    let mut current = read_some(&path)?.and_then(|d| serde_json::from_slice::<Value>(&d).ok()).and_then(|v| v.as_object().cloned()).unwrap_or_default();
    let count = rules::fill_drafts(&mut current, &swift);
    if count > 0 {
        let data = serde_json::to_vec(&Value::Object(current)).map_err(|e| e.to_string())?;
        threadbear_core::private_file::write(&path, &data).map_err(|e| format!("couldn't bring the Swift app's drafts across: {e}"))?;
    }
    Ok(count)
}

/// The small files this app keeps the rest in, made from the Swift app's
/// defaults when this app hasn't one yet. Says the keys that came.
fn bring_side_files(home: &Path, defaults: &std::collections::BTreeMap<String, Plist>) -> Result<Vec<String>, String> {
    let mut filled = Vec::new();
    for (file, value, key) in rules::side_files(defaults) {
        let path = home.join(file);
        if path.exists() {
            continue;
        }
        let text = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
        threadbear_core::private_file::write(&path, text.as_bytes()).map_err(|e| format!("couldn't write {}: {e}", path.display()))?;
        filled.push(key.to_owned());
    }
    Ok(filled)
}

impl Threadbear {
    /// The note over the main panel saying what came across: ten seconds,
    /// unless the pointer's on it, or until it's clicked.
    pub(crate) fn show_import_note(&mut self, text: String, cx: &mut Context<Self>) {
        self.import_note = Some(text.clone());
        cx.spawn(async move |this, cx| loop {
            cx.background_executor().timer(Duration::from_secs(10)).await;
            let keep = this.update(cx, |this, cx| {
                if this.import_note.as_deref() != Some(text.as_str()) {
                    return false;
                }
                if this.toast_hovered {
                    return true;
                }
                this.import_note = None;
                cx.notify();
                false
            });
            if !matches!(keep, Ok(true)) {
                break;
            }
        })
        .detach();
        cx.notify();
    }

    /// The note as the inbox-end note looks (`InboxEndToast`): its words and,
    /// quieter, what came; a click takes it down.
    pub(crate) fn render_import_note(&mut self, text: String, cx: &mut Context<Self>) -> AnyElement {
        let (secondary, accent) = (crate::style::secondary_text(cx), crate::style::accent(cx));
        let (title, detail) = text.split_once(": ").unwrap_or((text.as_str(), ""));
        self.toast_capsule("import-note", cx)
            .gap(px(7.))
            .max_w(px(640.))
            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                this.import_note = None;
                cx.notify();
            }))
            .tip(text.clone())
            .child(crate::icons::symbol("square.and.arrow.down").with_size(px(crate::style::size::LABEL)).text_color(accent))
            .child(div().flex_none().child(title.to_owned()))
            .child(div().min_w_0().truncate().text_color(secondary).child(detail.to_owned()))
            .into_any_element()
    }
}
