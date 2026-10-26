//! **The road a real key takes into a task window**, pressed by evdev code
//! through the seat, with a stand-in daemon.
//!
//! Nothing here calls `Server::a_key_reached_a_task_window` or
//! `AChangeWaiting::pressed` directly: both were finished and tested while a
//! task window could not receive one key, and a test that calls a link in the
//! road is the fault this file exists to have fixed.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::os::unix::fs::PermissionsExt;

use alo_changing::{Answered, Answering, Done, Standing};
use alo_models::InferenceSource;
use alo_tasks::ATask;
use smithay::backend::input::KeyState;
use smithay::input::keyboard::XkbConfig;

use super::the_key_the_task_took;
use crate::Server;
use crate::a_change_waiting_reaches_the_person::WhichAnswer;

/// evdev's codes for the keys these tests press.
const ESCAPE: u32 = 1;
const TAB: u32 = 15;
const ENTER: u32 = 28;
const A: u32 = 30;

/// A daemon holding what a test tells it to, and remembering what it was told.
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
        Some(Answered::It(Box::new(Done::Renamed { now_at: None })))
    }

    fn decline(&self, number: u64) -> Option<Answered> {
        self.declined.borrow_mut().push(number);
        Some(Answered::It(Box::new(Done::Renamed { now_at: None })))
    }
}

/// A daemon holding one change, read from the wire form a daemon sends.
fn holding(number: u64, sentence: &str) -> ADaemon {
    let wire = format!(
        r#"{{"number":{number},"sentence":{{"text":{sentence:?},"came_from":"the-source"}}}}"#
    );
    ADaemon {
        holding: vec![serde_json::from_str(&wire).expect("a waiting change reads from the wire")],
        ..ADaemon::default()
    }
}

/// A session with a keyboard on a US layout.
fn a_session() -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind_keyboard(
        runtime.path(),
        // A socket name, so hyphens and no spaces — the Bar's tests name theirs
        // the same way.
        "a-task-window-takes-keys",
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    (runtime, server)
}

/// A task window a person just asked for, holding the keyboard.
fn a_task_holding_the_keyboard(server: &mut Server) -> crate::the_tasks_a_person_opened::WhichTask {
    let which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "tidy my invoices",
        InferenceSource::ThisMachine,
    ));
    server.this_task_took_the_keyboard(which);
    which
}

/// Press and release one key, exactly as the lane hands it after libinput.
fn press(server: &mut Server, code: u32, daemon: &ADaemon) {
    for state in [KeyState::Pressed, KeyState::Released] {
        the_key_the_task_took(
            server,
            crate::DirectKeyEvent {
                code,
                state,
                time: 1,
            },
            Some(daemon),
        )
        .expect("the seat took the key");
    }
}

/// The reply box's contents.
fn draft(server: &Server, which: crate::the_tasks_a_person_opened::WhichTask) -> String {
    server
        .tasks()
        .the(which)
        .expect("the task is open")
        .the_draft()
        .to_owned()
}

/// **What is typed into a task window reaches its reply box** — the thing that
/// had never happened on a running machine.
#[test]
fn what_is_typed_reaches_the_reply_box() {
    let (_runtime, mut server) = a_session();
    let which = a_task_holding_the_keyboard(&mut server);
    let daemon = ADaemon::default();

    press(&mut server, A, &daemon);

    assert_eq!(draft(&server, which), "a");
}

/// **Tab moves to the answers, and Enter gives the one selected** — by the
/// number the daemon gave.
#[test]
fn tab_reaches_the_answers_and_enter_gives_the_one_selected() {
    let (_runtime, mut server) = a_session();
    let which = a_task_holding_the_keyboard(&mut server);
    let daemon = holding(7, "Rename march.pdf");
    server.an_answer_came_back(which, "I can do that.".to_owned(), false, Some(&daemon));

    press(&mut server, TAB, &daemon);
    assert_eq!(
        server.the_change_waiting.selected(),
        Some(WhichAnswer::AllowOnce)
    );

    press(&mut server, ENTER, &daemon);
    assert_eq!(*daemon.approved.borrow(), [7]);
    assert!(!server.the_change_waiting.is_asking());
}

/// **Enter with no answer selected sends the reply, and approves nothing.**
///
/// The one key where *nothing proceeds on silence* could go wrong: a person who
/// typed a reply and pressed Enter to send it has not answered the card.
#[test]
fn enter_with_no_answer_selected_sends_the_reply_and_approves_nothing() {
    let (_runtime, mut server) = a_session();
    let which = a_task_holding_the_keyboard(&mut server);
    let daemon = holding(7, "Delete everything");
    server.an_answer_came_back(which, "Shall I?".to_owned(), false, Some(&daemon));

    press(&mut server, A, &daemon);
    press(&mut server, ENTER, &daemon);

    assert!(
        daemon.approved.borrow().is_empty(),
        "Enter approved the change"
    );
    assert!(
        daemon.declined.borrow().is_empty(),
        "Enter declined the change"
    );
    assert!(
        server.the_change_waiting.is_asking(),
        "the question was taken down"
    );
    assert_eq!(draft(&server, which), "", "the reply was not sent");
}

/// **Escape on a selected answer says no.**
#[test]
fn escape_on_a_selected_answer_says_no() {
    let (_runtime, mut server) = a_session();
    let which = a_task_holding_the_keyboard(&mut server);
    let daemon = holding(4, "Delete everything");
    server.an_answer_came_back(which, "Shall I?".to_owned(), false, Some(&daemon));

    press(&mut server, TAB, &daemon);
    press(&mut server, ESCAPE, &daemon);

    assert_eq!(*daemon.declined.borrow(), [4]);
    assert!(daemon.approved.borrow().is_empty());
}

/// **Escape on an empty box, nothing selected, gives the keyboard up**, so a
/// person can always get back to their applications.
#[test]
fn escape_on_an_empty_box_lets_go_of_the_keyboard() {
    let (_runtime, mut server) = a_session();
    let which = a_task_holding_the_keyboard(&mut server);
    let daemon = ADaemon::default();

    press(&mut server, A, &daemon);
    press(&mut server, ESCAPE, &daemon);
    assert!(
        server.a_task_is_taking_the_keys(),
        "the first Escape only empties the box"
    );
    assert_eq!(draft(&server, which), "");

    press(&mut server, ESCAPE, &daemon);
    assert!(
        !server.a_task_is_taking_the_keys(),
        "the second Escape did not let go"
    );
}

/// **A question arriving never takes the keyboard.**
///
/// The safety half of ordinary focus: a proposal that could take the keyboard
/// could be approved by somebody typing *Tab* and *Enter* into a document.
#[test]
fn a_question_arriving_never_takes_the_keyboard() {
    let (_runtime, mut server) = a_session();
    let which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "tidy my invoices",
        InferenceSource::ThisMachine,
    ));
    let daemon = holding(7, "Rename march.pdf");

    server.an_answer_came_back(which, "Shall I?".to_owned(), false, Some(&daemon));

    assert!(
        server.the_change_waiting.is_asking(),
        "the control: a question arrived"
    );
    assert!(
        !server.a_task_is_taking_the_keys(),
        "a question arriving took the keyboard"
    );
}

/// **A question in one window cannot be answered from another.** A person may
/// only answer what is in front of them.
#[test]
fn a_question_in_another_window_cannot_be_answered_from_this_one() {
    let (_runtime, mut server) = a_session();
    let asking = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "tidy my invoices",
        InferenceSource::ThisMachine,
    ));
    let daemon = holding(7, "Rename march.pdf");
    server.an_answer_came_back(asking, "Shall I?".to_owned(), false, Some(&daemon));
    let _typing_here = a_task_holding_the_keyboard(&mut server);

    press(&mut server, TAB, &daemon);
    press(&mut server, ENTER, &daemon);

    assert_eq!(
        server.the_change_waiting.selected(),
        None,
        "Tab reached another window's card"
    );
    assert!(
        daemon.approved.borrow().is_empty(),
        "another window's change was approved"
    );
}

/// **A click on a task window gives it the keyboard, and the Bar lets go.**
///
/// Before 2026-10-26 nothing hit-tested a task window, so a press on one fell
/// through to whatever application was underneath. The control is that no task
/// held the keyboard before the click.
#[test]
fn a_click_on_a_task_window_gives_it_the_keyboard() {
    let (_runtime, mut server) = a_session();
    server.enable_pointer().expect("a pointer on this seat");
    let which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "tidy my invoices",
        InferenceSource::ThisMachine,
    ));
    let view = smithay::utils::Rectangle::new((0, 0).into(), (1280, 800).into());
    server.tasks_mut().put_whatever_has_nowhere_to_be(
        view,
        alo_strings::Direction::LeftToRight,
        &[],
    );
    let window = server
        .tasks()
        .where_each_went()
        .get(which.at())
        .copied()
        .flatten()
        .expect("the task window was placed");
    server.the_composer.it_took_the_keyboard();
    assert!(
        !server.a_task_is_taking_the_keys(),
        "the control: a task held it already"
    );

    let (x, y) = (f64::from(window.loc.x + 20), f64::from(window.loc.y + 20));
    server.pointer_motion(x, y, 1).expect("the pointer moved");
    let delivered = server
        .pointer_button(0x110, smithay::backend::input::ButtonState::Pressed, 2)
        .expect("the press was handled");

    assert!(
        !delivered,
        "a press on a task window reached an application behind it"
    );
    assert!(
        server.a_task_is_taking_the_keys(),
        "the task window did not take the keyboard"
    );
    assert!(
        !server.the_bar_is_taking_the_keys(),
        "the Bar kept the keyboard"
    );
}
