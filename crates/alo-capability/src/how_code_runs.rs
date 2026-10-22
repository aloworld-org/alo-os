//! How code the agent writes or fetches is allowed to run, which the person
//! chooses.
//!
//! [ADR 0064](../../../docs/decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
//! replaced ADR 0001 §1 for this one question. The broker's verb list is
//! unchanged — typed, closed, walked by the compiler, and **running code is not
//! a verb on it**. It is a separate capability, at one of three levels, and the
//! person picks which.
//!
//! Law 2 in `CLAUDE.md` reads **no code runs unless the person chose it**, and
//! this type is the choosing.
//!
//! # What this is, and the one thing it is not
//!
//! It is the **level**, and nothing else: what the person decided. It is not a
//! runner, not a sandbox and not a policy engine. The sealed box is enforced by
//! the kernel ([ADR 0013](../../../docs/decisions/0013-the-grant-is-enforced-by-the-kernel.md)),
//! and **that boundary is not built**, so nothing in this repository can yet
//! run code at the default level. Recording the choice before the runner exists
//! is the right order: the runner has to ask something what the person decided,
//! and this is the something.
//!
//! **It is not a setting this crate reads**, for the reason
//! [`crate::a_terminal_for_the_agent`] gives: a person's choices live in their
//! own settings and this crate is handed the answer, as it is handed a grant
//! rather than reading the grants file.
//!
//! # Law 5 is the shape of this file
//!
//! > *The person chooses. alo never takes the choice away.*
//!
//! Three consequences, and each is a test rather than a sentence:
//!
//! **Every level is reachable.** There is no level this type can be put into
//! and not brought out of, and none that another level can forbid. A type that
//! could only tighten would be a wall wearing a setting's clothes.
//!
//! **The cost is said, and only said.** [`HowCodeRuns::what_it_costs`] carries
//! the plain words for full trust, and carrying them is all it does. ADR 0064:
//! *it informs and does not forbid.*
//!
//! **Three things stay on at every level**, and none of them is a setting here
//! or anywhere — see [`AlwaysOn`]. They are the two kinds law 5 keeps: the ones
//! that restrict nothing, and the ones that guard the person from others.

use serde::{Deserialize, Serialize};

use crate::words;

/// How code the agent writes or fetches runs on this machine.
///
/// The three levels of ADR 0064, in the order that ADR lists them, which is
/// also the order from least to most that the agent may do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HowCodeRuns {
    /// Inside a box that sees the project and nothing else.
    ///
    /// **The default.** No network except destinations the person approved,
    /// each appearing in the egress indicator; no powers of its own; no road to
    /// the broker; limits on processor, memory and time; thrown away after each
    /// run. **The lock is the kernel's, not the agent's promise.**
    ///
    /// **The box runs a command as of 2026-10-16.** `alo-running` is it, and
    /// `alo-running/tests/the_kernel_refuses_what_a_run_was_not_given.rs`
    /// measures it end to end on a real kernel: the same command, the same
    /// box, one line different in the bound, reading the file in one case and
    /// `EACCES` in the other.
    ///
    /// *This read "the box is not built. A machine at this level therefore
    /// runs nothing" until 2026-10-22, and had been false for six days —
    /// nothing fails when a comment naming what is missing stops being true.
    /// `docs/features.md` had the correct date the whole time.*
    #[default]
    InASealedBox,
    /// The agent shows the exact command, and it runs once the person approves.
    ///
    /// On the person's own machine, with everything the person may do. What is
    /// approved is the command shown, which is the rule every approval in this
    /// crate follows.
    AskingEachTime,
    /// The agent runs code directly, without asking.
    ///
    /// The person switches this on themself and can switch it off at any time.
    /// [`HowCodeRuns::what_it_costs`] is the sentence shown when they do, and
    /// ADR 0064 is explicit that it informs rather than forbids.
    AtFullTrust,
}

/// A protection that stays on however code is allowed to run.
///
/// ADR 0064: *at every level, always on.* Law 5 permits exactly two kinds of
/// protection to be unchangeable — those that restrict nothing, and those that
/// guard the person from others — and all three of these are the first kind.
///
/// **They are listed as a type so that they can be asserted.** A sentence in a
/// document saying *the record is always written* is a sentence; a list every
/// level must answer the same is a test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlwaysOn {
    /// Every command and its output are written to the record.
    TheRecord,
    /// The project or the affected files are snapshotted before each run, so
    /// *undo what the agent did* works **even at full trust**.
    TheSnapshotBeforeEachRun,
    /// Everything that leaves the machine is shown.
    TheEgressIndicator,
}

impl AlwaysOn {
    /// All three, which is the whole list.
    pub const ALL: [Self; 3] = [
        Self::TheRecord,
        Self::TheSnapshotBeforeEachRun,
        Self::TheEgressIndicator,
    ];

    /// What this protection is, in words a person reads.
    #[must_use]
    pub const fn the_word(self) -> alo_strings::Word {
        match self {
            Self::TheRecord => words::ALWAYS_THE_RECORD,
            Self::TheSnapshotBeforeEachRun => words::ALWAYS_THE_SNAPSHOT,
            Self::TheEgressIndicator => words::ALWAYS_THE_INDICATOR,
        }
    }
}

impl HowCodeRuns {
    /// Every level, in the order ADR 0064 lists them.
    pub const ALL: [Self; 3] = [Self::InASealedBox, Self::AskingEachTime, Self::AtFullTrust];

    /// What this level is, in words a person reads.
    #[must_use]
    pub const fn the_word(self) -> alo_strings::Word {
        match self {
            Self::InASealedBox => words::RUNS_IN_A_SEALED_BOX,
            Self::AskingEachTime => words::RUNS_AFTER_ASKING,
            Self::AtFullTrust => words::RUNS_AT_FULL_TRUST,
        }
    }

    /// What choosing this level costs, where it costs anything.
    ///
    /// [`None`] for the two levels that cost nothing a person needs warning
    /// about: a sealed box and asking each time both leave the person in
    /// control of every run.
    ///
    /// [`Some`] for [`Self::AtFullTrust`], carrying ADR 0064's own words:
    /// *text in a web page, an email or a downloaded project can steer the
    /// agent, and at this level whatever it steers the agent to run, runs.*
    ///
    /// **It is said and nothing more.** No road here refuses the level, delays
    /// it, or asks twice. Law 5: *anything with a risk is offered with the risk
    /// in plain words and switched on by the person.* An `Option` rather than a
    /// sentence for all three, because a warning on a level that carries no
    /// risk teaches a person to dismiss warnings.
    #[must_use]
    pub const fn what_it_costs(self) -> Option<alo_strings::Word> {
        match self {
            Self::InASealedBox | Self::AskingEachTime => None,
            Self::AtFullTrust => Some(words::WHAT_FULL_TRUST_COSTS),
        }
    }

    /// Whether the agent may run code without asking first.
    ///
    /// A method rather than a `matches!` at each site that asks, for the reason
    /// [`crate::a_terminal_for_the_agent::ATerminalForTheAgent::may_open_one`]
    /// gives: so that the sites cannot come to disagree.
    #[must_use]
    pub const fn runs_without_asking(self) -> bool {
        matches!(self, Self::AtFullTrust)
    }

    /// Whether a run at this level is confined by the kernel.
    ///
    /// True only of the sealed box. The other two run with everything the
    /// person may do, which is what the person chose.
    #[must_use]
    pub const fn the_kernel_confines_it(self) -> bool {
        matches!(self, Self::InASealedBox)
    }

    /// What stays on at this level, which is the same at all three.
    ///
    /// **A method on the level, although the answer does not depend on it.**
    /// That is the point: asking each level and getting the same three is how
    /// *at every level, always on* becomes checkable rather than remembered. A
    /// later change that made one of them depend on the level would have to
    /// edit this signature, which is a visible act.
    #[must_use]
    pub const fn what_stays_on(self) -> [AlwaysOn; 3] {
        AlwaysOn::ALL
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "how_code_runs_tests.rs"]
mod tests;
