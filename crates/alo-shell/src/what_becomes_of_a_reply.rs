//! What becomes of a reply a person sends from a task window's reply box.
//!
//! # Today: it is not sent, and the window says so
//!
//! Walking a booted machine on 2026-10-28, a person replied in a finished task
//! window and pressed Enter. The window switched to *alo is working* and stayed
//! there: the reply had been added to the conversation and then dropped, and
//! the journal showed no `alo-asking@` unit — nothing was ever asked. **A window
//! that says alo is working when nothing was asked is the one claim this
//! surface must never make.**
//!
//! No road carries a reply to the agent. `alo-agent` takes one request per turn
//! and keeps nothing between turns, so sending the reply's words as if they
//! were a new request would hand a model *"yes, the second one"* with no first
//! one — and the person, reading a conversation, would believe alo had it.
//!
//! So the reply stays in the conversation where the person wrote it, and the
//! line beneath it says it did not reach alo and where to ask instead. That
//! ends the conversation on something alo's side said, so the task reads as
//! what it is — finished, or still waiting for an answer on its card — and
//! never as working.
//!
//! # What this does not decide
//!
//! **Whether a reply continues the conversation with what came before it.**
//! `docs/features.md` promises *follow-ups stay in that window*, and the
//! design's interaction contract says the conversation *accepts user replies
//! while work runs* and explains *whether a reply is queued or redirects at the
//! next safe step*. Neither says whether alo is given the earlier turns. That
//! is a change to what the agent is sent — and, for a provider, to how much of
//! a conversation leaves the machine on each reply — so it is the owner's to
//! decide, and this file is where the answer lands.

use alo_strings::{Filling, Strings};
use alo_tasks::Turn;

use crate::the_reply_boxs_words::A_REPLY_DOES_NOT_REACH_ALO_YET;
use crate::the_tasks_a_person_opened::WhichTask;

impl crate::Server {
    /// A person sent this reply from the task window `which`.
    ///
    /// The reply is already in the conversation and out of the box —
    /// `crate::a_key_reaches_a_task_window` did both. This says, beneath it,
    /// that it did not reach alo.
    pub(crate) fn a_reply_was_sent(&mut self, which: WhichTask, strings: &Strings) {
        let Some(task) = self.tasks_mut().the_mut(which) else {
            return;
        };
        task.alo_says(Turn::from_alo(
            strings
                .say(&A_REPLY_DOES_NOT_REACH_ALO_YET.key(), &Filling::nothing())
                .text()
                .to_owned(),
        ));
    }
}
