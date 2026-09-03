//! A question that was sent and not answered, and the departure it left with.
//!
//! [`crate::asked`]'s twin, and it exists because of the half of law 1 that is
//! easiest to lose: **a question that failed still left the machine.** The bytes
//! went out, the indicator showed them going, and the record has to be able to
//! say so afterwards — a machine that wrote down only the questions that were
//! answered would report a quieter day than it had.
//!
//! So the departure comes back on this path exactly as it does on the other:
//! *the authorisation comes back either way* (`alo-files`, item 6a), about a
//! departure.
//!
//! ```text
//! Err(NotAsked::DidNotAnswer(unanswered)) => {
//!     record.keep(Entry::left(unanswered.departing()));
//!     let failed = unanswered.ended(&mut indicator);
//!     // …and `failed` is `alo-answering`'s from here: what the person is told,
//!     //  and the offer they may answer. Nothing asks anything on its own.
//! }
//! ```
//!
//! # What this file deliberately does not have
//!
//! **No way back to another attempt.** Holding one of these permits nothing:
//! the only thing inside it is an `alo_answering::Failed`, whose only door is an
//! offer a person answered. That is ADR 0008's *never a silent fallback*, and
//! the shape of the guarantee is that a second attempt would need a whole type
//! that does not exist rather than a second line in this file.

use alo_answering::Failed;
use alo_egress::{Departing, Indicator};

/// A question that was put somewhere and did not come back with an answer.
///
/// Not `Clone`: neither a `Departing` nor a `Failed` is, and for one reason
/// between them — a copy of either would be a second go at something that is
/// worth exactly one.
#[derive(Debug)]
pub struct DidNotAnswer {
    /// What left, still on the indicator and still to be written down.
    departing: Departing,
    /// What the person may be told, and what they may be asked.
    failed: Failed,
}

impl DidNotAnswer {
    /// Made by [`crate::Asking::to_a_provider`] and by nothing else.
    pub(crate) fn new(departing: Departing, failed: Failed) -> Self {
        Self { departing, failed }
    }

    /// What left this machine — for `alo_record::Entry::left`, because a
    /// question that failed left it just the same.
    #[must_use]
    pub fn departing(&self) -> &Departing {
        &self.departing
    }

    /// What went wrong, before the line comes off the indicator.
    #[must_use]
    pub fn failed(&self) -> &Failed {
        &self.failed
    }

    /// Take the line off the indicator, and keep the failure.
    #[must_use]
    pub fn ended(self, indicator: &mut Indicator) -> Failed {
        indicator.ended(self.departing);
        self.failed
    }
}
