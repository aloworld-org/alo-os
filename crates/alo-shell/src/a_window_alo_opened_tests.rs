//! Which new windows are alo's: an application alo started by either road,
//! remembered until its window comes or the time it may take has passed.
//! That such a window is placed in empty space with the view moving to it is
//! `tests/a_new_window_glides_to_empty_space`, on real client connections.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;
use std::time::{Duration, Instant};

use crate::Server;

/// A server on a socket of its own.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, server)
}

fn app(named: &str) -> alo_dock::AppId {
    alo_dock::AppId::named(named).expect("a named application")
}

/// The applications alo is waiting for, by name.
fn waited_for(server: &Server) -> Vec<String> {
    server
        .alo_is_waiting_for
        .iter()
        .map(|(it, _)| it.name().to_owned())
        .collect()
}

/// **An application alo started is waited for, and one the person started is
/// not** — the person's has its own road, which also brings it in front.
#[test]
fn an_application_alo_started_is_waited_for_and_the_persons_is_not() {
    let (_directory, mut server) = server("alo-started");
    server.its_agent_asked_to_start.push(app("org.kde.dolphin"));
    server.asked_to_start.push(app("org.mozilla.firefox"));
    let not_started = server.start_what_was_asked_for(&|_, _| true);
    assert!(not_started.is_empty(), "the control: both were sent");
    assert_eq!(waited_for(&server), ["org.kde.dolphin"]);
}

/// **One that could not be started is not waited for**: nothing is coming.
#[test]
fn an_application_that_did_not_start_is_not_waited_for() {
    let (_directory, mut server) = server("alo-not-started");
    server.its_agent_asked_to_start.push(app("org.kde.dolphin"));
    let not_started = server.start_what_was_asked_for(&|_, _| false);
    assert_eq!(not_started.len(), 1);
    assert!(waited_for(&server).is_empty());
}

/// **A file alo opened is waited for too**, once per application however
/// often it is asked.
#[test]
fn a_file_alo_opened_is_waited_for_once() {
    let (_directory, mut server) = server("alo-opened-a-file");
    let now = Instant::now();
    server.alo_opened_a_file_in("com.vscodium.codium", now);
    server.alo_opened_a_file_in("com.vscodium.codium", now);
    assert_eq!(waited_for(&server), ["com.vscodium.codium"]);
}

/// **The wait ends**: a file opened in a copy already running brings no new
/// window, so a later window of that application is not taken for alo's.
#[test]
fn the_wait_ends_after_the_time_a_window_may_take() {
    let (_directory, mut server) = server("alo-wait-ends");
    let then = Instant::now();
    server.alo_started(app("com.vscodium.codium"), then);
    server.alo_forgets_what_is_late(then + super::COMES_WITHIN);
    assert_eq!(
        waited_for(&server),
        ["com.vscodium.codium"],
        "forgotten before the time a window may take"
    );
    server.alo_forgets_what_is_late(then + super::COMES_WITHIN + Duration::from_secs(1));
    assert!(
        waited_for(&server).is_empty(),
        "the ask outlived the time its window may take"
    );
}
