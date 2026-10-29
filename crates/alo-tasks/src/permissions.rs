//! What a task may reach, and how it asks for more.
//!
//! The owner's words, 2026-10-12: *Permissions shows what this task may access
//! or change, with controls to grant or revoke access*, and *when alo needs
//! permission, show the specific proposed action in the conversation with Allow
//! once and Deny. Broader access should be a separate, deliberate choice.*
//!
//! # Two things that must not become one
//!
//! **Allow once answers one proposed action.** It is not a grant, it does not
//! outlive the action, and it never appears in the list below. `CLAUDE.md`
//! settles it: *what a person approves is that sentence, and an approval is
//! never a session.*
//!
//! **A standing grant is the other thing**, and a person reaches it
//! deliberately — it is a row in this area with its own control, not a
//! consequence of having said yes to something.
//!
//! The failure this file is shaped to refuse is the easy one: *they allowed it
//! once, so stop asking.* That reads as kindness and is how an approval becomes
//! a session. So [`ThePermissions`] can answer an action **without any way to
//! widen anything**, and widening takes a different call with a different name.
//!
//! # Where the lists come from
//!
//! Nowhere here. `alo_granted::Seen` is a row of what is granted and holds its
//! own revocation; `alo_approving::Approving` holds the one change in front of
//! a person. This type reads both and adds the one thing neither has: that they
//! are two different questions on one screen.

/// What a task may reach, and whatever it is asking for now.
///
/// **Holds neither list itself.** The grants live where grants live and the
/// pending change lives where changes live; a copy here would be a second
/// account of each, and the one that drifts is always the copy on the screen.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ThePermissions {
    /// What this task is asking to do right now, worded as a person reads it.
    ///
    /// **One, not a queue.** `alo_approving::Approving` holds one change per
    /// surface and this follows it: a list of pending approvals is a list a
    /// person clears by clicking, and clicking through is how somebody approves
    /// the fourth one without reading it.
    asking: Option<String>,
    /// How many standing grants this task holds, for the sentence that counts
    /// them.
    ///
    /// The rows themselves are read from `alo-granted` at the moment of
    /// drawing. This is the count a person is told **in the conversation**,
    /// where there is no room for the rows.
    holds: usize,
}

/// What answering a proposed action did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answered {
    /// The person allowed this one action. **Nothing was widened.**
    AllowedOnce,
    /// The person said no. Nothing happened.
    Denied,
    /// Nothing was being asked, so nothing was answered.
    ///
    /// **A real answer rather than a silent no-op**, because the two callers
    /// that can produce it are a double-click and a stale window — and a
    /// surface that treated them as *denied* would record a refusal the person
    /// never made.
    NothingWasAsked,
}

impl ThePermissions {
    /// A task holding nothing and asking for nothing.
    #[must_use]
    pub fn nothing_yet() -> Self {
        Self::default()
    }

    /// A task holding this many standing grants.
    #[must_use]
    pub const fn holding(holds: usize) -> Self {
        Self {
            asking: None,
            holds,
        }
    }

    /// This task is asking to do this, in the words a person reads.
    ///
    /// **Replaces whatever was being asked**, which cannot happen while one
    /// change is in front of a person — `Approving` refuses a second — and is
    /// written as a replacement rather than a refusal because this type is a
    /// reading. A reading that could disagree with what it reads would be the
    /// second account this file exists to avoid.
    pub fn is_asking(&mut self, sentence: impl Into<String>) {
        self.asking = Some(sentence.into());
    }

    /// What it is asking, if it is asking.
    #[must_use]
    pub fn asking(&self) -> Option<&str> {
        self.asking.as_deref()
    }

    /// How many standing grants it holds.
    #[must_use]
    pub const fn how_many_grants(&self) -> usize {
        self.holds
    }

    /// Allow the one action being asked about.
    ///
    /// **It cannot widen anything**, and that is the whole of this method: the
    /// only thing it touches is the pending question. A caller that wanted a
    /// standing grant has to go and ask for one, by a different name, having
    /// shown the person what it costs.
    pub fn allow_once(&mut self) -> Answered {
        match self.asking.take() {
            Some(_) => Answered::AllowedOnce,
            None => Answered::NothingWasAsked,
        }
    }

    /// Say no to the one action being asked about.
    pub fn deny(&mut self) -> Answered {
        match self.asking.take() {
            Some(_) => Answered::Denied,
            None => Answered::NothingWasAsked,
        }
    }

    /// **The question lapsed unanswered**: the turn it lived in ended, by a
    /// Stop or by running out of time. Neither allowed nor denied, and nothing
    /// is widened or recorded as an answer (ADR 0139).
    pub fn lapsed(&mut self) {
        self.asking = None;
    }
}

#[cfg(test)]
#[path = "permissions_tests.rs"]
mod tests;
