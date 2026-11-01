//! A press while the launcher is open, and the arrows that walk its rows.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use crate::Server;
use crate::composer_keys::ComposerKey;
use crate::display_scale::DisplayScale;
use crate::where_the_bar_opens::{WhatItHolds, where_the_bar_opens};
use smithay::input::keyboard::XkbConfig;
use smithay::utils::{Physical, Point, Size};
use std::os::unix::fs::PermissionsExt;

/// A seat with a keyboard, the launcher open on "files", drawn on 1440 × 960
/// with no application found and the row that asks offered.
fn open(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        what,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.the_composer.it_took_the_keyboard();
    for letter in "files".chars() {
        drop(server.a_key_reached_the_composer(ComposerKey::Letter(letter)));
    }
    let places = where_the_bar_opens(
        Size::from((1440, 960)),
        DisplayScale::ONE,
        WhatItHolds {
            found: 0,
            asks: true,
            chosen: None,
        },
    );
    server.the_launcher_was_drawn(places);
    (runtime, server)
}

/// **Close closes it**, as Escape does: the field empties and lets go.
#[test]
fn a_press_on_close_closes_the_launcher() {
    let (_runtime, mut server) = open("launcher-close");
    let close = server.the_launchers_places.as_ref().expect("drawn").close;
    let middle = Point::<i32, Physical>::from((
        close.loc.x + close.size.w / 2,
        close.loc.y + close.size.h / 2,
    ));
    assert!(server.a_press_on_the_launcher(middle));
    assert!(!server.the_composer.has_the_keyboard());
    assert!(server.the_composer.typed().is_empty());
}

/// **A press on the dimmed screen closes it too, and reaches nothing behind.**
#[test]
fn a_press_on_the_dimmed_screen_closes_it_and_is_taken() {
    let (_runtime, mut server) = open("launcher-outside");
    assert!(server.a_press_on_the_launcher(Point::from((20, 900))));
    assert!(!server.the_composer.has_the_keyboard());
}

/// **A press on the panel itself, away from any row, changes nothing** — and
/// is still taken, so nothing behind the panel hears it.
#[test]
fn a_press_on_the_panel_is_taken_and_changes_nothing() {
    let (_runtime, mut server) = open("launcher-panel");
    let origin = server.the_launchers_places.as_ref().expect("drawn").origin;
    assert!(server.a_press_on_the_launcher(Point::from((origin.loc.x + 300, origin.loc.y + 5))));
    assert!(server.the_composer.has_the_keyboard());
    assert_eq!(server.the_composer.typed(), "files");
}

/// **With the launcher shut, a press is not its own** — the refusal.
#[test]
fn a_shut_launcher_takes_no_press() {
    let (_runtime, mut server) = open("launcher-shut");
    drop(server.a_key_reached_the_composer(ComposerKey::Clear));
    assert!(!server.a_press_on_the_launcher(Point::from((700, 400))));
}

/// **Down reaches the row that asks and stops there; Up goes back.** With no
/// application found, that row is the only one.
#[test]
fn the_arrows_walk_the_rows_and_stop_at_the_ends() {
    let (_runtime, mut server) = open("launcher-arrows");
    assert_eq!(server.how_many_rows_the_launcher_has(), 1);
    drop(server.a_key_reached_the_composer(ComposerKey::Down));
    assert_eq!(server.which_row_in_the_bar, Some(0));
    assert!(server.the_ask_row_is_chosen());
    drop(server.a_key_reached_the_composer(ComposerKey::Down));
    assert_eq!(server.which_row_in_the_bar, Some(0), "it stops at the last");
    drop(server.a_key_reached_the_composer(ComposerKey::Up));
    assert_eq!(server.which_row_in_the_bar, Some(0), "and at the first");
    // Typing again forgets the choice: the rows are new rows.
    drop(server.a_key_reached_the_composer(ComposerKey::Letter('x')));
    assert_eq!(server.which_row_in_the_bar, None);
}

/// **Enter on the row that asks asks alo** with what was typed, and the field
/// lets go.
#[test]
fn enter_on_the_ask_row_asks_alo() {
    let (_runtime, mut server) = open("launcher-ask");
    drop(server.a_key_reached_the_composer(ComposerKey::Down));
    let did = server.a_key_reached_the_composer(ComposerKey::Ask);
    assert_eq!(
        did,
        crate::the_composer_a_person_types_in::WhatTheComposerDid::TheyAsked("files".to_owned())
    );
    assert_eq!(
        server.take_what_was_asked_of_alo(),
        vec!["files".to_owned()]
    );
}
