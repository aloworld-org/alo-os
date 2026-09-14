//! What this machine holds about the other machines, behind one lock.
//!
//! The pairings this machine keeps and the proposals waiting on it are read
//! and changed from more than one side: the wire, on every message; the
//! person's own surface, which proposes, confirms and revokes; and a test
//! standing in for that surface. A pairing revoked on the person's side has
//! to refuse the very next verb and the very next question, and *the very
//! next* only means something if there is one list and one lock over it. So
//! there is one [`Shared`], one [`Mutex`], and the serving loop takes it for
//! exactly the length of one message.
//!
//! What is **not** here is the machine: the network's door onto it
//! (`alo_corridor::Doorway`, with the proofs it has seen) borrows the one
//! `alo_turn::Machine` the serving loop owns, in the loop's own thread, and
//! a verb from the network waits on a local turn the way `crate::serving`
//! says. The lock here is over the lists; the lock over the machine is the
//! loop.

use std::sync::{Mutex, MutexGuard, PoisonError};

use alo_nearby::{MachineId, Pairings, Proposals};

/// The pairings this machine holds and the proposals waiting on it.
#[derive(Debug)]
pub struct Shared {
    /// Every pairing two people made with this machine, until revoked or
    /// expired.
    pairings: Pairings,
    /// Every proposal waiting on this machine, either way round.
    proposals: Proposals,
}

impl Shared {
    /// The pairings.
    #[must_use]
    pub const fn pairings(&self) -> &Pairings {
        &self.pairings
    }

    /// The pairings, to keep or revoke one.
    pub const fn pairings_mut(&mut self) -> &mut Pairings {
        &mut self.pairings
    }

    /// The proposals.
    #[must_use]
    pub const fn proposals(&self) -> &Proposals {
        &self.proposals
    }

    /// The proposals, to move one along.
    pub const fn proposals_mut(&mut self) -> &mut Proposals {
        &mut self.proposals
    }

    /// Both at once, for a confirmation that completes a pairing.
    pub const fn both(&mut self) -> (&mut Proposals, &mut Pairings) {
        (&mut self.proposals, &mut self.pairings)
    }
}

/// The one lock over what this machine holds about the other machines.
#[derive(Debug)]
pub struct TheNetwork {
    /// The lists.
    shared: Mutex<Shared>,
}

impl TheNetwork {
    /// This machine, paired with nothing and with nothing waiting.
    ///
    /// Which is what every machine starts as: pairings are not yet kept
    /// anywhere between restarts, and this file says so rather than reading
    /// a list nothing wrote.
    #[must_use]
    pub fn on(here: MachineId) -> Self {
        Self {
            shared: Mutex::new(Shared {
                pairings: Pairings::none(),
                proposals: Proposals::on(here),
            }),
        }
    }

    /// Take the lock, for the length of one message or one act on the
    /// person's surface.
    ///
    /// A poisoned lock is taken anyway: what poisons it is a thread that
    /// panicked while holding it, and the lists are two `Vec`s that are
    /// either whole or the panic was in this crate's tests. Refusing every
    /// message afterwards would be a machine nobody can pair with because of
    /// a test that failed.
    pub fn locked(&self) -> MutexGuard<'_, Shared> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
