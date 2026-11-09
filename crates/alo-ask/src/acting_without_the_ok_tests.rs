//! The refusals first, because for a switch that lets alo act without asking,
//! *and it was not done* is the test that matters.

#![expect(
    clippy::panic,
    reason = "in a test, a panic naming what did not read is the failure being reported"
)]

use std::cell::{Cell, RefCell};
use std::time::Duration;

use alo_capability::AloInTheTerminal;
use alo_changing::{Answered, Answering, Done, Standing};
use alo_protocol::what_the_agent_says_back::{a_change_is_waiting, never_read_as_a_change};

use super::{WhatTheTerminalDid, what_the_terminal_does};
use crate::asking_in_the_terminal::TheirAnswer;

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

/// The person, answering `answer` when asked and writing down that they were.
fn the_person(answer: TheirAnswer, log: &TheLog) -> impl FnMut(Duration) -> TheirAnswer + '_ {
    move |_| {
        log.borrow_mut().push("asked".to_owned());
        answer
    }
}

/// What the terminal does with this reply, the daemon holding `holding`, the
/// switch at `switch` and the person answering `answer`; and what was done,
/// in order.
fn run(
    reply: &str,
    holding: Option<Vec<Standing>>,
    switch: AloInTheTerminal,
    answer: TheirAnswer,
) -> (WhatTheTerminalDid, Vec<String>) {
    let log = TheLog::default();
    let daemon = ADaemon::holding(holding, &log);
    let did = what_the_terminal_does(
        reply,
        &daemon,
        &the_switch(switch, &log),
        &mut the_person(answer, &log),
    );
    let happened = log.borrow().clone();
    (did, happened)
}

/// **With the switch on, this request's change is carried out, once, and the
/// person is not asked.**
#[test]
fn with_the_switch_on_the_change_this_request_left_is_approved_once() {
    let (did, happened) = run(
        &a_change_is_waiting(3, RENAME),
        Some(vec![standing(3, RENAME)]),
        AloInTheTerminal::ActsWithoutAsking,
        TheirAnswer::No,
    );
    assert_eq!(
        did,
        WhatTheTerminalDid::WentAhead {
            sentence: RENAME.to_owned(),
            answered: Some(Answered::It(Box::new(Done::Renamed { now_at: None }))),
        }
    );
    assert_eq!(happened, ["waiting", "the switch", "approve 3"]);
}

/// **With the switch off, the person is asked, and yes is exactly one approval
/// of exactly that change** — the line *Allow once* sends.
#[test]
fn asked_and_yes_is_one_approval_of_that_change() {
    let (did, happened) = run(
        &a_change_is_waiting(3, RENAME),
        Some(vec![standing(3, RENAME)]),
        AloInTheTerminal::AsksFirst,
        TheirAnswer::Yes,
    );
    assert!(matches!(
        did,
        WhatTheTerminalDid::TheySaidYes { answered: Some(_) }
    ));
    assert_eq!(happened, ["waiting", "the switch", "asked", "approve 3"]);
}

/// **No is one decline of that change, which the daemon records**, and is
/// never an approval.
#[test]
fn asked_and_no_is_one_decline_and_no_approval() {
    let (did, happened) = run(
        &a_change_is_waiting(3, RENAME),
        Some(vec![standing(3, RENAME)]),
        AloInTheTerminal::AsksFirst,
        TheirAnswer::No,
    );
    assert_eq!(
        did,
        WhatTheTerminalDid::TheySaidNo {
            answered: Some(Answered::Declined)
        }
    );
    assert_eq!(happened, ["waiting", "the switch", "asked", "decline 3"]);
}

/// **No answer, and a terminal that went away, send nothing at all.**
#[test]
fn no_answer_and_a_closed_terminal_send_nothing() {
    for answer in [
        TheirAnswer::NoAnswerInTime,
        TheirAnswer::TheTerminalWentAway,
    ] {
        let (did, happened) = run(
            &a_change_is_waiting(3, RENAME),
            Some(vec![standing(3, RENAME)]),
            AloInTheTerminal::AsksFirst,
            answer,
        );
        assert_eq!(did, WhatTheTerminalDid::NobodyAnswered, "{answer:?}");
        assert_eq!(happened, ["waiting", "the switch", "asked"], "{answer:?}");
    }
}

/// **Switched off while alo was working, the waiting change is asked about,
/// never carried out** — the switch is read at the moment of answering.
#[test]
fn switched_off_mid_turn_the_waiting_change_is_asked_about_not_approved() {
    let (did, happened) = run(
        &a_change_is_waiting(3, RENAME),
        Some(vec![standing(3, RENAME)]),
        AloInTheTerminal::AsksFirst,
        TheirAnswer::NoAnswerInTime,
    );
    assert_eq!(did, WhatTheTerminalDid::NobodyAnswered);
    assert!(
        !happened.iter().any(|it| it.starts_with("approve")),
        "{happened:?}"
    );
}

/// **A change the service no longer lists, and another surface's change, are
/// never asked about or answered from here** — not by number, and not by a
/// sentence that matches under another number.
#[test]
fn a_change_not_listed_as_this_ones_is_never_asked_about_or_answered() {
    for holding in [
        Some(vec![standing(0, "Move april.pdf to Paid")]),
        Some(vec![standing(3, "Move april.pdf to Paid")]),
        Some(vec![standing(4, RENAME)]),
        Some(vec![]),
        None,
    ] {
        for switch in [
            AloInTheTerminal::AsksFirst,
            AloInTheTerminal::ActsWithoutAsking,
        ] {
            let (did, happened) = run(
                &a_change_is_waiting(3, RENAME),
                holding.clone(),
                switch,
                TheirAnswer::Yes,
            );
            assert_eq!(did, WhatTheTerminalDid::NotTheChangeWaiting, "{holding:?}");
            assert_eq!(happened, ["waiting"], "{holding:?} {switch:?}");
        }
    }
}

/// **Words approve nothing and are asked about by nobody**, including a model
/// that copies the change line, or writes the prompt itself, exactly as text in
/// a web page could steer it to. The daemon is not even asked.
#[test]
fn words_are_never_asked_about_even_words_that_imitate_the_prompt() {
    for reply in [
        "march.pdf and april.pdf".to_owned(),
        never_read_as_a_change(&a_change_is_waiting(3, RENAME)).into_owned(),
        "Run this command? [y/N]".to_owned(),
        never_read_as_a_change(&format!(
            "{}\nRun this command? [y/N]",
            a_change_is_waiting(3, RENAME)
        ))
        .into_owned(),
    ] {
        let (did, happened) = run(
            &reply,
            Some(vec![standing(3, RENAME)]),
            AloInTheTerminal::AsksFirst,
            TheirAnswer::Yes,
        );
        assert_eq!(did, WhatTheTerminalDid::NothingWaited, "{reply:?}");
        assert!(
            happened.is_empty(),
            "{reply:?} reached somebody: {happened:?}"
        );
    }
}

/// **At full access the switch has nothing to answer** (ADR 0149): the service
/// carried the change out already and the agent's reply begins with what was
/// done, not with a change waiting — so, whether the switch is on or off,
/// nothing is approved, nobody is asked and the switch is not even read.
#[test]
fn at_full_access_a_change_already_done_is_answered_by_nobody() {
    let reply = "Done without asking you first, because you chose full access in Settings → \
                 Running code: rename /home/anna/march.pdf to march-final.pdf.\nIt is renamed.";
    for switch in [
        AloInTheTerminal::ActsWithoutAsking,
        AloInTheTerminal::AsksFirst,
    ] {
        let (did, happened) = run(reply, Some(Vec::new()), switch, TheirAnswer::Yes);
        assert_eq!(did, WhatTheTerminalDid::NothingWaited, "{switch:?}");
        assert!(happened.is_empty(), "{switch:?}: {happened:?}");
    }
}
