//! Which starts a new window may answer by coming in front. The windows
//! themselves, on real client connections, are
//! `tests/a_window_the_person_opened`.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use crate::Server;
use crate::which_window_is_in_front::AKindOfWindow;

/// A server on a socket of its own.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, server)
}

/// An application by its identifier.
fn app(named: &str) -> alo_dock::AppId {
    alo_dock::AppId::named(named).expect("a named application")
}

/// **The person's start, sent, is waited for**; the agent's is not.
#[test]
fn only_the_persons_own_start_is_waited_for() {
    let (_directory, mut server) = server("only-the-persons");
    server.the_person_asked_to_start(app("org.kde.dolphin"));
    server
        .its_agent_asked_to_start
        .push(app("org.mozilla.firefox"));
    let not_started = server.start_what_was_asked_for(&|_, _| true);
    assert!(not_started.is_empty(), "the control: both were started");
    assert_eq!(
        server.the_person_is_waiting_for,
        [app("org.kde.dolphin")],
        "what the agent opened would come in front, or what the person opened would not"
    );
}

/// **A start that failed is not waited for**: nothing is coming.
#[test]
fn a_refused_start_is_not_waited_for() {
    let (_directory, mut server) = server("a-refused-start");
    server.the_person_asked_to_start(app("org.kde.dolphin"));
    let not_started = server.start_what_was_asked_for(&|_, _| false);
    assert_eq!(
        not_started,
        [app("org.kde.dolphin")],
        "the control: it was refused"
    );
    assert!(server.the_person_is_waiting_for.is_empty());
}

/// **Going somewhere else forgets what was started**, so a window that
/// comes later does not take the person away from where they went.
#[test]
fn going_elsewhere_forgets_what_was_started() {
    let (_directory, mut server) = server("going-elsewhere");
    server.the_person_asked_to_start(app("org.kde.dolphin"));
    let _ = server.start_what_was_asked_for(&|_, _| true);
    assert!(!server.the_person_is_waiting_for.is_empty(), "the control");
    server.the_person_went_to(AKindOfWindow::TheTerminal);
    assert!(
        server.the_person_is_waiting_for.is_empty(),
        "the person went to the terminal and a window started earlier would still take them away"
    );
}

/// **One application is waited for once**, and the list has a bound.
#[test]
fn an_application_is_waited_for_once_and_the_list_is_bounded() {
    let (_directory, mut server) = server("waited-once");
    server.the_person_started(app("org.kde.dolphin"));
    server.the_person_started(app("org.kde.dolphin"));
    assert_eq!(server.the_person_is_waiting_for.len(), 1);
    for n in 0..200 {
        server.the_person_started(app(&format!("org.example.App{n}")));
    }
    assert!(
        server.the_person_is_waiting_for.len() <= super::AT_MOST_REMEMBERED,
        "the list grows for ever"
    );
    assert_eq!(
        server.the_person_is_waiting_for.last(),
        Some(&app("org.example.App199")),
        "the newest start was the one dropped"
    );
}

/// **With nothing new on the screen, nothing moves.**
#[test]
fn nothing_new_moves_nothing() {
    let (_directory, mut server) = server("nothing-new");
    let before = server.which_is_in_front();
    server.what_the_person_opened_comes_in_front();
    assert_eq!(server.which_is_in_front(), before);
    assert!(!server.the_applications_are_in_front());
}
