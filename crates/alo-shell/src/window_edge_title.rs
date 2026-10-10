//! A window's title on its edge: what is shown, and what is still said.
//!
//! The contract, `docs/design/the-external-window-edge.md`: a shell-decorated
//! window's edge carries *a movement affordance, the window title, and alo's
//! window controls*. This is the title, and only the deciding of it — the
//! drawing is `crate::painted_text`'s, which every other sentence on this
//! machine already goes through.
//!
//! # Shown and whole are two different strings, deliberately
//!
//! The owner's direction of 2026-10-09:
//!
//! > Truncate long titles with an ellipsis before the controls; **preserve the
//! > full title for accessibility.**
//!
//! So [`FittedTitle`] carries both. A reader is told the whole title however little
//! of it is drawn, which is the difference between a window somebody can find
//! by name and one they cannot. Anything that read the drawn string and
//! announced it would quietly make long-titled windows unfindable.
//!
//! # The title is the application's own string
//!
//! It is never translated and never invented. `Server::the_name_of` answers
//! what a frame is called — the title the application set, then its class,
//! then the machine's own phrase for a window that named neither. **Only the
//! third is alo speaking**, and this file does not word it: a caller with the
//! application's own words passes them, and a caller without them says so.
//!
//! # What Unicode this gets right, and the one place it approximates
//!
//! Shaping is `cosmic-text`'s `Shaping::Advanced` through
//! `crate::painted_text`, the same path the sign-in screen uses — so font
//! fallback, combining marks and right-to-left runs are its answer and not
//! this file's.
//!
//! **Truncation cuts at grapheme boundaries**, so an accented letter is never
//! split from its accent and an emoji sequence is never split into the parts
//! it is built from. `unicode-segmentation` answers where those boundaries
//! are, and it was already in this tree — **`cosmic-text` depends on it**, so
//! the text stack this shell draws with had the answer and nothing new enters
//! the supply chain by asking for it directly.
//!
//! An earlier version of this file cut at character boundaries and said so as
//! a limitation. The owner's direction of 2026-10-09 was to fix it rather than
//! record it: *do not split accented letters, emoji sequences or other
//! combined characters.*

use cosmic_text::{FontSystem, Metrics};
use unicode_segmentation::UnicodeSegmentation;

/// The ellipsis a truncated title ends with.
///
/// One character rather than three dots: three periods are three glyphs a
/// reader announces one at a time, and the single character is what a screen
/// reader knows to call an ellipsis.
pub const AN_ELLIPSIS: &str = "…";

/// A title fitted to the room its edge has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FittedTitle {
    /// What is drawn — the whole title, or as much as fits with an ellipsis.
    pub shown: String,
    /// The whole title, always, for whoever says it aloud.
    pub whole: String,
    /// Whether anything was cut, so a caller can offer the rest some other
    /// way without measuring again.
    pub cut: bool,
}

/// Fit `title` into `room` logical units, ending with an ellipsis if it must.
///
/// `room` is what the drag region leaves after the grip — the contract's
/// *keep controls at the trailing edge and give the remaining usable space to
/// dragging and the title*.
///
/// Returns the whole title untouched when it fits, which is the ordinary case
/// and costs one measurement.
#[must_use]
pub fn fitted(fonts: &mut FontSystem, title: &str, room: i32, metrics: Metrics) -> FittedTitle {
    let whole = title.to_owned();
    if room <= 0 {
        return FittedTitle {
            shown: String::new(),
            whole,
            cut: !title.is_empty(),
        };
    }
    if crate::painted_text::how_wide(fonts, title, metrics) <= room {
        return FittedTitle {
            shown: whole.clone(),
            whole,
            cut: false,
        };
    }
    // Too wide. Take the longest prefix whose ellipsis still fits, by halving
    // rather than stepping: a title is shaped once per probe and a long one
    // stepped one cluster at a time would shape it hundreds of times.
    //
    // **Grapheme boundaries, not character ones.** A cut between a letter and
    // its combining accent, or inside an emoji built from several code
    // points, is a cut that renders as something the person never typed.
    let boundaries: Vec<usize> = title
        .grapheme_indices(true)
        .map(|(at, _)| at)
        .chain(std::iter::once(title.len()))
        .collect();
    let mut fits = 0;
    let mut beyond = boundaries.len();
    while beyond - fits > 1 {
        let probe = fits + (beyond - fits) / 2;
        let Some(end) = boundaries.get(probe) else {
            break;
        };
        let Some(prefix) = title.get(..*end) else {
            break;
        };
        let candidate = format!("{prefix}{AN_ELLIPSIS}");
        if crate::painted_text::how_wide(fonts, &candidate, metrics) <= room {
            fits = probe;
        } else {
            beyond = probe;
        }
    }
    let shown = boundaries
        .get(fits)
        .and_then(|end| title.get(..*end))
        .map(|prefix| format!("{prefix}{AN_ELLIPSIS}"))
        .unwrap_or_default();
    FittedTitle {
        shown,
        whole,
        cut: true,
    }
}

#[cfg(test)]
#[path = "window_edge_title_tests.rs"]
mod tests;
