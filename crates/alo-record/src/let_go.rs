//! What the machine let go of, so that a turn can no longer be put back.
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md)'s first
//! two accepted terms both end in a snapshot being **removed** — the window no
//! longer reaches it, or the disk needed the room — and both say the same thing
//! about what a person is owed afterwards: *the record says those turns can no
//! longer be undone*, naming the turns rather than counting them. This module
//! is that sentence's two halves as types, and
//! [`crate::Happened::LetGo`] is where they are kept.
//!
//! # Why the turns are copied in rather than pointed at
//!
//! For [`crate::Happened::Undone`]'s reason, which is the record's reason
//! generally: the file is appended to and shortened
//! (`crates/alo-keeping/src/pruning.rs`), so a position in it is not a name
//! that lasts, and an entry that pointed at one would become a line about
//! nothing the day the original was pruned. Each [`Forgone`] therefore carries
//! the moment the turn ran and **the sentence the person approved at the time**,
//! copied from what the machine kept beside the snapshot it is removing.
//!
//! # And why it is words rather than a number
//!
//! The owner's second term says it outright: *what a person is told names the
//! turns that lost it rather than a number*. **Four undos expired** is a
//! sentence nobody can act on and nobody can check; *the folder you asked the
//! agent to tidy on Tuesday* is one they recognise. So there is no count in
//! this module, and [`Forgone::did`] is the only thing in it a person reads.

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::line::Line;

/// Why the machine let go of what it was keeping.
///
/// The two reasons a machine of its own accord stops being able to undo a turn.
/// A person forgetting everything an undo could put back
/// (`alo_keeping_up::WhatWasKept::forgetting`) is a third and is **not** here:
/// it is the person's own deliberate act, written down when something builds
/// it, and adding a member for it then is additive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WhyLetGo {
    /// How far back an undo reaches no longer reaches this turn
    /// (`alo_keeping_up::HowFarBack`, ADR 0045's first term).
    OutsideTheWindow,
    /// The disk was below the amount of free space the machine keeps, so the
    /// oldest went first (ADR 0045's second term). **The machine never fills a
    /// disk to preserve an undo.**
    TheDiskNeededTheRoom,
}

/// One turn that can no longer be put back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Forgone {
    /// The moment the turn ran.
    done: SystemTime,
    /// What it did, in the words the person approved when it ran.
    did: Line,
}

impl WhyLetGo {
    /// Both of them, for a test that walks the pair.
    pub const EVERY: [Self; 2] = [Self::OutsideTheWindow, Self::TheDiskNeededTheRoom];
}

impl Forgone {
    /// A turn that ran at `done`, described by the sentence the person
    /// approved — [`None`] for a sentence with nothing in it.
    ///
    /// A blank sentence is refused rather than kept, because an entry naming a
    /// turn a person cannot recognise keeps the term's letter and loses the
    /// whole of what it was for.
    #[must_use]
    pub fn of(done: SystemTime, did: &str) -> Option<Self> {
        let did = did.trim();
        (!did.is_empty()).then(|| Self {
            done,
            did: Line::of(did),
        })
    }

    /// The moment the turn ran.
    #[must_use]
    pub const fn done(&self) -> SystemTime {
        self.done
    }

    /// What it did, in the words the person approved.
    #[must_use]
    pub const fn did(&self) -> &Line {
        &self.did
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// A moment far enough from the epoch to read as a real one.
    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// **A turn is kept by its moment and the words the person approved.**
    #[test]
    fn a_turn_is_its_moment_and_the_sentence_the_person_approved() {
        let forgone = Forgone::of(noon(), "  move March.pdf into Invoices  ").unwrap();
        assert_eq!(forgone.done(), noon());
        assert!(forgone.did().is("move March.pdf into Invoices"));
    }

    /// **A turn with nothing to say about it is not written down**, because the
    /// whole of the second term is that a person is told *which* turns lost
    /// their undo.
    #[test]
    fn a_turn_with_no_sentence_is_refused() {
        assert_eq!(Forgone::of(noon(), ""), None);
        assert_eq!(Forgone::of(noon(), "   \n "), None);
    }

    /// **Both reasons read back as themselves**, in the words the file spells
    /// them with.
    #[test]
    fn both_reasons_are_written_and_read_the_way_the_file_spells_them() {
        for why in WhyLetGo::EVERY {
            let written = serde_json::to_string(&why).unwrap();
            assert_eq!(serde_json::from_str::<WhyLetGo>(&written).unwrap(), why);
        }
        assert_eq!(
            serde_json::to_string(&WhyLetGo::OutsideTheWindow).unwrap(),
            r#""outside-the-window""#
        );
        assert_eq!(
            serde_json::to_string(&WhyLetGo::TheDiskNeededTheRoom).unwrap(),
            r#""the-disk-needed-the-room""#
        );
    }
}
