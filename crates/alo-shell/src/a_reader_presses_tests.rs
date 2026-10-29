//! That a reader finds Close on every task window, and that pressing it does
//! what clicking it does: a finished task closes, a waiting one does not and
//! its question is not answered, a running one does not.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::os::unix::fs::PermissionsExt as _;

use alo_access::{Surface, TurnedOn};
use alo_changing::{Answered, Answering, Standing};
use alo_models::InferenceSource;
use alo_tasks::ATask;
use alo_tasks::conversation::Turn;

use super::WhatAReaderPressCameTo;
use crate::Server;
use crate::access_action::Pressed;
use crate::approval_testing::words;
use crate::closing_a_task_window::WhatClosingCameTo;
use crate::the_tasks_a_person_opened::WhichTask;

/// A daemon holding what a test tells it to, and remembering every answer.
#[derive(Debug, Default)]
struct ADaemon {
    holding: Vec<Standing>,
    approved: RefCell<Vec<u64>>,
    declined: RefCell<Vec<u64>>,
}

impl Answering for ADaemon {
    fn waiting(&self) -> Option<Vec<Standing>> {
        Some(self.holding.clone())
    }

    fn approve(&self, number: u64) -> Option<Answered> {
        self.approved.borrow_mut().push(number);
        None
    }

    fn decline(&self, number: u64) -> Option<Answered> {
        self.declined.borrow_mut().push(number);
        None
    }
}

/// A display of this test's own.
fn a_session() -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(runtime.path(), "a-reader-presses").expect("a display");
    (runtime, server)
}

/// A task window, placed on a 1280 × 800 view.
fn a_task(server: &mut Server, asked: &str) -> WhichTask {
    let which = server
        .tasks_mut()
        .and_now_this_one(ATask::asked_for(asked, InferenceSource::ThisMachine));
    let view = smithay::utils::Rectangle::new((0, 0).into(), (1280, 800).into());
    server.tasks_mut().put_whatever_has_nowhere_to_be(
        view,
        alo_strings::Direction::LeftToRight,
        &[],
    );
    which
}

/// alo answered, so the task's work is over.
fn finished(server: &mut Server, which: WhichTask) {
    server
        .tasks_mut()
        .the_mut(which)
        .expect("the task is open")
        .alo_says(Turn::from_alo("Your invoices are tidy."));
}

/// One window as a reader is told about it: its name, and each control under
/// it with what pressing that does.
type AWindowHeard = (String, Vec<(String, Option<Pressed>)>);

/// Each window a reader is told is open, with the controls under it.
fn heard(server: &Server) -> Vec<AWindowHeard> {
    let strings = words();
    let tree =
        server.read_aloud_with_the_frames_open(&strings, &[Surface::Desktop], &TurnedOn::nothing());
    // The desktop's list, found by its own name: other surfaces are lists
    // too, and the first list in the tree is not this one.
    let named = Surface::Desktop
        .read_aloud()
        .into_iter()
        .find(|control| control.role == alo_access::Role::List)
        .map(|control| {
            strings
                .say(&control.name.key(), &alo_strings::Filling::nothing())
                .text()
                .to_owned()
        })
        .expect("the desktop is read as a list of the windows open");
    let list = tree
        .nodes()
        .iter()
        .find(|node| node.name == named)
        .expect("the tree carries the list of the windows open");
    list.children
        .iter()
        .filter_map(|at| tree.nodes().get(*at))
        .map(|window| {
            let under = window
                .children
                .iter()
                .filter_map(|at| tree.nodes().get(*at))
                .map(|control| (control.name.clone(), control.press.clone()))
                .collect();
            (window.name.clone(), under)
        })
        .collect()
}

/// The word Close, as a person reading English hears it.
fn close() -> String {
    words()
        .say(
            &alo_shortcuts::Action::CloseWindow.word().key(),
            &alo_strings::Filling::nothing(),
        )
        .text()
        .to_owned()
}

/// **Every open task window is in the tree, named by its request, with Close
/// under it** — and that Close presses that window and no other.
#[test]
fn every_task_window_carries_its_own_close() {
    let (_runtime, mut server) = a_session();
    let first = a_task(&mut server, "tidy my invoices");
    let second = a_task(&mut server, "explain an invoice");

    assert_eq!(
        heard(&server),
        [
            (
                "tidy my invoices".to_owned(),
                vec![(close(), Some(Pressed::CloseTheTask(first)))]
            ),
            (
                "explain an invoice".to_owned(),
                vec![(close(), Some(Pressed::CloseTheTask(second)))]
            ),
        ]
    );
}

/// **With no task open there is nothing to press**: the control for a tree
/// that offers presses only where a task window is.
#[test]
fn with_no_task_window_nothing_can_be_pressed() {
    let (_runtime, server) = a_session();
    let tree =
        server.read_aloud_with_the_frames_open(&words(), &Surface::ALL, &TurnedOn::nothing());
    assert!(tree.what_can_be_pressed().is_empty());
}

/// **A reader's Close closes a finished task**, and the reader is no longer
/// told about it.
#[test]
fn a_readers_close_closes_a_finished_task() {
    let (_runtime, mut server) = a_session();
    let which = a_task(&mut server, "tidy my invoices");
    finished(&mut server, which);

    assert_eq!(
        server.a_reader_pressed(Pressed::CloseTheTask(which)),
        WhatAReaderPressCameTo::ATaskWindow(WhatClosingCameTo::Closed)
    );
    assert!(!server.tasks().is_open(which), "the window did not close");
    assert!(
        heard(&server).is_empty(),
        "a reader is still told it is open"
    );
    assert_eq!(
        server.a_reader_pressed(Pressed::CloseTheTask(which)),
        WhatAReaderPressCameTo::ATaskWindow(WhatClosingCameTo::NothingToClose),
        "pressing it again closed something"
    );
}

/// **A reader's Close on a task waiting for an approval does not close it,
/// and answers nothing**: the change is neither approved nor declined, and
/// the question still stands.
#[test]
fn a_readers_close_never_answers_a_waiting_change() {
    let (_runtime, mut server) = a_session();
    let which = a_task(&mut server, "rename my invoices");
    let wire = r#"{"number":7,"sentence":{"text":"Delete everything in Invoices","came_from":"the-source"}}"#;
    let daemon = ADaemon {
        holding: vec![serde_json::from_str(wire).expect("a waiting change reads from the wire")],
        ..ADaemon::default()
    };
    server.an_answer_came_back(
        which,
        "Shall I?".to_owned(),
        false,
        None,
        Some(&daemon),
        &words(),
    );
    assert!(
        server.the_change_waiting.is_asking(),
        "the control: a change is waiting"
    );

    assert_eq!(
        server.a_reader_pressed(Pressed::CloseTheTask(which)),
        WhatAReaderPressCameTo::ATaskWindow(WhatClosingCameTo::StillRunning)
    );
    assert!(server.tasks().is_open(which), "a waiting task was closed");
    assert!(daemon.approved.borrow().is_empty(), "closing approved it");
    assert!(daemon.declined.borrow().is_empty(), "closing declined it");
    assert!(server.the_change_waiting.is_asking(), "the question went");
    assert_eq!(heard(&server).len(), 1, "a reader lost the waiting window");
}

/// **A reader's Close on a task alo is still working on does not close it**,
/// and does not ask it to stop: that is Stop's, and Stop is not built.
#[test]
fn a_readers_close_leaves_a_running_task_open() {
    let (_runtime, mut server) = a_session();
    let which = a_task(&mut server, "tidy my invoices");
    assert_eq!(
        server.tasks().the(which).expect("open").what_is_happening(),
        alo_tasks::words::ALO_IS_WORKING,
        "the control: alo is working on it"
    );

    assert_eq!(
        server.a_reader_pressed(Pressed::CloseTheTask(which)),
        WhatAReaderPressCameTo::ATaskWindow(WhatClosingCameTo::StillRunning)
    );
    assert!(server.tasks().is_open(which), "a running task was closed");
    assert!(
        !server
            .tasks()
            .the(which)
            .expect("open")
            .stopping()
            .is_waiting(),
        "closing asked the work to stop, which nobody chose"
    );
}
