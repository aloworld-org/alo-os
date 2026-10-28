//! A stand-in daemon, and what the road between it and a person does.

use std::cell::RefCell;

use alo_changing::{Answered, Answering};
use alo_changing::{Done, Standing};
use alo_models::InferenceSource;
use alo_tasks::ATask;

use super::{AChangeWaiting, WhatArrived, WhatThePressDid, WhichAnswer};
use crate::approval_keys::ApprovalKey;

/// A daemon that holds what a test tells it to, and remembers what it was
/// told.
#[derive(Debug, Default)]
struct ADaemon {
    /// What it is holding.
    holding: Vec<Standing>,
    /// Every number approved, in order.
    approved: RefCell<Vec<u64>>,
    /// Every number declined, in order.
    declined: RefCell<Vec<u64>>,
    /// Whether it answers at all. `false` is a daemon that is not running.
    answers: bool,
}

impl ADaemon {
    /// A running daemon holding these changes.
    fn holding(changes: Vec<Standing>) -> Self {
        Self {
            holding: changes,
            answers: true,
            ..Self::default()
        }
    }

    /// A daemon nobody is running.
    fn absent() -> Self {
        Self::default()
    }
}

impl Answering for ADaemon {
    fn waiting(&self) -> Option<Vec<Standing>> {
        self.answers.then(|| self.holding.clone())
    }

    fn approve(&self, number: u64) -> Option<Answered> {
        self.answers.then(|| {
            self.approved.borrow_mut().push(number);
            Answered::It(Box::new(Done::Renamed { now_at: None }))
        })
    }

    fn decline(&self, number: u64) -> Option<Answered> {
        self.answers.then(|| {
            self.declined.borrow_mut().push(number);
            Answered::It(Box::new(Done::Renamed { now_at: None }))
        })
    }
}

/// One change the daemon is holding, read from the wire form a daemon sends.
///
/// **Not a constructor only tests call.** `alo_protocol::testing` is private
/// to that crate and `Standing::of` needs the whole approvals machinery, so
/// this reads the bytes instead — which holds these tests to what really
/// crosses the socket. A field renamed on the wire breaks them, and should.
fn standing(number: u64, sentence: &str) -> Standing {
    let wire = format!(
        r#"{{"number":{number},"sentence":{{"text":{sentence:?},"came_from":"the-source"}}}}"#
    );
    serde_json::from_str(&wire)
        .unwrap_or_else(|why| panic!("a waiting change did not read from {wire}: {why}"))
}

/// A task with nothing asked of it.
fn a_task() -> ATask {
    ATask::asked_for("tidy my invoices", InferenceSource::ThisMachine)
}

/// **The question reaches the person**: the sentence is in their conversation
/// and on the task they answer from, and it is the daemon's sentence unchanged.
#[test]
fn a_waiting_change_is_put_in_front_of_the_person() {
    let said = "Rename march.pdf to invoice-march.pdf in /home/anna/Invoices";
    let daemon = ADaemon::holding(vec![standing(7, said)]);
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();

    assert_eq!(waiting.arrived(&mut task, &daemon), WhatArrived::AQuestion);
    assert!(waiting.is_asking());
    assert_eq!(task.permissions().asking(), Some(said));
    assert!(
        task.conversation()
            .each()
            .iter()
            .any(|turn| turn.said() == said && turn.is_waiting_for_the_person()),
        "the sentence is not in the conversation as something waiting"
    );
}

/// **The number answered is the number the daemon gave**, which is the one
/// thing a surface cannot read off the screen.
#[test]
fn allowing_once_answers_the_number_the_daemon_named() {
    let daemon = ADaemon::holding(vec![standing(7, "Rename march.pdf")]);
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();
    waiting.arrived(&mut task, &daemon);

    assert!(waiting.allow_once(&mut task, &daemon).is_some());
    assert_eq!(*daemon.approved.borrow(), [7]);
    assert!(daemon.declined.borrow().is_empty());
    assert!(!waiting.is_asking(), "the question stayed up after a yes");
}

/// Saying no answers the same number, and nothing is carried about why.
#[test]
fn denying_answers_the_same_number() {
    let daemon = ADaemon::holding(vec![standing(3, "Delete everything")]);
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();
    waiting.arrived(&mut task, &daemon);

    assert!(waiting.deny(&mut task, &daemon).is_some());
    assert_eq!(*daemon.declined.borrow(), [3]);
    assert!(daemon.approved.borrow().is_empty());
}

/// **A second answer sends nothing**, because there is nothing in front of the
/// person to answer.
///
/// *One approval is one execution* is the daemon's rule and not this file's —
/// a second answer that did reach it would be refused there. This asserts the
/// weaker and separate thing: a surface that pressed twice does not send
/// twice, so the daemon is not asked to refuse what was never a second answer.
#[test]
fn answering_twice_sends_one_answer() {
    let daemon = ADaemon::holding(vec![standing(7, "Rename march.pdf")]);
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();
    waiting.arrived(&mut task, &daemon);

    assert!(waiting.allow_once(&mut task, &daemon).is_some());
    assert!(waiting.allow_once(&mut task, &daemon).is_none());
    assert!(waiting.deny(&mut task, &daemon).is_none());
    assert_eq!(*daemon.approved.borrow(), [7], "it was approved twice");
}

/// **A question in front of somebody is never replaced.** Asking again while
/// one stands leaves it alone, because a question that changed under a person
/// about to answer it is how somebody approves one thing believing it was
/// another.
#[test]
fn a_question_already_in_front_of_them_is_not_replaced() {
    let first = "Rename march.pdf";
    let daemon = ADaemon::holding(vec![standing(7, first), standing(8, "Delete everything")]);
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();

    assert_eq!(waiting.arrived(&mut task, &daemon), WhatArrived::AQuestion);
    assert_eq!(
        waiting.arrived(&mut task, &daemon),
        WhatArrived::TheOneAlreadyThere
    );
    assert_eq!(task.permissions().asking(), Some(first));

    // And the one answered is the one that was shown.
    waiting.allow_once(&mut task, &daemon);
    assert_eq!(*daemon.approved.borrow(), [7]);
}

/// **One at a time**, which is `ThePermissions`' own rule: a list of pending
/// approvals is a list a person clears by clicking.
#[test]
fn only_the_first_of_several_is_put_in_front_of_them() {
    let daemon = ADaemon::holding(vec![
        standing(1, "Rename march.pdf"),
        standing(2, "Delete everything"),
    ]);
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();
    waiting.arrived(&mut task, &daemon);

    let proposed = task
        .conversation()
        .each()
        .iter()
        .filter(|turn| turn.is_waiting_for_the_person())
        .count();
    assert_eq!(proposed, 1, "more than one question was drawn at once");
}

/// **A daemon nobody is running draws no question**, and is not an answer
/// either: nothing is asked and nothing is sent.
#[test]
fn no_daemon_means_no_question_and_no_answer() {
    let daemon = ADaemon::absent();
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();

    assert_eq!(waiting.arrived(&mut task, &daemon), WhatArrived::Nothing);
    assert!(!waiting.is_asking());
    assert_eq!(task.permissions().asking(), None);
    assert!(waiting.allow_once(&mut task, &daemon).is_none());
}

/// A running daemon holding nothing is the same to a surface as one that said
/// nothing: there is no question to draw.
#[test]
fn a_daemon_holding_nothing_draws_no_question() {
    let daemon = ADaemon::holding(Vec::new());
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();

    assert_eq!(waiting.arrived(&mut task, &daemon), WhatArrived::Nothing);
    assert!(!waiting.is_asking());
}

/// **Nothing is preselected**, and Enter on a question nobody has moved on
/// answers nothing at all.
///
/// A person who presses Enter to make something go away has not approved
/// anything. A surface that opened with *Allow once* under the finger would
/// pass every other test in this file and would be wrong in the one way that
/// matters.
#[test]
fn enter_with_nothing_selected_answers_nothing() {
    let daemon = ADaemon::holding(vec![standing(7, "Delete everything")]);
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();
    waiting.arrived(&mut task, &daemon);

    assert_eq!(waiting.selected(), None, "an answer was preselected");
    assert_eq!(
        waiting.pressed(ApprovalKey::Choose, &mut task, &daemon),
        WhatThePressDid::Nothing
    );
    assert!(daemon.approved.borrow().is_empty(), "Enter approved it");
    assert!(daemon.declined.borrow().is_empty(), "Enter declined it");
    assert!(
        waiting.is_asking(),
        "the question was taken down unanswered"
    );
}

/// **Both answers are reachable, and Tab reaches them in reading order.**
#[test]
fn tab_moves_between_the_two_answers() {
    let daemon = ADaemon::holding(vec![standing(7, "Rename march.pdf")]);
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();
    waiting.arrived(&mut task, &daemon);

    assert_eq!(
        waiting.pressed(ApprovalKey::NextAnswer, &mut task, &daemon),
        WhatThePressDid::ItMoved
    );
    assert_eq!(waiting.selected(), Some(WhichAnswer::AllowOnce));

    waiting.pressed(ApprovalKey::NextAnswer, &mut task, &daemon);
    assert_eq!(waiting.selected(), Some(WhichAnswer::Deny));

    // And back, because with two answers next and previous are one move.
    waiting.pressed(ApprovalKey::PreviousAnswer, &mut task, &daemon);
    assert_eq!(waiting.selected(), Some(WhichAnswer::AllowOnce));
}

/// Choosing the selected answer sends it, by the number the daemon gave.
#[test]
fn choosing_an_answer_sends_that_answer() {
    let daemon = ADaemon::holding(vec![standing(7, "Rename march.pdf")]);
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();
    waiting.arrived(&mut task, &daemon);

    waiting.pressed(ApprovalKey::NextAnswer, &mut task, &daemon);
    let did = waiting.pressed(ApprovalKey::Choose, &mut task, &daemon);
    assert!(matches!(did, WhatThePressDid::ItWasAnswered(Some(_))));
    assert_eq!(*daemon.approved.borrow(), [7]);
    assert!(!waiting.is_asking());
    assert_eq!(
        waiting.selected(),
        None,
        "a selection outlived its question"
    );
}

/// **Escape says no**, whatever is selected and whether or not anything is.
///
/// Leaving a question a person did not answer is a refusal rather than a
/// maybe, which is what `alo_access::leaving` says Escape means here.
#[test]
fn escape_declines_whatever_is_selected() {
    let daemon = ADaemon::holding(vec![standing(4, "Delete everything")]);
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();
    waiting.arrived(&mut task, &daemon);

    // Nothing selected, and it still says no.
    let did = waiting.pressed(ApprovalKey::Decline, &mut task, &daemon);
    assert!(matches!(did, WhatThePressDid::ItWasAnswered(Some(_))));
    assert_eq!(*daemon.declined.borrow(), [4]);
    assert!(daemon.approved.borrow().is_empty());
}

/// A press with no question in front of the person does nothing, which is a
/// window somebody left open and came back to.
#[test]
fn a_press_with_nothing_waiting_does_nothing() {
    let daemon = ADaemon::holding(Vec::new());
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();

    for key in [
        ApprovalKey::NextAnswer,
        ApprovalKey::Choose,
        ApprovalKey::Decline,
    ] {
        assert_eq!(
            waiting.pressed(key, &mut task, &daemon),
            WhatThePressDid::Nothing
        );
    }
    assert!(daemon.approved.borrow().is_empty());
    assert!(daemon.declined.borrow().is_empty());
}

/// **The agent's announcement of a waiting change is not said twice.** Found
/// walking a booted machine on 2026-10-28: the conversation read *a change is
/// waiting for you to answer, as number 0:* and the sentence, directly above
/// the card that already showed the sentence. The control is words of alo's
/// own, which are kept before the card.
#[test]
fn the_agents_announcement_is_not_said_above_the_card() {
    let sentence = "rename /home/alo/Documents/march-invoice.pdf to invoice-march.pdf";
    for (said, kept) in [
        (
            format!("a change is waiting for you to answer, as number 0:\n  {sentence}\n"),
            false,
        ),
        ("I can rename that for you.".to_owned(), true),
    ] {
        let daemon = ADaemon::holding(vec![standing(0, sentence)]);
        let mut task = a_task();
        let mut waiting = AChangeWaiting::nothing_yet();

        assert_eq!(
            waiting.arrived_after(Some(said.clone()), &mut task, &daemon),
            WhatArrived::AQuestion
        );

        let turns: Vec<&str> = task
            .conversation()
            .each()
            .iter()
            .map(alo_tasks::Turn::said)
            .collect();
        let expected: Vec<&str> = if kept {
            vec!["tidy my invoices", said.as_str(), sentence]
        } else {
            vec!["tidy my invoices", sentence]
        };
        assert_eq!(turns, expected, "for {said:?}");
        assert!(
            !turns.iter().any(|turn| turn.contains("as number")),
            "the daemon's number reached the conversation"
        );
    }
}

/// **With nothing waiting, the agent's words are all there is**, and are kept
/// whatever they say.
#[test]
fn with_nothing_waiting_the_agents_words_are_kept() {
    let daemon = ADaemon::holding(Vec::new());
    let mut task = a_task();
    let mut waiting = AChangeWaiting::nothing_yet();

    assert_eq!(
        waiting.arrived_after(Some("It is in Invoices.".to_owned()), &mut task, &daemon),
        WhatArrived::Nothing
    );
    assert_eq!(
        task.conversation().each().last().map(alo_tasks::Turn::said),
        Some("It is in Invoices.")
    );
}
