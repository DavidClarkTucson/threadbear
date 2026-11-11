// Chips among a message's words (rich.picturechips, rich.cardchips,
// rich.threadchips): in Mike's bubble a "[Picture N]" is a chip for that
// picture, a Notion card's or another thread's link a chip with its title
// (RowRenderer.userText, PictureChips.swift, CardChips.swift). Where they go
// is threadbear_core::chips; this draws them, lays the bubble's lines out
// round them, and shows a chip's card while the pointer rests on it
// (TextKit's hoverChip and ChipCardWindow).
use crate::open_gate::{self, Origin};
use crate::style::{self, size};
use crate::thread::ThreadPane;
use gpui_kit::base::SelectableText;
use gpui_kit::component::{h_flex, v_flex, ActiveTheme as _};
use gpui_kit::prelude::*;
use gpui_kit::*;
// The zoom-aware px (style.rs), in place of GPUI's.
use crate::style::px;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use threadbear_core::chips::{self as found, Mention, MentionKind, Piece};
use threadbear_core::inbox::ThreadSummary;
use threadbear_core::reducer::{ImageRef, UserMessage};

/// A chip at the bubble's 15 pt (PictureChip.Metrics): 20 tall, a 14-point
/// picture 3 in from its edge, 5 to its label at 13 pt, 7 after it.
const CHIP_ICON: f32 = 14.;
const CHIP_INSET: f32 = 3.;
const CHIP_GAP: f32 = 5.;
const CHIP_TRAILING: f32 = 7.;
const CHIP_RADIUS: f32 = 6.;
const CHIP_LABEL: f32 = 13.;
/// The widest a card's title gets before it's cut short (CardChip.maxLabel).
const CARD_LABEL_MAX: f32 = 230.;

/// How long the pointer rests on a chip before its card shows: a thread's
/// and a picture's quickly, a page's a little longer, as links are
/// everywhere (TextKit's hoverChip and hoverPeek).
const QUICK: Duration = Duration::from_millis(350);
const SLOWER: Duration = Duration::from_millis(500);

// ---------------------------------------------------------------- hover cards

/// The chip the pointer is resting on, and since when: its card shows once
/// it's been there a moment, and goes when it leaves.
static HOVERED: Mutex<Option<(SharedString, Instant)>> = Mutex::new(None);

fn hover(key: &SharedString, on: bool, delay: Duration, window: &mut Window, cx: &mut App) {
    let mut hovered = HOVERED.lock().unwrap_or_else(|e| e.into_inner());
    if on {
        *hovered = Some((key.clone(), Instant::now()));
        cx.spawn(async move |cx| {
            cx.background_executor().timer(delay + Duration::from_millis(20)).await;
            cx.update(|cx| cx.refresh_windows());
        })
        .detach();
    } else if hovered.as_ref().is_some_and(|(k, _)| k == key) {
        *hovered = None;
        window.refresh();
    }
}

fn card_shown(key: &str, delay: Duration) -> bool {
    HOVERED.lock().ok().and_then(|h| h.as_ref().map(|(k, at)| k.as_ref() == key && at.elapsed() >= delay)).unwrap_or(false)
}

/// Shows `key`'s card now, as a script's `peek` does.
#[cfg(feature = "automation")]
pub(crate) fn show_card(key: Option<&str>) {
    let mut hovered = HOVERED.lock().unwrap_or_else(|e| e.into_inner());
    *hovered = key.map(|k| (SharedString::from(k.to_owned()), Instant::now() - Duration::from_secs(10)));
}

/// A card under a chip `height` tall (ChipCardWindow): its left edge on the
/// chip's, 4 below it, kept in the window.
fn under(height: f32, card: AnyElement) -> AnyElement {
    div()
        .absolute()
        .left_0()
        .top(px(height + 4.))
        .child(deferred(anchored().snap_to_window_with_margin(px(8.)).child(div().occlude().child(card))).with_priority(2))
        .into_any_element()
}

fn under_chip(card: AnyElement) -> AnyElement {
    under(size::CHIP_HEIGHT, card)
}

/// The card's look: a small rounded window's (ChipCardWindow), 8 round with
/// a hairline edge.
fn card_frame(cx: &App) -> Div {
    div().rounded(px(8.)).bg(cx.theme().popover).border(gpui_kit::px(0.5)).border_color(style::card_border(cx)).shadow_lg()
}

/// A thread's or a page's card (ThreadChipCard, LinkPeekCard): its title,
/// where it is, and that a click opens it.
pub(crate) fn words_card(title: &str, detail: &str, cx: &App) -> AnyElement {
    card_frame(cx)
        .w(px(300.))
        .px(px(12.))
        .py(px(10.))
        .child(
            v_flex()
                .gap(px(3.))
                .child(div().text_size(px(size::LABEL)).line_height(px(style::natural_line(size::LABEL))).font_weight(FontWeight::SEMIBOLD).text_color(style::label_text(cx)).line_clamp(2).child(title.to_owned()))
                .child(div().text_size(px(size::DETAIL)).line_height(px(style::natural_line(size::DETAIL))).text_color(style::secondary_label(cx)).truncate().child(detail.to_owned()))
                .child(div().pt(px(3.)).text_size(px(size::SMALL)).line_height(px(style::natural_line(size::SMALL))).text_color(style::tertiary_label(cx)).child("Click to open")),
        )
        .into_any_element()
}

/// A picture chip's card: the picture alone in a 276 by 170 box, as its
/// file name means nothing to Mike (LinkPeekCard, pictureOnly).
fn picture_card(image: &ImageRef, cx: &App) -> AnyElement {
    const BOX: (f32, f32) = (276., 170.);
    // Fitted to the box, a small picture made bigger too, as SwiftUI's
    // resizable image fits it; its own size where the runner gave it, else
    // the box's shape.
    let (w, h) = match (image.width, image.height) {
        (Some(w), Some(h)) if w > 0 && h > 0 => (w as f32, h as f32),
        _ => BOX,
    };
    let scale = (BOX.0 / w).min(BOX.1 / h);
    card_frame(cx)
        .p(px(6.))
        .child(
            div()
                .w(px(BOX.0))
                .h(px(BOX.1))
                .flex()
                .items_center()
                .justify_center()
                .child(img(PathBuf::from(&image.path)).w(px((w * scale).floor())).h(px((h * scale).floor())).rounded(px(3.)).object_fit(ObjectFit::Contain)),
        )
        .into_any_element()
}

/// The card for a thread, or None if it's gone (ThreadChipCard.make).
pub(crate) fn thread_card(thread_id: &str, threads: &HashMap<String, ThreadSummary>, accounts: &[threadbear_core::logins::Account], cx: &App) -> Option<AnyElement> {
    let thread = threads.get(thread_id)?;
    let title = if thread.title.is_empty() { "Untitled thread" } else { thread.title.as_str() };
    let account = (!thread.account_id.is_empty()).then(|| threadbear_core::logins::short_account_name(&thread.account_id, accounts));
    let status = threadbear_core::highlights::thread_status(thread, &threadbear_core::chrono::Local::now());
    let detail = found::thread_detail(&threadbear_core::new_thread::short_folder(&thread.cwd), account.as_deref(), &status);
    Some(words_card(title, &detail, cx))
}

// ---------------------------------------------------------------- Mike's bubble

/// What a chip in the bubble stands for.
#[derive(Clone)]
enum ChipKind {
    /// The message's picture at this index.
    Picture(usize),
    Card { title: String, url: String },
    Thread { title: String, id: String },
}

impl ChipKind {
    fn label(&self) -> String {
        match self {
            ChipKind::Picture(index) => format!("Picture {}", index + 1),
            ChipKind::Card { title, .. } | ChipKind::Thread { title, .. } => title.clone(),
        }
    }

    /// The bubble's symbol for it: the photo (the pictures are over the
    /// words there), a card, or a speech bubble for a thread.
    fn symbol(&self) -> &'static str {
        match self {
            ChipKind::Picture(_) => "photo",
            ChipKind::Card { .. } => "square.text.square",
            ChipKind::Thread { .. } => "bubble.left",
        }
    }

    /// For the dump: "picture: Picture 1", "card: Fix the button",
    /// "thread: Morning brief".
    fn described(&self) -> String {
        let kind = match self {
            ChipKind::Picture(_) => "picture",
            ChipKind::Card { .. } => "card",
            ChipKind::Thread { .. } => "thread",
        };
        format!("{kind}: {}", self.label())
    }
}

/// A piece of a bubble's line: words, or a chip.
enum Token {
    Words(String),
    Chip(ChipKind),
}

fn chip_kind(mention: &Mention) -> ChipKind {
    match (mention.kind, mention.thread_id()) {
        (MentionKind::Thread, Some(id)) => ChipKind::Thread { title: mention.title.clone(), id: id.to_owned() },
        _ => ChipKind::Card { title: mention.title.clone(), url: mention.url.clone() },
    }
}

/// The chips Mike's bubble draws among its words, for the dump (both apps'
/// `chips`).
pub(crate) fn described(message: &UserMessage) -> Option<Vec<String>> {
    if message.shows_as_note() || message.agent_header().is_some() {
        return None;
    }
    let chips: Vec<String> = found::message_pieces(&message.text, message.images.len())
        .iter()
        .filter_map(|p| match p {
            Piece::Picture(index) => Some(ChipKind::Picture(*index).described()),
            Piece::Mention(mention) => Some(chip_kind(mention).described()),
            Piece::Words(_) => None,
        })
        .collect();
    (!chips.is_empty()).then_some(chips)
}

fn text_width(text: &str, font_size: f32, window: &Window) -> Pixels {
    let style = window.text_style();
    let run = TextRun { len: text.len(), font: style.font(), color: style.color, background_color: None, underline: None, strikethrough: None };
    window.text_system().shape_line(text.to_owned().into(), px(font_size), &[run], None).width
}

/// A chip's width: its picture and gap, its label (a card's cut short), the
/// room after it, and its edges.
fn chip_width(kind: &ChipKind, window: &Window) -> Pixels {
    let mut label = text_width(&kind.label(), CHIP_LABEL, window).ceil();
    if !matches!(kind, ChipKind::Picture(_)) {
        label = label.min(px(CARD_LABEL_MAX));
    }
    px(CHIP_INSET + CHIP_ICON + CHIP_GAP + CHIP_TRAILING) + label
}

/// The bubble's lines, broken where the words would wrap at `max` with each
/// chip as one unbreakable piece, as TextKit lays the Swift bubble's
/// attachments out.
fn lay_out(pieces: Vec<Piece>, max: Pixels, window: &Window) -> Vec<Vec<Token>> {
    let mut lines: Vec<Vec<Token>> = vec![Vec::new()];
    // What the line's taken up to its last piece's end, its trailing spaces
    // counted (they hang past the edge when it wraps).
    let mut used = Pixels::ZERO;
    let add = |lines: &mut Vec<Vec<Token>>, used: &mut Pixels, token: Token, ink: Pixels, full: Pixels| {
        if lines.last().is_some_and(|l| !l.is_empty()) && *used + ink > max {
            lines.push(Vec::new());
            *used = Pixels::ZERO;
        }
        *used += full;
        lines.last_mut().expect("a line").push(token);
    };
    for piece in pieces {
        match piece {
            Piece::Words(text) => {
                for (n, paragraph) in text.split('\n').enumerate() {
                    if n > 0 {
                        lines.push(Vec::new());
                        used = Pixels::ZERO;
                    }
                    for word in paragraph.split_inclusive(' ') {
                        let full = text_width(word, size::BODY, window);
                        let ink = text_width(word.trim_end_matches(' '), size::BODY, window);
                        add(&mut lines, &mut used, Token::Words(word.to_owned()), ink, full);
                    }
                }
            }
            Piece::Picture(index) => {
                let kind = ChipKind::Picture(index);
                let width = chip_width(&kind, window);
                add(&mut lines, &mut used, Token::Chip(kind), width, width);
            }
            Piece::Mention(mention) => {
                let kind = chip_kind(&mention);
                let width = chip_width(&kind, window);
                add(&mut lines, &mut used, Token::Chip(kind), width, width);
            }
        }
    }
    lines
}

/// A line's width as it's drawn: each run of words between chips measured
/// whole, as one text draws it, the line's last without its trailing
/// spaces, and the chips.
fn drawn_width(tokens: &[Token], window: &Window) -> Pixels {
    let mut width = Pixels::ZERO;
    let mut run = String::new();
    let measure = |run: &mut String, last: bool| -> Pixels {
        let text = if last { run.trim_end_matches(' ') } else { run.as_str() };
        let width = if text.is_empty() { Pixels::ZERO } else { text_width(text, size::BODY, window) };
        run.clear();
        width
    };
    for token in tokens {
        match token {
            Token::Words(words) => run.push_str(words),
            Token::Chip(kind) => {
                width += measure(&mut run, false);
                width += chip_width(kind, window);
            }
        }
    }
    width + measure(&mut run, true)
}

impl ThreadPane {
    /// The bubble's words with its chips among them, laid out line by line,
    /// and how wide its widest line is; None for a message with no chips,
    /// which stays one piece of selectable text.
    pub(crate) fn chip_lines(&mut self, ix: usize, message: &UserMessage, max: Pixels, window: &Window, cx: &mut Context<Self>) -> Option<(Pixels, AnyElement)> {
        let pieces = found::message_pieces(&message.text, message.images.len());
        if pieces.iter().all(|p| matches!(p, Piece::Words(_))) {
            return None;
        }
        let lines = lay_out(pieces, max, window);
        let widest = lines.iter().map(|l| drawn_width(l, window).min(max)).fold(Pixels::ZERO, Pixels::max).ceil();
        let mut body = v_flex();
        for (n, tokens) in lines.into_iter().enumerate() {
            let mut line = h_flex().min_h(px(size::BODY_LINE)).items_center();
            // Words between chips as one run of selectable text each.
            let mut words = String::new();
            let (mut runs, mut chips) = (0, 0);
            let flush = |line: Div, words: &mut String, runs: &mut usize| -> Div {
                if words.is_empty() {
                    return line;
                }
                *runs += 1;
                let id = ("user-chip-text", ix * 100_000 + n * 100 + *runs);
                line.child(div().flex_none().child(SelectableText::new(id, std::mem::take(words))))
            };
            for token in tokens {
                match token {
                    Token::Words(text) => words.push_str(&text),
                    Token::Chip(kind) => {
                        line = flush(line, &mut words, &mut runs);
                        chips += 1;
                        line = line.child(self.bubble_chip(ix, n, chips, kind, &message.images, window.scale_factor() >= 2., cx));
                    }
                }
            }
            line = flush(line, &mut words, &mut runs);
            body = body.child(line);
        }
        Some((widest, body.into_any_element()))
    }

    /// One chip in the bubble (PictureChip.draw, CardChip.draw): a small
    /// rounded tag with its symbol and label, which opens what it stands
    /// for and shows its card after a moment.
    fn bubble_chip(&self, ix: usize, line: usize, at: usize, kind: ChipKind, images: &[ImageRef], retina: bool, cx: &mut Context<Self>) -> AnyElement {
        let (fill, secondary) = (style::chip_fill(cx), style::secondary_label(cx));
        // Its half-point edge (NSColor.separatorColor): a pixel on a Retina
        // screen; on a 1x one AppKit's comes out at half strength, so this
        // does too.
        let edge = if retina { style::hairline(cx) } else { style::hairline(cx).opacity(0.5) };
        let key = SharedString::from(format!("bubble-chip-{ix}-{line}-{at}-{}", kind.label()));
        let label = div().text_size(px(CHIP_LABEL)).line_height(px(16.)).text_color(secondary).whitespace_nowrap().when(!matches!(kind, ChipKind::Picture(_)), |d| d.max_w(px(CARD_LABEL_MAX)).truncate()).child(kind.label());
        // SF's symbols fill the 14-point box; Lucide's draw a little inside theirs.
        let symbol = div().size(px(CHIP_ICON)).flex().items_center().justify_center().child(crate::icons::symbol(kind.symbol()).size(px(13.)).text_color(secondary));
        let chip = div()
            .id(ElementId::Name(key.clone()))
            .relative()
            .flex_none()
            .h(px(size::CHIP_HEIGHT))
            .pl(px(CHIP_INSET))
            .pr(px(CHIP_TRAILING))
            .flex()
            .items_center()
            .gap(px(CHIP_GAP))
            .rounded(px(CHIP_RADIUS))
            .bg(fill)
            .border(gpui_kit::px(0.5))
            .border_color(edge)
            .cursor_pointer()
            .child(symbol)
            .child(label)
            // Not the start of a selection in the words round it.
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());
        let origin = self.origin();
        let name = kind.label();
        let chip = match kind {
            ChipKind::Picture(index) => {
                // Only this Mac's pictures: another Mac's path names a file there.
                let picture = images.get(index).filter(|_| origin == Origin::ThisMac);
                let shown = picture.filter(|_| card_shown(&key, QUICK)).map(|p| under_chip(picture_card(p, cx)));
                let images = images.to_vec();
                let hover_key = key.clone();
                chip.on_hover(move |on, window, cx| hover(&hover_key, *on, QUICK, window, cx))
                    // The in-app viewer, as a click on the picture over the words does.
                    .on_click(move |_, window, cx| crate::viewer::open_from_tile(&images, index, window, cx))
                    .children(shown)
            }
            ChipKind::Card { title, url } => {
                let shown = card_shown(&key, SLOWER).then(|| under_chip(words_card(&page_title(&title, &url), &threadbear_core::highlights::text::address(&url), cx)));
                let cwd = self.summary.cwd.clone();
                let hover_key = key.clone();
                chip.on_hover(move |on, window, cx| hover(&hover_key, *on, SLOWER, window, cx))
                    // Through the open gate, as the reply's links go.
                    .on_click(move |_, _, cx| open_gate::open_link(&url, origin, &cwd, cx))
                    .children(shown)
            }
            ChipKind::Thread { id, .. } => {
                let shown = card_shown(&key, QUICK).then(|| self.thread_card_for(&id, cx)).flatten().map(under_chip);
                let hover_key = key.clone();
                chip.on_hover(move |on, window, cx| hover(&hover_key, *on, QUICK, window, cx))
                    .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| this.open_thread_link(&id, window, cx)))
                    .children(shown)
            }
        };
        // Its label is what it's called to a screen reader, and to a script.
        chip.test_support().aria_label(name).into_any_element()
    }

    /// The thread's hover card, from the app's thread list.
    pub(crate) fn thread_card_for(&self, id: &str, cx: &App) -> Option<AnyElement> {
        let app = self.composer.read(cx).app.upgrade()?;
        let app = app.read(cx);
        thread_card(id, &app.threads, &app.logins.accounts, cx)
    }
}

/// A page's card title: the link's words, unless they're the address again
/// (LinkPeekCard.title).
fn page_title(words: &str, url: &str) -> String {
    let address = threadbear_core::highlights::text::address(url);
    if !words.is_empty() && !address.starts_with(words) && !url.starts_with(words) {
        return words.to_owned();
    }
    address
}

// ---------------------------------------------------------------- scripts

/// `peek`, as the Swift app's: `{"row": id}` lists a message's chips with a
/// card ("Picture 1=/path | Fix the button=https://…"); with `words` (a
/// chip's label, or part of it) shows its card as resting the pointer on it
/// would ("ok window 0"), and `show: false` closes it; `press: true` presses
/// a picture's chip through real pointer events and says what the viewer
/// opened, as the Swift command says what Quick Look would ("ok quicklook
/// <index> of [paths]").
#[cfg(feature = "automation")]
pub(crate) fn peek_command(app: &Entity<crate::app::Threadbear>, cmd: &serde_json::Value, window: &mut Window, cx: &mut App) -> Result<serde_json::Value, String> {
    use gpui_kit::test::{TestEventExt as _, TestQueryExt as _, TestWindowExt as _};
    use serde_json::json;
    use threadbear_core::reducer::RowContent;
    if cmd["show"].as_bool() == Some(false) {
        show_card(None);
        window.refresh();
        return Ok(json!("ok"));
    }
    let pane = app.read(cx).pane.clone().ok_or("no thread open")?;
    // With no `row`, the newest message with chips.
    let (ix, prefix, message) = {
        let rows = pane.read(cx).feed.model().rows();
        let ix = match cmd["row"].as_str() {
            Some(prefix) => rows.iter().position(|r| r.id.starts_with(prefix)).ok_or_else(|| format!("no row starting {prefix:?}"))?,
            None => rows.iter().rposition(|r| matches!(&r.content, RowContent::User(m) if described(m).is_some())).ok_or("no message with chips")?,
        };
        let RowContent::User(message) = &rows[ix].content else { return Err(format!("{} isn't a message", rows[ix].id)) };
        (ix, rows[ix].id.clone(), message.clone())
    };
    let chips: Vec<ChipKind> = found::message_pieces(&message.text, message.images.len())
        .iter()
        .filter_map(|p| match p {
            Piece::Picture(i) => Some(ChipKind::Picture(*i)),
            Piece::Mention(m) => Some(chip_kind(m)),
            Piece::Words(_) => None,
        })
        .collect();
    let Some(words) = cmd["words"].as_str() else {
        let lines: Vec<String> = chips
            .iter()
            .filter_map(|kind| match kind {
                ChipKind::Picture(i) => message.images.get(*i).map(|image| format!("{}={}", kind.label(), image.path)),
                ChipKind::Card { url, .. } => Some(format!("{}={url}", kind.label())),
                ChipKind::Thread { .. } => None,
            })
            .collect();
        return Ok(json!(if lines.is_empty() { "ok none".to_owned() } else { format!("ok {}", lines.join(" | ")) }));
    };
    let Some(kind) = chips.iter().find(|k| k.label().contains(words)).cloned() else { return Err(format!("no chip with {words} in {prefix}")) };
    // On screen first, as a person would see it.
    pane.update(cx, |p, cx| p.scroll_command(None, Some(&prefix), 120., cx))?;
    window.render_frame(cx);
    let mine = format!("bubble-chip-{ix}-");
    let element = window
        .find_all_by_label(&kind.label())
        .into_iter()
        .find(|e| e.visible() && e.path().iter().any(|id| matches!(id, ElementId::Name(n) if n.starts_with(&mine))))
        .ok_or_else(|| format!("the chip {words} isn't on screen"))?;
    if cmd["press"].as_bool() == Some(true) {
        let ChipKind::Picture(index) = kind else { return Err(format!("{words} isn't a picture's chip")) };
        let at = element.bounds().center();
        window.pointer_move(at, None, cx);
        window.pointer_down(at, MouseButton::Left, cx);
        window.pointer_up(at, MouseButton::Left, cx);
        let side = app.read(cx).side.clone();
        let opened = crate::viewer::viewer_command(&side, &json!({}), cx)?;
        if opened.as_str().is_some_and(|s| s.starts_with("ok closed")) {
            return Err("nothing opened".into());
        }
        let paths: Vec<&str> = message.images.iter().map(|i| i.path.as_str()).collect();
        return Ok(json!(format!("ok quicklook {index} of {}", serde_json::to_string(&paths).unwrap_or_default())));
    }
    // Its card, as resting the pointer on it shows it.
    let key = element.path().iter().rev().find_map(|id| match id {
        ElementId::Name(n) if n.starts_with(&mine) => Some(n.to_string()),
        _ => None,
    });
    show_card(key.as_deref());
    window.refresh();
    window.render_frame(cx);
    Ok(json!("ok window 0"))
}
