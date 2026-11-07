//! How a command ended: the exit codes of `docs/contracts/terminal-commands.md`,
//! *What it says, and the exit codes*.
//!
//! | Code | Meaning | Did anything happen? |
//! |---|---|---|
//! | 0 | done | yes |
//! | 1 | the application refused, and said why | no |
//! | 64 | the command was not written in this shape | no |
//! | 65 | a value was refused | no |
//! | 69 | the application is not installed, or does not offer that ability | no |
//! | 75 | the request was sent and the application did not answer | not known |
//!
//! **76 and 77 are not produced by anything in this release, and that is not an
//! omission in this file.** 76 is the screen road's *the keys or the click were
//! delivered and reading the window again could not confirm it*, and the screen
//! road is not built. 77 is *an organisation's policy forbids it* (ADR 0016),
//! and no policy reaches a person's own command yet. A variant nobody can
//! produce would be a promise this file cannot keep, so neither is here.

use std::process::ExitCode;

/// How one act, or one whole command line, ended.
///
/// Ordered from the best to the worst, so a command whose pipe ran several
/// lines ends with [`Ended::worst`] of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ended {
    /// Done.
    Done,
    /// The application, or the machine on its behalf, refused and said why.
    Refused,
    /// A value was refused: the wrong kind, out of range, not one of the
    /// options, or a character a sentence cannot hold.
    ValueRefused,
    /// The application is not installed, does not offer that ability, or the
    /// road to it is not there.
    NotOffered,
    /// The command was not written in this shape.
    NotThisShape,
    /// The request was sent and nothing answered: whether it happened is not
    /// known, and it is written down as having run.
    NotKnown,
}

impl Ended {
    /// The number a shell reads.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Done => 0,
            Self::Refused => 1,
            Self::NotThisShape => 64,
            Self::ValueRefused => 65,
            Self::NotOffered => 69,
            Self::NotKnown => 75,
        }
    }

    /// The worse of two endings.
    #[must_use]
    pub fn worst(self, other: Self) -> Self {
        self.max(other)
    }

    /// Whether anything may have happened.
    #[must_use]
    pub const fn may_have_happened(self) -> bool {
        matches!(self, Self::Done | Self::NotKnown)
    }
}

impl From<Ended> for ExitCode {
    fn from(ended: Ended) -> Self {
        Self::from(ended.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The numbers are the contract's**, and nothing else is one.
    #[test]
    fn every_ending_is_the_contracts_number() {
        let all = [
            Ended::Done,
            Ended::Refused,
            Ended::ValueRefused,
            Ended::NotOffered,
            Ended::NotThisShape,
            Ended::NotKnown,
        ];
        let codes: Vec<u8> = all.iter().map(|ended| ended.code()).collect();
        assert_eq!(codes, [0, 1, 65, 69, 64, 75]);
        assert!(!codes.contains(&76) && !codes.contains(&77));
    }

    /// **The worst of several lines is what the command ends with**, and *not
    /// known* is worse than any refusal, because something may have happened.
    #[test]
    fn the_worst_of_several_is_kept() {
        assert_eq!(Ended::Done.worst(Ended::Refused), Ended::Refused);
        assert_eq!(Ended::NotKnown.worst(Ended::ValueRefused), Ended::NotKnown);
        assert_eq!(Ended::NotOffered.worst(Ended::Done), Ended::NotOffered);
        assert!(Ended::NotKnown.may_have_happened());
        assert!(!Ended::Refused.may_have_happened());
    }
}
