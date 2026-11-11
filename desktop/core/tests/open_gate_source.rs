// Nothing in the desktop apps hands a link or a file to the system except the
// open gate (desktop/gpui/src/open_gate.rs), as OpenGateSourceTests is for the
// Swift app. Each of Daybreak's findings 58, 59 and 64 (9 Nov 2026) was a new
// button or link that called the system's opener itself, with a URL made by
// pasting a path into `file://`.
use std::path::{Path, PathBuf};

/// What reaches the system's opener, or starts a program, directly. The
/// kit's own rich text, links and menu links count: with no handler of their
/// own they call `open_url` themselves (Codex review of the 10 Nov fix), so
/// they're made only by the gate's `text_view` and `markdown_view`.
const OPENERS: &[&str] = &[
    ".open_url(",
    ".reveal_path(",
    "open_with_system",
    "Command::new(",
    // The terminal library's own way of starting a program in a pty.
    "tty::new(",
    "NSWorkspace",
    "ShellExecute",
    "xdg-open",
    "format!(\"file://",
    "TextView::new(",
    "TextView::markdown(",
    "TextView::html(",
    "PopupMenuItem::link(",
    "Link::new(",
    "EditorHost",
];

/// The gate itself, a test helper that starts the mock runner, and the
/// connection bar's Start runner, which runs launchctl on the runner's own
/// LaunchAgent (and `id -u` for the user) with fixed arguments: nothing a link,
/// a file or the runner names.
const ALLOWED: &[&str] = &["gpui/src/open_gate.rs", "client/tests/mock_runner.rs", "gpui/src/runner_launcher.rs"];

/// Single lines elsewhere, each by its file and the text it holds, allowed
/// exactly as many times as listed, so anything else in the same file is
/// still caught (Codex's review, 11 Nov: the whole Terminal file was let off).
/// The Terminal panel starts your login shell (`$SHELL -l` through `env -u`,
/// threadbear_core::terminal::launch) in a pty in a thread's folder or your
/// home folder, as Terminal would, and runs nothing a reply, a link or a file
/// says (a shell block's command is typed at its prompt for you to run).
/// The hang watch runs macOS's `sample` on the app itself, inside
/// `sandbox-exec` with core's fixed profile (hang_sample.rs), and nothing else.
/// The first launch reads the Swift app's settings with `defaults export` of
/// its own domain, once (swift_import.rs). The self-update (app_update.rs)
/// reads main's tree with git, runs the update script from the app's own
/// bundle under `taskpolicy -b`, and opens the installed app again once this
/// one has quit: fixed programs, on the app's own checkout and bundle.
const ALLOWED_LINES: &[(&str, &str, usize)] = &[
    ("gpui/src/terminal.rs", "let pty = tty::new(&options, self.size.window_size(), 0)?;", 1),
    ("gpui/src/hang_watch.rs", "let mut sample = Command::new(\"/usr/bin/sandbox-exec\");", 1),
    ("gpui/src/app_update.rs", "let started = Command::new(\"/bin/sh\").arg(\"-c\").arg(wait).arg(\"sh\").args(open).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();", 1),
    ("gpui/src/app_update.rs", "let out = Command::new(\"/usr/bin/git\").args(rules::rev_parse_arguments(checkout)).stdin(Stdio::null()).output().ok()?;", 1),
    ("gpui/src/app_update.rs", "let child = Command::new(program).args(arguments).current_dir(installed::home_dir()).stdin(Stdio::null()).stdout(out).stderr(err).spawn();", 1),
    ("gpui/src/swift_import.rs", "let out = Command::new(\"/usr/bin/defaults\").args([\"export\", \"com.mikerosoft.threadbear\", \"-\"]).stderr(Stdio::null()).output().ok()?;", 1),
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target" || n == "golden") {
                continue;
            }
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn nothing_opens_outside_the_gate() {
    let desktop = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_owned();
    let mut files = Vec::new();
    // web/ too: the web runtime's pages are where a link would most want out.
    for crate_dir in ["gpui", "dock", "client", "core/src", "web"] {
        rust_files(&desktop.join(crate_dir), &mut files);
    }
    assert!(files.iter().any(|f| f.ends_with("gpui/src/open_gate.rs")), "the gate wasn't found, so this test isn't looking in the right place");
    let mut found = Vec::new();
    let mut allowed_seen = vec![0; ALLOWED_LINES.len()];
    for file in &files {
        let rel = file.strip_prefix(&desktop).unwrap().to_string_lossy().replace('\\', "/");
        if ALLOWED.contains(&rel.as_str()) {
            continue;
        }
        let text = std::fs::read_to_string(file).unwrap();
        for (n, line) in text.lines().enumerate() {
            if let Some(i) = ALLOWED_LINES.iter().position(|(f, l, _)| *f == rel && line.trim() == *l) {
                allowed_seen[i] += 1;
                continue;
            }
            // Comments may name them; `file://` itself has a `//` in it, so
            // only whole comment lines are passed over.
            if line.trim_start().starts_with("//") {
                continue;
            }
            for opener in OPENERS {
                if line.contains(opener) {
                    found.push(format!("{rel}:{}: {}", n + 1, line.trim()));
                }
            }
        }
    }
    assert!(found.is_empty(), "open links and files through open_gate.rs (open_web, open_link, open_file, reveal), not directly:\n{}", found.join("\n"));
    for ((file, line, times), seen) in ALLOWED_LINES.iter().zip(allowed_seen) {
        assert_eq!(seen, *times, "{file} should have `{line}` {times} time(s): if it has moved or changed, say why it's safe and change ALLOWED_LINES");
    }
}

/// The gate's own sink is the only `open_url` in it, so everything in it
/// goes through the checks above it.
#[test]
fn the_gate_opens_in_one_place() {
    let gate = Path::new(env!("CARGO_MANIFEST_DIR")).join("../gpui/src/open_gate.rs");
    let text = std::fs::read_to_string(gate).unwrap();
    assert_eq!(text.matches(".open_url(").count(), 1);
    assert!(!text.contains("format!(\"file://"));
}
