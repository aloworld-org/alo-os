//! The end of what is typed stays in view, cut only between graphemes.

use super::*;

/// Ten pixels a grapheme, whatever it is.
fn ten_each(text: &str) -> i32 {
    i32::try_from(text.graphemes(true).count() * 10).unwrap_or(i32::MAX)
}

#[test]
fn what_fits_is_shown_whole() {
    assert_eq!(the_end_that_fits("files", 100, ten_each), "files");
}

#[test]
fn what_does_not_fit_loses_its_start_and_keeps_its_end() {
    assert_eq!(the_end_that_fits("open the spreadsheet", 50, ten_each), "sheet");
}

#[test]
fn a_letter_and_its_accent_are_never_parted() {
    // "e" and a combining acute are one grapheme.
    let typed = "cafe\u{301} au lait";
    assert_eq!(the_end_that_fits(typed, 90, ten_each), "e\u{301} au lait");
    assert_eq!(the_end_that_fits(typed, 80, ten_each), " au lait");
    assert_eq!(the_end_that_fits(typed, 120, ten_each), typed);
}

#[test]
fn no_room_shows_nothing() {
    assert_eq!(the_end_that_fits("files", 0, ten_each), "");
    assert_eq!(the_end_that_fits("files", 5, ten_each), "");
}
