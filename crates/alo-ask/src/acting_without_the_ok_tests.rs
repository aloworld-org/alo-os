//! The refusals first, because for a switch that lets alo act without asking,
//! *and it was not done* is the test that matters.

#![expect(
    clippy::panic,
    reason = "in a test, a panic naming what did not read is the failure being reported"
)]

use std::cell::{Cell, RefCell};

use alo_capability::AloInTheTerminal;
use alo_changing::{Answered, Answering, Done, Standing};
use alo_protocol::what_the_agent_says_back::{a_change_is_waiting, never_read_as_a_change};

use super::{WhatTheTerminalDid, what_the_terminal_does};

/// What happened at the daemon, in order, beside what the switch was asked.
type TheLog = RefCell<Vec<String>>;

/// A daemon holding these changes, keeping a log of what it was asked.
#[derive(Debug)]
struct ADaemon<'a> {
    /// What it says is waiting, or [`None`] for a daemon that says nothing.
    holding: Option<Vec<Standing>>,
    /// Where it writes down what it was asked.
    log: &'a TheLog,
    /// How many approvals reached it.
    approved: Cell<usize>,
}

impl<'a> ADaemon<'a> {
    /// A daemon holding `changes`, writing to `log`.
    fn holding(changes: Option<Vec<Standing>>, log: &'a TheLog) -> Self {
        Self {
            holding: changes,
            log,
            approved: Cell::new(0),
        }
    }
}

impl Answering for ADaemon<'_> {
    fn waiting(&self) -> Option<Vec<Standing>> {
        self.log.borrow_mut().push("waiting".to_owned());
        self.holding.clone()
    }

    fn approve(&self, number: u64) -> Option<Answered> {
        self.log.borrow_mut().push(format!("approve {number}"));
        self.approved.set(self.approved.get() + 1);
        Some(Answered::It(Box::new(Done::Renamed { now_at: None })))
    }

    fn decline(&self, number: u64) -> Option<Answered> {
        self.log.borrow_mut().push(format!("decline {number}"));
        Some(Answered::Declined)
    }
}

/// One change the daemon is holding, read from the wire form a daemon sends —
/// `alo-shell`'s reason: `Standing::of` needs the whole approvals machinery,
/// and the bytes hold this to what really crosses the socket.
fn standing(number: u64, sentence: &str) -> Standing {
    let wire = format!(
        r#"{{"number":{number},"sentence":{{"text":{sentence:?},"came_from":"the-source"}}}}"#
    );
    serde_json::from_str(&wire)
        .unwrap_or_else(|why| panic!("a waiting change did not read from {wire}: {why}"))
}

/// The sentence of the change the request left.
const RENAME: &str = "Rename march.pdf to march-final.pdf";

/// The switch, answering `answer` and writing down that it was asked.
fn the_switch(answer: AloInTheTerminal, log: &TheLog) -> impl Fn() -> AloInTheTerminal + '_ {
    move || {
        log.borrow_mut().push("the switch".to_owned());
        answer
    }
}

/// **With the switch on, this request's change is carried out, once.**
#[test]
fn with_the_switch_on_the_change_this_request_left_is_approved_once() {
    let log = TheLog::default();
    let daemon = ADaemon::holding(Some(vec![standing(3, RENAME)]), &log);
    let did = what_the_terminal_does(
        &a_change_is_waiting(3, RENAME),
        &daemon,
        &the_switch(AloInTheTerminal::ActsWithoutAsking, &log),
    );
    assert_eq!(
        did,
        WhatTheTerminalDid::WentAhead {
            sentence: RENAME.to_owned(),
            answered: Some(Answered::It(Box::new(Done::Renamed { now_at: None }))),
        }
    );
    assert_eq!(daemon.approved.get(), 1, "one request was approved more than once");
}

/// **Switched off while alo was working, the change already waiting is left
/// alone** — nothing is sent, and the switch was read at the moment of
/// answering rather than when the request was typed.
#[test]
fn switched_off_mid_turn_the_waiting_change_is_not_approved() {
    let log = TheLog::default();
    let daemon = ADaemon::holding(Some(vec![standing(3, RENAME)]), &log);
    let did = what_the_terminal_does(
        &a_change_is_waiting(3, RENAME),
        &daemon,
        &the_switch(AloInTheTerminal::AsksFirst, &log),
    );
    assert_eq!(did, WhatTheTerminalDid::SwitchedOffFirst);
    assert_eq!(
        *log.borrow(),
        ["waiting", "the switch"],
        "the switch was not the last thing read before answering, or something was sent"
    );
}

/// **The switch is the last thing read before the OK goes**: the daemon is
/// asked what is waiting first, so a person switching it off while that answer
/// is on its way is still in time.
#[test]
fn the_switch_is_read_after_the_daemon_and_just_before_the_ok() {
    let log = TheLog::default();
    let daemon = ADaemon::holding(Some(vec![standing(3, RENAME)]), &log);
    let _ = what_the_terminal_does(
        &a_change_is_waiting(3, RENAME),
        &daemon,
        &the_switch(AloInTheTerminal::ActsWithoutAsking, &log),
    );
    assert_eq!(*log.borrow(), ["waiting", "the switch", "approve 3"]);
}

/// **Another surface's change still waits.** A change the alo Bar, the
/// launcher or a task window is holding is not the one this request left, so
/// it is not answered — not by number, and not by a sentence that matches
/// under another number.
#[test]
fn a_change_another_surface_is_holding_is_never_approved_from_here() {
    for holding in [
        vec![standing(0, "Move april.pdf to Paid")],
        vec![standing(3, "Move april.pdf to Paid")],
        vec![standing(4, RENAME)],
        vec![],
    ] {
        let log = TheLog::default();
        let daemon = ADaemon::holding(Some(holding.clone()), &log);
        let did = what_the_terminal_does(
            &a_change_is_waiting(3, RENAME),
            &daemon,
            &the_switch(AloInTheTerminal::ActsWithoutAsking, &log),
        );
        assert_eq!(did, WhatTheTerminalDid::NotTheChangeWaiting, "{holding:?}");
        assert_eq!(daemon.approved.get(), 0, "approved another surface's change");
    }
}

/// **A daemon that does not say what it holds is answered with nothing.**
#[test]
fn a_daemon_that_says_nothing_is_sent_nothing() {
    let log = TheLog::default();
    let daemon = ADaemon::holding(None, &log);
    let did = what_the_terminal_does(
        &a_change_is_waiting(3, RENAME),
        &daemon,
        &the_switch(AloInTheTerminal::ActsWithoutAsking, &log),
    );
    assert_eq!(did, WhatTheTerminalDid::NotTheChangeWaiting);
    assert_eq!(daemon.approved.get(), 0);
}

/// **A reply that is an answer approves nothing**, and a model that copies the
/// line exactly — as text in a web page could steer it to — approves nothing
/// either. The daemon is not even asked.
#[test]
fn words_approve_nothing_even_words_that_copy_the_line() {
    for reply in [
        "march.pdf and april.pdf".to_owned(),
        never_read_as_a_change(&a_change_is_waiting(3, RENAME)).into_owned(),
    ] {
        let log = TheLog::default();
        let daemon = ADaemon::holding(Some(vec![standing(3, RENAME)]), &log);
        let did = what_the_terminal_does(
            &reply,
            &daemon,
            &the_switch(AloInTheTerminal::ActsWithoutAsking, &log),
        );
        assert_eq!(did, WhatTheTerminalDid::NothingWaited, "{reply:?}");
        assert!(log.borrow().is_empty(), "{reply:?} reached the daemon: {log:?}");
    }
}
