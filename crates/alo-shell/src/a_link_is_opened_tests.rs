//! That a link the person followed is recorded with its link, sent once with
//! it, and brings its window in front as a press does.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::cell::RefCell;
use std::os::unix::fs::PermissionsExt as _;

use alo_acting::{Answer, ToOpenALink};
use alo_applications::{Application, Link};

use crate::Server;
use crate::an_application_is_started::WhoAskedToStart;
use crate::the_application_door::CarriesOutAnApplicationVerb as _;

/// A server on a socket of its own.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, server)
}

fn firefox_with(address: &str) -> ToOpenALink {
    ToOpenALink::with(
        Application::identified("org.mozilla.firefox").expect("an application"),
        Link::checked(address).expect("a link"),
    )
}

/// **The ask waits, and is then sent once, with its link and as the
/// person's** — and its first window is the one the person is waiting for.
#[test]
fn a_link_is_sent_once_with_its_link_and_its_window_comes_in_front() {
    let (_directory, mut server) = server("a-link");
    let answer = server.opening_a_link(&firefox_with("https://example.com/invoice"));
    assert_eq!(answer, Answer::Acted);
    assert_eq!(server.how_many_are_waiting_to_start(), 1);
    let sent = RefCell::new(Vec::new());
    let not_started = server.start_what_was_asked_for(&|identifier, who| {
        sent.borrow_mut().push((identifier.to_owned(), who));
        true
    });
    assert!(not_started.is_empty());
    assert_eq!(
        sent.into_inner(),
        [(
            "org.mozilla.firefox".to_owned(),
            WhoAskedToStart::ALinkTheyFollowed(
                Link::checked("https://example.com/invoice").expect("a link")
            )
        )]
    );
    assert_eq!(server.how_many_are_waiting_to_start(), 0);
    assert!(
        server
            .the_person_is_waiting_for
            .iter()
            .any(|it| it.name() == "org.mozilla.firefox"),
        "the window a link opens would not come in front"
    );
}

/// **A link that could not be started is said, not retried**, and a machine
/// with no sender reports it as not started.
#[test]
fn a_link_that_could_not_be_started_is_reported_once() {
    let (_directory, mut server) = server("a-link-refused");
    server.opening_a_link(&firefox_with("mailto:anna@example.com"));
    let not_started = server.start_what_was_asked_for(&|_, _| false);
    assert_eq!(not_started.len(), 1);
    assert_eq!(server.how_many_are_waiting_to_start(), 0);
    assert!(server.the_person_is_waiting_for.is_empty());

    server.opening_a_link(&firefox_with("mailto:anna@example.com"));
    assert_eq!(server.start_what_was_asked_for_now().len(), 1);
    assert_eq!(server.how_many_are_waiting_to_start(), 0);
}
