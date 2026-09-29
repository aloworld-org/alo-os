//! Whether a machine answered, including the answer *we have not asked yet*.
//!
//! # Why this is an enum and not an `Option<bool>`
//!
//! There are three states and only three, and the third is the one a boolean
//! loses: a machine added a moment ago has not failed to answer — nobody has
//! asked it. Showing that as *did not answer* would have a person looking for a
//! network fault that does not exist, thirty seconds after they added a machine
//! that is fine.
//!
//! An `Option<bool>` can hold three states too. It is refused because `None` and
//! `Some(false)` read the same at a glance in a match arm, and the whole point
//! of this type is that the two must never be confused.

use crate::words;

/// Whether a machine the person added answered when this machine last asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Reaching {
    /// It answered.
    Answered,
    /// It was asked and did not answer.
    ///
    /// **This is a state, not an absence.** A machine in this state is still on
    /// the person's list, still named, and still says so.
    DidNotAnswer,
    /// It has not been asked since the person added it.
    ///
    /// Distinct from [`Self::DidNotAnswer`] on purpose: *we have not asked* and
    /// *it did not reply* are different facts, and the second is alarming while
    /// the first is not.
    NotAskedYet,
}

impl Reaching {
    /// Every state, so a caller drawing them cannot quietly handle two of three.
    pub const ALL: &'static [Self] = &[Self::Answered, Self::DidNotAnswer, Self::NotAskedYet];

    /// Whether work can be sent right now.
    ///
    /// Only [`Self::Answered`] is usable. [`Self::NotAskedYet`] is deliberately
    /// **not** usable: optimism here would mean sending work to a machine
    /// nothing has ever reached and reporting the failure later, when the
    /// honest answer was available before anything was sent.
    #[must_use]
    pub const fn can_be_worked_with(self) -> bool {
        matches!(self, Self::Answered)
    }

    /// Whether this is something the person may want to act on.
    ///
    /// Used to decide what to draw attention to, never to decide what to list.
    /// Nothing in this crate lets a caller list only the machines for which this
    /// is false — see [`crate::TheMachines`].
    #[must_use]
    pub const fn is_worth_saying(self) -> bool {
        matches!(self, Self::DidNotAnswer)
    }

    /// The key of the sentence a person reads for this state.
    ///
    /// The key rather than the sentence: what a person reads is whatever their
    /// language says under it, and that lookup belongs where a language has
    /// been chosen rather than in a model that has no opinion about one.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Answered => words::ANSWERED,
            Self::DidNotAnswer => words::DID_NOT_ANSWER,
            Self::NotAskedYet => words::NOT_ASKED_YET,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The third state is the one a boolean loses.** A machine added a moment
    /// ago has not failed; nobody has asked it.
    #[test]
    fn not_asked_yet_is_not_a_failure() {
        assert!(!Reaching::NotAskedYet.is_worth_saying());
        assert!(Reaching::DidNotAnswer.is_worth_saying());
    }

    /// **Optimism is refused.** A machine nothing has reached is not one work
    /// may be sent to, because the honest answer is available before anything is
    /// sent rather than after it fails.
    #[test]
    fn a_machine_nobody_has_asked_is_not_ready_for_work() {
        assert!(!Reaching::NotAskedYet.can_be_worked_with());
        assert!(!Reaching::DidNotAnswer.can_be_worked_with());
        assert!(Reaching::Answered.can_be_worked_with());
    }

    /// Every state has its own sentence, and no two share one.
    #[test]
    fn each_state_says_something_different() {
        let mut seen = Vec::new();
        for state in Reaching::ALL {
            let word = state.word();
            assert!(!seen.contains(&word), "two states share the key {word}");
            seen.push(word);
        }
        assert_eq!(seen.len(), 3, "a state was added without a sentence");
    }
}
