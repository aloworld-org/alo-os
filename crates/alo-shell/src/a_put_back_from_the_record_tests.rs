//! **Put back from the record window, by keyboard**: Tab to the change, Enter
//! for the confirmation, Enter again to ask — and what the service answers.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt;

use crate::record_testing::{A_PLACE, AChangeKept, a_change_with_a_place_kept, words};
use crate::{OnTheSheet, RecordKey, RecordOpened, RecordShows, Server};

/// A display with the record window open on one change alo made after a place
/// was kept, the places told.
fn open(change: &AChangeKept) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().unwrap();
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut server = Server::bind(runtime.path(), "put-back").unwrap();
    server.the_record_is_kept(Ok(change.kept.recounting()));
    server.the_places_are_kept(change.whose.path().to_path_buf(), change.home());
    assert_eq!(server.the_record_is_opened(), Some(RecordOpened::Shown));
    (runtime, server)
}

/// One key, as the record window takes it.
fn pressed(server: &mut Server, key: RecordKey, change: &AChangeKept) {
    let opened = server.record.pressed(key, &change.kept.recounting());
    server.the_record_answered(opened);
}

/// What the confirmation is, while it is open.
fn the_sheet(server: &Server) -> Option<crate::TheSheet> {
    match server.record.shows() {
        RecordShows::Account { put_back, .. } => put_back.sheet.clone(),
        other => panic!("no account: {other:?}"),
    }
}

/// **Tab, Enter, Enter**: the keyboard reaches the change, the confirmation
/// opens with every folder, and confirming asks the service for exactly those
/// folders from exactly that place — once.
#[test]
fn tab_enter_enter_asks_the_service() {
    let change = a_change_with_a_place_kept();
    let (_runtime, mut server) = open(&change);

    pressed(&mut server, RecordKey::NextPutBack, &change);
    pressed(&mut server, RecordKey::Press, &change);
    let at = server
        .take_the_plan_asked()
        .expect("Enter asked for the plan");
    server.the_plan_is_made(at, &words());
    let sheet = the_sheet(&server).expect("the confirmation is open");
    assert_eq!(sheet.on, OnTheSheet::PutBack);
    assert!(sheet.can_put_back(), "{sheet:?}");

    pressed(&mut server, RecordKey::Press, &change);
    let (place, folders) = server.take_the_put_back_asked().expect("confirmed");
    assert_eq!(place, A_PLACE);
    assert_eq!(folders.len(), 2, "{folders:?}");
    assert!(the_sheet(&server).is_some_and(|sheet| sheet.waiting));

    // **One confirmation is one request**: pressing again while it is being
    // done asks nothing more.
    pressed(&mut server, RecordKey::Press, &change);
    assert_eq!(server.take_the_put_back_asked(), None);
}

/// **Nothing put back keeps the confirmation open, saying why**, with the
/// keyboard on *Cancel*; Escape closes it and not the window.
#[test]
fn nothing_put_back_says_why_and_escape_closes_only_the_confirmation() {
    let change = a_change_with_a_place_kept();
    let (_runtime, mut server) = open(&change);
    pressed(&mut server, RecordKey::NextPutBack, &change);
    pressed(&mut server, RecordKey::Press, &change);
    let at = server.take_the_plan_asked().unwrap();
    server.the_plan_is_made(at, &words());
    pressed(&mut server, RecordKey::Press, &change);
    let _ = server.take_the_put_back_asked().unwrap();

    server.that_put_back_answered(Err("nothing was changed".to_owned()));
    let sheet = the_sheet(&server).unwrap();
    assert_eq!(sheet.refused.as_deref(), Some("nothing was changed"));
    assert_eq!(sheet.on, OnTheSheet::Cancel);
    assert!(!sheet.can_put_back());

    pressed(&mut server, RecordKey::Close, &change);
    assert_eq!(the_sheet(&server), None);
    assert!(
        server.record.is_open(),
        "Escape closed the confirmation, not the window"
    );
}

/// **Cancel changes nothing and asks nothing.**
#[test]
fn cancel_asks_nothing() {
    let change = a_change_with_a_place_kept();
    let (_runtime, mut server) = open(&change);
    pressed(&mut server, RecordKey::NextPutBack, &change);
    pressed(&mut server, RecordKey::Press, &change);
    let at = server.take_the_plan_asked().unwrap();
    server.the_plan_is_made(at, &words());
    pressed(&mut server, RecordKey::NextPutBack, &change);
    assert_eq!(the_sheet(&server).unwrap().on, OnTheSheet::Cancel);
    pressed(&mut server, RecordKey::Press, &change);
    assert_eq!(the_sheet(&server), None);
    assert_eq!(server.take_the_put_back_asked(), None);
}

/// **Put back, and the record is read again**: the confirmation is gone.
#[test]
fn put_back_reads_the_record_again() {
    let change = a_change_with_a_place_kept();
    let (_runtime, mut server) = open(&change);
    pressed(&mut server, RecordKey::NextPutBack, &change);
    pressed(&mut server, RecordKey::Press, &change);
    let at = server.take_the_plan_asked().unwrap();
    server.the_plan_is_made(at, &words());
    pressed(&mut server, RecordKey::Press, &change);
    let _ = server.take_the_put_back_asked().unwrap();
    server.that_put_back_answered(Ok(vec!["invoices is as it was".to_owned()]));
    assert_eq!(the_sheet(&server), None);
    assert!(server.record.is_open());
}
