//! That a finished task window closes and hands the keyboard to the window now
//! in front, that a running one asks rather than closing, and that closing
//! never answers a change.
//!
//! Alt+F4 is pressed by evdev code through the seat, the road a person's key
//! takes, rather than by calling the closing function.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::os::unix::fs::PermissionsExt as _;

use alo_changing::{Answered, Answering, Standing};
use alo_models::InferenceSource;
use alo_tasks::ATask;
use alo_tasks::conversation::Turn;
use smithay::backend::input::KeyState;
use smithay::input::keyboard::XkbConfig;
use smithay::utils::{Physical, Point};

use super::WhatClosingCameTo;
use crate::Server;
use crate::the_tasks_a_person_opened::WhichTask;

/// evdev's codes for Alt and F4.
const LEFT_ALT: u32 = 56;
const F4: u32 = 62;
const TAB: u32 = 15;
const ENTER: u32 = 28;
const ESCAPE: u32 = 1;

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

/// The card's and the reply box's words, untranslated.
fn in_english() -> alo_strings::Strings {
    let mut vocabulary = alo_strings::Vocabulary::empty();
    crate::the_cards_words::declare_the_cards_words(&mut vocabulary)
        .expect("the card's words declare");
    crate::the_closing_words::declare_the_closing_words(&mut vocabulary)
        .expect("the closing words declare");
    crate::the_reply_boxs_words::declare_the_reply_boxs_words(&mut vocabulary)
        .expect("the reply box's words declare");
    alo_strings::Strings::of(vocabulary)
}

/// A session with a keyboard on a US layout and the shipped shortcuts.
fn a_session() -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "closing-a-task-window",
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.the_shortcuts_are(alo_shortcuts::Shortcuts::shipped());
    (runtime, server)
}

/// A task window, placed on a 1280 × 800 view and holding the keyboard.
fn a_task(server: &mut Server, asked: &str) -> WhichTask {
    let which = server
        .tasks_mut()
        .and_now_this_one(ATask::asked_for(asked, InferenceSource::ThisMachine));
    let view = smithay::utils::Rectangle::new((0, 0).into(), (1280, 800).into());
    server.tasks_mut().put_whatever_has_nowhere_to_be(
        view,
        alo_strings::Direction::LeftToRight,
        &[],
        100,
    );
    server.this_task_took_the_keyboard(which);
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

/// The middle of this task's window on the screen.
fn the_middle_of(server: &Server, which: WhichTask) -> Point<i32, Physical> {
    let at = server
        .tasks()
        .where_each_went()
        .get(which.at())
        .copied()
        .flatten()
        .expect("the window was placed");
    (at.loc.x + at.size.w / 2, at.loc.y + at.size.h / 2).into()
}

/// Alt+F4, held and released the way a person presses it.
fn alt_f4(server: &mut Server, daemon: &ADaemon) {
    let strings = in_english();
    let mut one = |code, state| {
        crate::a_key_reaches_the_task_in_front::the_key_the_task_took(
            server,
            crate::DirectKeyEvent {
                code,
                state,
                time: 1,
            },
            Some(daemon),
            &strings,
        )
        .expect("the seat took the key");
    };
    one(LEFT_ALT, KeyState::Pressed);
    one(F4, KeyState::Pressed);
    one(F4, KeyState::Released);
    one(LEFT_ALT, KeyState::Released);
}

/// **Alt+F4 closes a finished task window**: it is no longer drawn, pressed
/// or counted, and with no other task window open the keyboard is let go.
#[test]
fn alt_f4_closes_a_finished_task_window() {
    let (_runtime, mut server) = a_session();
    let which = a_task(&mut server, "tidy my invoices");
    finished(&mut server, which);
    assert_eq!(
        server.tasks().how_many(),
        1,
        "the control: one window is open"
    );
    let middle = the_middle_of(&server, which);

    alt_f4(&mut server, &ADaemon::default());

    assert!(!server.tasks().is_open(which), "the window did not close");
    assert_eq!(server.tasks().how_many(), 0);
    assert_eq!(
        server.tasks().where_each_went(),
        [None],
        "it is still drawn"
    );
    assert_eq!(
        server.tasks().the_one_under(middle),
        None,
        "a press where it was still lands on it"
    );
    assert!(
        !server.a_task_is_taking_the_keys(),
        "the closed window kept the keyboard, so typing would go nowhere"
    );
}

/// **A closed window is never put back**, however many frames place windows.
#[test]
fn a_closed_window_is_not_placed_again() {
    let (_runtime, mut server) = a_session();
    let which = a_task(&mut server, "tidy my invoices");
    finished(&mut server, which);

    assert_eq!(
        server.the_person_closes_a_task(which),
        WhatClosingCameTo::Closed
    );
    let view = smithay::utils::Rectangle::new((0, 0).into(), (1280, 800).into());
    let placed = server.tasks_mut().put_whatever_has_nowhere_to_be(
        view,
        alo_strings::Direction::LeftToRight,
        &[],
        100,
    );

    assert_eq!(placed, 0, "a closed window was placed again");
    assert_eq!(
        server.the_person_closes_a_task(which),
        WhatClosingCameTo::NothingToClose,
        "closing it twice closed something"
    );
}

/// **A running task does not close at the press**: it is asked about, and
/// keeps its window and its keyboard while the person chooses.
/// Nothing stops until *Stop and close* is chosen (ADR 0139).
#[test]
fn a_running_task_does_not_close() {
    let (_runtime, mut server) = a_session();
    let which = a_task(&mut server, "tidy my invoices");
    assert_eq!(
        server.tasks().the(which).expect("open").what_is_happening(),
        alo_tasks::words::ALO_IS_WORKING,
        "the control: alo is working on it"
    );

    alt_f4(&mut server, &ADaemon::default());

    assert!(server.tasks().is_open(which), "a running task was closed");
    assert!(server.a_task_is_taking_the_keys());
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

/// **Closing never answers a waiting change**: Alt+F4 on the window holding
/// it approves nothing, declines nothing, and the question still stands.
#[test]
fn closing_never_answers_a_waiting_change() {
    let (_runtime, mut server) = a_session();
    let which = a_task(&mut server, "rename my invoices");
    let daemon = holding(7, "Delete everything in Invoices");
    server.an_answer_came_back(
        which,
        "Shall I?".to_owned(),
        false,
        None,
        Some(&daemon),
        &in_english(),
    );
    assert!(
        server.the_change_waiting.is_asking(),
        "the control: a change is waiting"
    );

    alt_f4(&mut server, &daemon);

    assert!(daemon.approved.borrow().is_empty(), "closing approved it");
    assert!(daemon.declined.borrow().is_empty(), "closing declined it");
    assert!(server.the_change_waiting.is_asking());
    assert!(server.tasks().is_open(which));
}

/// **An answer for a later task still reaches it** after an earlier one
/// closes — which is why a closed task keeps its place in the list.
#[test]
fn an_answer_for_another_task_still_reaches_it() {
    let (_runtime, mut server) = a_session();
    let first = a_task(&mut server, "tidy my invoices");
    finished(&mut server, first);
    let second = a_task(&mut server, "explain an invoice");

    assert_eq!(
        server.the_person_closes_a_task(first),
        WhatClosingCameTo::Closed
    );
    server.an_answer_came_back(
        second,
        "An invoice asks to be paid.".to_owned(),
        false,
        None,
        None,
        &in_english(),
    );

    let said_last = |which| {
        server
            .tasks()
            .the(which)
            .expect("the task is still in the list")
            .conversation()
            .each()
            .last()
            .expect("something was said")
            .said()
            .to_owned()
    };
    assert_eq!(said_last(second), "An invoice asks to be paid.");
    assert_eq!(said_last(first), "Your invoices are tidy.");
}

/// One key, pressed and released, the way a person presses it.
fn press(server: &mut Server, code: u32, daemon: &ADaemon) {
    let strings = in_english();
    for state in [KeyState::Pressed, KeyState::Released] {
        crate::a_key_reaches_the_task_in_front::the_key_the_task_took(
            server,
            crate::DirectKeyEvent {
                code,
                state,
                time: 1,
            },
            Some(daemon),
            &strings,
        )
        .expect("the seat took the key");
    }
}

/// **Closing a running task asks *Stop and close* or *Cancel***, and
/// *Cancel* closes nothing and stops nothing (ADR 0139).
#[test]
fn closing_a_running_task_asks_and_cancel_changes_nothing() {
    let (_runtime, mut server) = a_session();
    let which = a_task(&mut server, "tidy my invoices");
    let daemon = ADaemon::default();

    alt_f4(&mut server, &daemon);
    assert!(
        server.closing_asked_about(which).is_some(),
        "closing a running task asked nothing"
    );
    press(&mut server, TAB, &daemon);
    press(&mut server, TAB, &daemon);
    press(&mut server, ENTER, &daemon);

    assert!(server.closing_asked_about(which).is_none());
    assert!(server.tasks().is_open(which), "Cancel closed the window");
    assert!(
        server.take_the_stops().is_empty(),
        "Cancel stopped the work"
    );
}

/// **Escape is *Cancel*, and Enter with nothing chosen does nothing** — a
/// person who presses Enter to dismiss something has not chosen to stop.
#[test]
fn escape_cancels_and_enter_alone_chooses_nothing() {
    let (_runtime, mut server) = a_session();
    let which = a_task(&mut server, "tidy my invoices");
    let daemon = ADaemon::default();
    alt_f4(&mut server, &daemon);

    press(&mut server, ENTER, &daemon);
    assert!(
        server.take_the_stops().is_empty(),
        "Enter alone stopped the work"
    );
    press(&mut server, ESCAPE, &daemon);
    assert!(server.closing_asked_about(which).is_none());
    assert!(server.tasks().is_open(which));
    assert!(server.take_the_stops().is_empty());
}

/// **Stop and close stops alo, and the window closes only once alo has
/// really stopped** — never at the press.
#[test]
fn stop_and_close_closes_only_after_alo_has_stopped() {
    let (_runtime, mut server) = a_session();
    let which = a_task(&mut server, "tidy my invoices");
    let daemon = ADaemon::default();
    alt_f4(&mut server, &daemon);
    press(&mut server, TAB, &daemon);
    press(&mut server, ENTER, &daemon);

    let stops = server.take_the_stops();
    assert_eq!(stops.len(), 1, "Stop and close did not stop");
    assert!(server.tasks().is_open(which), "closed before alo stopped");
    assert_eq!(
        server.tasks().the(which).expect("open").what_is_happening(),
        alo_tasks::words::STOPPING
    );

    server.it_really_stopped(which);
    assert!(
        !server.tasks().is_open(which),
        "did not close once alo stopped"
    );
}

/// **Stop and close never answers a waiting change**: it lapses with the
/// turn, neither approved nor declined.
#[test]
fn stop_and_close_never_answers_a_waiting_change() {
    let (_runtime, mut server) = a_session();
    let which = a_task(&mut server, "rename my invoices");
    let daemon = holding(7, "Delete everything in Invoices");
    server.an_answer_came_back(
        which,
        "Shall I?".to_owned(),
        false,
        None,
        Some(&daemon),
        &in_english(),
    );
    alt_f4(&mut server, &daemon);
    press(&mut server, TAB, &daemon);
    press(&mut server, ENTER, &daemon);

    assert!(
        daemon.approved.borrow().is_empty(),
        "Stop approved the change"
    );
    assert!(
        daemon.declined.borrow().is_empty(),
        "Stop declined the change"
    );
    assert!(!server.the_change_waiting.is_asking(), "the card stayed up");
    assert_eq!(server.take_the_stops().len(), 1);
}

/// **The walk of 2026-10-30, pressed again**: Escape twice in a finished
/// task's empty reply box, then Alt+F4. Escape used to let go, so Alt+F4 found
/// *no focused window to close* and the window stayed in front. Now Escape
/// keeps the keyboard and Alt+F4 closes the window.
#[test]
fn escape_then_alt_f4_closes_a_finished_task_window() {
    let (_runtime, mut server) = a_session();
    let which = a_task(&mut server, "tidy my invoices");
    finished(&mut server, which);
    let daemon = ADaemon::default();

    press(&mut server, ESCAPE, &daemon);
    press(&mut server, ESCAPE, &daemon);
    assert!(
        server.a_task_is_taking_the_keys(),
        "Escape gave the keyboard up"
    );
    alt_f4(&mut server, &daemon);

    assert!(!server.tasks().is_open(which), "Alt+F4 did not close it");
}

/// **After a window closes, the keyboard goes to the task window now in
/// front**, which is drawn over everything else, rather than falling to
/// whatever is hidden behind it.
#[test]
fn closing_hands_the_keyboard_to_the_task_window_now_in_front() {
    let (_runtime, mut server) = a_session();
    let behind = a_task(&mut server, "tidy my invoices");
    let front = a_task(&mut server, "explain an invoice");
    finished(&mut server, front);
    assert_eq!(
        server.the_task_with_the_keyboard,
        Some(front),
        "the control: the newest window has the keyboard"
    );

    alt_f4(&mut server, &ADaemon::default());

    assert!(!server.tasks().is_open(front));
    assert_eq!(
        server.the_task_with_the_keyboard,
        Some(behind),
        "the keyboard did not go to the window now in front"
    );
}

/// **The keyboard goes nowhere new when the closed window did not hold it.**
/// A person typing elsewhere who presses another window's Close has not asked
/// to type into a task window.
#[test]
fn closing_a_window_without_the_keyboard_leaves_the_keyboard_where_it_was() {
    let (_runtime, mut server) = a_session();
    let _open = a_task(&mut server, "tidy my invoices");
    let closing = a_task(&mut server, "explain an invoice");
    finished(&mut server, closing);
    server.the_task_gave_the_keyboard_up();

    assert_eq!(
        server.the_person_closes_a_task(closing),
        WhatClosingCameTo::Closed
    );
    assert!(
        !server.a_task_is_taking_the_keys(),
        "closing a window that did not hold the keyboard handed it to a task"
    );
}

/// **Alt+F4 on a task already stopping closes nothing and keeps the
/// keyboard**: it closes by itself once alo has really stopped.
#[test]
fn alt_f4_on_a_stopping_task_closes_nothing() {
    let (_runtime, mut server) = a_session();
    let which = a_task(&mut server, "tidy my invoices");
    let daemon = ADaemon::default();
    alt_f4(&mut server, &daemon);
    press(&mut server, TAB, &daemon);
    press(&mut server, ENTER, &daemon);
    assert_eq!(
        server.tasks().the(which).expect("open").what_is_happening(),
        alo_tasks::words::STOPPING,
        "the control: Stop and close was chosen"
    );

    alt_f4(&mut server, &daemon);

    assert!(server.tasks().is_open(which), "a stopping task was closed");
    assert!(server.closing_asked_about(which).is_none(), "it asked again");
    assert!(server.a_task_is_taking_the_keys());
}
