//! What the task is doing, and the technical log behind *Details*.
//!
//! The owner's words, 2026-10-12: *Activity shows actions, progress, results
//! and errors. Expand Details for technical logs.* And the sentence that
//! decides this file's shape: **the logs support the conversation; they are not
//! the whole interface.**
//!
//! # Details is shut, and a type is how that stays true
//!
//! A line of activity and its technical log are **one value with two parts**
//! rather than two lists that a surface decides between. With two lists, every
//! drawing chooses what to show and one of them eventually shows the log —
//! which is how a window for asking a machine to do something becomes a console
//! for watching it do it.
//!
//! So an [`Act`] has a sentence a person reads, and `details` that are
//! [`Option`] and separate. A caller that wants the detail has to ask for it by
//! name.
//!
//! # Errors are activity, not a different surface
//!
//! *Actions, progress, results and errors* — one list, in order. An error
//! raised somewhere else is an error a person has to go and find, and the
//! moment it happened is the thing that explains it.

/// How an act ended, or that it has not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HowItWent {
    /// Still going.
    ///
    /// **A state and not an absence.** A list that showed only finished acts
    /// would leave a person watching nothing while the machine worked, which
    /// is the commonest moment in a task's life.
    Doing,
    /// It did what it said.
    Done,
    /// It did not, and this is activity like any other.
    WentWrong,
}

/// One thing the task did, or is doing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Act {
    /// What a person is told, in their own words.
    said: String,
    /// How it went.
    went: HowItWent,
    /// The machine's own account, for whoever opens **Details**.
    ///
    /// [`None`] where there is nothing more to say than the sentence — which is
    /// most acts, and is why this is an `Option` rather than an empty string: a
    /// Details control that opened on nothing is a control that lies about
    /// having something behind it.
    details: Option<String>,
}

impl Act {
    /// Something the task is doing now.
    #[must_use]
    pub fn doing(said: impl Into<String>) -> Self {
        Self {
            said: said.into(),
            went: HowItWent::Doing,
            details: None,
        }
    }

    /// Something it finished.
    #[must_use]
    pub fn done(said: impl Into<String>) -> Self {
        Self {
            said: said.into(),
            went: HowItWent::Done,
            details: None,
        }
    }

    /// Something that went wrong.
    #[must_use]
    pub fn went_wrong(said: impl Into<String>) -> Self {
        Self {
            said: said.into(),
            went: HowItWent::WentWrong,
            details: None,
        }
    }

    /// With the machine's own account behind Details.
    #[must_use]
    pub fn and_the_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// What a person is told.
    #[must_use]
    pub fn said(&self) -> &str {
        &self.said
    }

    /// How it went.
    #[must_use]
    pub const fn went(&self) -> HowItWent {
        self.went
    }

    /// The technical log, for whoever opened Details.
    #[must_use]
    pub fn details(&self) -> Option<&str> {
        self.details.as_deref()
    }

    /// Whether there is anything behind Details at all.
    ///
    /// So a surface can leave the control off rather than offering one that
    /// opens on nothing.
    #[must_use]
    pub const fn has_details(&self) -> bool {
        self.details.is_some()
    }
}

/// Everything the task has done, in order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TheActivity {
    /// The acts, oldest first.
    acts: Vec<Act>,
    /// Whether the person has opened Details.
    ///
    /// **Shut until they open it, and theirs once they have.** Not reset by a
    /// new act: somebody who opened the log to watch something is watching, and
    /// a surface that closed it under them each time a line arrived would be
    /// unusable exactly when it was wanted.
    details_are_open: bool,
}

impl TheActivity {
    /// A task that has done nothing yet.
    #[must_use]
    pub fn nothing_yet() -> Self {
        Self::default()
    }

    /// And then this happened.
    pub fn and_then(&mut self, act: Act) {
        self.acts.push(act);
    }

    /// Every act, oldest first.
    #[must_use]
    pub fn each(&self) -> &[Act] {
        &self.acts
    }

    /// What it is doing now, if anything.
    ///
    /// The **last** act that is still going, which is what a minimised task
    /// shows in the panel: a person glancing at it wants *what is it doing*,
    /// not a history.
    #[must_use]
    pub fn doing_now(&self) -> Option<&Act> {
        self.acts
            .iter()
            .rev()
            .find(|act| matches!(act.went(), HowItWent::Doing))
    }

    /// Whether anything went wrong.
    #[must_use]
    pub fn anything_went_wrong(&self) -> bool {
        self.acts
            .iter()
            .any(|act| matches!(act.went(), HowItWent::WentWrong))
    }

    /// Whether the person has Details open.
    #[must_use]
    pub const fn details_are_open(&self) -> bool {
        self.details_are_open
    }

    /// The person opened or shut Details.
    pub const fn details_are_now(&mut self, open: bool) {
        self.details_are_open = open;
    }
}

#[cfg(test)]
#[path = "activity_tests.rs"]
mod tests;
