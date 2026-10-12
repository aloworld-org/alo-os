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
use crate::permissions::ThePermissions;
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

    /// Whether it is stopping.
    #[must_use]
    pub const fn stopping(&self) -> &Stopping {
        &self.stopping
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

    /// The work ended, having changed this.
    pub fn it_stopped(&mut self, changed: crate::changed::WhatChanged) {
        self.stopping = Stopping::stopped(changed);
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
            waiting_for_you: self.conversation.is_waiting_for_the_person(),
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
