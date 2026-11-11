//! **Every road into the Bar opens the Dock's own field** — a click on it,
//! `Super`+`Space` and `Super`+`A` — and none of them the centred panel.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_shortcuts::{Action, Shortcuts};

use crate::Server;
use crate::a_press_on_the_dock::WhatAPressOnTheDockDoes;
use crate::bar_raster::TheBarOpens;

fn a_session(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(runtime.path(), what).expect("a display");
    (runtime, server)
}

#[test]
fn with_the_keyboard_elsewhere_the_bar_shows_nothing() {
    let (_runtime, server) = a_session("bar-face-shut");
    assert_eq!(server.which_face_the_bar_shows(), None);
}

#[test]
fn a_click_on_the_docks_field_opens_the_field_and_not_the_panel() {
    let (_runtime, mut server) = a_session("bar-face-click");
    assert!(
        server.carry_out_what_the_bar_was_asked_for(Some(
            WhatAPressOnTheDockDoes::TypeIntoTheComposer
        ))
    );
    assert_eq!(
        server.which_face_the_bar_shows(),
        Some(TheBarOpens::AboveTheDocksField)
    );
}

/// **Both chords land in the same field** — the owner's *Super+Space: same as
/// clicking the field*. Read from the shipped list, so a changed default is
/// still the chord tested.
#[test]
fn super_space_and_super_a_land_in_the_docks_field() {
    let shortcuts = Shortcuts::shipped();
    for action in [Action::Launcher, Action::TheAgent] {
        let (_runtime, mut server) = a_session("bar-face-chord");
        let chord = shortcuts
            .chord_for(action)
            .expect("the release ships this chord");
        assert!(server.a_chord_opens_the_bar(&shortcuts, chord), "{action:?}");
        assert_eq!(
            server.which_face_the_bar_shows(),
            Some(TheBarOpens::AboveTheDocksField),
            "{action:?}"
        );
    }
}

/// **Escape closes it**, and with it the field's hold on the keyboard.
#[test]
fn escape_closes_the_field() {
    let (_runtime, mut server) = a_session("bar-face-escape");
    server.the_composer.it_took_the_keyboard();
    drop(server.a_key_reached_the_composer(crate::composer_keys::ComposerKey::Letter('f')));
    drop(server.a_key_reached_the_composer(crate::composer_keys::ComposerKey::Clear));
    assert_eq!(server.which_face_the_bar_shows(), None);
    assert!(server.composer().typed().is_empty());
}
