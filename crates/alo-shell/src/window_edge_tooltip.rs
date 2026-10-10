//! The whole title, for somebody who can see and is reading it now.
//!
//! # What this answers, and what it deliberately does not
//!
//! The owner's ruling of 2026-10-10, recorded in
//! `docs/design/the-external-window-edge.md`: *hovering the title or focusing
//! it with the keyboard reveals the full title in a wrapping tooltip,
//! positioned within the screen and clear of the window controls.*
//!
//! It is **one of three roads to the same string** and the convenient one. A
//! tooltip is transient by nature, so it cannot be read slowly, cannot be
//! copied, and on a touch screen there is no hover to produce it. The window
//! menu carries the other half — the full title as selectable text — and the
//! ellipsis carries the third, which is telling somebody there is more.
//!
//! # Clear of the controls by construction, not by arithmetic
//!
//! *The tooltip must not interfere with dragging or button interaction.* The
//! cheap way to satisfy that is to compute an overlap against every
//! [`EdgeControl::target`] and nudge. This does something stronger and simpler:
//! **it is never inside the edge's region at all.** It sits below
//! [`THE_REGION_IS_TALL`], so the drag region and all four control targets are
//! above it and no arithmetic can put it on one.
//!
//! That also means it overlays the application's own content, which is what a
//! tooltip is. The edge's *permanent* promise — `no application content
//! obscured` — is about the edge, and a transient panel a person asked for by
//! pointing at something is not the same claim.
//!
//! # Within the screen, which is the clamp and not a refusal
//!
//! [`EdgePicture::of`] answers `None` for an edge that would fall off its
//! output, because an edge drawn half off a screen is wrong. A tooltip is the
//! other case: it is **moved** to fit rather than withheld, because withholding
//! it would leave the person who asked for the title with nothing. So a title
//! on a window near the right edge gets a panel shifted left, and one on a
//! window near the bottom gets a panel that is still below the edge and as far
//! down as the output allows.
//!
//! # Only when something is hidden
//!
//! A title that fits is a title already on the screen, and a tooltip repeating
//! it would be a panel over somebody's work saying what they can already read.
//! [`tooltip_of`] answers `None` unless [`FittedTitle::cut`].

use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::window_edge::{THE_REGION_IS_TALL, WindowEdge};
use crate::window_edge_title::FittedTitle;

/// How wide the panel may be, before it wraps.
///
/// Wide enough for a sentence and narrow enough to read: a panel as wide as the
/// screen would put a long title on one line a person has to track across.
pub const AT_MOST_WIDE: i32 = 320;

/// The space between the words and the panel's edge, on every side.
pub const AROUND_THE_WORDS: i32 = 8;

/// One line of the title, in the edge's own size.
pub const A_LINE_IS_TALL: i32 = 18;

/// How many lines a title may wrap to.
///
/// **Four, and the rest is the menu's.** A title longer than four lines of 320
/// logical pixels is one somebody needs to read slowly or copy, which is what
/// the window menu's selectable title is for. A tooltip that grew without limit
/// would cover the work it is describing.
pub const AT_MOST_LINES: usize = 4;

/// The gap between the edge's region and the panel below it.
pub const BELOW_THE_EDGE: i32 = 4;

/// Why the whole title is being shown.
///
/// Kept as a reason rather than a boolean because the ruling names two roads to
/// the same panel and a reader of a frame should be able to tell which one a
/// person took.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Because {
    /// A pointer is on the title.
    APointerIsOnIt,
    /// The keyboard has focused the title.
    TheKeyboardIsOnIt,
}

/// The whole title, laid out below its window's edge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tooltip {
    /// The panel, including the space around the words.
    pub panel: Rectangle<i32, Logical>,
    /// Each line, and where its text begins.
    pub lines: Vec<(String, Point<i32, Logical>)>,
    /// Which road produced it.
    pub because: Because,
}

/// The whole title as a panel, or [`None`] when there is nothing hidden to show.
///
/// `fitted` is what the edge drew, and its [`FittedTitle::cut`] is the whole
/// condition: a title that fits is already readable.
///
/// `output` is the display in logical units, which is what the panel is clamped
/// to. `fonts` shapes the wrap in the face the edge draws with — see this file's
/// header for why a wrap measured in another font is a wrap that overruns.
#[must_use]
pub fn tooltip_of(
    edge: &WindowEdge,
    fitted: &FittedTitle,
    because: Because,
    fonts: &mut cosmic_text::FontSystem,
    output: (i32, i32),
) -> Option<Tooltip> {
    if !fitted.cut {
        return None;
    }
    let at = edge.title?;
    let room = AT_MOST_WIDE - 2 * AROUND_THE_WORDS;
    let lines = wrapped(&fitted.whole, room, fonts);
    if lines.is_empty() {
        return None;
    }

    let wide = lines
        .iter()
        .map(|line| crate::painted_text::how_wide(fonts, line, the_metrics()))
        .max()
        .unwrap_or(room)
        .min(room);
    let size = Size::from((
        wide + 2 * AROUND_THE_WORDS,
        i32::try_from(lines.len()).unwrap_or(1) * A_LINE_IS_TALL + 2 * AROUND_THE_WORDS,
    ));

    // **Below the region, always.** The drag area and every control target are
    // inside it, so nothing here can land on one.
    let under_the_edge = edge.region.loc.y + THE_REGION_IS_TALL + BELOW_THE_EDGE;
    let put = Point::from((at.x - AROUND_THE_WORDS, under_the_edge));
    let panel = within(Rectangle::new(put, size), output, under_the_edge);

    let lines = lines
        .into_iter()
        .enumerate()
        .map(|(which, line)| {
            let down = i32::try_from(which).unwrap_or(0) * A_LINE_IS_TALL;
            (
                line,
                Point::from((
                    panel.loc.x + AROUND_THE_WORDS,
                    panel.loc.y + AROUND_THE_WORDS + down,
                )),
            )
        })
        .collect();
    Some(Tooltip {
        panel,
        lines,
        because,
    })
}

/// The metrics the edge's title is drawn with.
fn the_metrics() -> cosmic_text::Metrics {
    cosmic_text::Metrics::new(12.0, 16.0)
}

/// Move `panel` so it is inside `output`, keeping it below `floor`.
///
/// **Moved and never shrunk.** A narrower panel would re-wrap, and a panel that
/// re-wrapped as a person moved their window would be one whose shape depends
/// on where it is rather than on what it says.
fn within(
    panel: Rectangle<i32, Logical>,
    output: (i32, i32),
    floor: i32,
) -> Rectangle<i32, Logical> {
    let x = panel.loc.x.min(output.0 - panel.size.w).max(0);
    // Below the edge first, inside the output second: a panel pushed up to fit
    // a short output would land on the controls, which is the one place it may
    // not be.
    let y = panel.loc.y.min(output.1 - panel.size.h).max(floor);
    Rectangle::new(Point::from((x, y)), panel.size)
}

/// `whole`, broken into lines that each fit `room`.
///
/// **Breaks between graphemes and prefers a space**, for the reason
/// `window_edge_title` cuts on graphemes: a line break inside a combining
/// sequence is a broken letter, and one inside a word is harder to read than a
/// short line. A word longer than `room` on its own is broken rather than
/// allowed to overrun — a single long token is the case where there is no good
/// answer and overrunning is the worse one.
fn wrapped(whole: &str, room: i32, fonts: &mut cosmic_text::FontSystem) -> Vec<String> {
    use unicode_segmentation::UnicodeSegmentation;

    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in whole.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if crate::painted_text::how_wide(fonts, &candidate, the_metrics()) <= room {
            line = candidate;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
            if lines.len() >= AT_MOST_LINES {
                return lines;
            }
        }
        // The word alone, broken by graphemes if even that does not fit.
        let mut rest = word;
        while crate::painted_text::how_wide(fonts, rest, the_metrics()) > room {
            let mut kept = String::new();
            let mut cut = 0;
            for (at, grapheme) in rest.grapheme_indices(true) {
                let next = format!("{kept}{grapheme}");
                if crate::painted_text::how_wide(fonts, &next, the_metrics()) > room {
                    cut = at;
                    break;
                }
                kept = next;
                cut = at + grapheme.len();
            }
            if cut == 0 {
                // Not one grapheme fits. Nothing can be laid out in this room.
                return lines;
            }
            lines.push(kept);
            if lines.len() >= AT_MOST_LINES {
                return lines;
            }
            rest = &rest[cut..];
        }
        line = rest.to_owned();
    }
    if !line.is_empty() && lines.len() < AT_MOST_LINES {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
#[path = "window_edge_tooltip_tests.rs"]
mod tests;
