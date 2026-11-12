//! A press while the alo Bar is open in the Dock's field, and the arrows that
//! walk the rows risen from it.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use crate::Server;
use crate::composer_keys::ComposerKey;
use crate::display_scale::DisplayScale;
use crate::where_the_bars_matches_rise::WhatItHolds;
use smithay::input::keyboard::XkbConfig;
use smithay::utils::{Point, Size};
use std::os::unix::fs::PermissionsExt;

/// A seat with a keyboard in the Dock's field with "files" typed, and the
/// matches drawn rising from a field on a bottom Dock of a 1440 × 960 display:
/// no application found, the row that asks offered.
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
    let places = crate::where_the_bars_matches_rise::where_the_matches_rise(
        Size::from((1440, 960)),
        DisplayScale::ONE,
        WhatItHolds {
            found: 0,
            asks: true,
            says: true,
            chosen: None,
        },
        crate::where_the_bars_matches_rise::TheFieldOnTheDock {
            field: smithay::utils::Rectangle::new((560, 886).into(), (344, 48).into()),
            band: smithay::utils::Rectangle::new((300, 872).into(), (840, 76).into()),
            edge: alo_dock::Edge::Bottom,
        },
        None,
    );
    assert!(places.is_some(), "the list rose");
    server.the_bars_matches_were_drawn(places);
    (runtime, server)
}

/// **With the Bar shut, a press is not its own** — the refusal.
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

/// **A press on the row that asks, in the list above the field, asks alo** —
/// the pointer and Enter do one thing to one row.
#[test]
fn a_press_on_the_risen_ask_row_asks_alo() {
    let (_runtime, mut server) = open("matches-ask");
    let ask = server
        .the_bars_matches_places
        .as_ref()
        .and_then(|places| places.ask)
        .expect("the row that asks");
    assert!(server.a_press_on_the_launcher(Point::from((ask.loc.x + 20, ask.loc.y + 20))));
    assert_eq!(
        server.take_what_was_asked_of_alo(),
        vec!["files".to_owned()]
    );
    assert!(!server.the_composer.has_the_keyboard());
}

/// **A press on the field is the field's**: not taken by the list, and the
/// keyboard and what is typed stay.
#[test]
fn a_press_on_the_docks_field_keeps_the_typing() {
    let (_runtime, mut server) = open("matches-field");
    assert!(!server.a_press_on_the_launcher(Point::from((600, 900))));
    assert!(server.the_composer.has_the_keyboard());
    assert_eq!(server.the_composer.typed(), "files");
}

/// **A press anywhere else closes the list and goes on to what it landed on**
/// — not taken, for nothing behind the list is dimmed — and the words stay
/// in the field for when the person comes back.
#[test]
fn a_press_outside_the_risen_list_closes_it_and_is_not_taken() {
    let (_runtime, mut server) = open("matches-outside");
    assert!(!server.a_press_on_the_launcher(Point::from((100, 100))));
    assert!(!server.the_composer.has_the_keyboard());
    assert!(server.the_bars_matches_places.is_none());
    assert_eq!(server.the_composer.typed(), "files");
}

/// **Between the rows, on the list itself, a press is taken and changes
/// nothing**, so it never reaches the window behind the list.
#[test]
fn a_press_on_the_risen_list_away_from_a_row_is_taken() {
    let (_runtime, mut server) = open("matches-panel");
    let panel = server
        .the_bars_matches_places
        .as_ref()
        .map(|places| places.panel)
        .expect("drawn");
    assert!(server.a_press_on_the_launcher(Point::from((panel.loc.x + 2, panel.loc.y + 2))));
    assert!(server.the_composer.has_the_keyboard());
}

/// **With the keyboard in the field and nothing typed there is no list**, and
/// a press elsewhere still gives the keyboard back.
#[test]
fn a_press_elsewhere_with_nothing_typed_gives_the_keyboard_back() {
    let (_runtime, mut server) = open("matches-empty");
    server.the_bars_matches_were_drawn(None);
    drop(server.a_key_reached_the_composer(ComposerKey::Clear));
    server.the_composer.it_took_the_keyboard();
    assert!(!server.a_press_on_the_launcher(Point::from((700, 400))));
    assert!(!server.the_composer.has_the_keyboard());
}
