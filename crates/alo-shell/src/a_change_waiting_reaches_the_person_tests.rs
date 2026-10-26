//! A stand-in daemon, and what the road between it and a person does.

use std::cell::RefCell;

use alo_changing::{Answered, Answering};
use alo_changing::{Done, Standing};
use alo_models::InferenceSource;
use alo_tasks::ATask;

use super::{AChangeWaiting, WhatArrived};

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
