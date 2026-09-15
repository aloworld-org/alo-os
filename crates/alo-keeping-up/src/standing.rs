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
//! **An update carries nothing else.** [`Ready`] holds two digests and has no
//! room for a priority, a severity or a deadline. Each of those is a lever
//! somebody pulls to justify interrupting a person, and the lever is easier to
//! leave out than to hold shut.

use alo_strings::{Filling, Said, Strings};
use serde::Serialize;

use crate::checking::Offered;
use crate::digest::Digest;
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

    /// An update between two builds, for this crate's own unit tests, which
    /// cannot make an indicator. Not compiled into the crate.
    #[cfg(test)]
    pub(crate) fn for_a_test(running: Digest, offered: Digest) -> Self {
        Self { running, offered }
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
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let word = match self {
            Self::UpToDate => words::UP_TO_DATE,
            Self::Ready(_) => words::READY,
        };
        strings.say(&word.key(), &Filling::nothing())
    }
}
