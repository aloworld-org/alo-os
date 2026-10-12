//! That F6 takes the keyboard to the Dock and back to the work, and that no
//! other chord moves it.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_shortcuts::{Chord, Key, Modifier, Modifiers, Shortcuts};

use crate::Server;
use crate::which_area_has_the_keyboard::AnArea;

/// A server on a socket of its own.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let bound = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, bound)
}

/// F6, as this release ships it.
fn f6() -> Chord {
    Chord::checked(Modifiers::none(), Key::F6).expect("a function key may stand alone")
}

/// Shift+F6.
fn shift_f6() -> Chord {
    Chord::checked(Modifiers::just(Modifier::Shift), Key::F6)
        .expect("a function key may stand alone with Shift")
}

/// **F6 takes the keyboard to the Dock, and again takes it back to the work.**
///
/// Two areas today: the canvas and the Dock. The panel, notifications and the
/// status area have nothing a person can act on yet and are passed over — so
/// this is also the test that says the skipping works on a real `Server`
/// rather than only against a fixture's answers.
#[test]
fn f6_takes_the_keyboard_to_the_dock_and_back_to_the_work() {
    let (_directory, mut server) = server("f6-to-the-dock");
    let shortcuts = Shortcuts::shipped();
    assert_eq!(
        server.which_area_has_the_keyboard, None,
        "the premise: the keyboard starts with the work"
    );

    assert!(
        server.a_chord_moves_the_keyboard_between_areas(&shortcuts, f6()),
        "F6 was not taken"
    );
    assert_eq!(
        server.which_area_has_the_keyboard,
        Some(AnArea::TheDock),
        "F6 did not reach the Dock"
    );

    assert!(server.a_chord_moves_the_keyboard_between_areas(&shortcuts, f6()));
    assert_eq!(
        server.which_area_has_the_keyboard, None,
        "a second F6 did not come back to the work, so the cycle has nowhere to leave to"
    );
}

/// **Arriving on the Dock lands on a slot.**
///
/// A person who pressed F6 asked to be somewhere, and a screen reader with
/// nothing focused has nothing to read out. Leaving forgets it, because coming
/// back is a new arrival and the icon that was there may not be.
#[test]
fn arriving_on_the_dock_lands_on_a_slot_and_leaving_forgets_it() {
    let (_directory, mut server) = server("f6-lands");
    let shortcuts = Shortcuts::shipped();

    server.a_chord_moves_the_keyboard_between_areas(&shortcuts, f6());
    assert_eq!(
        server.which_slot_on_the_bar,
        Some(0),
        "arriving on the Dock focused no slot, so a reader has nothing to say"
    );

    server.a_chord_moves_the_keyboard_between_areas(&shortcuts, f6());
    assert_eq!(
        server.which_slot_on_the_bar, None,
        "leaving the Dock kept a slot focused, so a ring would be drawn on a bar nobody is in"
    );
}

/// **Shift+F6 goes the other way**, which from the work is the last area with
/// anything in it rather than the first.
#[test]
fn shift_f6_goes_the_other_way() {
    let (_directory, mut server) = server("shift-f6");
    let shortcuts = Shortcuts::shipped();

    assert!(server.a_chord_moves_the_keyboard_between_areas(&shortcuts, shift_f6()));

    assert_eq!(
        server.which_area_has_the_keyboard,
        Some(AnArea::TheDock),
        "Shift+F6 from the work did not reach the last area with anything in it"
    );
}

/// **No other chord moves the keyboard**, and the answer says so.
///
/// This is a layer over the other chord roads rather than a gate in front of
/// them: a chord it does not take must go on untouched, or `⊞`+I would stop
/// opening Settings the day this file was added.
#[test]
fn another_chord_is_not_taken_and_moves_nothing() {
    let (_directory, mut server) = server("not-taken");
    let shortcuts = Shortcuts::shipped();
    let settings = Chord::checked(Modifiers::just(Modifier::Super), Key::I).expect("a chord");

    assert!(
        !server.a_chord_moves_the_keyboard_between_areas(&shortcuts, settings),
        "a chord that is not one of the two was taken"
    );
    assert_eq!(
        server.which_area_has_the_keyboard, None,
        "a chord that is not one of the two moved the keyboard"
    );
}

/// **A chord bound to nothing is not taken either.**
///
/// A person may unbind these, which `alo_shortcuts` allows for every action.
/// Then F6 is a key like any other and belongs to whatever has focus — rather
/// than being swallowed by a shell that remembered what it used to mean.
#[test]
fn an_unbound_chord_is_not_taken() {
    let (_directory, mut server) = server("unbound");
    let mut nothing_bound = Shortcuts::shipped();
    nothing_bound.unbind(alo_shortcuts::Action::TheNextArea);

    assert!(
        !server.a_chord_moves_the_keyboard_between_areas(&nothing_bound, f6()),
        "F6 was taken although this person has it bound to nothing"
    );
}
