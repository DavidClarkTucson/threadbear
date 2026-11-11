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

#[cfg(test)]
mod tests {
    use super::*;

    fn info(origin: Option<&str>) -> BuildInfo {
        BuildInfo { origin: origin.map(str::to_owned), ..Default::default() }
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
