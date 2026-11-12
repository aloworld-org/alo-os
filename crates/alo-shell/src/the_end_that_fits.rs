//! **The end of a line of typing that fits in a field**, so what a person just
//! typed, and the caret after it, are never the part cut off.
//!
//! Every field a person types into on this machine scrolls this way: once the
//! words are wider than the field, the start goes out of view and the end stays.
//! Cutting at a grapheme, never inside one, so an accented letter or an emoji
//! built of several code points is shown whole or not at all.

use unicode_segmentation::UnicodeSegmentation;

/// The longest end of `typed` that `how_wide` says fits in `room`.
///
/// `how_wide` is asked a handful of times — the cut is found by halving, not
/// by trying every grapheme — because a field holds up to
/// `crate::the_composer_a_person_types_in::THE_MOST_THAT_CAN_BE_TYPED` of them
/// and this runs on every frame.
pub(crate) fn the_end_that_fits(
    typed: &str,
    room: i32,
    mut how_wide: impl FnMut(&str) -> i32,
) -> &str {
    if room <= 0 {
        return "";
    }
    if how_wide(typed) <= room {
        return typed;
    }
    let starts: Vec<usize> = typed.grapheme_indices(true).map(|(at, _)| at).collect();
    // The first start whose end fits; the last start (one grapheme) is tried
    // too, and if even that does not fit nothing is shown.
    let (mut low, mut high) = (0, starts.len());
    while low < high {
        let middle = low + (high - low) / 2;
        let from = starts.get(middle).copied().unwrap_or(typed.len());
        if how_wide(typed.get(from..).unwrap_or("")) <= room {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    let from = starts.get(low).copied().unwrap_or(typed.len());
    typed.get(from..).unwrap_or("")
}

#[cfg(test)]
#[path = "the_end_that_fits_tests.rs"]
mod tests;
