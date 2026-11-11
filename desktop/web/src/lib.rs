// The web runtime for the GPUI app (docs/gpui-port.md, "Choice 1: the web
// view"): CEF's official Chromium build, fetched the first time something
// needs it (download.rs, checked by threadbear_core::web_runtime), started
// from the data folder in the app's own process (mac/host.rs), and the pages
// it shows, all sealed the way the Swift app seals its web views (mac/sealed.rs).
//
// Nothing here draws GPUI; desktop/gpui shows the state and hosts the pages.
// On Windows and Linux there's no runtime yet, so everything says so and the
// app shows source and offers the browser, as it does before an install.
pub mod download;
#[cfg(target_os = "macos")]
mod mac;

pub use download::Progress;

#[cfg(target_os = "macos")]
pub use mac::host::{bundle_identifier, start, status, Launch, Status};
#[cfg(target_os = "macos")]
pub use mac::offscreen::{picture, png_size, Offscreen, Options, Reply};

/// The helper processes' main (the threadbear-web-helper binary).
pub fn helper_main() -> i32 {
    #[cfg(target_os = "macos")]
    return mac::helper::run();
    #[cfg(not(target_os = "macos"))]
    {
        eprintln!("threadbear-web-helper: there's no web runtime for this system yet");
        1
    }
}

#[cfg(not(target_os = "macos"))]
mod elsewhere {
    use std::path::PathBuf;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Status {
        NotStarted,
        Starting,
        Ready,
        Failed(String),
    }

    pub struct Launch {
        pub bundle: PathBuf,
        pub framework: PathBuf,
        pub helper: PathBuf,
        pub profile: PathBuf,
        pub log_file: PathBuf,
    }

    pub fn status() -> Status {
        Status::Failed("there's no web runtime for this system yet".into())
    }

    pub fn start(_launch: Launch) -> Result<(), String> {
        Err("there's no web runtime for this system yet".into())
    }

    pub fn bundle_identifier() -> Option<String> {
        None
    }
}
#[cfg(not(target_os = "macos"))]
pub use elsewhere::*;
