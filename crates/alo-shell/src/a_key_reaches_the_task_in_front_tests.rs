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
use std::io::Read as _;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;

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
    /// The agent's end of the turn's socket, when a test hands one over.
    agents_end: Option<UnixStream>,
    /// Whether the turn was still open each time an answer arrived here.
    open_when_answered: RefCell<Vec<bool>>,
    /// Whether it answers nothing at all, as a daemon that never replied.
    says_nothing: bool,
}

impl ADaemon {
    /// Note whether the agent still holds its turn, as the answer arrives.
    fn notes_the_turn(&self) {
        if let Some(agents_end) = &self.agents_end {
            self.open_when_answered
                .borrow_mut()
                .push(still_open(agents_end));
        }
    }
}

/// Whether the shell still holds its end: a read that would wait is open, and
/// one that reads nothing at all is closed.
fn still_open(agents_end: &UnixStream) -> bool {
    agents_end
        .set_nonblocking(true)
        .expect("the agent's end can be asked without waiting");
    let open = matches!(
        (&*agents_end).read(&mut [0_u8; 1]),
        Err(why) if why.kind() == std::io::ErrorKind::WouldBlock
    );
    agents_end
        .set_nonblocking(false)
        .expect("the agent's end waits again");
    open
}

impl Answering for ADaemon {
    fn waiting(&self) -> Option<Vec<Standing>> {
        Some(self.holding.clone())
    }

    fn approve(&self, number: u64) -> Option<Answered> {
        self.notes_the_turn();
        self.approved.borrow_mut().push(number);
        (!self.says_nothing).then(|| Answered::It(Box::new(Done::Renamed { now_at: None })))
    }

    /// `Declined`, which is what the real daemon answers a kept decline with.
    fn decline(&self, number: u64) -> Option<Answered> {
        self.notes_the_turn();
        self.declined.borrow_mut().push(number);
        (!self.says_nothing).then_some(Answered::Declined)
    }
}

/// The card's words, untranslated — what a person reads after answering.
fn in_english() -> alo_strings::Strings {
    let mut vocabulary = alo_strings::Vocabulary::empty();
    crate::the_cards_words::declare_the_cards_words(&mut vocabulary)
        .expect("the card's words declare");
    alo_strings::Strings::of(vocabulary)
}

/// What alo said last in this task.
fn the_last_thing_said(
    server: &Server,
    which: crate::the_tasks_a_person_opened::WhichTask,
) -> String {
    server
        .tasks()
        .the(which)
        .expect("the task is open")
        .conversation()
        .each()
        .last()
        .expect("something was said")
        .said()
        .to_owned()
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
            &in_english(),
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
    server.an_answer_came_back(
        which,
        "I can do that.".to_owned(),
        false,
        None,
        Some(&daemon),
        &in_english(),
    );

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
    server.an_answer_came_back(
        which,
        "Shall I?".to_owned(),
        false,
        None,
        Some(&daemon),
        &in_english(),
    );

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
    server.an_answer_came_back(
        which,
        "Shall I?".to_owned(),
        false,
        None,
        Some(&daemon),
        &in_english(),
    );

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
    server.an_answer_came_back(
        asking,
        "Shall I?".to_owned(),
        false,
        None,
        Some(&daemon),
        &in_english(),
    );
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

/// **The agent keeps its turn while the question is on screen, and loses it
/// only after the answer was carried out.**
///
/// The change lives in the agent's turn, so a turn that ended first would take
/// the change with it — which is what the walk of 2026-10-26 saw: alo proposed
/// the right change and the card never came. The daemon here notes whether the
/// turn was still open at the moment each answer reached it.
#[test]
fn the_agents_turn_is_held_until_the_person_answers() {
    for (answer, the_answer_is) in [(ENTER, "allowed"), (ESCAPE, "denied")] {
        let (_runtime, mut server) = a_session();
        let which = a_task_holding_the_keyboard(&mut server);
        let (ours, agents_end) = UnixStream::pair().expect("a socket pair");
        let daemon = ADaemon {
            agents_end: Some(agents_end),
            ..holding(7, "Rename march.pdf")
        };

        server.an_answer_came_back(
            which,
            "Shall I?".to_owned(),
            false,
            Some(ours),
            Some(&daemon),
            &in_english(),
        );
        let held = daemon.agents_end.as_ref().is_some_and(still_open);
        assert!(held, "the turn ended before the person was asked");

        press(&mut server, TAB, &daemon);
        press(&mut server, answer, &daemon);

        assert_eq!(
            *daemon.open_when_answered.borrow(),
            [true],
            "the turn was not open when the {the_answer_is} change reached the daemon"
        );
        let after = daemon.agents_end.as_ref().is_some_and(still_open);
        assert!(
            !after,
            "the turn was still held after it was {the_answer_is}"
        );
    }
}

/// **An answer with nothing waiting lets the agent go at once.** A held turn
/// would only keep the next request out.
#[test]
fn an_answer_with_nothing_waiting_lets_the_agent_go() {
    let (_runtime, mut server) = a_session();
    let which = a_task_holding_the_keyboard(&mut server);
    let (ours, agents_end) = UnixStream::pair().expect("a socket pair");
    let daemon = ADaemon::default();

    server.an_answer_came_back(
        which,
        "It is in Invoices.".to_owned(),
        false,
        Some(ours),
        Some(&daemon),
        &in_english(),
    );

    assert!(
        !server.the_change_waiting.is_asking(),
        "the control: nothing was waiting"
    );
    assert!(
        !still_open(&agents_end),
        "the agent was kept with nothing to wait for"
    );
}

/// evdev's codes for the modifiers and letters the shortcut tests hold.
const LEFT_CTRL: u32 = 29;
const LEFT_ALT: u32 = 56;
const LEFT_SUPER: u32 = 125;
const T: u32 = 20;

/// Hold these keys down, press and release one more, then let go of them in
/// reverse — the way a person presses a shortcut.
fn press_holding(server: &mut Server, held: &[u32], code: u32, daemon: &ADaemon) {
    let mut one = |code, state| {
        the_key_the_task_took(
            server,
            crate::DirectKeyEvent {
                code,
                state,
                time: 1,
            },
            Some(daemon),
            &in_english(),
        )
        .expect("the seat took the key");
    };
    for &down in held {
        one(down, KeyState::Pressed);
    }
    one(code, KeyState::Pressed);
    one(code, KeyState::Released);
    for &up in held.iter().rev() {
        one(up, KeyState::Released);
    }
}

/// **Ctrl+Alt+T in a task window opens the terminal, types nothing, and the
/// keyboard goes with it.**
///
/// Found walking a booted machine on 2026-10-26: with a task window holding
/// the keyboard, every shortcut was swallowed. The control is a plain `t`,
/// which is typed — so the shortcut was told apart by its modifiers, not by
/// the task refusing the letter.
#[test]
fn a_shortcut_in_a_task_window_is_carried_out_and_not_typed() {
    let (_runtime, mut server) = a_session();
    server.the_shortcuts_are(alo_shortcuts::Shortcuts::shipped());
    let which = a_task_holding_the_keyboard(&mut server);
    let daemon = ADaemon::default();

    press(&mut server, T, &daemon);
    assert_eq!(
        draft(&server, which),
        "t",
        "the control: a plain t is typed"
    );

    press_holding(&mut server, &[LEFT_CTRL, LEFT_ALT], T, &daemon);

    assert_eq!(
        draft(&server, which),
        "t",
        "the shortcut was typed into the reply box"
    );
    assert!(
        server.alos_own_terminal().is_some(),
        "Ctrl+Alt+T in a task window opened no terminal"
    );
    assert!(
        !server.a_task_is_taking_the_keys(),
        "the terminal opened and the task kept the keyboard, so typing would not reach it"
    );
}

/// **The agent's shortcut leaves the task holding the keyboard**, because the
/// Bar is in front of it: when the Bar lets go, typing comes back to the task.
#[test]
fn a_shortcut_to_a_surface_in_front_leaves_the_task_its_keyboard() {
    let (_runtime, mut server) = a_session();
    server.the_shortcuts_are(alo_shortcuts::Shortcuts::shipped());
    let which = a_task_holding_the_keyboard(&mut server);
    let daemon = ADaemon::default();

    press_holding(&mut server, &[LEFT_SUPER], A, &daemon);

    assert!(
        server.the_bar_is_taking_the_keys(),
        "Super+A in a task window did not open the Bar"
    );
    assert!(
        server.a_task_is_taking_the_keys(),
        "the task let go for a surface drawn in front of it"
    );
    assert_eq!(draft(&server, which), "", "the shortcut was typed");
}

/// Whether this task still says it is waiting for the person.
fn still_waiting(server: &Server, which: crate::the_tasks_a_person_opened::WhichTask) -> bool {
    server
        .tasks()
        .the(which)
        .expect("the task is open")
        .what_is_happening()
        .key()
        == alo_tasks::words::WAITING_FOR_YOUR_PERMISSION.key()
}

/// **After an answer, the person is told what it did, and the task stops
/// waiting.** Found walking a booted machine on 2026-10-26: Allow once was
/// pressed, the daemon refused, and the screen said nothing — the task still
/// read *Waiting for your permission*.
///
/// Each of the three answers a daemon can give, pressed the way a person
/// presses them; the control is that every one was waiting before.
#[test]
fn the_person_is_told_what_their_answer_did() {
    for (keys, says_nothing, told) in [
        (&[TAB, ENTER][..], false, "Done."),
        (
            &[TAB, ESCAPE][..],
            false,
            "You said no, so nothing was changed.",
        ),
        (
            &[TAB, ENTER][..],
            true,
            "This computer did not say what happened to that change.",
        ),
    ] {
        let (_runtime, mut server) = a_session();
        let which = a_task_holding_the_keyboard(&mut server);
        let daemon = ADaemon {
            says_nothing,
            ..holding(7, "Rename march.pdf")
        };
        server.an_answer_came_back(
            which,
            "Shall I?".to_owned(),
            false,
            None,
            Some(&daemon),
            &in_english(),
        );
        assert!(
            still_waiting(&server, which),
            "the control: a question was waiting"
        );

        for &key in keys {
            press(&mut server, key, &daemon);
        }

        assert_eq!(the_last_thing_said(&server, which), told);
        assert!(
            !still_waiting(&server, which),
            "the task still waits after it was answered with {keys:?}"
        );
    }
}

/// **A turn that stopped without answering says so, and stops working.**
/// Found walking a booted machine on 2026-10-27: a request went round eight
/// times and the agent gave up, which reaches the shell as *the machine could
/// not be asked*. The reason went beside the conversation and nothing went in
/// it, so the window read *alo is working* for as long as it stayed open.
#[test]
fn a_turn_that_stopped_without_answering_says_so_and_is_no_longer_working() {
    let (_runtime, mut server) = a_session();
    let which = a_task_holding_the_keyboard(&mut server);
    let is_working = |server: &Server| {
        server
            .tasks()
            .the(which)
            .expect("the task is open")
            .what_is_happening()
            .key()
            == alo_tasks::words::ALO_IS_WORKING.key()
    };
    assert!(
        is_working(&server),
        "a request just sent is alo's to answer"
    );

    let daemon = holding(7, "Rename march.pdf");
    server.an_answer_came_back(
        which,
        "your agent closed without answering".to_owned(),
        true,
        None,
        Some(&daemon),
        &in_english(),
    );

    assert!(!is_working(&server), "the window still says alo is working");
    assert_eq!(
        the_last_thing_said(&server, which),
        "alo stopped without finishing this."
    );
    assert!(
        !server.the_change_waiting.is_asking(),
        "a turn that stopped left a question standing"
    );
}
