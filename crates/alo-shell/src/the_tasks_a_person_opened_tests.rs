//! That a request is recorded once, that an empty one is not a request, and
//! that windows keep their order.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_models::InferenceSource;
use alo_tasks::ATask;
use alo_tasks::conversation::Turn;

use crate::Server;

/// A server on a socket of its own.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let bound = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, bound)
}

/// **A request is recorded and taken once.**
///
/// One Enter opens one window however many frames pass before the binary asks.
/// A list read rather than taken would open a window per frame for ever after
/// one keypress.
#[test]
fn a_request_is_recorded_and_taken_once() {
    let (_directory, mut server) = server("asked-once");

    assert!(server.a_person_asked_alo("Tidy up my invoices folder"));

    assert_eq!(
        server.take_what_was_asked_of_alo(),
        vec!["Tidy up my invoices folder".to_owned()]
    );
    assert!(
        server.take_what_was_asked_of_alo().is_empty(),
        "the same request was taken twice, so one Enter would open two windows"
    );
}

/// **An empty composer asks nothing.**
///
/// Somebody who presses Enter on a blank field has not asked for anything, and
/// a window with no request in it is one they have to close. Spaces are the
/// same: a person who typed nothing but a space typed nothing.
#[test]
fn an_empty_composer_asks_nothing() {
    let (_directory, mut server) = server("asked-nothing");

    assert!(!server.a_person_asked_alo(""));
    assert!(!server.a_person_asked_alo("   "));

    assert!(
        server.take_what_was_asked_of_alo().is_empty(),
        "an empty composer opened a window"
    );
}

/// **What is recorded is what they typed, trimmed.**
#[test]
fn what_is_recorded_is_what_they_typed() {
    let (_directory, mut server) = server("asked-trimmed");

    server.a_person_asked_alo("  Install Blender  ");

    assert_eq!(
        server.take_what_was_asked_of_alo(),
        vec!["Install Blender".to_owned()]
    );
}

/// **Windows keep the order they were opened in.**
///
/// A list that moved the busy one to the top would move the window somebody was
/// reaching for — worse here than on the Dock, because these are windows and a
/// window that moves under a hand is one somebody closes by accident.
#[test]
fn task_windows_keep_the_order_they_opened_in() {
    let (_directory, mut server) = server("task-order");
    let first = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "Tidy my invoices",
        InferenceSource::ThisMachine,
    ));
    let second = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "Install Blender",
        InferenceSource::ThisMachine,
    ));

    // The second one becomes the busy one.
    server
        .tasks_mut()
        .the_mut(second)
        .expect("the second task is there")
        .alo_says(Turn::alo_asking("Which version?"));

    let order: Vec<&str> = server
        .tasks()
        .each()
        .iter()
        .filter_map(|task| task.conversation().the_request().map(|turn| turn.said()))
        .collect();
    assert_eq!(
        order,
        vec!["Tidy my invoices", "Install Blender"],
        "a task window moved because something happened in it"
    );
    assert_eq!(first.at(), 0);
    assert_eq!(second.at(), 1);
}

/// **Which tasks are waiting is answered in one place.**
///
/// The panel, the Dock and a window list all want it, and three readers working
/// it out separately is three answers to *which of these needs me*.
#[test]
fn which_tasks_are_waiting_is_answered_once() {
    let (_directory, mut server) = server("task-waiting");
    server.tasks_mut().and_now_this_one(ATask::asked_for(
        "Tidy my invoices",
        InferenceSource::ThisMachine,
    ));
    let asking = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "Install Blender",
        InferenceSource::ThisMachine,
    ));
    server
        .tasks_mut()
        .the_mut(asking)
        .expect("the task is there")
        .alo_says(Turn::alo_asking("Which version?"));

    let waiting: Vec<usize> = server
        .tasks()
        .waiting_for_the_person()
        .map(|which| which.at())
        .collect();

    assert_eq!(
        waiting,
        vec![1],
        "the tasks waiting on the person are not the ones that asked something"
    );
}
