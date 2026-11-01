//! Which of the design's three kinds of entry each of the record's outcomes
//! is, and today's count of each — the facts the record window draws as an
//! icon, a word and a number, and never as a sentence of its own.
//!
//! `560:142` draws three kinds: **a change** (a file), **something that left**
//! (an arrow out), and **something refused** (a circle struck through). The
//! record has twenty-eight outcomes (`alo_recounting::Outcome`), so this is an
//! exhaustive match from those to the three and a fourth for everything else —
//! a question answered here, a pairing, an update — which is drawn with the
//! window's own clock on its tile and no word, since a file would say a file
//! changed. An outcome added to the record and not sorted
//! here is a build that fails, never an entry with the wrong icon.
//!
//! # A count is not a summary
//!
//! `crate::record_window` says the window words nothing, because a surface
//! able to word what happened is a machine with two accounts of one moment.
//! **The counts word nothing about any entry**: each is a number of entries of
//! one kind, today, and the number can be checked against the list under it.

use std::time::SystemTime;

use alo_recounting::{Account, Outcome};

use crate::TheDay;
use crate::record_clock::RecordClock;

/// What kind of entry the design draws an outcome as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryKind {
    /// Something of the person's, or of the machine's, was changed.
    Changed,
    /// Something left this computer.
    Left,
    /// It was asked for and did not happen.
    Refused,
    /// Anything else the record keeps: a question answered here, a pairing,
    /// an update, what the machine let go of.
    Else,
}

impl EntryKind {
    /// The kind an outcome is drawn as.
    pub(crate) const fn of(outcome: Outcome) -> Self {
        match outcome {
            Outcome::Ran
            | Outcome::RanACommand
            | Outcome::MachineChangeHandedOn
            | Outcome::PutBack => Self::Changed,
            Outcome::Left | Outcome::LeftOnItsOwn => Self::Left,
            Outcome::NobodyWasAsked
            | Outcome::ThePersonSaidNo
            | Outcome::TheGrantsSaidNo
            | Outcome::NeverBecameACall
            | Outcome::NeverPutAnywhere
            | Outcome::NotBounded
            | Outcome::ACommandWasNotRun
            | Outcome::HeldBack
            | Outcome::MachineChangeRefused
            | Outcome::NotPutBack => Self::Refused,
            Outcome::AnsweredHere
            | Outcome::AnsweredForAnotherMachine
            | Outcome::GrantsNotReadAgain
            | Outcome::Paired
            | Outcome::WorkspaceOpened
            | Outcome::Updated
            | Outcome::RolledBack
            | Outcome::CarriedOnAfterSleep
            | Outcome::StoppedBySleep
            | Outcome::LetGoOutsideTheWindow
            | Outcome::LetGoTheDiskNeededTheRoom
            | Outcome::LetGoThePersonAskedToForget => Self::Else,
        }
    }
}

/// How many of today's entries are of each counted kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct Today {
    /// Changes the person said yes to.
    pub(crate) allowed: u64,
    /// Things that left this computer.
    pub(crate) left: u64,
    /// Things refused.
    pub(crate) refused: u64,
}

impl Today {
    /// Today's counts in `account`, *today* being the person's own day as
    /// `clock` says it is.
    pub(crate) fn in_the(account: &Account, clock: RecordClock<'_>) -> Self {
        let is_today = |at: SystemTime| clock.formats.day_of(at, clock.now) == Some(TheDay::Today);
        account
            .told()
            .iter()
            .filter(|told| is_today(told.at()))
            .fold(Self::default(), |mut today, told| {
                if told.approved().is_some() {
                    today.allowed += 1;
                }
                match EntryKind::of(told.outcome()) {
                    EntryKind::Left => today.left += 1,
                    EntryKind::Refused => today.refused += 1,
                    EntryKind::Changed | EntryKind::Else => {}
                }
                today
            })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or variant is the failure being reported"
)]
mod tests {
    use super::*;

    /// **Every outcome is one kind**, and each of the design's three is
    /// reached.
    #[test]
    fn every_outcome_is_one_kind() {
        assert_eq!(EntryKind::of(Outcome::Ran), EntryKind::Changed);
        assert_eq!(EntryKind::of(Outcome::Left), EntryKind::Left);
        assert_eq!(EntryKind::of(Outcome::ThePersonSaidNo), EntryKind::Refused);
        assert_eq!(EntryKind::of(Outcome::AnsweredHere), EntryKind::Else);
    }

    /// **Today's counts are of today's entries**: the afternoon the tests keep
    /// is today at the fixture's clock, with one change approved, one question
    /// that left and the four ways it was refused.
    #[test]
    fn todays_counts_are_counted_from_the_record() {
        let kept = crate::record_testing::an_afternoon_kept();
        let mut window = crate::RecordWindow::on_an_output();
        window.opened_by_hand(&kept.recounting());
        let crate::RecordShows::Account { account, .. } = window.shows() else {
            panic!("the afternoon did not open");
        };
        let today = Today::in_the(account, crate::record_testing::a_clock());
        assert_eq!(today.allowed, 1, "{today:?}");
        assert_eq!(
            today.left, 2,
            "one asked for and one alo OS's own: {today:?}"
        );
        assert_eq!(today.refused, 4, "{today:?}");
        let refused = account
            .told()
            .iter()
            .filter(|told| EntryKind::of(told.outcome()) == EntryKind::Refused)
            .count();
        assert_eq!(u64::try_from(refused).unwrap(), today.refused);
    }
}
