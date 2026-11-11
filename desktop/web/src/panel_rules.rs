// What a web panel's page may do, with nothing of Chromium's in it, so it's
// the same on every system and tested anywhere: which sites count as the
// panel's own (WebPanelPool.sameSite and leavesPanel in the Swift app), and
// what a page tells the app. mac/panel.rs acts on them.

/// What a panel's page tells the app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// A link, form or new window to another site, for the app to open.
    Leave(String),
    /// Loading started or stopped, going back became possible or not, or the
    /// title or address changed.
    Changed,
}

/// Whose page it is, which decides where its links may go.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A patch's url panel: its own site stays, other sites leave.
    Patch,
    /// The Convex embedded dashboard: only its own site stays, and the page
    /// answers its credentials request (docs/protocol.md, "The Convex panel")
    /// with this script, run in it before its own.
    Convex { credentials_script: String },
}

/// `https://a.b:8443/x` → `https://a.b:8443`: what counts as the same site
/// (scheme, host and port, with the default ports). None for anything that
/// isn't an http or https address.
pub fn site(url: &str) -> Option<String> {
    let (scheme, rest) = url.split_once("://")?;
    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let authority = authority.rsplit('@').next().unwrap_or(authority).to_ascii_lowercase();
    let default = if scheme == "https" { "443" } else { "80" };
    let (host, port) = if let Some(rest) = authority.strip_prefix('[') {
        // An IPv6 address, its colons inside the brackets.
        let (inside, after) = rest.split_once(']')?;
        (format!("[{inside}]"), after.strip_prefix(':').unwrap_or(""))
    } else {
        match authority.rsplit_once(':') {
            Some((host, port)) => (host.to_owned(), port),
            None => (authority.clone(), ""),
        }
    };
    let port = if port.is_empty() { default } else { port };
    (!host.is_empty()).then(|| format!("{scheme}://{host}:{port}"))
}

/// Whether a navigation to `url` from a page whose site is `home` leaves the
/// panel: a web page on another site (for the Convex dashboard, any but its
/// own). Not a web page (about:blank, data:): it stays, and Chromium's own
/// rules keep it in its frame.
pub fn leaves(url: &str, home: &str) -> bool {
    site(url).is_some_and(|target| target != home)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_counts_as_the_same_site() {
        assert_eq!(site("https://Dash.Example.com/a?b").as_deref(), Some("https://dash.example.com:443"));
        assert_eq!(site("http://localhost:3000/x").as_deref(), Some("http://localhost:3000"));
        assert_eq!(site("http://localhost/x").as_deref(), Some("http://localhost:80"));
        assert_eq!(site("https://user@example.com/").as_deref(), Some("https://example.com:443"));
        assert_eq!(site("http://[::1]:8080/").as_deref(), Some("http://[::1]:8080"));
        assert_eq!(site("about:blank"), None);
        assert_eq!(site("file:///tmp/a.html"), None);
        assert_eq!(site("data:text/html,hi"), None);
        let home = site("http://localhost:3000/").unwrap();
        assert!(!leaves("http://localhost:3000/settings", &home), "same site stays");
        assert!(leaves("https://youtube.com/watch", &home), "another site leaves");
        assert!(leaves("http://localhost:3001/", &home), "another port is another site");
        assert!(leaves("https://localhost:3000/", &home), "another scheme is another site");
        assert!(!leaves("about:blank", &home));
        // A data: page has no site of its own, so every web link leaves it.
        assert!(leaves("https://example.com/", ""));
    }
}
