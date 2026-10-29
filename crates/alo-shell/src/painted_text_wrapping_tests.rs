//! That a wrap keeps to its room and its number of lines, says when it left
//! something out, and that an ellipsis line still fits.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use cosmic_text::Metrics;

use super::{ending_in_an_ellipsis, wrapped};
use crate::painted_text::how_wide;

/// The faces the shell draws with, and only those.
fn fonts() -> cosmic_text::FontSystem {
    crate::WindowControlLabels::new()
        .expect("the bundled faces")
        .fonts
}

/// The tooltip's own size.
fn metrics() -> Metrics {
    Metrics::new(11.0, 18.0)
}

/// **A name that fits is one line, and nothing was left out.**
#[test]
fn a_name_that_fits_is_one_line() {
    let (lines, cut) = wrapped("Files", 200, &mut fonts(), metrics(), 2);
    assert_eq!(lines, ["Files"]);
    assert!(!cut);
}

/// **Every line fits its room, there are never more than asked for, and a
/// wrap that stopped early says so.**
#[test]
fn every_line_fits_and_a_wrap_that_stopped_says_so() {
    let mut fonts = fonts();
    let long = "one two three four five six seven eight nine ten eleven twelve thirteen";
    let (lines, cut) = wrapped(long, 60, &mut fonts, metrics(), 2);
    assert_eq!(lines.len(), 2);
    assert!(cut, "a wrap that left words out did not say so");
    for line in &lines {
        assert!(
            how_wide(&mut fonts, line, metrics()) <= 60,
            "{line:?} overruns"
        );
    }
    let (all, cut) = wrapped(long, 60, &mut fonts, metrics(), 99);
    assert!(
        !cut,
        "a wrap with room for everything said it left something out"
    );
    assert_eq!(all.join(" "), long, "words were lost or reordered");
}

/// **A word longer than the room is broken rather than overrunning.**
#[test]
fn a_word_longer_than_the_room_is_broken() {
    let mut fonts = fonts();
    let (lines, _) = wrapped("Supercalifragilistic", 40, &mut fonts, metrics(), 9);
    assert!(lines.len() > 1, "the control: this word does not fit in 40");
    for line in &lines {
        assert!(
            how_wide(&mut fonts, line, metrics()) <= 40,
            "{line:?} overruns"
        );
    }
    assert_eq!(lines.concat(), "Supercalifragilistic");
}

/// **A line ending in an ellipsis still fits**, and ends in the mark.
#[test]
fn a_line_ending_in_an_ellipsis_still_fits() {
    let mut fonts = fonts();
    let line = "eleven twelve thirteen fourteen";
    let shortened = ending_in_an_ellipsis(line, 60, &mut fonts, metrics());
    assert!(shortened.ends_with('\u{2026}'), "{shortened:?}");
    assert!(
        how_wide(&mut fonts, &shortened, metrics()) <= 60,
        "{shortened:?} overruns"
    );
    assert!(
        !shortened.contains(" \u{2026}"),
        "a space before the mark: {shortened:?}"
    );
}
