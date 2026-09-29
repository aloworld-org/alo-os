//! The ways adding a machine does not happen.

use crate::words;

/// Why a machine was not added.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotElsewhere {
    /// Nothing was left of the name after the spaces either side came off.
    #[error("a machine needs a name the person will recognise")]
    Unnamed,
    /// The name is past what a list can show.
    #[error("that name is {how_long} characters and the most is {at_most}")]
    NameTooLong {
        /// How long the name the person typed is, in characters.
        how_long: usize,
        /// The most a name may be.
        at_most: usize,
    },
    /// That machine is already on the person's list.
    #[error("that machine is already added")]
    AlreadyAdded,
    /// A grant whose end is not after its beginning.
    #[error("a grant must end after it begins")]
    EndsBeforeItBegins,
    /// A grant over that machine already exists.
    #[error("that machine is already granted to the agent")]
    AlreadyDriving,
    /// Nothing was left of the goal after the spaces either side came off.
    #[error("work needs a goal saying what is wanted")]
    NoGoal,
    /// The goal is longer than what crosses.
    #[error("that goal is {how_long} characters and the most is {at_most}")]
    GoalTooLong {
        /// How long the goal is, in characters.
        how_long: usize,
        /// The most a goal may be.
        at_most: usize,
    },
    /// A result arrived for work the person had stopped.
    #[error("that work was stopped before its result arrived")]
    ItWasStopped,
}

impl NotElsewhere {
    /// The key of the sentence a person reads for this refusal.
    ///
    /// The `Display` text above is for a log and for a developer; this is the
    /// one a person sees, and they are deliberately not the same sentence.
    #[must_use]
    pub const fn word(&self) -> &'static str {
        match self {
            Self::Unnamed => words::UNNAMED,
            Self::NameTooLong { .. } => words::NAME_TOO_LONG,
            Self::AlreadyAdded => words::ALREADY_ADDED,
            Self::EndsBeforeItBegins => words::ENDS_BEFORE_IT_BEGINS,
            Self::AlreadyDriving => words::ALREADY_DRIVING,
            Self::NoGoal => words::NO_GOAL,
            Self::GoalTooLong { .. } => words::GOAL_TOO_LONG,
            Self::ItWasStopped => words::IT_WAS_STOPPED,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every refusal has a sentence a person can read, and no two share one.
    #[test]
    fn every_refusal_says_something_of_its_own() {
        let every = [
            NotElsewhere::Unnamed,
            NotElsewhere::NameTooLong {
                how_long: 65,
                at_most: 64,
            },
            NotElsewhere::AlreadyAdded,
            NotElsewhere::EndsBeforeItBegins,
            NotElsewhere::AlreadyDriving,
            NotElsewhere::NoGoal,
            NotElsewhere::GoalTooLong {
                how_long: 501,
                at_most: 500,
            },
            NotElsewhere::ItWasStopped,
        ];
        let mut seen = Vec::new();
        for refusal in &every {
            let word = refusal.word();
            assert!(!seen.contains(&word), "two refusals share {word}");
            seen.push(word);
        }
        assert_eq!(seen.len(), 8, "a refusal was added without a sentence");
    }

    /// **The developer's sentence and the person's are not the same string.**
    /// One goes in a log, the other is read by somebody adding a machine, and
    /// writing one for both ends up serving neither.
    #[test]
    fn what_a_log_says_is_not_what_a_person_reads() {
        let refused = NotElsewhere::AlreadyAdded;
        assert_eq!(refused.to_string(), "that machine is already added");
        assert_eq!(refused.word(), "elsewhere.not-added.already-added");
    }
}
