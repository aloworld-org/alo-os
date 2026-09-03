//! One change waiting for a person to answer it.
//!
//! The number, the sentence they are being asked, and how long is left. It is
//! what comes back when an agent proposes something, and what a shell draws
//! when it asks what is waiting — one shape rather than two, because it is one
//! question read from two sides.
//!
//! # The sentence is not optional and there is no shape without it
//!
//! ADR 0001 §5: what a person approves is a sentence. A wire that carried the
//! number alone would be a wire where a shell could offer *approve change 7*,
//! and an approval of a number nobody read out is the thing the whole
//! capability model exists to prevent. So [`Standing::of`] takes the vocabulary
//! and renders the proposal's own sentence, and there is no constructor that
//! takes a number by itself.
//!
//! # How long is left is a duration, and never a moment
//!
//! `docs/contracts/daemon-protocol.md` says nothing on the wire names a moment,
//! and that rule is about requests: a request naming one could revive a grant
//! that expired an hour ago. It is kept here anyway, for a second reason that
//! is the client's rather than the machine's — two ends of a socket have two
//! clocks, and a shell told *this lapses at 14:05* would be a shell drawing a
//! countdown against a clock that is not the one the daemon will answer by.
//! A duration is the same fact with nothing to disagree about.

use std::time::SystemTime;

use alo_capability::Waiting;
use alo_strings::Strings;
use serde::{Deserialize, Serialize};

use crate::wording::Wording;

/// A change put to a person, as a client is told about it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Standing {
    /// The number to answer it by.
    number: u64,
    /// What the person is being asked, in the language they read.
    sentence: Wording,
    /// How many seconds are left to answer, and nothing once the question has
    /// stopped standing.
    lapses_in: Option<u64>,
}

impl Standing {
    /// A change waiting, as it goes on the wire.
    ///
    /// `now` is the machine's moment, as everywhere else in this workspace:
    /// nothing here reads a clock, so what a shell is told about the time left
    /// is measured against the same moment the daemon will answer by.
    #[must_use]
    pub fn of(waiting: &Waiting, strings: &Strings, now: SystemTime) -> Self {
        Self {
            number: waiting.id.as_u64(),
            sentence: Wording::of(&waiting.proposal.sentence(strings)),
            lapses_in: waiting
                .proposal
                .lapses_in(now)
                .map(|left| left.as_secs().max(1)),
        }
    }

    /// The number to answer it by.
    ///
    /// A number and not a handle, on the way back as on the way in: what it
    /// names is found among the changes actually waiting, by the turn, at the
    /// moment it is answered.
    #[must_use]
    pub fn number(&self) -> u64 {
        self.number
    }

    /// What the person is being asked.
    #[must_use]
    pub fn sentence(&self) -> &Wording {
        &self.sentence
    }

    /// How many seconds are left to answer, and nothing once it has lapsed.
    #[must_use]
    pub fn lapses_in(&self) -> Option<u64> {
        self.lapses_in
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_change_waiting, an_hour_in, the_change, the_moment};
    use crate::wording::CameFrom;

    /// **A change waiting carries the sentence it is waiting on.** A number on
    /// its own would be a shell asking somebody to approve *change 7*.
    #[test]
    fn a_change_waiting_carries_the_sentence_the_person_is_being_asked() {
        let (approvals, strings) = a_change_waiting();
        let waiting = the_change(&approvals);
        let standing = Standing::of(waiting, &strings, the_moment());
        assert_eq!(standing.number(), waiting.id.as_u64());
        assert!(
            standing.sentence().text().contains("march-final.pdf"),
            "{standing:?}"
        );
        assert_eq!(standing.sentence().came_from(), CameFrom::TheSource);
    }

    /// The seconds left are counted from the machine's moment, and a question
    /// that has stopped standing carries nothing rather than a zero somebody
    /// would draw as *no time at all*.
    #[test]
    fn how_long_is_left_is_a_duration_and_nothing_once_it_has_lapsed() {
        let (approvals, strings) = a_change_waiting();
        let waiting = the_change(&approvals);
        let standing = Standing::of(waiting, &strings, the_moment());
        assert_eq!(standing.lapses_in(), Some(300));

        let lapsed = Standing::of(waiting, &strings, an_hour_in());
        assert_eq!(lapsed.lapses_in(), None);
    }

    /// Nothing here reads a clock, so the same change at the same moment is the
    /// same answer however many times it is asked for.
    #[test]
    fn nothing_here_reads_a_clock() {
        let (approvals, strings) = a_change_waiting();
        let waiting = the_change(&approvals);
        assert_eq!(
            Standing::of(waiting, &strings, the_moment()),
            Standing::of(waiting, &strings, the_moment())
        );
    }

    /// What is written is what is read back.
    #[test]
    fn a_change_waiting_reads_back_as_what_was_written() {
        let (approvals, strings) = a_change_waiting();
        let standing = Standing::of(the_change(&approvals), &strings, the_moment());
        let written = serde_json::to_string(&standing).unwrap();
        let back: Standing = serde_json::from_str(&written).unwrap();
        assert_eq!(back, standing);
    }

    /// The sentence is one this machine really declares, so what crosses is a
    /// sentence rather than a key — which is the thing a client cannot check
    /// for the daemon.
    #[test]
    fn the_sentence_that_crosses_is_one_the_machine_declares() {
        let (approvals, strings) = a_change_waiting();
        let standing = Standing::of(the_change(&approvals), &strings, the_moment());
        assert!(!standing.sentence().is_a_bug(), "{standing:?}");
    }
}
