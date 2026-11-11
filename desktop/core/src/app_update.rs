//! What the installed app was built from, and the rules it follows main by,
//! a port of the Swift app's `BuildInfo` and `AppUpdate`. The bundle's
//! `Contents/Resources/build.json` says the commit, the tree of `desktop/` at
//! it and the checkout whose main to follow; crash reports go by its origin
//! (only Mike's own repo reports to his Sentry).
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// `build.json`, as desktop/scripts/make-bundle.sh writes it: the same fields
/// as the Swift app's, so one reader would do for both.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BuildInfo {
    pub commit: String,
    pub short: String,
    /// The tree hash of `desktop/` at that commit (`git rev-parse
    /// <commit>:desktop`). Only a change under it changes what runs, so this,
    /// not the commit, says whether a build would be any different.
    pub app_tree: String,
    pub built_at: String,
    /// The checkout it was built from, whose main the updater follows.
    pub checkout: String,
    pub configuration: String,
    /// The checkout's origin remote when it was built.
    pub origin: Option<String>,
}

impl BuildInfo {
    pub fn path_in(bundle: &Path) -> PathBuf {
        bundle.join("Contents").join("Resources").join("build.json")
    }

    pub fn read(bundle: &Path) -> Option<BuildInfo> {
        let data = std::fs::read(Self::path_in(bundle)).ok()?;
        serde_json::from_slice(&data).ok()
    }

    /// github.com/mikecann/threadbear, however the URL was written.
    pub fn is_mikes_repo(&self) -> bool {
        self.origin.as_deref().is_some_and(|o| repo_key(o) == "github.com/mikecann/threadbear")
    }
}

/// An origin URL as host/owner/name, lowercased, however it was written
/// (`isMikesRepo`, `repoKey` in runner/src/upstream.ts).
pub fn repo_key(origin: &str) -> String {
    let mut key = origin.trim().to_lowercase();
    if let Some(rest) = key.strip_prefix("git@") {
        key = rest.replace(':', "/");
    }
    if let Some(at) = key.find("://") {
        key = key[at + 3..].to_owned();
    }
    if let Some(at) = key.find('@') {
        if !key[..at].contains('/') {
            key = key[at + 1..].to_owned();
        }
    }
    let key = key.strip_suffix('/').unwrap_or(&key);
    key.strip_suffix(".git").unwrap_or(key).to_owned()
}

/// The bundle an executable runs from: `<bundle>/Contents/MacOS/<exe>`.
pub fn bundle_of(exe: &Path) -> Option<PathBuf> {
    let macos = exe.parent()?;
    let contents = macos.parent()?;
    if macos.file_name()? != "MacOS" || contents.file_name()? != "Contents" {
        return None;
    }
    contents.parent().map(Path::to_path_buf)
}

// Following main (the Swift app's `AppUpdate`): what to build, and putting a
// staged build in the installed one's place.

/// `git rev-parse` for main's `desktop/` tree in a checkout: what a build of
/// main would be made from. The build takes three files from the Swift app's
/// folder too (two pictures and the diagram library): a change to only those
/// comes in with the next change under desktop/, rather than every change to
/// the Swift app's resources making a build.
pub fn rev_parse_arguments(checkout: &str) -> Vec<String> {
    ["-C", checkout, "rev-parse", "refs/heads/main:desktop"].map(String::from).to_vec()
}

/// The tree from `git rev-parse`'s output.
pub fn tree_from_rev_parse(output: &str) -> String {
    output.lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("+")
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Nothing,
    /// Main's tree differs from what's running and nothing staged, being
    /// built or known to fail matches it.
    Build(String),
    /// What's staged no longer matches main, and main is what's running
    /// already: drop it.
    DropStaged,
}

pub fn decide(main_tree: &str, running: &BuildInfo, staged: Option<&BuildInfo>, building: Option<&str>, failed: Option<&str>) -> Action {
    if main_tree == running.app_tree {
        return if staged.is_some() { Action::DropStaged } else { Action::Nothing };
    }
    if staged.is_some_and(|s| s.app_tree == main_tree) || building == Some(main_tree) || failed == Some(main_tree) {
        return Action::Nothing;
    }
    Action::Build(main_tree.to_owned())
}

/// Puts the staged bundle in the installed one's place in one exchange, so
/// there's never a moment with no app there: two renames left none if the
/// app was killed between them (Codex's review, 12 Nov). The old bundle (the
/// running one, when this is done on quit) then goes to `old`, to be deleted
/// at the next launch. Killed before that, the old one is left staged, and
/// `apply_staged` won't put an older build back.
pub fn swap(install: &Path, next: &Path, old: &Path) -> std::io::Result<()> {
    if old.exists() {
        std::fs::remove_dir_all(old)?;
    }
    if !install.exists() {
        return std::fs::rename(next, install);
    }
    exchange(next, install)?;
    std::fs::rename(next, old)
}

/// Swaps what two paths name, at once.
#[cfg(target_os = "macos")]
fn exchange(a: &Path, b: &Path) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt as _;
    let a = std::ffi::CString::new(a.as_os_str().as_bytes())?;
    let b = std::ffi::CString::new(b.as_os_str().as_bytes())?;
    // SAFETY: two NUL-terminated paths that live until the call returns.
    if unsafe { libc::renamex_np(a.as_ptr(), b.as_ptr(), libc::RENAME_SWAP) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(target_os = "linux")]
fn exchange(a: &Path, b: &Path) -> std::io::Result<()> {
    use std::os::unix::ffi::OsStrExt as _;
    let a = std::ffi::CString::new(a.as_os_str().as_bytes())?;
    let b = std::ffi::CString::new(b.as_os_str().as_bytes())?;
    // SAFETY: two NUL-terminated paths that live until the call returns.
    if unsafe { libc::renameat2(libc::AT_FDCWD, a.as_ptr(), libc::AT_FDCWD, b.as_ptr(), libc::RENAME_EXCHANGE) } == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error())
    }
}

/// Elsewhere (Windows has its own way in), two renames, putting the old one
/// back if the second fails.
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn exchange(a: &Path, b: &Path) -> std::io::Result<()> {
    let aside = b.with_extension("swapping");
    std::fs::rename(b, &aside)?;
    if let Err(e) = std::fs::rename(a, b) {
        let _ = std::fs::rename(&aside, b);
        return Err(e);
    }
    std::fs::rename(&aside, a)
}

/// Swaps a staged bundle in if it would change anything, and says what was
/// swapped in. A staged bundle that matches what's running (a build that
/// finished after the same tree went in another way), or that has no
/// build.json, is dropped. Leftovers of an earlier swap go first.
pub fn apply_staged(install: &Path, next: &Path, old: &Path, running: Option<&BuildInfo>) -> Result<Option<BuildInfo>, String> {
    if old.exists() {
        let _ = std::fs::remove_dir_all(old);
    }
    if !next.exists() {
        return Ok(None);
    }
    // Not what's running, and not older than it: an old bundle can be left
    // staged by a swap killed half done.
    let Some(staged) = BuildInfo::read(next).filter(|s| running.is_none_or(|r| r.app_tree != s.app_tree && !older(s, r))) else {
        let _ = std::fs::remove_dir_all(next);
        return Ok(None);
    };
    swap(install, next, old).map_err(|e| format!("couldn't put {} in place: {e}", next.display()))?;
    Ok(Some(staged))
}

/// Built before `than`, by their `built_at` times (ISO, so they sort as
/// strings); unknown either side counts as not older.
fn older(build: &BuildInfo, than: &BuildInfo) -> bool {
    !build.built_at.is_empty() && !than.built_at.is_empty() && build.built_at < than.built_at
}

/// The idle time, with the app in front, that counts as not using it.
pub const AUTO_RESTART_IDLE_SECONDS: f64 = 5.;

/// Whether to restart on its own to take a staged build: nothing typed would
/// be lost, the threads are in, and Mike isn't in the app (another app is in
/// front), or he hasn't touched it for a while and the restart wouldn't be
/// seen (`in_place`: a handover with the window on screen). Running threads
/// don't hold it up: the runner owns the turns and the app replays what it
/// missed when it reconnects (`shouldAutoRestart`).
pub fn should_auto_restart(staged: bool, loaded: bool, has_drafts: bool, active: bool, idle_for: f64, in_place: bool) -> bool {
    if !staged || !loaded || has_drafts {
        return false;
    }
    !active || (in_place && idle_for >= AUTO_RESTART_IDLE_SECONDS)
}

/// The update script's arguments, run at background priority (`taskpolicy
/// -b`, as Mike asks of every build on his M5): throttled CPU and disk, off
/// the cores he's using, inherited by everything it starts, the copy to a
/// build host and back included.
pub fn build_command(script: &str, checkout: &str, next: &str) -> (String, Vec<String>) {
    ("/usr/sbin/taskpolicy".into(), ["-b", "/bin/bash", script, checkout, "--next", next].map(String::from).to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(origin: Option<&str>) -> BuildInfo {
        BuildInfo { origin: origin.map(str::to_owned), ..Default::default() }
    }

    fn build(short: &str, tree: &str) -> BuildInfo {
        BuildInfo { commit: format!("{short}000"), short: short.into(), app_tree: tree.into(), configuration: "release".into(), ..Default::default() }
    }

    /// A bundle folder with a build.json and a marker file naming it.
    fn bundle(dir: &Path, name: &str, info: &BuildInfo) -> PathBuf {
        let url = dir.join(name);
        std::fs::create_dir_all(url.join("Contents/Resources")).unwrap();
        std::fs::write(BuildInfo::path_in(&url), serde_json::to_vec(info).unwrap()).unwrap();
        std::fs::write(url.join("Contents/marker"), &info.short).unwrap();
        url
    }

    fn marker(url: &Path) -> Option<String> {
        std::fs::read_to_string(url.join("Contents/marker")).ok()
    }

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tb-app-update-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_build_runs_at_background_priority() {
        let (program, args) = build_command("/b/stage-update.sh", "/repo", "/next");
        assert_eq!(program, "/usr/sbin/taskpolicy");
        assert_eq!(args, ["-b", "/bin/bash", "/b/stage-update.sh", "/repo", "--next", "/next"]);
    }

    #[test]
    fn decides_whether_to_build() {
        let running = build("aaa", "t1");
        assert_eq!(decide("t1", &running, None, None, None), Action::Nothing);
        assert_eq!(decide("t2", &running, None, None, None), Action::Build("t2".into()));
        // Already staged, being built or known to fail: leave it.
        assert_eq!(decide("t2", &running, Some(&build("bbb", "t2")), None, None), Action::Nothing);
        assert_eq!(decide("t2", &running, None, Some("t2"), None), Action::Nothing);
        assert_eq!(decide("t2", &running, None, None, Some("t2")), Action::Nothing);
        // Main moved on past a failed or staged tree: build the new one.
        assert_eq!(decide("t3", &running, Some(&build("bbb", "t2")), None, Some("t2")), Action::Build("t3".into()));
        // Main is back to what's running while something else is staged.
        assert_eq!(decide("t1", &running, Some(&build("bbb", "t2")), None, None), Action::DropStaged);
    }

    #[test]
    fn restarts_by_itself_only_when_nothing_would_be_interrupted() {
        let idle = AUTO_RESTART_IDLE_SECONDS;
        // Another app in front: straight away, handover or not.
        assert!(should_auto_restart(true, true, false, false, 0., true));
        assert!(should_auto_restart(true, true, false, false, 0., false));
        // In front: only after a while with no input, and only unseen.
        assert!(!should_auto_restart(true, true, false, true, idle - 1., true));
        assert!(should_auto_restart(true, true, false, true, idle, true));
        assert!(!should_auto_restart(true, true, false, true, idle * 100., false));
        // Unsent words, no build or no thread list: never.
        assert!(!should_auto_restart(true, true, true, false, idle, true));
        assert!(!should_auto_restart(false, true, false, false, idle, true));
        assert!(!should_auto_restart(true, false, false, false, idle, true));
    }

    #[test]
    fn the_tree_is_desktops_from_rev_parse() {
        assert_eq!(rev_parse_arguments("/repo"), ["-C", "/repo", "rev-parse", "refs/heads/main:desktop"]);
        assert_eq!(tree_from_rev_parse("aaa111\n"), "aaa111");
    }

    #[test]
    fn swap_puts_the_staged_bundle_in_place_and_keeps_the_old() {
        let dir = temp("swap");
        let install = bundle(&dir, "Threadbear GPUI.app", &build("aaa", "t1"));
        let next = bundle(&dir, ".Threadbear GPUI.app.next", &build("bbb", "t2"));
        let old = dir.join(".Threadbear GPUI.app.old");
        swap(&install, &next, &old).unwrap();
        assert_eq!(marker(&install).as_deref(), Some("bbb"));
        assert_eq!(marker(&old).as_deref(), Some("aaa"));
        assert!(!next.exists());
        // A swap that can't happen leaves the installed one where it is: the
        // exchange is one step, done or not.
        assert!(swap(&install, &dir.join("gone.app"), &old).is_err());
        assert_eq!(marker(&install).as_deref(), Some("bbb"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_staged_swaps_only_a_different_build() {
        let dir = temp("apply");
        let install = bundle(&dir, "Threadbear GPUI.app", &build("aaa", "t1"));
        let old = dir.join(".Threadbear GPUI.app.old");
        // The same tree as what's running: dropped, nothing swapped.
        let next = bundle(&dir, ".Threadbear GPUI.app.next", &build("aab", "t1"));
        assert_eq!(apply_staged(&install, &next, &old, Some(&build("aaa", "t1"))), Ok(None));
        assert!(!next.exists());
        assert_eq!(marker(&install).as_deref(), Some("aaa"));
        // A leftover from an earlier swap is cleared, and a new build goes in.
        bundle(&dir, ".Threadbear GPUI.app.old", &build("zzz", "t0"));
        let next = bundle(&dir, ".Threadbear GPUI.app.next", &build("bbb", "t2"));
        assert_eq!(apply_staged(&install, &next, &old, Some(&build("aaa", "t1"))).unwrap().map(|b| b.short), Some("bbb".to_owned()));
        assert_eq!(marker(&install).as_deref(), Some("bbb"));
        assert_eq!(marker(&old).as_deref(), Some("aaa"));
        // Nothing staged: nothing happens.
        assert_eq!(apply_staged(&install, &next, &old, Some(&build("bbb", "t2"))), Ok(None));
        // A staged folder with no build.json is dropped.
        std::fs::create_dir_all(&next).unwrap();
        assert_eq!(apply_staged(&install, &next, &old, Some(&build("bbb", "t2"))), Ok(None));
        assert!(!next.exists());
        // An older build left staged (a swap killed after its exchange, before
        // its last rename) isn't put back over a newer one.
        let mut older = build("ccc", "t3");
        older.built_at = "2026-11-11T10:00:00Z".into();
        let mut newer = build("bbb", "t2");
        newer.built_at = "2026-11-12T10:00:00Z".into();
        let next = bundle(&dir, ".Threadbear GPUI.app.next", &older);
        assert_eq!(apply_staged(&install, &next, &old, Some(&newer)), Ok(None));
        assert!(!next.exists());
        assert_eq!(marker(&install).as_deref(), Some("bbb"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // Only a build of Mike's own repo reports to his Sentry.
    #[test]
    fn only_mikes_repo_counts_as_mikes() {
        for url in ["https://github.com/mikecann/threadbear.git", "https://github.com/MikeCann/threadbear", "git@github.com:mikecann/threadbear.git", "ssh://git@github.com/mikecann/threadbear", "https://x-access-token@github.com/mikecann/threadbear/"] {
            assert!(info(Some(url)).is_mikes_repo(), "{url}");
        }
        assert!(!info(Some("https://github.com/someone/threadbear.git")).is_mikes_repo());
        assert!(!info(Some("")).is_mikes_repo());
        assert!(!info(None).is_mikes_repo(), "built before origin was recorded, or no origin");
    }

    #[test]
    fn reads_what_the_bundle_script_writes() {
        let dir = std::env::temp_dir().join(format!("tb-build-info-{}", std::process::id()));
        let bundle = dir.join("Threadbear GPUI.app");
        std::fs::create_dir_all(bundle.join("Contents/Resources")).unwrap();
        std::fs::write(
            BuildInfo::path_in(&bundle),
            r#"{"commit":"abc1234def","short":"abc1234","appTree":"t9","builtAt":"2026-11-11T10:00:00+00:00","checkout":"/repo","configuration":"release","origin":"git@github.com:mikecann/threadbear.git"}"#,
        )
        .unwrap();
        let read = BuildInfo::read(&bundle).unwrap();
        assert_eq!((read.short.as_str(), read.app_tree.as_str()), ("abc1234", "t9"));
        assert!(read.is_mikes_repo());
        assert_eq!(BuildInfo::read(&dir.join("missing.app")), None);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_bundle_is_found_from_its_executable() {
        let exe = Path::new("/Users/me/Applications/Threadbear GPUI.app/Contents/MacOS/threadbear-gpui");
        assert_eq!(bundle_of(exe).as_deref(), Some(Path::new("/Users/me/Applications/Threadbear GPUI.app")));
        assert_eq!(bundle_of(Path::new("/repo/desktop/target/debug/threadbear-gpui")), None);
    }
}
