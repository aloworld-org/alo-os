//! A place as a window shows it is meant by the names typing takes: its own
//! name, or — with the keyboard, or unnamed — what its window's title holds.

use super::{APlaceForText, is_meant_by};

const FILE: &str = "hello.txt";
const WINDOW: &str = "hello.txt - VSCodium";

#[test]
fn a_place_as_shown_has_the_keyboard_it_was_shown_with() {
    assert!(APlaceForText::as_shown(FILE, WINDOW, true).has_the_keyboard());
    assert!(!APlaceForText::as_shown(FILE, WINDOW, false).has_the_keyboard());
}

/// **The window's title is a name typing takes**, for the place with the
/// keyboard: VSCodium's entry is named after the file, and a model that names
/// it by the window it read is typing into the same place.
#[test]
fn the_title_names_the_place_with_the_keyboard() {
    let place = APlaceForText::as_shown(FILE, WINDOW, true);
    for said in [FILE, WINDOW] {
        assert!(is_meant_by(&place, said), "{said}");
    }
    assert!(!is_meant_by(&place, "Document"));
}

/// **And only that one**: a named place without the keyboard is meant by its
/// own name alone.
#[test]
fn the_title_does_not_name_a_named_place_without_the_keyboard() {
    let place = APlaceForText::as_shown(FILE, WINDOW, false);
    assert!(is_meant_by(&place, FILE));
    assert!(!is_meant_by(&place, WINDOW));
}

/// **An unnamed place is meant by its window's title**, keyboard or not.
#[test]
fn the_title_names_an_unnamed_place() {
    assert!(is_meant_by(
        &APlaceForText::as_shown("", WINDOW, false),
        FILE
    ));
}
