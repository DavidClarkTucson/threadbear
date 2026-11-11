// ```mermaid and ```html blocks in a reply, drawn as what they show rather
// than as code: a port of the Swift app's RichBlock.swift, whose pages were
// security-reviewed there. Which fences count, each block's id (the same as
// the Swift app's, so the two apps share the pictures in
// ~/Library/Caches/Threadbear/rich), the colours, and the pages the web
// runtime is given: an agent's html behind a Content-Security-Policy that
// allows only inline styles and data: pictures, and the app's own diagram
// page that Pele (pele.run, bundled, MIT) draws into, the diagram's source
// handed over as a string, never as markup.
use sha2::{Digest as _, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RichKind {
    Mermaid,
    Html,
}

impl RichKind {
    pub fn name(self) -> &'static str {
        match self {
            RichKind::Mermaid => "mermaid",
            RichKind::Html => "html",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RichBlock {
    pub kind: RichKind,
    pub source: String,
    /// Stable for the same kind and source, so a picture, a switch to the
    /// source and a page for the window are found again after a restart.
    pub id: String,
}

impl RichBlock {
    /// Past this a block stays plain code: Pele refuses more than 50,000
    /// characters itself, and a page that size is a file, not a reply.
    pub const MAX_SOURCE: usize = 50_000;

    pub fn new(kind: RichKind, source: impl Into<String>) -> Self {
        let source = source.into();
        let digest = Sha256::digest(format!("{}\n{source}", kind.name()).as_bytes());
        let id = digest[..10].iter().map(|b| format!("{b:02x}")).collect();
        RichBlock { kind, source, id }
    }

    /// The block for a fence's language (```mermaid, ```HTML title=x: only
    /// the first word counts), or None for anything else, an empty one, or
    /// one too long.
    pub fn from(language: Option<&str>, code: &str) -> Option<RichBlock> {
        let word = language?.split([' ', '\t', '{']).find(|w| !w.is_empty())?.to_lowercase();
        let kind = match word.as_str() {
            "mermaid" => RichKind::Mermaid,
            "html" => RichKind::Html,
            _ => return None,
        };
        let source = code.trim_end_matches('\n');
        if source.trim().is_empty() || source.chars().count() > Self::MAX_SOURCE {
            return None;
        }
        Some(RichBlock::new(kind, source))
    }

    /// What the box's header calls it.
    pub fn label(&self) -> &'static str {
        self.kind.name()
    }

    /// The left half of the switch.
    pub fn drawn_title(&self) -> &'static str {
        match self.kind {
            RichKind::Mermaid => "Diagram",
            RichKind::Html => "Preview",
        }
    }

    /// Where its pictures are kept, at the zoom they were drawn for
    /// (RichBlocks.key in the Swift app).
    pub fn key(&self, zoom: u32) -> String {
        format!("{}-{zoom}", self.id)
    }

    /// The window's title (RichRenderer.showWindow).
    pub fn window_title(&self) -> &'static str {
        match self.kind {
            RichKind::Mermaid => "Diagram",
            RichKind::Html => "Page",
        }
    }
}

/// Whether text could hold a rich block, so only those replies are looked at.
pub fn may_hold_rich(text: &str) -> bool {
    if !text.contains("```") && !text.contains("~~~") {
        return false;
    }
    let lowered = text.to_lowercase();
    lowered.contains("mermaid") || lowered.contains("html")
}

/// Pele's colours for Threadbear, matching the transcript's code box so a
/// diagram sits in it as if drawn there. `bg` is only what labels are backed
/// with; the rest of the picture is see-through.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RichPalette {
    pub bg: &'static str,
    pub fg: &'static str,
    pub muted: &'static str,
    pub line: &'static str,
    pub surface: &'static str,
    pub surface_alt: &'static str,
    pub border: &'static str,
    pub accent: &'static str,
}

impl RichPalette {
    pub const DARK: RichPalette = RichPalette { bg: "#282828", fg: "#ececec", muted: "#9a9a9a", line: "#8e8e93", surface: "#333335", surface_alt: "#3a3a3c", border: "#5a5a5e", accent: "#0a84ff" };
    pub const LIGHT: RichPalette = RichPalette { bg: "#f7f7f7", fg: "#1d1d1f", muted: "#6e6e73", line: "#86868b", surface: "#ffffff", surface_alt: "#f0f0f2", border: "#c7c7cc", accent: "#007aff" };

    /// Pele's CSS variables (pele.run/theming), without the selector.
    fn variables(&self) -> String {
        format!(
            "--pele-bg:{};--pele-fg:{};--pele-muted:{};--pele-line:{};--pele-surface:{};--pele-surface-alt:{};--pele-border:{};--pele-accent:{};--pele-radius:6px;--pele-font:-apple-system,system-ui,sans-serif;--pele-font-mono:ui-monospace,Menlo,monospace",
            self.bg, self.fg, self.muted, self.line, self.surface, self.surface_alt, self.border, self.accent
        )
    }

    pub fn css(&self) -> String {
        format!(".pele{{{}}}", self.variables())
    }
}

/// Every sealed page gets this as its Content-Security-Policy: nothing from
/// anywhere, except inline styles and pictures written into the page itself.
/// The runtime's rules (desktop/web's sealed.rs) stop the network as well;
/// this is the page's own say on top.
pub const POLICY: &str = "default-src 'none'; style-src 'unsafe-inline'; img-src data:; font-src data:; form-action 'none'; base-uri 'none'; frame-src 'none'";

fn escape_attribute(text: &str) -> String {
    text.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;")
}

/// The agent's HTML with the policy in front of it. A `<meta>` policy only
/// counts in the head before anything it governs, so it goes first (after a
/// doctype, or the page lays out in quirks mode), ahead of the agent's own
/// head; browsers take what follows as the same document.
pub fn html_page(source: &str) -> String {
    format!("<!doctype html><meta http-equiv=\"Content-Security-Policy\" content=\"{}\"><meta name=\"color-scheme\" content=\"light dark\">{source}", escape_attribute(POLICY))
}

/// The page the diagram view loads once, with both palettes; the draw script
/// renders into it. Its own policy lets no script of the page's run: Pele and
/// the draw script run apart from the page, in a world of their own.
pub fn diagram_page() -> String {
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\">\n<meta http-equiv=\"Content-Security-Policy\" content=\"{}\">\n<style>\nhtml,body{{margin:0;padding:0;background:transparent}}\n#out{{display:inline-block;padding:2px}}\n#out.dark{{{}}}\n#out.light{{{}}}\n</style></head><body><div id=\"out\"></div></body></html>",
        escape_attribute(POLICY),
        RichPalette::DARK.variables(),
        RichPalette::LIGHT.variables()
    )
}

/// The biggest a diagram is drawn, in CSS pixels: a side, and an area.
pub const DIAGRAM_MAX_SIDE: u32 = 2400;
pub const DIAGRAM_MAX_AREA: u32 = 2400 * 1600;

/// Run in the diagram page's own world: renders `source` and gives back its
/// size, or `{error}` with Pele's message. A function body taking `source`,
/// `dark`, `fontSize`, `maxSide` and `maxArea` (the Swift app's drawScript).
pub const DRAW_SCRIPT: &str = r#"
const out = document.getElementById('out');
out.className = dark ? 'dark' : 'light';
try {
  const result = pele.render(source, { links: false, images: false, responsive: false, fontSize: fontSize,
    fontFamily: '-apple-system, system-ui, sans-serif', idPrefix: dark ? 'dk' : 'lt' });
  out.innerHTML = result.svg;
  // The box round the SVG and all it draws (a few shapes reach past its
  // viewBox), so the picture takes all of it.
  const svg = out.querySelector('svg');
  // A diagram can ask for any size (spacing in its config): drawn smaller
  // past a budget, so a hostile one can't make a picture of gigabytes.
  const w = parseFloat(svg.getAttribute('width')) || 1;
  const h = parseFloat(svg.getAttribute('height')) || 1;
  const scale = Math.min(1, maxSide / w, maxSide / h, Math.sqrt(maxArea / (w * h)));
  if (scale < 1) {
    svg.setAttribute('width', String(Math.floor(w * scale)));
    svg.setAttribute('height', String(Math.floor(h * scale)));
  }
  let right = 0, bottom = 0;
  for (const el of [out, svg, ...svg.querySelectorAll('*')]) {
    const r = el.getBoundingClientRect();
    if (r.width || r.height) { right = Math.max(right, r.right); bottom = Math.max(bottom, r.bottom); }
  }
  return { width: Math.min(Math.ceil(right) + 2, maxSide + 8), height: Math.min(Math.ceil(bottom) + 2, maxSide + 8), svg: out.innerHTML };
} catch (error) {
  out.textContent = '';
  return { error: String(error && error.message || error) };
}
"#;

/// The draw script called with its arguments, each written as JSON (a
/// string literal for the source, never markup), for DevTools to evaluate.
pub fn draw_call(source: &str, dark: bool, font_size: f64) -> String {
    let source = serde_json::to_string(source).unwrap_or_else(|_| "\"\"".into());
    format!("(function (source, dark, fontSize, maxSide, maxArea) {{{DRAW_SCRIPT}}})({source}, {dark}, {font_size}, {DIAGRAM_MAX_SIDE}, {DIAGRAM_MAX_AREA})")
}

/// A page for the full-size window: the diagram as it was drawn, standing
/// alone (no scripts needed to show it), centred, in both looks.
pub fn diagram_pane_page(dark_svg: &str, light_svg: &str, title: &str) -> String {
    format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{}</title>\n<meta http-equiv=\"Content-Security-Policy\" content=\"{}\">\n<meta name=\"color-scheme\" content=\"light dark\">\n<style>\nhtml,body{{margin:0;min-height:100%}}\nbody{{display:flex;justify-content:center;padding:24px;box-sizing:border-box;background:{}}}\n.look.dark{{display:none}}\n{}\n@media (prefers-color-scheme: dark){{\n  body{{background:{}}}\n  .look.light{{display:none}}.look.dark{{display:block}}\n}}\n.look.dark .pele{{{}}}\nsvg{{max-width:100%;height:auto}}\n</style></head><body><div class=\"look light\">{light_svg}</div><div class=\"look dark\">{dark_svg}</div></body></html>",
        escape_attribute(title),
        escape_attribute(POLICY),
        RichPalette::LIGHT.bg,
        RichPalette::LIGHT.css(),
        RichPalette::DARK.bg,
        RichPalette::DARK.variables()
    )
}

/// The window's copy of a diagram scales it to fill the window, bigger or
/// smaller (an SVG's viewBox keeps its shape), overriding Pele's max-width.
pub const FIT_TO_WINDOW: &str = "<style>html,body{height:100%}body{align-items:center}.look{width:100%;height:100%}.look svg{width:100%!important;height:100%!important;max-width:none!important;max-height:none!important}</style>";

/// The pane page fitted to its window: FIT_TO_WINDOW just before `</head>`.
pub fn fitted_to_window(page: &str) -> String {
    match page.find("</head>") {
        Some(at) => format!("{}{FIT_TO_WINDOW}{}", &page[..at], &page[at..]),
        None => page.to_owned(),
    }
}

/// How a block's picture sits in its box: a diagram at its own size or
/// smaller, centred; a page filling the width, cut past `max_body`. All in
/// points (RichBlockCell.pictureSize).
pub fn picture_size(kind: RichKind, natural: (f32, f32), room: f32, max_body: f32, pad: f32) -> (f32, f32) {
    let (width, height) = natural;
    if width <= 0. || height <= 0. {
        return (0., 0.);
    }
    match kind {
        RichKind::Html => {
            let scale = room / width;
            (room, (height * scale).round())
        }
        RichKind::Mermaid => {
            let room = room - pad * 2.;
            let scale = 1f32.min(room / width).min((max_body - pad * 2.) / height);
            ((width * scale).round(), (height * scale).round())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn which_fences_count() {
        let block = RichBlock::from(Some("mermaid"), "graph TD\n  A --> B\n\n").unwrap();
        assert_eq!(block.kind, RichKind::Mermaid);
        assert_eq!(block.source, "graph TD\n  A --> B");
        assert_eq!(RichBlock::from(Some("HTML"), "<p>hi</p>").unwrap().kind, RichKind::Html);
        assert_eq!(RichBlock::from(Some("html title=x"), "<p>hi</p>").unwrap().kind, RichKind::Html);
        assert_eq!(RichBlock::from(Some("mermaid{init}"), "graph TD").unwrap().kind, RichKind::Mermaid);
        assert!(RichBlock::from(Some("js"), "x").is_none());
        assert!(RichBlock::from(None, "x").is_none());
        assert!(RichBlock::from(Some("mermaid"), "  \n\n").is_none(), "an empty block stays code");
        assert!(RichBlock::from(Some("html"), &"x".repeat(RichBlock::MAX_SOURCE + 1)).is_none(), "too long to be a reply's");
        assert_eq!(RichBlock::from(Some("mermaid"), "graph TD").unwrap().drawn_title(), "Diagram");
        assert_eq!(RichBlock::from(Some("html"), "<b>x</b>").unwrap().drawn_title(), "Preview");
    }

    #[test]
    fn ids_are_the_swift_apps() {
        // The first ten bytes of SHA-256("mermaid\ngraph TD"), as
        // RichBlock.swift makes them, so both apps find the same pictures.
        let block = RichBlock::new(RichKind::Mermaid, "graph TD");
        let digest = Sha256::digest(b"mermaid\ngraph TD");
        assert_eq!(block.id, digest[..10].iter().map(|b| format!("{b:02x}")).collect::<String>());
        assert_eq!(block.id.len(), 20);
        assert_eq!(block.key(115), format!("{}-115", block.id));
        assert_ne!(block.id, RichBlock::new(RichKind::Html, "graph TD").id, "the kind is part of it");
    }

    #[test]
    fn only_replies_that_could_hold_one_are_looked_at() {
        assert!(may_hold_rich("see:\n```mermaid\ngraph TD\n```"));
        assert!(may_hold_rich("~~~HTML\n<p>\n~~~"));
        assert!(!may_hold_rich("plain words about mermaid"));
        assert!(!may_hold_rich("```rust\nfn main() {}\n```"));
    }

    #[test]
    fn an_agents_page_gets_the_policy_first() {
        let page = html_page("<html><head><script>alert(1)</script></head><body>hi</body></html>");
        assert!(page.starts_with("<!doctype html><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'; img-src data:;"));
        // The policy is ahead of anything the agent wrote.
        assert!(page.find("Content-Security-Policy").unwrap() < page.find("<script>").unwrap());
        assert!(POLICY.contains("default-src 'none'") && POLICY.contains("form-action 'none'") && POLICY.contains("frame-src 'none'") && POLICY.contains("base-uri 'none'"));
        assert!(!POLICY.contains("script-src"), "no script source is allowed, so default-src 'none' covers scripts");
        // Quotes in the policy can't close the attribute early.
        assert!(!escape_attribute("a\"b<c>").contains('"'));
    }

    #[test]
    fn the_source_goes_to_the_draw_script_as_a_string() {
        let hostile = "graph TD\n A[\"</script><script>alert(1)</script>\"] --> B\n`${x}`'\\";
        let call = draw_call(hostile, true, 14.);
        let literal = serde_json::to_string(hostile).unwrap();
        assert!(call.ends_with(&format!("({literal}, true, 14, 2400, 3840000)")), "{call}");
        // The diagram page itself has no script and lets none run.
        let page = diagram_page();
        assert!(!page.contains("<script"));
        assert!(page.contains("default-src 'none'"));
        assert!(page.contains("#out.dark{--pele-bg:#282828"));
    }

    #[test]
    fn the_window_page_fills_its_window() {
        let page = diagram_pane_page("<svg id=d></svg>", "<svg id=l></svg>", "Diagram");
        assert!(page.contains("<title>Diagram</title>"));
        assert!(page.contains("default-src 'none'"), "the window's page is sealed too");
        let fitted = fitted_to_window(&page);
        assert!(fitted.contains(&format!("{FIT_TO_WINDOW}</head>")));
    }

    #[test]
    fn pictures_fit_their_box() {
        // A diagram is never drawn bigger than it is, and fits the room.
        assert_eq!(picture_size(RichKind::Mermaid, (300., 200.), 700., 560., 12.), (300., 200.));
        assert_eq!(picture_size(RichKind::Mermaid, (1352., 400.), 700., 560., 12.), (676., 200.));
        // A page fills the width.
        assert_eq!(picture_size(RichKind::Html, (720., 1000.), 640., 560., 12.), (640., 889.));
    }
}
