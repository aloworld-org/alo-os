//! What a put-aside window says about the work alo is doing in it.
//!
//! Task 7 of `docs/autonomy/putting-a-window-aside.md`: the task alo was given, the scope it
//! may change, progress, the last confirmed action, **Stop available immediately**, and
//! *Requires you* when a decision is waiting.
//!
//! # Stop is not a field, and that is the guarantee
//!
//! *Stop available immediately* is held by there being **nothing here that could withhold
//! it**. There is no `can_stop`, no `stopping_allowed`, no state in which this type exists
//! and stopping does not. A boolean would be a promise a caller could set to `false`, and
//! the one thing a person needs from a window they cannot see is the ability to make it
//! stop.
//!
//! So the rule is: **if this value exists, Stop exists.** A surface drawing an `AtWork`
//! draws Stop, unconditionally, and there is no branch for it to get wrong.
//!
//! # Every field refuses its own empty version
//!
//! ADR 0009: *the agent's surfaces disappear rather than nag*, and *a greyed-out feature is
//! an advertisement*. In this surface that means an agent section is **absent or complete** —
//! never present and hollow.
//!
//! - The task is named, and a blank name is refused. *alo is working on* followed by nothing
//!   is the empty control in its purest form.
//! - The scope distinguishes reading from changing, rather than carrying a list that happens
//!   to be empty.
//! - The last confirmed action is an [`Option`] and **absent is a real state** — alo has
//!   confirmed nothing yet — which is different from a blank line where a sentence should be.
//! - Progress is optional for the same reason: a task whose extent nobody knows has no
//!   progress to draw, and a bar at zero would say *begun and got nowhere*.
//! - *Requires you* carries the question or is `No`.
//!
//! # What is absent is the machine's answer, not this type's
//!
//! With AI switched off there is no `AtWork` anywhere, because
//! [`crate::what_alo_is_doing::WhatAloIsDoing::Nothing`] is what a preview carries. This
//! type has no *disabled* variant and no empty constructor: **the absence is one level up**,
//! where a surface can see it and draw nothing at all.

use crate::how_far_alo_has_got::Progress;
use crate::requires_you::RequiresYou;
use crate::the_scope_alo_may_change::Scope;

/// Why a report of alo's work could not be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotAtWork {
    /// A task with no name.
    #[error("alo is at work on a task with no name, which a person cannot be shown")]
    ItHasNoName,
    /// A last confirmed action that says nothing.
    ///
    /// **Different from having confirmed nothing**, which is [`None`] and is honest.
    #[error("the last confirmed action is blank, which is not the same as none having happened")]
    ItConfirmedNothing,
}

/// What alo is doing in a window that is put aside.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtWork {
    /// The task it was given, in the words it was given in.
    task: String,
    /// What it may touch.
    scope: Scope,
    /// How far through, where that is knowable.
    progress: Option<Progress>,
    /// The last thing it did that the person approved.
    last_confirmed: Option<String>,
    /// Whether it has stopped and needs an answer.
    requires_you: RequiresYou,
}

impl AtWork {
    /// A report of alo working on a named task.
    ///
    /// Takes every part at once, because a report assembled field by field is a report that
    /// can be drawn half-built — and half-built is the empty control this whole surface
    /// refuses.
    ///
    /// # Errors
    ///
    /// [`NotAtWork::ItHasNoName`] for a blank task, and
    /// [`NotAtWork::ItConfirmedNothing`] for a blank last-confirmed action. Pass [`None`] to
    /// say nothing has been confirmed yet, which is a different and honest answer.
    pub fn on(
        task: &str,
        scope: Scope,
        progress: Option<Progress>,
        last_confirmed: Option<&str>,
        requires_you: RequiresYou,
    ) -> Result<Self, NotAtWork> {
        if task.trim().is_empty() {
            return Err(NotAtWork::ItHasNoName);
        }
        if last_confirmed.is_some_and(|it| it.trim().is_empty()) {
            return Err(NotAtWork::ItConfirmedNothing);
        }
        Ok(Self {
            task: task.to_owned(),
            scope,
            progress,
            last_confirmed: last_confirmed.map(str::to_owned),
            requires_you,
        })
    }

    /// The task it was given. **Never blank.**
    #[must_use]
    pub fn task(&self) -> &str {
        &self.task
    }

    /// What it may touch.
    #[must_use]
    pub const fn scope(&self) -> &Scope {
        &self.scope
    }

    /// How far through, where that is knowable.
    #[must_use]
    pub const fn progress(&self) -> Option<Progress> {
        self.progress
    }

    /// The last thing the person approved, if anything yet.
    #[must_use]
    pub fn last_confirmed(&self) -> Option<&str> {
        self.last_confirmed.as_deref()
    }

    /// Whether it has stopped and needs an answer.
    #[must_use]
    pub const fn requires_you(&self) -> &RequiresYou {
        &self.requires_you
    }

    /// Whether Stop is available.
    ///
    /// **Always true**, and it is a method rather than a constant so that a surface can ask
    /// the question in the same shape as the others. There is deliberately no way for it to
    /// answer anything else: *Stop available immediately* is the one promise a person cannot
    /// check for themselves from a window they cannot see.
    #[must_use]
    pub const fn can_be_stopped(&self) -> bool {
        true
    }
}
