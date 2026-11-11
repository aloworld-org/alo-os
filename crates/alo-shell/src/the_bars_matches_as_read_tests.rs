//! **A reader is told the matches as they are drawn**: the row that asks by
//! its own sentence, the one Enter acts on as selected, and nothing at all
//! while no list is shown.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use crate::Server;
use crate::composer_keys::ComposerKey;
use crate::display_scale::DisplayScale;
use crate::where_the_bar_opens::WhatItHolds;
use smithay::utils::{Rectangle, Size};

fn english() -> alo_strings::Strings {
    alo_strings::Strings::of(alo_saying::everything_this_machine_can_say().expect("the vocabulary"))
}

fn typing(what: &str, asks: bool) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind(runtime.path(), what).expect("a display");
    server.the_composer.it_took_the_keyboard();
    for letter in "files".chars() {
        drop(server.a_key_reached_the_composer(ComposerKey::Letter(letter)));
    }
    let places = crate::where_the_bars_matches_rise::where_the_matches_rise(
        Size::from((1440, 960)),
        DisplayScale::ONE,
        WhatItHolds {
            found: 0,
            asks,
            chosen: None,
        },
        crate::where_the_bars_matches_rise::TheFieldOnTheDock {
            field: Rectangle::new((26, 300).into(), (48, 344).into()),
            band: Rectangle::new((12, 100).into(), (76, 700).into()),
            edge: alo_dock::Edge::Left,
        },
        None,
    );
    server.the_bars_matches_were_drawn(places);
    (runtime, server)
}

#[test]
fn the_row_that_asks_is_read_in_its_own_words_and_is_selected() {
    let (_runtime, server) = typing("matches-read", true);
    let rows = server.the_bars_matches_as_read(&english());
    assert_eq!(rows, vec![("Ask alo \u{201c}files\u{201d}".to_owned(), true)]);
}

#[test]
fn with_ai_off_a_reader_is_told_of_no_row_that_asks() {
    let (_runtime, server) = typing("matches-read-off", false);
    assert!(server.the_bars_matches_as_read(&english()).is_empty());
}

#[test]
fn with_the_keyboard_gone_a_reader_is_told_of_no_list() {
    let (_runtime, mut server) = typing("matches-read-gone", true);
    server.the_composer.it_gave_the_keyboard_up();
    assert!(server.the_bars_matches_as_read(&english()).is_empty());
}
