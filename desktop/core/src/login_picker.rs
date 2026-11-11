// The composer's login picker and the model picker's pinned line: the logins
// grouped by provider, each with its note and the room it has left
// (AccountMenu and AccountUsageText.menuLine and subtitle in
// AccountUsageText.swift), and where Unpin puts a thread pinned to a login
// (Pools.unpinTarget, ThreadSettings.pinnedLine). The GPUI app draws these
// (desktop/gpui/src/composer_account.rs); its scripts read the same lines.
use crate::logins::{provider_title, Account, Pool};
use crate::new_thread::{chip_badge, login_name, main_windows, new_thread_pool, percent, pool_for_folder, reset_time, tightest, usage_of, AccountUsage, ChipBadge, UsageWindow};
use chrono::{DateTime, TimeZone};

/// "5h", "week": the short names the menu uses (AccountUsageText.shortName).
fn short_name(window: &UsageWindow) -> String {
    match window.kind.as_str() {
        "five_hour" => "5h".to_owned(),
        "weekly" => "week".to_owned(),
        _ => window.title.clone(),
    }
}

/// A moment in milliseconds in `now`'s zone.
fn at<Tz: TimeZone>(ms: i64, now: &DateTime<Tz>) -> DateTime<Tz> {
    now.timezone().timestamp_millis_opt(ms).single().unwrap_or_else(|| now.clone())
}

/// The menu's usage line (AccountUsageText.menuLine): "5h 89% · week 78%,
/// resets Thu 1 pm", the reset being the tighter window's; "Limit reached,
/// resets 3:40 pm" for a login at its limit. None with nothing known.
pub fn menu_line<Tz: TimeZone>(usage: &AccountUsage, now: &DateTime<Tz>) -> Option<String> {
    let main = main_windows(usage);
    if usage.blocked {
        let reset = main.iter().filter(|w| w.remaining_percent < 1.).filter_map(|w| w.resets_at).min().or_else(|| main.iter().filter_map(|w| w.resets_at).min());
        return Some(match reset {
            Some(ms) => format!("Limit reached, resets {}", reset_time(&at(ms, now), now)),
            None => "Limit reached".to_owned(),
        });
    }
    if main.is_empty() {
        return None;
    }
    let mut line = main.iter().map(|w| format!("{} {}", short_name(w), percent(w.remaining_percent))).collect::<Vec<_>>().join(" · ");
    if let Some(ms) = tightest(usage).and_then(|w| w.resets_at) {
        line.push_str(&format!(", resets {}", reset_time(&at(ms, now), now)));
    }
    Some(line)
}

/// A login's line under its name (AccountUsageText.subtitle): its note,
/// then the room left.
pub fn subtitle<Tz: TimeZone>(account: &Account, now: &DateTime<Tz>) -> Option<String> {
    let parts: Vec<String> = [account.note.clone(), usage_of(account).and_then(|u| menu_line(&u, now))].into_iter().flatten().filter(|p| !p.is_empty()).collect();
    (!parts.is_empty()).then(|| parts.join(" · "))
}

/// A login in the picker (AccountMenu.Item).
#[derive(Clone, Debug, PartialEq)]
pub struct LoginItem {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub is_selected: bool,
    pub is_blocked: bool,
    /// Under a quarter left (orange) or at its limit (red), as on the chip.
    pub badge: Option<ChipBadge>,
}

/// A provider's logins (AccountMenu.Section).
#[derive(Clone, Debug, PartialEq)]
pub struct LoginSection {
    pub id: String,
    pub title: String,
    pub items: Vec<LoginItem>,
}

/// The picker's sections (AccountMenu.sections): one a provider, Claude then
/// Codex then any other in the order they come, each login by its address
/// with its note and room under it. A login at its limit stays pickable.
pub fn login_sections<Tz: TimeZone>(accounts: &[Account], selected: &str, now: &DateTime<Tz>) -> Vec<LoginSection> {
    let mut sections: Vec<LoginSection> = Vec::new();
    for account in accounts {
        let provider = account.provider_id().to_owned();
        let item = LoginItem {
            id: account.id.clone(),
            title: account.name(),
            subtitle: subtitle(account, now),
            is_selected: account.id == selected,
            is_blocked: usage_of(account).is_some_and(|u| u.blocked),
            badge: chip_badge(Some(account)),
        };
        match sections.iter_mut().find(|s| s.id == provider) {
            Some(section) => section.items.push(item),
            None => sections.push(LoginSection { title: provider_title(&provider), id: provider, items: vec![item] }),
        }
    }
    let rank = |id: &str| ["claude", "codex"].iter().position(|p| *p == id).unwrap_or(2);
    // Stable, so the others keep their order.
    sections.sort_by_key(|s| rank(&s.id));
    sections
}

/// The picker as text, for the automation's `composer-menu` (AccountMenu.
/// lines): `#` on a section, `[x]` on the chosen login, its line after a
/// dash, then where the numbers came from.
pub fn login_lines(sections: &[LoginSection], footer: Option<&str>) -> Vec<String> {
    let mut lines = Vec::new();
    for section in sections {
        lines.push(format!("# {}", section.title));
        for item in &section.items {
            let mut line = format!("{}{}", if item.is_selected { "[x] " } else { "[ ] " }, item.title);
            if let Some(subtitle) = &item.subtitle {
                line.push_str(&format!(" - {subtitle}"));
            }
            lines.push(line);
        }
    }
    if let Some(footer) = footer {
        lines.push(format!("--- {footer}"));
    }
    lines
}

/// "Pinned to gmail", under the model picker's effort levels for a thread
/// pinned to a login (ThreadSettings.pinnedLine).
pub fn pinned_line(account_id: &str, accounts: &[Account]) -> String {
    format!("Pinned to {}", login_name(account_id, accounts))
}

/// The pool Unpin puts a pinned thread in (Pools.unpinTarget): one holding
/// its login, so it stays where it is until that login runs dry, its
/// folder's or the last one picked first; with none holding it, the pool a
/// new thread there would start in.
pub fn unpin_target(pools: &[Pool], account_id: &str, cwd: &str, home: &str, last: Option<&str>) -> Option<String> {
    let holding: Vec<&Pool> = pools.iter().filter(|p| p.account_ids.iter().any(|a| a == account_id)).collect();
    let folder = pool_for_folder(pools, cwd, home);
    for candidate in [folder.as_deref(), last].into_iter().flatten() {
        if holding.iter().any(|p| p.id == candidate) {
            return Some(candidate.to_owned());
        }
    }
    if let Some(first) = holding.first() {
        return Some(first.id.clone());
    }
    new_thread_pool(pools, None, cwd, home, last, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::FixedOffset;
    use serde_json::json;

    fn account(id: &str, provider: &str, email: &str, note: Option<&str>, usage: serde_json::Value) -> Account {
        Account { id: id.into(), provider: provider.into(), title: email.into(), email: Some(email.into()), note: note.map(str::to_owned), usage: (!usage.is_null()).then_some(usage), ..Account::default() }
    }

    fn now() -> DateTime<FixedOffset> {
        FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2026, 11, 11, 9, 0, 0).unwrap()
    }

    #[test]
    fn a_logins_line_says_its_note_and_room() {
        let usage = json!({ "windows": [
            { "kind": "five_hour", "title": "5-hour", "remainingPercent": 89.6, "resetsAt": "2026-11-11T13:40:00Z" },
            { "kind": "weekly", "title": "Weekly", "remainingPercent": 78, "resetsAt": "2026-11-12T13:00:00Z" },
            { "kind": "weekly_model", "title": "Weekly Fable", "remainingPercent": 2, "resetsAt": "2026-11-12T13:00:00Z" },
        ] });
        let main = account("claude-main", "claude", "mike@example.com", Some("Convex work only"), usage);
        // The tighter main window's reset; a model's own week never decides it.
        assert_eq!(subtitle(&main, &now()).as_deref(), Some("Convex work only · 5h 89% · week 78%, resets tomorrow 1 pm"));
        let blocked = account("claude-4", "claude", "mike+4@example.com", None, json!({ "blocked": true, "windows": [
            { "kind": "five_hour", "title": "5-hour", "remainingPercent": 0, "resetsAt": "2026-11-11T10:30:00Z" },
            { "kind": "weekly", "title": "Weekly", "remainingPercent": 31, "resetsAt": "2026-11-13T09:00:00Z" },
        ] }));
        assert_eq!(subtitle(&blocked, &now()).as_deref(), Some("Limit reached, resets 10:30 am"));
        assert_eq!(subtitle(&account("x", "claude", "x@y.z", None, serde_json::Value::Null), &now()), None);
        assert_eq!(subtitle(&account("x", "claude", "x@y.z", Some(""), json!({ "windows": [] })), &now()), None);
    }

    #[test]
    fn the_picker_groups_logins_by_provider_claude_first() {
        let accounts = vec![
            account("codex-main", "codex", "mike@example.com", None, json!({ "windows": [{ "kind": "weekly", "title": "Weekly", "remainingPercent": 12 }] })),
            account("claude-main", "claude", "mike@example.com", None, serde_json::Value::Null),
            account("gemini", "gemini", "g@example.com", None, serde_json::Value::Null),
            account("claude-2", "", "work@example.com", Some("Convex work only"), json!({ "blocked": true, "windows": [] })),
        ];
        let sections = login_sections(&accounts, "claude-2", &now());
        assert_eq!(sections.iter().map(|s| s.title.as_str()).collect::<Vec<_>>(), ["Claude", "Codex", "Gemini"]);
        assert_eq!(sections[0].items.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(), ["claude-main", "claude-2"], "a login with no provider is Claude's");
        assert!(sections[0].items[1].is_selected && sections[0].items[1].is_blocked);
        assert_eq!(sections[0].items[1].badge, Some(ChipBadge::Blocked));
        assert_eq!(sections[1].items[0].badge, Some(ChipBadge::Low("12%".into())));
        assert_eq!(
            login_lines(&sections, Some("Usage from Token Stats at 9 am")),
            [
                "# Claude",
                "[ ] mike@example.com",
                "[x] work@example.com - Convex work only · Limit reached",
                "# Codex",
                "[ ] mike@example.com - week 12%",
                "# Gemini",
                "[ ] g@example.com",
                "--- Usage from Token Stats at 9 am",
            ]
        );
    }

    #[test]
    fn unpin_puts_a_thread_in_a_pool_with_its_login() {
        let pool = |id: &str, accounts: &[&str], folders: &[&str]| Pool { id: id.into(), title: id.into(), account_ids: accounts.iter().map(|a| a.to_string()).collect(), icon: None, folders: folders.iter().map(|f| f.to_string()).collect() };
        let pools = vec![pool("claude", &["claude-main", "claude-3"], &[]), pool("convex", &["claude-2"], &["~/dev/convex"]), pool("side", &["codex-main", "claude-3"], &[])];
        assert_eq!(unpin_target(&pools, "claude-3", "/Users/m/dev/me", "/Users/m", None).as_deref(), Some("claude"), "the first holding its login");
        assert_eq!(unpin_target(&pools, "claude-3", "/Users/m/dev/me", "/Users/m", Some("side")).as_deref(), Some("side"), "the last picked, when it holds the login");
        assert_eq!(unpin_target(&pools, "claude-3", "/Users/m/dev/me", "/Users/m", Some("convex")).as_deref(), Some("claude"), "not one without the login");
        assert_eq!(unpin_target(&pools, "claude-9", "/Users/m/dev/convex/app", "/Users/m", None).as_deref(), Some("convex"), "no pool holds it: where a new thread there starts");
        assert_eq!(unpin_target(&[], "claude-3", "/", "/Users/m", None), None);
    }
}
