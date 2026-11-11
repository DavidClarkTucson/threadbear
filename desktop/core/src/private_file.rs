//! Files only this user's account may read, written whole: what's typed in a
//! box (drafts, review drafts), the automation port's token, the sign-in
//! script, crash and hang records, the app's own settings and caches.
//!
//! Each is written into a new file of its own beside the real one, made with
//! owner-only permissions from the start (never written first and tightened
//! after), under a name nobody can guess or make first, and then renamed over
//! the real one. Daybreak, 11 Nov (findings 88 and 94): drafts and review
//! drafts went to `<name>.json.tmp` with `fs::write`, so under umask 022 the
//! file was 0644 until the chmod after it, and another account on the machine
//! that opened it in that moment kept reading it after the chmod and rename.
//! A fixed temp name had two more holes: a file someone else made there first
//! was written into (theirs, and open to them), and a symlink there was
//! followed. A new file with `O_EXCL` has neither.
//!
//! Every private file in `desktop/` goes through here, so there's one place
//! to get this right (tests/private_file_source.rs fails on a writer that
//! doesn't). Off Unix there are no modes: a file and any folder made for it
//! take the access of the folder they're in, which in the user's own profile
//! is that user's.

use std::io::Write as _;
use std::path::{Path, PathBuf};

/// Writes `data` as `path`, readable and writable by this user only.
pub fn write(path: &Path, data: &[u8]) -> std::io::Result<()> {
    write_with_mode(path, data, 0o600)
}

/// `write`, with the file's mode given: 0o700 for a script only this user
/// may run. Group and other bits are never set, whatever's asked.
pub fn write_with_mode(path: &Path, data: &[u8], mode: u32) -> std::io::Result<()> {
    let dir = match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
        _ => PathBuf::from("."),
    };
    make_dir(&dir)?;
    let (mut file, temp) = new_file(&dir, path, mode & 0o700)?;
    let written = file.write_all(data).and_then(|()| file.sync_all());
    drop(file);
    if let Err(e) = written.and_then(|()| std::fs::rename(&temp, path)) {
        let _ = std::fs::remove_file(&temp);
        return Err(e);
    }
    Ok(())
}

/// A folder that isn't there yet is made for this user only. One that is
/// keeps the permissions it has.
fn make_dir(dir: &Path) -> std::io::Result<()> {
    if dir.is_dir() {
        return Ok(());
    }
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    builder.create(dir)
}

/// A file that wasn't there before, beside `path`, with only `mode`'s owner
/// bits. `create_new` is `O_CREAT | O_EXCL`, which also refuses to follow a
/// symlink at that name; the name has the process, the time and a count in
/// it, and a clash (someone made it first) just tries another.
fn new_file(dir: &Path, path: &Path, mode: u32) -> std::io::Result<(std::fs::File, PathBuf)> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNT: AtomicU64 = AtomicU64::new(0);
    let name = path.file_name().map_or_else(|| "file".into(), |n| n.to_string_lossy().into_owned());
    let mut last = None;
    for _ in 0..16 {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos());
        let temp = dir.join(format!(".{name}.{}-{nanos:x}-{}.tmp", std::process::id(), COUNT.fetch_add(1, Ordering::Relaxed)));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, mode);
        #[cfg(not(unix))]
        let _ = mode;
        match options.open(&temp) {
            Ok(file) => return Ok((file, temp)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => last = Some(e),
            Err(e) => return Err(e),
        }
    }
    Err(last.unwrap_or_else(|| std::io::Error::other("couldn't make a file to write into")))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt as _;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tb-private-file-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // As a runner home can be: listable and openable by everyone.
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        dir
    }

    fn mode(path: &Path) -> u32 {
        std::fs::symlink_metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn written_whole_and_only_for_its_owner() {
        let dir = scratch("whole");
        let path = dir.join("desktop-drafts.json");
        write(&path, b"{\"t\":{\"text\":\"secret\"}}").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"{\"t\":{\"text\":\"secret\"}}");
        assert_eq!(mode(&path), 0o600);
        // An older file someone made readable is replaced, not written into.
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        write(&path, b"two").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"two");
        assert_eq!(mode(&path), 0o600);
        // Nothing left behind.
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        write_with_mode(&dir.join("sign-in.sh"), b"#!/bin/sh\n", 0o755).unwrap();
        assert_eq!(mode(&dir.join("sign-in.sh")), 0o700, "never group or other bits");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_new_folder_is_the_owners_only() {
        let dir = scratch("folder");
        let path = dir.join("a/b/desktop-review-drafts.json");
        write(&path, b"x").unwrap();
        assert_eq!(mode(&dir.join("a")), 0o700);
        assert_eq!(mode(&dir.join("a/b")), 0o700);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The old writers' fixed `<name>.json.tmp`, made first by someone else:
    /// theirs to read, or a symlink to somewhere of theirs. Neither is
    /// touched, and the real file is still only the owner's.
    #[test]
    fn a_file_or_symlink_made_first_at_the_old_temp_name_is_left_alone() {
        let dir = scratch("planted");
        let path = dir.join("desktop-drafts.json");
        let planted = dir.join("desktop-drafts.json.tmp");
        std::fs::write(&planted, b"").unwrap();
        std::fs::set_permissions(&planted, std::fs::Permissions::from_mode(0o666)).unwrap();
        let elsewhere = dir.join("theirs");
        std::os::unix::fs::symlink(&elsewhere, dir.join("desktop-review-drafts.json.tmp")).unwrap();
        write(&path, b"secret").unwrap();
        write(&dir.join("desktop-review-drafts.json"), b"secret").unwrap();
        assert_eq!(std::fs::read(&planted).unwrap(), b"");
        assert!(!elsewhere.exists(), "the symlink was followed");
        assert_eq!(mode(&path), 0o600);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The race itself: another account opening each temp file as it
    /// appears. Played first against the old way (write, then chmod), to
    /// show the watcher can catch it here, then against `write`, which it
    /// must never catch with group or other bits set. A umask that already
    /// keeps them off (077) can't show the old way's hole, so only the
    /// second half is checked then.
    #[test]
    fn no_temp_file_is_ever_readable_by_anyone_else() {
        fn watch(dir: PathBuf, rounds: usize, writer: impl Fn(&Path, usize)) -> usize {
            let stop = Arc::new(AtomicBool::new(false));
            let caught = Arc::new(AtomicUsize::new(0));
            let watcher = {
                let (stop, caught, dir) = (stop.clone(), caught.clone(), dir.clone());
                std::thread::spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                            if let Ok(meta) = std::fs::symlink_metadata(entry.path()) {
                                if meta.permissions().mode() & 0o077 != 0 {
                                    caught.fetch_add(1, Ordering::Relaxed);
                                }
                            }
                        }
                    }
                })
            };
            let path = dir.join("desktop-drafts.json");
            for i in 0..rounds {
                writer(&path, i);
            }
            stop.store(true, Ordering::Relaxed);
            watcher.join().unwrap();
            caught.load(Ordering::Relaxed)
        }
        let old = scratch("race-old");
        let caught_old = watch(old.clone(), 3000, |path, i| {
            let temp = path.with_extension("json.tmp");
            std::fs::write(&temp, format!("secret {i}")).unwrap();
            std::fs::set_permissions(&temp, std::fs::Permissions::from_mode(0o600)).unwrap();
            std::fs::rename(&temp, path).unwrap();
        });
        if caught_old == 0 {
            eprintln!("the old way wasn't caught here (a strict umask?), so only today's is checked");
        }
        let new = scratch("race-new");
        let caught_new = watch(new.clone(), 1000, |path, i| write(path, format!("secret {i}").as_bytes()).unwrap());
        assert_eq!(caught_new, 0, "a temp file was readable by others");
        let _ = std::fs::remove_dir_all(&old);
        let _ = std::fs::remove_dir_all(&new);
    }
}
