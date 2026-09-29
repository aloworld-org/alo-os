//! What the record says about a period, gathered once and stated.
//!
//! Every field here is counted from [`alo_record::Entry`]s whose own `at` falls
//! inside the [`Period`] — never from when the statement was made, and never from
//! anything this crate decides for itself.
//!
//! # What it cannot attest, said in the statement rather than left to a reader
//!
//! An attestation that can only say what it found is half an artifact. This one
//! also carries [`Statement::entries_outside`]: how many entries the record held
//! that the period did not cover. A statement about a Tuesday, rendered from a
//! record that begins on Wednesday, is *true and about nothing*, and the only way
//! a reader can tell that from *a quiet Tuesday* is if the statement says how much
//! it looked past.
//!
//! That is the same rule as the skip count in this repository's gates: a number
//! nobody can see is a number nobody can weigh, and *nothing happened* and
//! *nothing was looked at* must not arrive in the same colour.

use std::time::SystemTime;

use alo_egress::{Destination, Errand, Why};
use alo_record::{Entry, Happened};

use crate::Period;

/// One departure, as the statement will render it.
///
/// The fields are the record's own — nothing is renamed on the way through,
/// because a second name for a destination is a second thing that can drift.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Departure {
    /// When it left.
    pub at: SystemTime,

    /// Whose authority it left under, as the record wrote it.
    pub agent: String,

    /// Where it went.
    pub destination: Destination,

    /// Why it left.
    pub why: Why,
}

/// One thing the egress policy refused to let leave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldBack {
    /// When it was refused.
    pub at: SystemTime,

    /// Whose authority it would have left under.
    pub agent: String,

    /// Where it would have gone.
    pub destination: Destination,

    /// Why it would have left.
    pub why: Why,

    /// Why it was not permitted, in the policy's own words.
    pub refused: String,
}

/// One time alo OS reached the network with nobody having asked it to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnItsOwn {
    /// When it reached.
    pub at: SystemTime,

    /// Which of the errands it was.
    pub errand: Errand,

    /// Where it reached.
    pub destination: Destination,
}

/// What the record says about one period.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Statement {
    /// The period this is about.
    period: Period,

    /// Everything that left, oldest first.
    departures: Vec<Departure>,

    /// Everything the policy refused, oldest first.
    held_back: Vec<HeldBack>,

    /// Everything alo OS reached on its own, oldest first.
    on_its_own: Vec<OnItsOwn>,

    /// How many questions were answered on this machine and never left.
    answered_here: usize,

    /// How many entries the record held that this period does not cover.
    entries_outside: usize,
}

impl Statement {
    /// Gather everything the record says about `period`.
    ///
    /// Entries are taken by their own `at`, and each list comes out **oldest
    /// first** — fixed here rather than inherited from whatever the record
    /// iterates in, because two auditors rendering one period must get one set of
    /// bytes.
    #[must_use]
    pub fn of<'a>(period: Period, entries: impl Iterator<Item = &'a Entry>) -> Self {
        let mut departures = Vec::new();
        let mut held_back = Vec::new();
        let mut on_its_own = Vec::new();
        let mut answered_here = 0_usize;
        let mut entries_outside = 0_usize;

        for entry in entries {
            if !period.holds(entry.at()) {
                entries_outside += 1;
                continue;
            }
            match entry.happened() {
                Happened::Left {
                    agent,
                    destination,
                    why,
                } => departures.push(Departure {
                    at: entry.at(),
                    agent: agent.as_str().to_owned(),
                    destination: destination.clone(),
                    why: *why,
                }),
                Happened::HeldBack {
                    agent,
                    destination,
                    why,
                    refused,
                } => held_back.push(HeldBack {
                    at: entry.at(),
                    agent: agent.as_str().to_owned(),
                    destination: destination.clone(),
                    why: *why,
                    refused: refused.as_str().to_owned(),
                }),
                Happened::LeftOnItsOwn {
                    errand,
                    destination,
                } => on_its_own.push(OnItsOwn {
                    at: entry.at(),
                    errand: *errand,
                    destination: destination.clone(),
                }),
                Happened::AnsweredHere { .. } => answered_here += 1,
                _ => {}
            }
        }

        departures.sort_by_key(|one| one.at);
        held_back.sort_by_key(|one| one.at);
        on_its_own.sort_by_key(|one| one.at);

        Self {
            period,
            departures,
            held_back,
            on_its_own,
            answered_here,
            entries_outside,
        }
    }

    /// The period it is about.
    #[must_use]
    pub fn period(&self) -> Period {
        self.period
    }

    /// Everything that left, oldest first.
    #[must_use]
    pub fn departures(&self) -> &[Departure] {
        &self.departures
    }

    /// Everything the policy refused to let leave, oldest first.
    #[must_use]
    pub fn held_back(&self) -> &[HeldBack] {
        &self.held_back
    }

    /// Everything alo OS reached on its own, oldest first.
    #[must_use]
    pub fn on_its_own(&self) -> &[OnItsOwn] {
        &self.on_its_own
    }

    /// How many questions were answered on this machine and never left.
    #[must_use]
    pub fn answered_here(&self) -> usize {
        self.answered_here
    }

    /// How many of the record's entries this period does not cover.
    ///
    /// Not a fault. It is what lets a reader tell *nothing happened* from
    /// *nothing was looked at*.
    #[must_use]
    pub fn entries_outside(&self) -> usize {
        self.entries_outside
    }

    /// How many departures were a question put to a model somewhere else.
    ///
    /// **This is the number law 1 promises to publish.** *With a local model a
    /// working day produces zero inference egress, measured at the network
    /// boundary — and we publish the measurement rather than the promise.* Zero
    /// here, beside a non-zero [`Self::answered_here`], is that promise kept on
    /// this machine over this period, as a fact rather than a sentence.
    #[must_use]
    pub fn inference_egress(&self) -> usize {
        self.departures
            .iter()
            .filter(|one| matches!(one.why, Why::Asking))
            .count()
    }
}
