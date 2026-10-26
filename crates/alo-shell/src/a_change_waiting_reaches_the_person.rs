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

use crate::approval_keys::ApprovalKey;

/// The change a person is being asked about, the number to answer it by, and
/// which answer their keyboard is on.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AChangeWaiting {
    /// The number the daemon answers by, while one is in front of the person.
    number: Option<u64>,
    /// Which answer is selected, and [`None`] until they move to one.
    ///
    /// **Nothing is preselected.** A question that opens with an answer under
    /// the finger is a question answered by somebody pressing Enter to make a
    /// dialogue go away, which is not an approval of anything.
    selected: Option<WhichAnswer>,
}

/// The two answers, in reading order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhichAnswer {
    /// Allow this one action. **Not a grant, and it widens nothing.**
    AllowOnce,
    /// Say no. Nothing happens and nothing is recorded about why.
    Deny,
}

/// What a press on a waiting question did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatThePressDid {
    /// The selection moved, so the screen is drawn again.
    ItMoved,
    /// An answer was given, and this is what the daemon said — [`None`] when
    /// it said nothing at all.
    ItWasAnswered(Option<Answered>),
    /// Nothing happened, which includes Enter with nothing selected.
    Nothing,
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
        Self {
            number: None,
            // **Nothing preselected**, here as well as after every answer.
            selected: None,
        }
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
        self.selected = None;
        task.allow_once();
        daemon.approve(number)
    }

    /// They said no. Nothing is carried about why, because nothing was asked.
    pub fn deny(&mut self, task: &mut ATask, daemon: &dyn Answering) -> Option<Answered> {
        let number = self.number.take()?;
        self.selected = None;
        task.deny();
        daemon.decline(number)
    }

    /// Which answer their keyboard is on, and [`None`] until they move to one.
    #[must_use]
    pub const fn selected(&self) -> Option<WhichAnswer> {
        self.selected
    }

    /// One key press on the question in front of them.
    ///
    /// **Enter with nothing selected does nothing**, which is *nothing
    /// proceeds on silence* in the one place it can be got wrong: a person who
    /// presses Enter to dismiss something has not answered it.
    ///
    /// **Escape says no** whatever is selected and whether or not anything is,
    /// because leaving a question unanswered is a refusal rather than a maybe.
    pub fn pressed(
        &mut self,
        key: ApprovalKey,
        task: &mut ATask,
        daemon: &dyn Answering,
    ) -> WhatThePressDid {
        if !self.is_asking() {
            return WhatThePressDid::Nothing;
        }
        match key {
            // **Two answers, so next and previous are the same move.** Written
            // as one arm rather than two identical ones: a third answer would
            // make them differ, and then they would be written differently.
            ApprovalKey::NextAnswer | ApprovalKey::PreviousAnswer => {
                self.selected = Some(match self.selected {
                    None | Some(WhichAnswer::Deny) => WhichAnswer::AllowOnce,
                    Some(WhichAnswer::AllowOnce) => WhichAnswer::Deny,
                });
                WhatThePressDid::ItMoved
            }
            ApprovalKey::Choose => match self.selected {
                Some(WhichAnswer::AllowOnce) => {
                    WhatThePressDid::ItWasAnswered(self.allow_once(task, daemon))
                }
                Some(WhichAnswer::Deny) => WhatThePressDid::ItWasAnswered(self.deny(task, daemon)),
                None => WhatThePressDid::Nothing,
            },
            ApprovalKey::Decline => WhatThePressDid::ItWasAnswered(self.deny(task, daemon)),
            ApprovalKey::Nothing => WhatThePressDid::Nothing,
        }
    }
}

impl crate::Server {
    /// What the agent said came back to a task window: put it in the
    /// conversation, and put in front of the person whatever the daemon is
    /// now holding for them.
    ///
    /// **Here and not in the frame loop**, so that a test can prove what it
    /// never does: **it never touches the keyboard.** A question arriving is
    /// not a reason to move it — see `crate::the_tasks_seat`.
    ///
    /// Asked when an answer arrives rather than once a frame: a turn ends by
    /// answering or by leaving a change waiting, so this is the moment one can
    /// exist. A machine never told where its daemon is asks nothing.
    pub(crate) fn an_answer_came_back(
        &mut self,
        which: crate::the_tasks_a_person_opened::WhichTask,
        said: String,
        the_machines_fault: bool,
        daemon: Option<&dyn Answering>,
    ) {
        // **The fields, not the methods**, so the compiler can see the question
        // and the task are different parts of the server.
        let waiting = &mut self.the_change_waiting;
        let Some(task) = self.the_tasks.the_mut(which) else {
            return;
        };
        // The machine failing and the agent answering are told apart, because a
        // person reads them differently: an answer belongs in the conversation
        // and *this machine could not be asked* belongs beside it.
        if the_machines_fault {
            task.and_then(alo_tasks::Act::done(said));
        } else {
            task.alo_says(Turn::from_alo(said));
        }
        if let Some(daemon) = daemon {
            waiting.arrived(task, daemon);
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on an answer that will not read is the failure being reported"
)]
#[path = "a_change_waiting_reaches_the_person_tests.rs"]
mod tests;
