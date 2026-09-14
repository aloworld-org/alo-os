//! What this machine holds about the other machines, behind one lock — and
//! where it is written down.
//!
//! The pairings this machine keeps and the proposals waiting on it are read
//! and changed from more than one side: the wire, on every message; the
//! person's own door, which proposes, confirms and revokes
//! (`crate::pairing`); and a test standing in for that door. A pairing
//! revoked on the person's side has to refuse the very next verb and the very
//! next question, and *the very next* only means something if there is one
//! list and one lock over it. So there is one [`Shared`], one [`Mutex`], and
//! the serving loop takes it for exactly the length of one message.
//!
//! # A pairing kept is written down, at the moment
//!
//! Since the person's door learned to pair, the pairings outlive a restart:
//! [`TheNetwork::remembering`] starts from what `alo-remembering` read back
//! off the disk, and [`Shared::written_down`] writes the list whole through
//! the one [`KeepingPairings`] the process was handed — at the moment a
//! pairing is kept or revoked, by whichever side did it. The file is written
//! **after** the list in memory changed, never instead of it: a revocation
//! takes effect at once whether or not the disk agrees, and a file that could
//! not be written is said to the person as *until a restart* rather than
//! reversing what they did. Where the file is, is `src/main.rs`'s alone; what
//! travels below it is this trait, which can write the list and read nothing.
//!
//! What is **not** here is the machine: the network's door onto it
//! (`alo_corridor::Doorway`, with the proofs it has seen) borrows the one
//! `alo_turn::Machine` the serving loop owns, in the loop's own thread, and
//! a verb from the network waits on a local turn the way `crate::serving`
//! says. The lock here is over the lists; the lock over the machine is the
//! loop.

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::SystemTime;

use alo_nearby::{MachineId, Pairings, Proposals};
use alo_remembering::NotRemembered;

/// Somewhere this machine's pairings are written down between restarts.
///
/// One method, and it writes the whole list. There is deliberately no way to
/// read through it — what was read is handed to [`TheNetwork::remembering`]
/// as a value by `src/main.rs`, once, before the socket exists — and no way to
/// name a path: [`crate::keeping_pairings::ThePairingsFile`] is the only
/// implementation that ships, and it holds the one path `main` gave it.
///
/// `Send + Sync` because [`TheNetwork`] is shared with the person's door and
/// with a test's client thread, and `Debug` for the reason every trait object
/// in this crate is.
pub trait KeepingPairings: Send + Sync + std::fmt::Debug {
    /// Write what is paired at this moment, whole.
    ///
    /// # Errors
    ///
    /// [`NotRemembered`], carried whole from `alo-remembering` rather than
    /// reworded: whoever reads it is whoever is standing the machine up, and
    /// that crate already names the file and what was wrong with it.
    fn keep(&self, pairings: &Pairings, now: SystemTime) -> Result<(), NotRemembered>;
}

/// Nowhere: the pairings are held for the session and written to nothing.
///
/// What a test hands in when it is not about the file, and what
/// [`TheNetwork::on`] uses. Nothing a machine runs is handed this —
/// `crate::starting` takes the file from `main`.
#[derive(Debug, Clone, Copy, Default)]
pub struct NothingKeepsPairings;

impl KeepingPairings for NothingKeepsPairings {
    fn keep(&self, _pairings: &Pairings, _now: SystemTime) -> Result<(), NotRemembered> {
        Ok(())
    }
}

/// The pairings this machine holds and the proposals waiting on it.
#[derive(Debug)]
pub struct Shared {
    /// Every pairing two people made with this machine, until revoked or
    /// expired.
    pairings: Pairings,
    /// Every proposal waiting on this machine, either way round.
    proposals: Proposals,
    /// Where the pairings are written down.
    keeping: Box<dyn KeepingPairings>,
}

impl Shared {
    /// The pairings.
    #[must_use]
    pub const fn pairings(&self) -> &Pairings {
        &self.pairings
    }

    /// The pairings, to keep or revoke one.
    ///
    /// Whoever changes them calls [`Shared::written_down`] afterwards; the
    /// two are separate so that the list in memory moves first and the disk
    /// second, which is what makes a revocation immediate whatever the disk
    /// says.
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

    /// Write the pairings down, whole, as they stand at this moment.
    ///
    /// # Errors
    ///
    /// [`NotRemembered`] when the file could not be written. The list in
    /// memory is exactly as it was before the call: what failed is the
    /// remembering, and the caller says so to whoever asked.
    pub fn written_down(&self, now: SystemTime) -> Result<(), NotRemembered> {
        self.keeping.keep(&self.pairings, now)
    }
}

/// The one lock over what this machine holds about the other machines.
#[derive(Debug)]
pub struct TheNetwork {
    /// The lists.
    shared: Mutex<Shared>,
}

impl TheNetwork {
    /// This machine, paired with nothing, with nothing waiting, and writing
    /// its pairings nowhere.
    ///
    /// For a test that is not about the file. A machine really starting is
    /// [`TheNetwork::remembering`].
    #[must_use]
    pub fn on(here: MachineId) -> Self {
        Self::remembering(here, Pairings::none(), Box::new(NothingKeepsPairings))
    }

    /// This machine, holding the pairings that were read back off the disk,
    /// with nothing waiting, and writing every change to `keeping`.
    #[must_use]
    pub fn remembering(
        here: MachineId,
        pairings: Pairings,
        keeping: Box<dyn KeepingPairings>,
    ) -> Self {
        Self {
            shared: Mutex::new(Shared {
                pairings,
                proposals: Proposals::on(here),
                keeping,
            }),
        }
    }

    /// Take the lock, for the length of one message or one act on the
    /// person's door.
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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;

    use alo_nearby::{MayAskIts, Pairings};
    use alo_remembering::NotRemembered;

    use super::{KeepingPairings, TheNetwork};
    use crate::testing::{noon, paired_between, reception, the_studio};

    /// Somewhere that remembers how long each list it was handed was, in
    /// order, where the test can read it back.
    #[derive(Debug, Default)]
    struct Remembering {
        /// Every list written, by its length.
        written: Arc<Mutex<Vec<usize>>>,
    }

    impl KeepingPairings for Remembering {
        fn keep(&self, pairings: &Pairings, _now: SystemTime) -> Result<(), NotRemembered> {
            self.written.lock().unwrap().push(pairings.every().len());
            Ok(())
        }
    }

    /// **A machine starts with what was read back, and writes the whole list
    /// at each change** — the list in memory first, then the disk, so that a
    /// revocation is immediate whatever the disk says.
    #[test]
    fn what_was_read_back_is_held_and_every_change_is_written_whole() {
        let (_, on_studio) =
            paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
        let mut read_back = Pairings::none();
        read_back.keep(on_studio);
        let keeping = Remembering::default();
        let written = Arc::clone(&keeping.written);
        let network = TheNetwork::remembering(the_studio(), read_back, Box::new(keeping));

        let mut shared = network.locked();
        assert!(shared.pairings().paired_with(&reception(), noon()));
        assert!(shared.pairings_mut().revoke(&reception()));
        assert!(!shared.pairings().paired_with(&reception(), noon()));
        shared.written_down(noon()).unwrap();
        // The disk saw the list after the revocation, and nothing before.
        assert_eq!(*written.lock().unwrap(), vec![0]);
    }

    /// **A file that cannot be written leaves the list as it is**: the
    /// refusal is the disk's and the pairing stands for the session.
    #[test]
    fn a_file_that_cannot_be_written_leaves_the_list_alone() {
        #[derive(Debug)]
        struct NoRoom;
        impl KeepingPairings for NoRoom {
            fn keep(&self, _: &Pairings, _: SystemTime) -> Result<(), NotRemembered> {
                Err(NotRemembered::NotWritten {
                    at: "/var/lib/alo/pairings.toml".into(),
                    why: "no space left on device".to_owned(),
                })
            }
        }
        let (_, on_studio) =
            paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
        let network = TheNetwork::remembering(the_studio(), Pairings::none(), Box::new(NoRoom));
        let mut shared = network.locked();
        shared.pairings_mut().keep(on_studio);
        assert!(matches!(
            shared.written_down(noon()),
            Err(NotRemembered::NotWritten { .. })
        ));
        assert!(shared.pairings().paired_with(&reception(), noon()));
    }
}
