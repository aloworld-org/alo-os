//! The road from a change the daemon is holding to the person who must answer
//! it.
//!
//! ADR 0001 §5: *any change to the machine is proposed with a sentence
//! describing it and waits for one approval. What a person approves is that
//! sentence.* Everything that sentence needs existed before this file and none
//! of it was joined: `alo_approving` held the rule, `alo_tasks::ThePermissions`
//! held the question, `alo_protocol` carried it both ways, the daemon honoured
//! it, and `alo_changing::Answering` asks. **Nothing put the question in front
//! of a person.**
//!
//! Walking a booted machine on 2026-10-25 it said *a change is waiting for you
//! to answer, as number 0* and there was nothing on the machine to answer with
//! — `docs/walks/2026-10-25-the-first-walk.md`.
//!
//! # What it holds, and why it is one and not a list
//!
//! **One change in front of one person at a time.**
//! `alo_tasks::ThePermissions` says why in its own words: *a list of pending
//! approvals is a list a person clears by clicking, and clicking through is
//! how somebody approves the fourth one without reading it.* So this keeps the
//! first change the daemon is holding and leaves the rest where they are. They
//! are not lost — the daemon still has them, and the next ask brings the next
//! one.
//!
//! # The number is here and the sentence is in the task
//!
//! A person approves the **sentence**, so the sentence lives on the task and
//! is drawn in their conversation. The **number** is how the daemon is told,
//! and it is nobody's but this file's: a task that carried it would be a model
//! holding a protocol handle, and a surface that read it off the screen would
//! be answering something other than what it showed.
//!
//! # What it does not do
//!
//! **It does not keep *one approval is one execution*.** That is
//! `alo_capability::Approved::redeem`'s, and a second answer to the same
//! number is refused there. This can send a second answer and will be told no,
//! which is correct: the daemon is the one that counts, and a guard here would
//! be a second place to get it wrong.
//!
//! **It words nothing.** The sentence arrives already worded by whoever
//! proposed the change, and a refusal arrives already worded by the daemon.
//! Both are passed through untouched.

use alo_changing::{Answered, Answering};
use alo_tasks::{ATask, Turn};

/// The change a person is being asked about, and the number to answer it by.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AChangeWaiting {
    /// The number the daemon answers by, while one is in front of the person.
    number: Option<u64>,
}

/// What putting a question in front of a person did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatArrived {
    /// A change is now in front of them, drawn in their conversation.
    AQuestion,
    /// The daemon is holding nothing, or said nothing at all. A surface does
    /// the same for both: there is no question to draw.
    Nothing,
    /// A question was already in front of them and stays there.
    ///
    /// **Not replaced.** A question that changed under somebody about to
    /// answer it is how a person approves one thing believing it was another.
    TheOneAlreadyThere,
}

impl AChangeWaiting {
    /// Nothing is being asked.
    #[must_use]
    pub const fn nothing_yet() -> Self {
        Self { number: None }
    }

    /// Whether a change is in front of the person now.
    #[must_use]
    pub const fn is_asking(&self) -> bool {
        self.number.is_some()
    }

    /// Ask the daemon what it is holding, and put the first one in front of
    /// this person.
    ///
    /// The sentence goes into the conversation as
    /// [`alo_tasks::Turn::alo_proposing`] **and** onto the task's permissions,
    /// which are the two halves of one thing: what they read, and what they
    /// answer.
    pub fn arrived(&mut self, task: &mut ATask, daemon: &dyn Answering) -> WhatArrived {
        if self.number.is_some() {
            return WhatArrived::TheOneAlreadyThere;
        }
        let Some(changes) = daemon.waiting() else {
            return WhatArrived::Nothing;
        };
        let Some(first) = changes.first() else {
            return WhatArrived::Nothing;
        };
        let sentence = first.sentence().text().to_owned();
        self.number = Some(first.number());
        task.is_asking(sentence.clone());
        task.alo_says(Turn::alo_proposing(sentence));
        WhatArrived::AQuestion
    }

    /// They allowed the one action being asked about.
    ///
    /// Answers [`None`] when nothing was being asked or no daemon answered,
    /// which a surface treats the same way: nothing to show.
    pub fn allow_once(&mut self, task: &mut ATask, daemon: &dyn Answering) -> Option<Answered> {
        let number = self.number.take()?;
        task.allow_once();
        daemon.approve(number)
    }

    /// They said no. Nothing is carried about why, because nothing was asked.
    pub fn deny(&mut self, task: &mut ATask, daemon: &dyn Answering) -> Option<Answered> {
        let number = self.number.take()?;
        task.deny();
        daemon.decline(number)
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on an answer that will not read is the failure being reported"
)]
#[path = "a_change_waiting_reaches_the_person_tests.rs"]
mod tests;
