//! Whether there is an update: the build running, the build offered, and
//! whether they differ.
//!
//! **Differ, not newer.** Digests have no order, and the place updates come
//! from is the authority on which build this machine should be running — so an
//! offer that differs is an update, including one that is the build before.
//! Whether an offer can be trusted at all is its signature's question (ADR
//! 0036), answered where it is fetched and applied, not by guessing at order
//! here.
//!
//! **An update carries nothing else.** [`Ready`] holds two digests and
//! whether anybody has vouched for the one offered, and has no room for a
//! priority, a severity or a deadline. Each of those is a lever somebody pulls
//! to justify interrupting a person, and the lever is easier to leave out than
//! to hold shut. What it does carry is the opposite kind of thing: a doubt the
//! person is told **before** they choose rather than after
//! ([`crate::Vouching`]).

use alo_strings::{Filling, Said, Strings};
use serde::Serialize;

use crate::checking::Offered;
use crate::digest::Digest;
use crate::vouching::Vouching;
use crate::words;

/// The build this machine is running.
///
/// Made from whatever reported it — the base's own status, in the task that
/// reads it. Reading it decides nothing; it is a name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Running {
    /// The build running.
    digest: Digest,
}

impl Running {
    /// The build this machine reports it is running.
    #[must_use]
    pub fn reported(digest: Digest) -> Self {
        Self { digest }
    }

    /// The build running.
    #[must_use]
    pub fn digest(&self) -> &Digest {
        &self.digest
    }
}

/// An update this machine could apply: the build it runs, and a different one
/// on offer.
///
/// Made only by [`Standing::between`], so a `Ready` whose two builds are the
/// same cannot exist. Serialises so it can be shown or written down, and does
/// not deserialise, for [`Offered`]'s reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Ready {
    /// The build this machine runs now.
    running: Digest,
    /// The build it would run after the update applies.
    offered: Digest,
    /// Whether the place offering it has vouched for it.
    vouching: Vouching,
}

impl Ready {
    /// The build this machine runs now.
    #[must_use]
    pub fn running(&self) -> &Digest {
        &self.running
    }

    /// The build it would run after the update applies.
    #[must_use]
    pub fn offered(&self) -> &Digest {
        &self.offered
    }

    /// Whether the place offering it has vouched for it.
    ///
    /// Carried so that the sentence a person reads before they choose says it
    /// ([`Standing::said`]). It is **not** the machine's signature policy's
    /// answer, and nothing here stages or refuses to stage anything on the
    /// strength of it — [`crate::vouching`] says why that line is drawn where
    /// it is.
    #[must_use]
    pub fn vouching(&self) -> Vouching {
        self.vouching
    }

    /// An update between two builds, for this crate's own unit tests, which
    /// cannot make an indicator. Not compiled into the crate.
    #[cfg(test)]
    pub(crate) fn for_a_test(running: Digest, offered: Digest, vouching: Vouching) -> Self {
        Self {
            running,
            offered,
            vouching,
        }
    }
}

/// Where this machine stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "standing")]
pub enum Standing {
    /// The build offered is the build running.
    UpToDate,
    /// A different build is offered.
    Ready(Ready),
}

impl Standing {
    /// Where this machine stands, from what it runs and what is offered.
    #[must_use]
    pub fn between(running: &Running, offered: &Offered) -> Self {
        if running.digest() == offered.digest() {
            Self::UpToDate
        } else {
            Self::Ready(Ready {
                running: running.digest().clone(),
                offered: offered.digest().clone(),
                vouching: offered.vouching(),
            })
        }
    }

    /// Whether there is an update.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }

    /// What a person is told.
    ///
    /// When an update is ready: that it is, and that it will apply when they
    /// choose. Never which build, and never *when it will be forced*, because
    /// there is no such moment.
    ///
    /// **An update nothing has vouched for is a different sentence**, and it
    /// is said here rather than after the person chooses, because the two
    /// sentences are about two different decisions. The ordinary one promises
    /// the update applies when they choose; the other one cannot promise that,
    /// and a person who is going to learn it should learn it while the choice
    /// is still in front of them (`crate::vouching`).
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let word = match self {
            Self::UpToDate => words::UP_TO_DATE,
            Self::Ready(ready) if ready.vouching().is_vouched_for() => words::READY,
            Self::Ready(_) => words::READY_NOT_VOUCHED_FOR,
        };
        strings.say(&word.key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, ready_between, ready_between_unvouched};

    /// **An update nobody has vouched for is its own sentence**, said in place
    /// of *an update is ready* and before the person is offered a choice.
    #[test]
    fn an_update_nobody_has_vouched_for_reads_differently_from_one_that_is() {
        let strings = in_english();
        let vouched = Standing::Ready(ready_between("aa", "bb")).said(&strings);
        let not = Standing::Ready(ready_between_unvouched("aa", "bb")).said(&strings);

        assert!(!vouched.is_a_bug(), "{vouched}");
        assert!(!not.is_a_bug(), "{not}");
        assert_ne!(vouched.text(), not.text());
        assert!(not.text().contains("cannot confirm"), "{not}");
        assert!(
            !not.text().contains("An update is ready"),
            "an update nothing vouches for is offered as though it were: {not}"
        );
    }

    /// The two builds and the vouching travel together: an update is the
    /// difference **and** what is known about the build offered.
    #[test]
    fn an_update_carries_whether_anybody_vouched_for_the_build_offered() {
        let ready = ready_between_unvouched("aa", "bb");
        assert_eq!(ready.vouching(), Vouching::NobodyHasVouchedForIt);
        assert!(!ready.vouching().is_vouched_for());
        assert_eq!(
            ready_between("aa", "bb").vouching(),
            Vouching::ThePlaceVouchesForIt
        );
    }

    /// **Written down, it says which**, so anything that shows an update
    /// afterwards can say the same thing the sentence says.
    #[test]
    fn an_update_written_down_says_whether_anybody_vouched_for_it() {
        let written = serde_json::to_string(&Standing::Ready(ready_between_unvouched("aa", "bb")))
            .unwrap_or_default();
        assert!(written.contains("nobody-has-vouched-for-it"), "{written}");
    }
}
