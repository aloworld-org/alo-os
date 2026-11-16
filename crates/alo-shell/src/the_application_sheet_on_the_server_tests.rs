//! The sheet as the server holds it: what was wanted goes up named as alo's
//! surfaces name it, each road to an answer is kept once, and **only a press
//! answers *granted***.

use std::time::Instant;

use crate::the_application_sheet::{
    AnApplicationWanted, ApplicationSheetKey, HowLongForAnApplication, OnTheApplicationSheet,
    THE_LONGEST, TheSheetAnswered,
};
use crate::the_tasks_a_person_opened::WhichTask;

/// A server, bound in a private directory of its own.
#[cfg(target_os = "linux")]
fn a_server(named: &str) -> (tempfile::TempDir, crate::Server) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().unwrap();
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let server = crate::Server::bind(directory.path(), named).unwrap();
    (directory, server)
}

/// VSCodium, wanted by the first task, by the name the server knows it by.
fn codium(called: &str) -> AnApplicationWanted {
    AnApplicationWanted::by(WhichTask::numbered(0), "com.vscodium.codium", called)
}

/// **What was wanted goes up, and nothing is answered until somebody
/// answers**: wanting is not granting.
#[cfg(target_os = "linux")]
#[test]
fn what_was_wanted_goes_up_and_nothing_is_answered_yet() {
    let (_directory, mut server) = a_server("application-sheet-up");
    server.applications_were_wanted(
        WhichTask::numbered(0),
        &["com.vscodium.codium".to_owned()],
        Instant::now(),
    );
    let shows = server.application_sheet.shows().unwrap();
    // **Named as alo's own surfaces name it**: by its identifier, where the
    // server was told no name.
    assert_eq!(shows.wanted, &codium("com.vscodium.codium"));
    assert!(server.take_what_the_application_sheet_answered().is_empty());
}

/// **A key grants the length the keyboard is on, once**: the answer is kept
/// for the frame loop and taken, never carried out twice.
#[cfg(target_os = "linux")]
#[test]
fn a_key_grants_once_and_the_answer_is_taken_once() {
    let (_directory, mut server) = a_server("application-sheet-key");
    server.applications_were_wanted(
        WhichTask::numbered(0),
        &["com.vscodium.codium".to_owned()],
        Instant::now(),
    );
    server.the_application_sheet_key(ApplicationSheetKey::Last);
    server.the_application_sheet_key(ApplicationSheetKey::Grant);
    assert_eq!(
        server.take_what_the_application_sheet_answered(),
        [TheSheetAnswered::Granted {
            wanted: codium("com.vscodium.codium"),
            how_long: HowLongForAnApplication::Remember,
        }]
    );
    assert!(server.take_what_the_application_sheet_answered().is_empty());
    // **And a second Enter on no sheet grants nothing.**
    server.the_application_sheet_key(ApplicationSheetKey::Grant);
    assert!(server.take_what_the_application_sheet_answered().is_empty());
}

/// **Not now, by click, is kept as *not now***.
#[cfg(target_os = "linux")]
#[test]
fn not_now_by_a_click_is_kept_as_not_now() {
    let (_directory, mut server) = a_server("application-sheet-not-now");
    server.applications_were_wanted(
        WhichTask::numbered(0),
        &["com.vscodium.codium".to_owned()],
        Instant::now(),
    );
    server.the_application_sheet_pressed(OnTheApplicationSheet::NotNow);
    assert_eq!(
        server.take_what_the_application_sheet_answered(),
        [TheSheetAnswered::NotNow {
            wanted: codium("com.vscodium.codium")
        }]
    );
}

/// **A reader's press is held to its sheet**, and answers as a click does.
#[cfg(target_os = "linux")]
#[test]
fn a_readers_press_is_held_to_its_sheet() {
    let (_directory, mut server) = a_server("application-sheet-reader");
    server.applications_were_wanted(
        WhichTask::numbered(0),
        &["com.vscodium.codium".to_owned()],
        Instant::now(),
    );
    let number = server.application_sheet.shows().unwrap().number;
    assert!(
        !server.a_reader_pressed_on_the_application_sheet(
            number + 1,
            OnTheApplicationSheet::LetAloUseIt
        )
    );
    assert!(server.take_what_the_application_sheet_answered().is_empty());
    assert!(
        server
            .a_reader_pressed_on_the_application_sheet(number, OnTheApplicationSheet::LetAloUseIt)
    );
    assert_eq!(
        server.take_what_the_application_sheet_answered(),
        [TheSheetAnswered::Granted {
            wanted: codium("com.vscodium.codium"),
            how_long: HowLongForAnApplication::ThisTask,
        }]
    );
}

/// **Silence and a sheet nobody can see are *not now***, never a grant — the
/// two answers no person gave.
#[cfg(target_os = "linux")]
#[test]
fn silence_and_a_sheet_nobody_can_see_are_not_now() {
    let (_directory, mut server) = a_server("application-sheet-silence");
    let now = Instant::now();
    server.applications_were_wanted(
        WhichTask::numbered(0),
        &[
            "com.vscodium.codium".to_owned(),
            "org.mozilla.firefox".to_owned(),
        ],
        now,
    );
    server.the_application_sheet_waited(now + THE_LONGEST);
    server.the_application_sheet_could_not_be_drawn();
    let answered = server.take_what_the_application_sheet_answered();
    assert_eq!(answered.len(), 2);
    assert!(
        answered
            .iter()
            .all(|it| matches!(it, TheSheetAnswered::NotNow { .. })),
        "{answered:?}"
    );
}
