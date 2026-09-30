//! What undo is holding, as its own line beside what is filling the disk.
//!
//! `docs/decisions/0045-what-undoing-rewinds-to.md`'s **fourth accepted term**
//! says *what is filling the disk counts snapshots, by name*. A person who opens *what is filling my disk* because the disk is full
//! is exactly the person whose disk is full of yesterday's turns, and **an
//! answer that hid it would send them hunting for space the machine itself was
//! keeping.**
//!
//! # A line, and never a node in the tree
//!
//! Every [`crate::Node`] promises one thing about its size: it is *the sum of
//! its children plus its own bytes*. A snapshot cannot keep that promise. Its
//! bytes are not inside the folder a person opened, and on `btrfs` they are
//! shared with the live files by copy-on-write — so a node for a snapshot would
//! be a claim about where the bytes are that is not true, and would count every
//! unchanged file twice.
//!
//! So this sits **beside** the tree, on [`crate::Holding`], and no size in the
//! tree moves by a byte because of it. Nothing here walks into a snapshot.
//!
//! # Three answers, and never a zero
//!
//! [`crate::Number`] is *a number or a sentence, and never a zero standing in
//! for a sentence*, and this needs the same honesty with one arm that one does
//! not have:
//!
//! - **holding** — the bytes, and where they were read;
//! - **not on this machine** — ADR 0045's sixth term. A machine installed on a
//!   filesystem without snapshots keeps no undo, and says so *until it is
//!   reinstalled*, because a filesystem is chosen at install and cannot be
//!   converted afterwards. **A zero here would be a lie**: zero says undo is
//!   holding nothing, and the truth is that this machine cannot hold one.
//!   **Every machine this repository has answers this today**, so it is the arm
//!   the first reader will see;
//! - **asked and not answered** — what the machine said, never a zero, for the
//!   same reason [`crate::Number`] never reports one.
//!
//! # Told, never fetched
//!
//! This crate does not know what undo kept and must not learn: `alo-letting-go`
//! is the crate that does, and depending on it would put a housekeeping road
//! inside a crate an agent's turn reaches. So the seam is the one this crate
//! already uses for the kernel — [`crate::Reading::of_kernel`] is *told* where
//! to read — and [`WhatUndoIsHolding`] is a trait this crate declares and
//! something outside it answers.
//!
//! [`crate::Holding::of`] **takes an answer rather than defaulting one**. A
//! caller that could say what is filling the disk without saying anything about
//! undo is a caller that eventually will, and the fourth term would become a
//! thing somebody has to remember. Taking it as an argument is what makes the
//! term structural: the compiler asks, not a test.

use alo_strings::{Filling, Said, Strings};

use crate::source::Source;
use crate::words;

/// What undo is holding on this machine — or why there is no number.
///
/// Never a zero standing in for a sentence. See the module documentation for
/// why each arm exists and which one a machine here answers today.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum UndoIsHolding {
    /// Undo is holding this many bytes, read from here.
    Bytes {
        /// The bytes the snapshots are keeping.
        value: u64,
        /// Where that was read, so a person can check it against the machine
        /// the way they can check every other number in this crate.
        from: Source,
    },

    /// This machine keeps no undo, and cannot until it is reinstalled.
    ///
    /// ADR 0045's sixth term: a snapshot needs a filesystem that has them, and
    /// a filesystem is chosen at install. Not a fault and emphatically not a
    /// zero — the machine is not holding nothing, it is unable to hold
    /// anything.
    NotOnThisMachine,

    /// Something was asked and did not answer.
    ///
    /// Never a zero: a machine that would not say how much it is keeping may
    /// be keeping a great deal.
    NotAnswered {
        /// What the machine said, as a sentence.
        why: String,
    },
}

impl UndoIsHolding {
    /// The bytes, if there is a number.
    ///
    /// [`None`] for both of the other arms, which are sentences rather than
    /// numbers — and neither is a zero.
    #[must_use]
    pub const fn value(&self) -> Option<u64> {
        match self {
            Self::Bytes { value, .. } => Some(*value),
            Self::NotOnThisMachine | Self::NotAnswered { .. } => None,
        }
    }

    /// Where the bytes were read, when there are bytes.
    #[must_use]
    pub const fn from(&self) -> Option<&Source> {
        match self {
            Self::Bytes { from, .. } => Some(from),
            Self::NotOnThisMachine | Self::NotAnswered { .. } => None,
        }
    }

    /// What a window shows in place of the number, in the reader's language.
    ///
    /// [`None`] when there are bytes: a number is shown as itself and needs no
    /// sentence, exactly as in [`crate::Number::instead`].
    #[must_use]
    pub fn instead(&self, strings: &Strings) -> Option<Said> {
        let word = match self {
            Self::Bytes { .. } => return None,
            Self::NotOnThisMachine => &words::UNDO_NOT_ON_THIS_MACHINE,
            Self::NotAnswered { .. } => &words::UNDO_NOT_ANSWERED,
        };
        Some(strings.say(&word.key(), &Filling::nothing()))
    }
}

/// Something that can say what undo is holding.
///
/// Declared here and answered outside, so that this crate never learns what
/// undo kept. See the module documentation on the seam.
pub trait WhatUndoIsHolding {
    /// What undo is holding, asked now.
    fn what_undo_is_holding(&self) -> UndoIsHolding;
}

/// The answer every machine this repository has gives today.
///
/// A person's home is not a subvolume on any machine here, so there is nothing
/// for a snapshot to be of. This is not a placeholder to be filled in later and
/// forgotten: it is the true answer, and it stops being the one to pass when a
/// machine can actually hold an undo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NoUndoHere;

impl WhatUndoIsHolding for NoUndoHere {
    fn what_undo_is_holding(&self) -> UndoIsHolding {
        UndoIsHolding::NotOnThisMachine
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// English, with nothing translated.
    fn in_english() -> Strings {
        Strings::of(crate::measuring_words().unwrap())
    }

    /// **Bytes are a number and need no sentence; the other two are sentences
    /// and have no number.** Nothing here is ever both, and — the point of the
    /// whole type — nothing is ever a zero standing in for a sentence.
    #[test]
    fn three_states_and_never_a_zero() {
        let holding = UndoIsHolding::Bytes {
            value: 4_096,
            from: Source::of("/var/home/.snapshots", "undo"),
        };
        assert_eq!(holding.value(), Some(4_096));
        assert!(holding.instead(&in_english()).is_none());
        assert!(holding.from().is_some());

        for sentence in [
            UndoIsHolding::NotOnThisMachine,
            UndoIsHolding::NotAnswered {
                why: "the machine did not answer".to_owned(),
            },
        ] {
            assert_eq!(
                sentence.value(),
                None,
                "a sentence must not also be a number"
            );
            assert!(sentence.from().is_none());
            let said = sentence.instead(&in_english()).unwrap();
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
        }
    }

    /// **Not on this machine is what every machine here answers**, and it is
    /// distinguishable from holding nothing. A zero would not be.
    #[test]
    fn this_machine_says_it_cannot_hold_one_rather_than_that_it_holds_none() {
        let answered = NoUndoHere.what_undo_is_holding();
        assert_eq!(answered, UndoIsHolding::NotOnThisMachine);
        assert_eq!(answered.value(), None);

        let none_at_all = UndoIsHolding::Bytes {
            value: 0,
            from: Source::of("/var/home/.snapshots", "undo"),
        };
        assert_ne!(
            answered, none_at_all,
            "cannot-hold-one and holds-nothing must not be the same answer"
        );
        assert_eq!(none_at_all.value(), Some(0));
    }
}
