//! What becomes of a reply a person sends from a task window's reply box.
//!
//! # It is sent to alo, with the conversation so far
//!
//! The owner decided it on 2026-10-28
//! ([ADR 0138](../../../docs/decisions/0138-a-follow-up-is-sent-with-the-conversation-so-far.md)):
//! *and the April one too* reaches alo together with everything said before it
//! in that window, so alo can tell what it refers to. Until then the reply
//! stayed in the window and the line beneath it said alo had not received it
//! (#982), because a reply sent alone would have handed a model *"yes, the
//! second one"* with no first one.
//!
//! The conversation is read **at the moment the reply is sent** and handed to
//! the frame loop, which owns the door to the agent — the same shape a request
//! from the Dock takes (`Server::take_what_was_asked_of_alo`). Nothing reads a
//! task's conversation at any other moment.
//!
//! # A reply alo cannot take yet is held, not refused
//!
//! While alo is still answering, or while a change above waits for Allow once
//! or Deny, the reply is **held**: it shows under the conversation marked as
//! waiting to be sent, and goes by itself — with the conversation as it then
//! stands — once alo's answer has arrived and nothing waits for the person.
//! Backspace in the empty box gives it back to change or delete
//! (`crate::a_key_reaches_a_task_window`). It cannot go sooner because the
//! daemon serves one turn at a time, and the turn before it is still open.
//!
//! # Too long to send is said, never shortened
//!
//! alo leaves nothing out of a conversation. One too long for a single question
//! to carry is not sent, and the window says so and where to continue.

use alo_strings::{Filling, Strings};
use alo_tasks::Turn;

use crate::the_reply_boxs_words::TOO_LONG_TO_SEND;
use crate::the_tasks_a_person_opened::WhichTask;
use crate::what_a_follow_up_carries::the_conversation_so_far;

/// A follow-up waiting for the frame loop to hand it to the agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AFollowUp {
    /// The window it was typed in, where the answer goes.
    pub which: WhichTask,
    /// What the person typed.
    pub said: String,
    /// What was said before it, whole.
    pub earlier: alo_instructing::TheConversationSoFar,
}

impl crate::Server {
    /// A person sent `said` from the task window `which`: send it now, or hold
    /// it until it can go.
    pub(crate) fn a_reply_was_sent(&mut self, which: WhichTask, said: String, strings: &Strings) {
        let Some(task) = self.tasks_mut().the_mut(which) else {
            return;
        };
        let alo_is_still_answering = task.conversation().is_alos_turn();
        let a_change_waits = task.permissions().asking().is_some();
        if alo_is_still_answering || a_change_waits || task.held().is_some() {
            task.hold(said);
            return;
        }
        self.send_the_reply(which, said, strings);
    }

    /// **Send what was held, if it can go now**: alo has answered and nothing
    /// waits for the person.
    ///
    /// Asked whenever either of those can have become true — an answer
    /// arriving, and the person answering a change — and a no-op otherwise.
    pub(crate) fn send_what_was_held(&mut self, which: WhichTask, strings: &Strings) {
        let Some(task) = self.tasks_mut().the_mut(which) else {
            return;
        };
        if task.conversation().is_alos_turn() || task.permissions().asking().is_some() {
            return;
        }
        if let Some(held) = task.take_held() {
            self.send_the_reply(which, held, strings);
        }
    }

    /// The reply joins the conversation and goes to alo with everything before
    /// it — or, too long for one question, joins it with a sentence saying it
    /// was not sent.
    fn send_the_reply(&mut self, which: WhichTask, said: String, strings: &Strings) {
        let Some(task) = self.tasks_mut().the_mut(which) else {
            return;
        };
        let earlier = the_conversation_so_far(task);
        task.the_person_says(said.clone());
        if !earlier.fits_in_a_question(&said) {
            task.alo_says(Turn::from_alo(
                strings
                    .say(&TOO_LONG_TO_SEND.key(), &Filling::nothing())
                    .text()
                    .to_owned(),
            ));
            return;
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
