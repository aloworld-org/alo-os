//! Whether what a person asks alo for **in the terminal** waits for their OK,
//! which the person chooses.
//!
//! `docs/features.md`, the owner's decision of 2026-11-06: *in Settings →
//! Terminal, the person can let what they ask alo for in the terminal run
//! without the OK — ADR 0064's third level, full trust, applied to the
//! terminal. Off until the person switches it on, offered with what it costs in
//! plain words, and off again in one press.*
//!
//! # What this is, and what it is not
//!
//! It is the **answer** and nothing else, beside [`crate::HowCodeRuns`] for the
//! same reason that type gives: a person's choices live in their own settings
//! and this crate is handed the answer. It is kept in the same file as that
//! level (`docs/contracts/how-code-runs-file.md`), under the same rules about
//! who may have written it.
//!
//! **It is not running code.** [`crate::HowCodeRuns`] is how code the agent
//! writes or fetches runs, for the whole machine. This is whether the changes
//! alo proposes for a request typed at a terminal prompt are carried out
//! without the person being shown them first. Neither answers for the other,
//! and the file keeps them as two keys.
//!
//! **It reaches only the terminal.** A request from the alo Bar, the launcher
//! or a task window still waits for the OK at every setting of this, because
//! nothing on those roads reads it — `alo-ask` is the one reader, and it is the
//! command a person types at their own prompt.
//!
//! # Law 5 is the shape of this file, as it is of `how_code_runs`
//!
//! **The cost is said, and only said.** [`AloInTheTerminal::what_it_costs`] is
//! the owner's own sentence, and carrying it is all it does.
//!
//! **What stays on is not in it.** The record, Put back and the egress
//! indicator are [`crate::AlwaysOn`], and this type has no road to any of them:
//! a change carried out without the OK is still carried out by the daemon, which
//! records it, keeps a copy from before it and lights the indicator for
//! whatever leaves, exactly as it does for one a person pressed *Allow once* on.

/// Whether what a person asks alo for in the terminal waits for their OK.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AloInTheTerminal {
    /// alo shows what it would do and waits for the OK.
    ///
    /// **Off, which is how a machine arrives**: the switch is off until the
    /// person switches it on. A machine with nothing written down answers
    /// this.
    #[default]
    AsksFirst,
    /// What the person asks for runs without the OK — ADR 0064's full trust,
    /// applied to the terminal.
    ActsWithoutAsking,
}

impl AloInTheTerminal {
    /// Whether a change alo proposes for a request typed in the terminal is
    /// carried out without the person being shown it first.
    ///
    /// A method rather than a `matches!` at each site that asks, for
    /// [`crate::HowCodeRuns::runs_without_asking`]'s reason: so that the
    /// sites cannot come to disagree.
    #[must_use]
    pub const fn acts_without_asking(self) -> bool {
        matches!(self, Self::ActsWithoutAsking)
    }

    /// The other answer, which is what one press of the switch gives.
    ///
    /// **One press either way**, and that is the promise: *off again in one
    /// press*. There is no confirmation on the way off, because turning a
    /// protection back on takes nothing from anybody.
    #[must_use]
    pub const fn the_other_one(self) -> Self {
        match self {
            Self::AsksFirst => Self::ActsWithoutAsking,
            Self::ActsWithoutAsking => Self::AsksFirst,
        }
    }

    /// What choosing this costs, where it costs anything.
    ///
    /// [`Some`] for [`Self::ActsWithoutAsking`] only, carrying the owner's
    /// words: *alo will send, change and delete what you ask for without
    /// showing you first.* [`None`] for asking first, which costs nothing a
    /// person needs warning about — for [`crate::HowCodeRuns::what_it_costs`]'s
    /// reason, a warning on a choice with no risk teaches a person to dismiss
    /// warnings.
    #[must_use]
    pub const fn what_it_costs(self) -> Option<alo_strings::Word> {
        match self {
            Self::AsksFirst => None,
            Self::ActsWithoutAsking => Some(crate::words::WHAT_ACTING_WITHOUT_ASKING_COSTS),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Off until the person switches it on.** A machine that has written
    /// nothing down answers with the default, and the default waits.
    #[test]
    fn a_machine_arrives_asking_first() {
        assert_eq!(AloInTheTerminal::default(), AloInTheTerminal::AsksFirst);
        assert!(!AloInTheTerminal::default().acts_without_asking());
    }

    /// **Off again in one press, and on again in one.** Neither direction is
    /// a wall, and neither takes two presses.
    #[test]
    fn one_press_turns_it_either_way() {
        for now in [AloInTheTerminal::AsksFirst, AloInTheTerminal::ActsWithoutAsking] {
            assert_ne!(now.the_other_one(), now);
            assert_eq!(now.the_other_one().the_other_one(), now);
        }
        assert_eq!(
            AloInTheTerminal::ActsWithoutAsking.the_other_one(),
            AloInTheTerminal::AsksFirst,
            "one press from on did not turn it off"
        );
    }

    /// **The cost is said for the one answer that has one**, in the owner's
    /// words, and not for the one that has none.
    #[test]
    fn only_acting_without_asking_says_what_it_costs() {
        assert_eq!(AloInTheTerminal::AsksFirst.what_it_costs(), None);
        assert_eq!(
            AloInTheTerminal::ActsWithoutAsking.what_it_costs(),
            Some(crate::words::WHAT_ACTING_WITHOUT_ASKING_COSTS)
        );
        assert_eq!(
            crate::words::WHAT_ACTING_WITHOUT_ASKING_COSTS.says(),
            "alo will send, change and delete what you ask for without showing you first",
            "the cost is not the owner's sentence any more"
        );
    }
}
