//! **A sentence broken into lines that each fit a width, up to a number of
//! lines** — and, where it did not all fit, the last line ending in an
//! ellipsis.
//!
//! Lifted out of `crate::window_edge_tooltip` on 2026-10-29, when the Dock's
//! names became the second tooltip to need it. Two tooltips wrapping a name by
//! two rules would be two answers to one question.
//!
//! **Breaks between graphemes and prefers a space**: a line break inside a
//! combining sequence is a broken letter, and one inside a word is harder to
//! read than a short line. A word longer than the room on its own is broken
//! rather than allowed to overrun, because a single long token is the case
//! where there is no good answer and overrunning is the worse one.
//!
//! Measured with [`crate::painted_text::how_wide`], which is the face every
//! sentence on this machine is drawn in, so a wrap cannot be measured in one
//! font and drawn in another.

use cosmic_text::{FontSystem, Metrics};
use unicode_segmentation::UnicodeSegmentation;

use crate::painted_text::how_wide;

/// The mark that says there is more than was drawn.
///
/// A mark rather than a word: U+2026 means the same in every language this
/// machine ships in, and the whole name is always given to a screen reader.
const THERE_IS_MORE: char = '\u{2026}';

/// `whole`, broken into at most `at_most` lines that each fit `room` pixels
/// in `metrics`, and whether anything was left over.
pub(crate) fn wrapped(
    whole: &str,
    room: i32,
    fonts: &mut FontSystem,
    metrics: Metrics,
    at_most: usize,
) -> (Vec<String>, bool) {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    let mut words = whole.split_whitespace().peekable();
    while let Some(word) = words.next() {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if how_wide(fonts, &candidate, metrics) <= room {
            line = candidate;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
            if lines.len() >= at_most {
                return (lines, true);
            }
        }
        // The word alone, broken by graphemes if even that does not fit.
        let mut rest = word;
        while how_wide(fonts, rest, metrics) > room {
            let mut kept = String::new();
            let mut cut = 0;
            for (at, grapheme) in rest.grapheme_indices(true) {
                let next = format!("{kept}{grapheme}");
                if how_wide(fonts, &next, metrics) > room {
                    cut = at;
                    break;
                }
                kept = next;
                cut = at + grapheme.len();
            }
            if cut == 0 {
                // Not one grapheme fits. Nothing can be laid out in this room.
                return (lines, true);
            }
            lines.push(kept);
            rest = rest.get(cut..).unwrap_or_default();
            if lines.len() >= at_most {
                return (lines, !rest.is_empty() || words.peek().is_some());
            }
        }
        line = rest.to_owned();
    }
    if !line.is_empty() {
        if lines.len() < at_most {
            lines.push(line);
        } else {
            return (lines, true);
        }
    }
    (lines, false)
}

/// `line`, shortened until it and an ellipsis fit `room` pixels.
///
/// For the last line of a wrap that did not hold everything. Never longer than
/// `room` unless the ellipsis alone is, which is a room nothing fits in.
pub(crate) fn ending_in_an_ellipsis(
    line: &str,
    room: i32,
    fonts: &mut FontSystem,
    metrics: Metrics,
) -> String {
    let mut kept: Vec<&str> = line.graphemes(true).collect();
    loop {
        let candidate = format!("{}{THERE_IS_MORE}", kept.concat().trim_end());
        if kept.is_empty() || how_wide(fonts, &candidate, metrics) <= room {
            return candidate;
        }
        kept.pop();
    }
}

#[cfg(test)]
#[path = "painted_text_wrapping_tests.rs"]
mod tests;
