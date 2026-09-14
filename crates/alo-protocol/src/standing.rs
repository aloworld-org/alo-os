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
//!
//! # Where it came from, when it came from another machine
//!
//! ADR 0003: a change a paired machine's agent proposed waits for **this**
//! machine's person, and what they approve is the sentence — so the sentence
//! has to say whose change it is. [`Standing::from_a_machine`] stamps a change
//! with the name this machine's person gave the machine it came from, the way
//! `alo_record::Entry::from_another_machine` stamps an entry: additive, absent
//! for every change an agent on this machine proposed, and a reader that has
//! never heard of it reads a message without it exactly as before.

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
    /// The machine this change was proposed from, when it was proposed from
    /// another one, by the name this machine's person gave it.
    ///
    /// Absent — not present and empty — for every change proposed on this
    /// machine, so a message written before there was such a thing as a
    /// remote turn reads back exactly as it was written.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    from: Option<String>,
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
            from: None,
        }
    }

    /// This change, as one a paired machine proposed (ADR 0003).
    ///
    /// `machine` is the name this machine's person gave the other when they
    /// paired with it — their own word for it. A stamp rather than a
    /// constructor, on purpose: the number and the sentence were decided by
    /// [`Standing::of`] from the same values a local change would have handed
    /// over, and what a remote turn can add is where it came from and nothing
    /// beside it. The name is not an authority: the number is still found
    /// among what is really waiting, by the turn, when it is answered.
    #[must_use]
    pub fn from_a_machine(mut self, machine: &str) -> Self {
        self.from = Some(machine.to_owned());
        self
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

    /// Which machine this change was proposed from, when it was proposed from
    /// another one — `None` for a change an agent on this machine proposed.
    #[must_use]
    pub fn from(&self) -> Option<&str> {
        self.from.as_deref()
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

    /// **A change from a paired machine says which machine**, by the name the
    /// person gave it; a change from this machine says nothing, and a message
    /// written without the field reads back as one from this machine.
    #[test]
    fn a_change_from_a_paired_machine_says_which_and_a_local_one_says_nothing() {
        let (approvals, strings) = a_change_waiting();
        let local = Standing::of(the_change(&approvals), &strings, the_moment());
        assert_eq!(local.from(), None);
        let written = serde_json::to_string(&local).unwrap();
        assert!(!written.contains(r#""from""#), "{written}");

        let remote = local.clone().from_a_machine("the reception machine");
        assert_eq!(remote.from(), Some("the reception machine"));
        assert_eq!(remote.number(), local.number());
        assert_eq!(remote.sentence(), local.sentence());
        let written = serde_json::to_string(&remote).unwrap();
        let back: Standing = serde_json::from_str(&written).unwrap();
        assert_eq!(back, remote);

        let older =
            r#"{"number":7,"sentence":{"text":"…","came_from":"the-source"},"lapses_in":300}"#;
        let back: Standing = serde_json::from_str(older).unwrap();
        assert_eq!(back.from(), None);
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
