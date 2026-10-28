//! **Super+R opens the record, and every way it cannot is said in the window or
//! on the error stream — never as an empty list.**
//!
//! An empty record window would read as *alo did nothing*, which is the one
//! sentence a refusal must never be mistaken for. So each refusal here asserts
//! what the window shows, not only that it opened.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_recounting::Recounting;
use alo_shortcuts::{Action, Chord, Shortcuts};

use crate::{RecordOpened, RecordShows, Server};

/// A session on this machine's seat, with a private runtime directory.
fn a_session(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(runtime.path(), what).expect("a display");
    (runtime, server)
}

/// The chord the release ships for the record, read out of the shipped list
/// rather than typed here, so this cannot drift from the binding.
fn the_records_chord(shortcuts: &Shortcuts) -> Chord {
    shortcuts
        .chord_for(Action::SeeWhatAloDid)
        .expect("the release ships a chord for the record")
}

#[test]
fn the_chord_opens_the_record_a_machine_keeps() {
    let (_runtime, mut server) = a_session("the-record-chord");
    let shortcuts = Shortcuts::shipped();
    let kept = crate::record_testing::an_afternoon_kept();
    server.the_record_is_kept(Ok(kept.recounting()));

    assert!(
        !server.the_record_is_taking_the_keys(),
        "nothing is open yet"
    );
    assert!(server.a_chord_opens_the_record(&shortcuts, the_records_chord(&shortcuts)));
    assert!(
        matches!(
            server.the_record_window().shows(),
            RecordShows::Account { .. }
        ),
        "the chord was taken and the record's account is not in the window"
    );
    assert!(server.the_record_is_taking_the_keys());
}

#[test]
fn another_chord_is_not_the_records() {
    let (_runtime, mut server) = a_session("not-the-record");
    let shortcuts = Shortcuts::shipped();
    server.the_record_is_kept(Ok(crate::record_testing::an_afternoon_kept().recounting()));
    let settings = shortcuts
        .chord_for(Action::Settings)
        .expect("the release ships a chord for Settings");

    assert!(!server.a_chord_opens_the_record(&shortcuts, settings));
    assert!(!server.the_record_is_taking_the_keys());
}

/// **A session never told where the record is opens nothing, and still takes
/// the chord**, so it never falls through to an application.
#[test]
fn a_session_never_told_where_the_record_is_opens_nothing() {
    let (_runtime, mut server) = a_session("never-told");
    let shortcuts = Shortcuts::shipped();

    assert!(server.a_chord_opens_the_record(&shortcuts, the_records_chord(&shortcuts)));
    assert!(!server.the_record_is_taking_the_keys());
    assert_eq!(server.the_record_is_opened(), None);
}

/// **No record file yet is a refusal in the window**, never an empty account.
#[test]
fn a_record_that_is_not_there_is_said_in_the_window() {
    let (_runtime, mut server) = a_session("no-record-yet");
    let nowhere = tempfile::tempdir().expect("a folder of this test's own");
    server.the_record_is_kept(Ok(Recounting::kept_at(
        &nowhere.path().join("record.jsonl"),
    )));

    assert_eq!(
        server.the_record_is_opened(),
        Some(RecordOpened::RefusedInTheWindow)
    );
    match server.the_record_window().shows() {
        RecordShows::Refusal(why) => assert!(why.there_is_no_record(), "{why:?}"),
        other => panic!("the window should say there is no record, and shows {other:?}"),
    }
}

/// **A machine that cannot say where its record is opens the window on that
/// sentence**, rather than leaving a person's key unanswered.
#[test]
fn a_machine_that_cannot_say_where_its_record_is_says_so_in_the_window() {
    let (_runtime, mut server) = a_session("no-description");
    let nowhere = tempfile::tempdir().expect("a folder of this test's own");
    let why = Recounting::described_at(&nowhere.path().join("agentd.toml"))
        .expect_err("a description that is not there names no record");
    server.the_record_is_kept(Err(why.clone()));

    assert_eq!(
        server.the_record_is_opened(),
        Some(RecordOpened::RefusedInTheWindow)
    );
    assert_eq!(
        server.the_record_window().shows(),
        RecordShows::Refusal(&why)
    );
    assert!(server.the_record_is_taking_the_keys());
}
