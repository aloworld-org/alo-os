//! The one control at the end of the composer, and which action it is.
//!
//! **The design draws one button, never two.** *One shared 48px circular
//! composer action: Send arrow when idle; solid Stop square while alo is
//! working. Never display separate Send and Stop controls* — the interaction
//! contract, frame `418:28202`, read on 2026-10-12 and recorded in
//! `docs/design/the-ask-alo-task-window.md`.
//!
//! # Why this is one function and not a condition at each drawing site
//!
//! Four surfaces need this answer: the window that draws the button, the
//! accessibility tree that must name it, the keyboard that must refuse to let
//! Enter reach Stop, and the panel a minimised task offers Stop from. A
//! condition written out four times is four chances to disagree, and the
//! disagreement that matters here is not cosmetic — a button that says Send
//! while the runtime is still working, or that stops a task somebody meant to
//! reply to, is a wrong action taken on a person's machine.
//!
//! So the state is derived once, here, from the task, and every surface asks.
//! This is the same reason [`crate::TheConversation::is_waiting_for_the_person`]
//! exists rather than each reader checking the last turn.
//!
//! # What it deliberately does not carry
//!
//! **No words.** The contract says *accessible names and tooltips follow the
//! current action*, and the words for Send, Stop and Stopping are
//! `alo-saying`'s like every other user-facing string. What is here is which of
//! the four states a task is in; what a person reads is decided where every
//! other sentence is.

use crate::ATask;

/// Which action the composer's one button offers, and whether it can be
/// pressed.
///
/// **Four states, not a pair of booleans.** `(is_stop, enabled)` has the same
/// cardinality and permits `(false, false)` to mean two different things — a
/// Send with nothing typed and a Stop already asked for — which are drawn
/// differently and named differently to a screen reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TheOneButton {
    /// **Send**, and there is something to send.
    Send,
    /// **Send**, greyed: the draft is empty.
    ///
    /// *Disable empty-draft Send* — the contract. Drawn as Send rather than as
    /// nothing, because a slot that empties when a person deletes their text
    /// moves every control beside it.
    NothingToSend,
    /// **Stop**: alo is working, or waiting for an answer.
    Stop,
    /// **Stop**, greyed: a stop was asked for and the runtime has not confirmed
    /// it.
    ///
    /// *Show Stopping until acknowledged, then return to Send*, and *disable
    /// repeated cancellation* — the contract. **This is the state a boolean
    /// cannot hold**, and the reason [`crate::Stopping`] is not one: a window
    /// that went straight from Stop to Send would claim the work had ended at
    /// the moment somebody asked it to, which is the one thing a Stop must
    /// never say. Law 3 and the owner's own wording: a Stop that reported
    /// success while something was still running would be worse than no Stop.
    Stopping,
}

impl TheOneButton {
    /// Whether this is the stopping action, in either of its two states.
    ///
    /// **What the keyboard asks.** Enter sends; it must never reach Stop, so
    /// the key handler needs to know what the button currently is without
    /// caring whether it happens to be pressable.
    #[must_use]
    pub const fn is_stop(self) -> bool {
        matches!(self, Self::Stop | Self::Stopping)
    }

    /// Whether a person can press it at all.
    #[must_use]
    pub const fn can_be_pressed(self) -> bool {
        matches!(self, Self::Send | Self::Stop)
    }
}

impl ATask {
    /// Which action the composer's one button is offering.
    ///
    /// The order of these tests is the whole of the rule, and each is here
    /// because the design or the contract puts it here:
    ///
    /// 1. **A stop already asked for wins over everything.** While stopping,
    ///    the work is still running, so every later test would still say
    ///    *Stop* — and it would say it as a pressable button, which is the
    ///    repeated cancellation the contract forbids.
    /// 2. **A confirmed stop ends the work, whatever the activity still says.**
    ///    The runtime's confirmation is the authority on whether anything is
    ///    running; the activity is a log of what happened. They can disagree,
    ///    and this is not hypothetical — it is what the test
    ///    `a_confirmed_stop_returns_to_send_with_the_draft_kept` caught: an act
    ///    left reading *doing* after the work it describes had ended kept the
    ///    button on Stop for a task nothing was running in. A person would
    ///    have been offered a second cancellation of work already over, with
    ///    no way to reply.
    /// 3. **Working, or waiting for the person, is Stop.** The design draws
    ///    Stop in five variants: Working, Model picker, Permissions, Dark —
    ///    and **Needs permission**, which settles a question the written
    ///    requirement left open. A task waiting for an approval is a task alo
    ///    is in the middle of, so the control is Stop.
    /// 4. **Nothing typed is a greyed Send.**
    /// 5. **Otherwise Send.** Which is the design's Stopped and Complete.
    #[must_use]
    pub fn the_one_button(&self) -> TheOneButton {
        if matches!(self.stopping(), crate::Stopping::Asked) {
            return TheOneButton::Stopping;
        }
        let it_stopped = matches!(self.stopping(), crate::Stopping::Stopped { .. });
        let working = self.activity().doing_now().is_some()
            || self.conversation().is_alos_turn()
            || self.is_waiting_for_the_person();
        if working && !it_stopped {
            return TheOneButton::Stop;
        }
        if self.the_draft().is_empty() {
            return TheOneButton::NothingToSend;
        }
        TheOneButton::Send
    }

    /// Whether pressing Enter now sends the draft.
    ///
    /// **Separate from the button on purpose.** *Enter sends; Shift+Enter adds
    /// a line*, and *conversation accepts user replies while work runs* — so a
    /// person may type and send a reply while the button shows Stop, and Enter
    /// must not become a cancel key because the slot beside it changed. The
    /// only thing that stops Enter is having nothing to send.
    #[must_use]
    pub fn enter_sends_the_draft(&self) -> bool {
        !self.the_draft().is_empty()
    }
}

#[cfg(test)]
#[path = "the_one_button_tests.rs"]
mod tests;
