// Another lab's review on the open card (RowRenderer.reviewText): a line
// saying who reviewed and how it came out, which opens the review's thread,
// then each finding with what the agent did about it (fixed, disputed and why,
// left for Mike, or no answer) and where. On the done card, and on a card of
// asks under them all (on the last page of a paged one). The ledger is
// `turn.completed`'s `review` (docs/protocol.md, "Review by another lab").
use crate::composer::Composer;
use crate::style::{self, size};
use gpui_kit::assets::IconName;
use gpui_kit::component::{h_flex, v_flex, Icon, Sizable as _};
use gpui_kit::prelude::*;
use gpui_kit::*;
// The zoom-aware px (style.rs), in place of GPUI's.
use crate::style::px;
use threadbear_core::reducer::{LabReview, ReviewFinding};
use crate::tips::Tip as _;

/// A finding's mark (SF's checkmark.circle, exclamationmark.bubble,
/// person.crop.circle and circle, as Lucide draws them), its word and its
/// colour, by what was done about it.
fn outcome(finding: &ReviewFinding, cx: &App) -> (IconName, &'static str, Hsla) {
    match finding.outcome.as_deref() {
        Some("fixed") => (IconName::CircleCheck, "Fixed", style::tick_green(cx)),
        Some("disputed") => (IconName::MessageSquareWarning, "Disputed", style::needs_you(cx)),
        Some("left") => (IconName::CircleUser, "Left for you", style::accent(cx)),
        _ => (IconName::Circle, "No answer", style::faint_text(cx)),
    }
}

/// The line under a finding: what was done, the agent's note, how serious,
/// and where.
pub(crate) fn finding_detail(finding: &ReviewFinding, word: &str) -> String {
    let mut detail = word.to_owned();
    if let Some(note) = finding.note.as_deref().filter(|n| !n.is_empty()) {
        detail.push_str(&format!(": {note}"));
    }
    if finding.severity == "high" {
        detail.push_str(" · serious");
    }
    if let Some(place) = finding.where_.as_deref().filter(|w| !w.is_empty()) {
        detail.push_str(&format!(" · {place}"));
    }
    detail
}

/// The words a finding's outcome reads as, for the dump.
pub(crate) fn outcome_word(finding: &ReviewFinding) -> &'static str {
    match finding.outcome.as_deref() {
        Some("fixed") => "Fixed",
        Some("disputed") => "Disputed",
        Some("left") => "Left for you",
        _ => "No answer",
    }
}

impl Composer {
    /// The ledger, `inset` in from the card's edge (the done card's rows are
    /// in by their hover fill's padding; the asks are at the edge).
    pub(crate) fn render_review(&self, review: &LabReview, inset: f32, cx: &mut Context<Self>) -> AnyElement {
        let secondary = style::secondary_label(cx);
        // The words start past the mark, as the done card's rows do.
        let words = inset + (size::BODY * 1.2).ceil() + 8.;
        let thread = review.thread_id.clone();
        let heading = div()
            .id("review-heading")
            .mt(px(10.))
            .pl(px(inset))
            .text_size(px(size::SMALL))
            .line_height(px(19.))
            .text_color(secondary)
            .child(review.heading())
            // A link to the review's thread, in the link colour, as the
            // Swift line's `.link` draws it.
            .when(!thread.is_empty(), |s| {
                s.cursor_pointer()
                    .text_color(style::accent(cx))
                    .hover(|s| s.opacity(0.8))
                    .tip("Open the review's thread")
                    .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                        if let Some(pane) = this.pane.upgrade() {
                            let url = format!("thread:{thread}");
                            pane.update(cx, |p, cx| p.link_clicked(&url, "", window, cx));
                        }
                    }))
            });
        let mut column = v_flex().w_full().child(heading);
        for finding in &review.findings {
            let (symbol, word, color) = outcome(finding, cx);
            column = column.child(
                h_flex()
                    .w_full()
                    .items_start()
                    .mt(px(6.))
                    .child(div().flex_none().w(px(words)).pl(px(inset)).h(px(size::BODY_LINE)).flex().items_center().child(Icon::from(symbol).with_size(px(15.)).text_color(color)))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(div().text_size(px(size::BODY)).line_height(px(size::BODY_LINE)).text_color(style::label_text(cx)).child(finding.claim.clone()))
                            .child(div().mt(px(2.)).text_size(px(size::SMALL)).line_height(px(17.)).text_color(secondary).child(finding_detail(finding, word))),
                    ),
            );
        }
        column.into_any_element()
    }
}

/// The ledger as lines, for the dump's `composer.card` (both apps): its
/// heading, then each finding as "Fixed: <claim>".
pub(crate) fn review_lines(review: &LabReview) -> Vec<String> {
    let mut lines = vec![review.heading()];
    lines.extend(review.findings.iter().map(|f| format!("  {}: {}", outcome_word(f), f.claim)));
    lines
}
