//! What becomes of a reply a person sends from a task window's reply box.
//!
//! # It is sent to alo, with the conversation so far
//!
//! The owner decided it on 2026-10-28
//! ([ADR 0136](../../../docs/decisions/0136-a-follow-up-is-sent-with-the-conversation-so-far.md)):
//! *and the April one too* reaches alo together with what was said before it
//! in that window, so alo can tell what it refers to. Until then the reply
//! stayed in the window and the line beneath it said alo had not received it
//! (#982), because a reply sent alone would have handed a model *"yes, the
//! second one"* with no first one.
//!
//! The conversation is built **at the press of Enter** and handed to the frame
//! loop, which owns the door to the agent — the same shape a request from the
//! Dock takes (`Server::take_what_was_asked_of_alo`). Nothing reads a task's
//! conversation at any other moment.
//!
//! # When it is not sent, the window says so
//!
//! Two cases, and neither may read as *alo is working*, because nothing was
//! asked:
//!
//! - **A change above is still waiting.** The agent that proposed it holds its
//!   turn until the person answers, and the daemon serves one turn at a time,
//!   so a reply sent now would be turned away by the machine. The person is
//!   told to answer the change first.
//! - **alo has not answered yet.** The same: the earlier turn is still running.
//!
//! And when a long conversation was sent in part, the activity says so — the
//! agent tells the model, and the person is told here.

use alo_instructing::TheConversationSoFar;
use alo_strings::{Filling, Strings};
use alo_tasks::{Act, Turn, WhoSaidIt};

use crate::the_reply_boxs_words::{
    A_REPLY_WAITS_FOR_ALO, A_REPLY_WAITS_FOR_THE_CHANGE, PART_OF_THE_CONVERSATION_WAS_NOT_SENT,
};
use crate::the_tasks_a_person_opened::WhichTask;
use crate::what_a_follow_up_carries::the_conversation_before_the_reply;

/// A follow-up waiting for the frame loop to hand it to the agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AFollowUp {
    /// The window it was typed in, where the answer goes.
    pub which: WhichTask,
    /// What the person typed.
    pub said: String,
    /// What was said before it, held to its bound.
    pub earlier: TheConversationSoFar,
}

impl crate::Server {
    /// A person sent `said` from the task window `which`.
    ///
    /// The reply is already in the conversation and out of the box —
    /// `crate::a_key_reaches_a_task_window` did both. This sends it on with the
    /// conversation before it, or says beneath it why it was not sent.
    pub(crate) fn a_reply_was_sent(&mut self, which: WhichTask, said: String, strings: &Strings) {
        let Some(task) = self.tasks_mut().the_mut(which) else {
            return;
        };
        let not_sent = |task: &mut alo_tasks::ATask, word: &alo_strings::Word| {
            task.alo_says(Turn::from_alo(
                strings
                    .say(&word.key(), &Filling::nothing())
                    .text()
                    .to_owned(),
            ));
        };
        if task.permissions().asking().is_some() {
            not_sent(task, &A_REPLY_WAITS_FOR_THE_CHANGE);
            return;
        }
        // The turn before the reply: the person's own means alo had not
        // answered it yet.
        let turns = task.conversation().each();
        let alo_had_not_answered = turns
            .len()
            .checked_sub(2)
            .and_then(|before| turns.get(before))
            .is_some_and(|turn| matches!(turn.who(), WhoSaidIt::ThePerson));
        if alo_had_not_answered {
            not_sent(task, &A_REPLY_WAITS_FOR_ALO);
            return;
        }
        let earlier = the_conversation_before_the_reply(task);
        if earlier.is_shortened() {
            task.and_then(Act::done(
                strings
                    .say(
                        &PART_OF_THE_CONVERSATION_WAS_NOT_SENT.key(),
                        &Filling::nothing(),
                    )
                    .text()
                    .to_owned(),
            ));
        }
        self.follow_ups.push(AFollowUp {
            which,
            said,
            earlier,
        });
    }

    /// Take the follow-ups waiting to be handed to the agent.
    ///
    /// **Taken rather than read**, so one Enter asks once however many frames
    /// pass — `Server::take_what_was_asked_of_alo`'s rule, for a reply.
    pub fn take_the_follow_ups(&mut self) -> Vec<AFollowUp> {
        std::mem::take(&mut self.follow_ups)
    }
}
