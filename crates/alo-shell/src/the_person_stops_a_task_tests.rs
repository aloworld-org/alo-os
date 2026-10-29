//! What pressing Stop does in the window at once, and what it waits for.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_models::InferenceSource;
use alo_tasks::ATask;
use alo_tasks::conversation::Turn;
use smithay::input::keyboard::XkbConfig;

use crate::Server;
use crate::the_tasks_a_person_opened::WhichTask;

/// A session, and a task in it that alo is working on.
fn a_working_task() -> (tempfile::TempDir, Server, WhichTask) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "a-task-is-stopped",
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    let which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "tidy my invoices",
        InferenceSource::ThisMachine,
    ));
    (runtime, server, which)
}

/// What the task says is happening.
fn happening(server: &Server, which: WhichTask) -> alo_strings::Word {
    server
        .tasks()
        .the(which)
        .expect("the task is open")
        .what_is_happening()
}

/// **At the press the task says *stopping*, never *stopped***, and one Stop
/// is handed on to travel to the agent.
#[test]
fn a_press_says_stopping_and_hands_one_stop_on() {
    let (_runtime, mut server, which) = a_working_task();
    assert!(server.the_person_stops_a_task(which));
    assert_eq!(happening(&server, which), alo_tasks::words::STOPPING);
    let stops = server.take_the_stops();
    assert_eq!(stops.len(), 1);
    assert!(
        stops
            .first()
            .is_some_and(|stop| stop.which == which && stop.held_turn.is_none())
    );
    assert!(
        server.take_the_stops().is_empty(),
        "one press stopped twice"
    );
}

/// ***Stopped* only once the agent has gone**, naming what the person allowed
/// before the Stop.
#[test]
fn stopped_is_said_when_the_agent_has_gone_and_names_what_changed() {
    let (_runtime, mut server, which) = a_working_task();
    {
        let task = server.tasks_mut().the_mut(which).expect("open");
        task.is_asking("Move march.pdf to Paid");
        task.alo_says(Turn::alo_proposing("Move march.pdf to Paid"));
        let _ = task.allow_once();
        task.the_person_says("and april too");
    }
    assert!(server.the_person_stops_a_task(which));
    server.it_really_stopped(which);
    assert_eq!(happening(&server, which), alo_tasks::words::STOPPED);
    let stopping = server.tasks().the(which).expect("open").stopping().clone();
    let alo_tasks::Stopping::Stopped { changed } = stopping else {
        panic!("not stopped: {stopping:?}");
    };
    assert_eq!(changed.each(), ["Move march.pdf to Paid"]);
}

/// **A finished task has nothing to stop**, and a press stops nothing.
#[test]
fn a_finished_task_has_nothing_to_stop() {
    let (_runtime, mut server, which) = a_working_task();
    server
        .tasks_mut()
        .the_mut(which)
        .expect("open")
        .alo_says(Turn::from_alo("They are tidy."));
    assert!(!server.the_person_stops_a_task(which));
    assert!(server.take_the_stops().is_empty());
}

/// **A held reply is never sent after a Stop**: it goes back to the reply box,
/// so nothing the person typed is lost.
#[test]
fn a_held_reply_goes_back_to_the_box_and_is_never_sent() {
    let (_runtime, mut server, which) = a_working_task();
    server
        .tasks_mut()
        .the_mut(which)
        .expect("open")
        .hold("and the April one too");
    assert!(server.the_person_stops_a_task(which));
    let task = server.tasks().the(which).expect("open");
    assert_eq!(task.held(), None);
    assert_eq!(task.the_draft(), "and the April one too");
    assert!(server.take_the_follow_ups().is_empty());
}
