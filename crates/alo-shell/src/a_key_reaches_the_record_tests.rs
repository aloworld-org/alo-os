//! **Keys reach the record window down the road the lane drives**, from an
//! evdev code on a seat with a real keymap, and Escape closes it — in the state
//! where the record was read, and in the state where it could not be found.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_recounting::Recounting;
use smithay::backend::input::KeyState;
use smithay::input::keyboard::XkbConfig;

use super::the_key_the_record_took;
use crate::{RecordShows, Server};

/// Escape.
const KEY_ESC: u32 = 1;
/// Down.
const KEY_DOWN: u32 = 108;
/// A.
const KEY_A: u32 = 30;

/// A session on this machine's seat, with a keyboard on a US layout.
fn a_session(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind_keyboard(
        runtime.path(),
        what,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    (runtime, server)
}

/// One press and its release, handed over as the lane hands them.
fn tapped(server: &mut Server, code: u32) {
    for state in [KeyState::Pressed, KeyState::Released] {
        the_key_the_record_took(
            server,
            crate::DirectKeyEvent {
                code,
                state,
                time: 1,
            },
        )
        .expect("a key on a seat with a keyboard");
    }
}

#[test]
fn escape_closes_the_record() {
    let (_runtime, mut server) = a_session("escape-closes-the-record");
    // **Bound, not a temporary**: the record's folder goes when this does.
    let kept = crate::record_testing::an_afternoon_kept();
    server.the_record_is_kept(Ok(kept.recounting()));
    server.the_record_is_opened();
    assert!(server.the_record_is_taking_the_keys());

    tapped(&mut server, KEY_ESC);
    assert!(
        !server.the_record_is_taking_the_keys(),
        "Escape left it open"
    );
}

/// **Down moves through the account**, which is the press reaching the window
/// and not only the seat.
#[test]
fn down_moves_to_an_older_entry() {
    let (_runtime, mut server) = a_session("down-moves-the-record");
    // **Bound, not a temporary**: the record's folder goes when this does.
    let kept = crate::record_testing::an_afternoon_kept();
    server.the_record_is_kept(Ok(kept.recounting()));
    server.the_record_is_opened();

    tapped(&mut server, KEY_DOWN);
    match server.the_record_window().shows() {
        RecordShows::Account { moved_past, .. } => assert_eq!(moved_past, 1),
        other => panic!("the account should be in the window, and it shows {other:?}"),
    }
}

/// **A letter does nothing**, and the window stays as it was.
#[test]
fn a_letter_does_nothing() {
    let (_runtime, mut server) = a_session("a-letter-at-the-record");
    // **Bound, not a temporary**: the record's folder goes when this does.
    let kept = crate::record_testing::an_afternoon_kept();
    server.the_record_is_kept(Ok(kept.recounting()));
    server.the_record_is_opened();

    tapped(&mut server, KEY_A);
    assert!(server.the_record_is_taking_the_keys());
    match server.the_record_window().shows() {
        RecordShows::Account { moved_past, .. } => assert_eq!(moved_past, 0),
        other => panic!("the account should be in the window, and it shows {other:?}"),
    }
}

/// **A window opened on a machine that cannot say where its record is still
/// closes on Escape**, and no other key reads a record it invented.
#[test]
fn escape_closes_a_window_whose_record_could_not_be_found() {
    let (_runtime, mut server) = a_session("escape-with-no-record");
    let nowhere = tempfile::tempdir().expect("a folder of this test's own");
    let why = Recounting::described_at(&nowhere.path().join("agentd.toml"))
        .expect_err("a description that is not there names no record");
    server.the_record_is_kept(Err(why.clone()));
    server.the_record_is_opened();

    tapped(&mut server, KEY_DOWN);
    assert_eq!(
        server.the_record_window().shows(),
        RecordShows::Refusal(&why)
    );
    tapped(&mut server, KEY_ESC);
    assert!(!server.the_record_is_taking_the_keys());
}
