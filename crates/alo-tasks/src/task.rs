//! One task: the six areas, and what a minimised one shows in the panel.
//!
//! *Each new task gets its own window; follow-ups stay in that window.
//! Minimising it keeps the task running and puts its preview, activity and Stop
//! control in the right-hand panel* — the owner, 2026-10-12.
//!
//! # Minimising keeps it running, and this type is why that is not a promise
//!
//! Nothing here stops when a window is put aside, because nothing here knows
//! what a window is. A task is a conversation, an activity, a model, a set of
//! permissions and a stopping state; the shell draws it and the panel shows a
//! preview of it. **The only thing that ends a task is [`Stopping`] reaching
//! stopped**, and a person minimising something never touches it.
//!
//! That is the difference between a design that says *minimising keeps it
//! running* and one where it is true: here there is no code path from *put this
//! window aside* to *stop this work*, because the two live in different crates
//! and only one of them can stop anything.

use alo_models::InferenceSource;

use crate::activity::{Act, TheActivity};
use crate::conversation::{TheConversation, Turn};
use crate::permissions::{Answered, ThePermissions};
use crate::stopping::Stopping;
use crate::which_model::WhichModel;

/// One task, as a person meets it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ATask {
    /// What was said, in order.
    conversation: TheConversation,
    /// What it did and is doing.
    activity: TheActivity,
    /// Which model answers, and which will answer next.
    model: WhichModel,
    /// What it may reach, and what it is asking for.
    permissions: ThePermissions,
    /// Whether anybody has asked it to stop.
    stopping: Stopping,
    /// What the person has typed in the reply box and not yet sent.
    ///
    /// **Held by the task, not by the window**, for two reasons the contract
    /// gives directly. *Preserve the draft* across a Stop: nothing here clears
    /// it, so it survives by construction rather than by a window remembering
    /// to carry it. And *minimising preserves the task* — a draft kept by a
    /// window would be lost the moment somebody put the task aside and brought
    /// it back, which is the one thing a person does while waiting for work to
    /// finish.
    draft: String,
}

/// What a minimised task shows in the right-hand panel.
///
/// **Three things and no more**: the request it is working on, what it is doing
/// now, and whether it is waiting for the person. A panel preview that carried
/// the conversation would be a second window in a column six icons wide.
///
/// Stop is a control rather than a field: the panel offers it, and pressing it
/// is [`ATask::stop`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InThePanel {
    /// What the person asked for.
    pub request: String,
    /// What it is doing now, or [`None`] when it is doing nothing.
    pub doing: Option<String>,
    /// Whether it is waiting for the person to answer something.
    pub waiting_for_you: bool,
    /// Whether anything has gone wrong in it.
    pub something_went_wrong: bool,
}

impl ATask {
    /// A task somebody has just asked for.
    #[must_use]
    pub fn asked_for(request: impl Into<String>, answering: InferenceSource) -> Self {
        Self {
            conversation: TheConversation::began_with(request),
            activity: TheActivity::nothing_yet(),
            model: WhichModel::answering_from(answering),
            permissions: ThePermissions::nothing_yet(),
            stopping: Stopping::nobody_asked(),
            draft: String::new(),
        }
    }

    /// What was said.
    #[must_use]
    pub const fn conversation(&self) -> &TheConversation {
        &self.conversation
    }

    /// What it did.
    #[must_use]
    pub const fn activity(&self) -> &TheActivity {
        &self.activity
    }

    /// Which model answers.
    #[must_use]
    pub const fn model(&self) -> &WhichModel {
        &self.model
    }

    /// What it may reach.
    #[must_use]
    pub const fn permissions(&self) -> &ThePermissions {
        &self.permissions
    }

    /// Whether this task is waiting for the person to answer something.
    ///
    /// **The conversation's last turn, or a change still asked about.** The
    /// last turn alone is not enough once a person can write beneath a card: a
    /// reply sent while a change waits ends the conversation on something else,
    /// and a task read from the last turn alone would stop saying *waiting for
    /// your permission* with the card and its two answers still on the screen.
    #[must_use]
    pub fn is_waiting_for_the_person(&self) -> bool {
        self.conversation.is_waiting_for_the_person() || self.permissions.asking().is_some()
    }

    /// Whether it is stopping.
    #[must_use]
    pub const fn stopping(&self) -> &Stopping {
        &self.stopping
    }

    /// A change is waiting for this person, and this is the sentence they
    /// answer.
    ///
    /// **The sentence is taken whole.** It arrives already worded by whoever
    /// proposed the change, in the language the person reads, and nothing here
    /// shortens it, heads it or rewraps it into a different claim — what a
    /// person approves is that sentence, so a task that edited it would be
    /// asking a different question from the one that will be carried out.
    pub fn is_asking(&mut self, sentence: impl Into<String>) {
        self.permissions.is_asking(sentence);
    }

    /// They allowed the one action being asked about. **Nothing is widened.**
    ///
    /// Answers [`crate::Answered::NothingWasAsked`] when nothing was, which is
    /// a stale window or a second press and is not a refusal the person made.
    pub fn allow_once(&mut self) -> Answered {
        self.permissions.allow_once()
    }

    /// They said no. Nothing happened, and nothing is recorded about why,
    /// because nothing was asked.
    pub fn deny(&mut self) -> Answered {
        self.permissions.deny()
    }

    /// A follow-up, which stays in this window.
    ///
    /// **It begins the next request**, which is where a waiting model change
    /// lands — the one place `answering` moves. A follow-up that did not would
    /// leave a person's chosen model waiting for ever in the one window where
    /// they chose it.
    pub fn the_person_says(&mut self, said: impl Into<String>) {
        self.conversation.and_then(Turn::from_the_person(said));
        self.model.the_next_request_begins();
    }

    /// Alo said something.
    pub fn alo_says(&mut self, turn: Turn) {
        self.conversation.and_then(turn);
    }

    /// Something happened.
    pub fn and_then(&mut self, act: Act) {
        self.activity.and_then(act);
    }

    /// The person pressed Stop.
    ///
    /// **Asks, and does not end.** What ends it is [`Self::it_stopped`], called
    /// when the work actually has — which is the whole of
    /// [`Stopping`]'s shape.
    pub fn stop(&mut self) {
        self.stopping = self.stopping.clone().asked();
    }

    /// What the person has typed and not sent.
    #[must_use]
    pub fn the_draft(&self) -> &str {
        &self.draft
    }

    /// The person typed in the reply box.
    ///
    /// **Typing alone does nothing else.** *Typing does not automatically
    /// submit or interrupt* — the contract — so this changes what the one
    /// button offers and never starts, continues or cancels any work.
    pub fn the_person_is_typing(&mut self, draft: impl Into<String>) {
        self.draft = draft.into();
    }

    /// The draft was sent, so there is no longer a draft.
    ///
    /// **Separate from [`Self::the_person_says`]**, which appends the turn:
    /// clearing the box and adding what was in it to the conversation are two
    /// things, and a caller that did only the first would lose what somebody
    /// typed.
    #[must_use]
    pub fn take_the_draft(&mut self) -> String {
        std::mem::take(&mut self.draft)
    }

    /// The work ended, having changed this.
    pub fn it_stopped(&mut self, changed: crate::changed::WhatChanged) {
        self.stopping = Stopping::stopped(changed);
    }

    /// Which word says what is happening in this task.
    ///
    /// **The model chooses the word and the shell says it.** Which of the five
    /// applies is a reading of this task — and a drawing surface that worked it
    /// out would be a second place deciding, which is how two surfaces come to
    /// disagree about whether alo is still working.
    ///
    /// The order is the design's variant table, and the first two tests are the
    /// ones that matter:
    ///
    /// 1. **A stop asked for and not confirmed is *stopping*,** never
    ///    *stopped*. The work is still running, and saying it has ended is the
    ///    one thing a Stop must not claim.
    /// 2. **A confirmed stop is *stopped*** whatever the activity still says,
    ///    for the reason [`Self::the_one_button`] gives: the runtime's word
    ///    outranks a log that nobody closed out.
    /// 3. Waiting on the person, which the design draws in navy rather than
    ///    teal, because alo is not acting.
    /// 4. Working, which is the only one drawn in teal.
    /// 5. Otherwise finished.
    #[must_use]
    pub fn what_is_happening(&self) -> alo_strings::Word {
        match self.stopping {
            Stopping::Asked => crate::words::STOPPING,
            Stopping::Stopped { .. } => crate::words::STOPPED,
            Stopping::NobodyAsked => {
                if self.is_waiting_for_the_person() {
                    crate::words::WAITING_FOR_YOUR_PERMISSION
                } else if self.activity.doing_now().is_some() || self.conversation.is_alos_turn() {
                    crate::words::ALO_IS_WORKING
                } else {
                    crate::words::COMPLETE
                }
            }
        }
    }

    /// What the right-hand panel shows while this is minimised.
    #[must_use]
    pub fn in_the_panel(&self) -> InThePanel {
        InThePanel {
            request: self
                .conversation
                .the_request()
                .map(|turn| turn.said().to_owned())
                .unwrap_or_default(),
            doing: self.activity.doing_now().map(|act| act.said().to_owned()),
            waiting_for_you: self.is_waiting_for_the_person(),
            something_went_wrong: self.activity.anything_went_wrong(),
        }
    }

    /// What it may reach, to be changed.
    pub const fn permissions_mut(&mut self) -> &mut ThePermissions {
        &mut self.permissions
    }

    /// Which model answers, to be changed.
    pub const fn model_mut(&mut self) -> &mut WhichModel {
        &mut self.model
    }

    /// The activity, to be read into or out of Details.
    pub const fn activity_mut(&mut self) -> &mut TheActivity {
        &mut self.activity
    }
}

#[cfg(test)]
#[path = "task_tests.rs"]
mod tests;
