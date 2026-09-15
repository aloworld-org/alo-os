//! The daemon's pairing door, as the person's side of the machine sees it.
//!
//! A grant's revocation is this crate's to write — the grants file is the
//! person's side's, and the daemon only re-reads it ([`crate::Knocking`]). A
//! pairing's is not: `/var/lib/alo/pairings.toml` is the daemon's alone, so
//! the only road the person's side has is to **ask** the daemon, over the
//! `revoke-pairing` message `docs/contracts/daemon-protocol.md` already
//! documents, and to carry back what it said. This trait is that asking.
//!
//! A trait for the reason [`crate::Knocking`] is one: [`crate::Changing`] has
//! to be tested against a door with a counter behind it without a test
//! standing a daemon up. [`crate::TheDaemonsDoor`] is the only implementation
//! that ships.
//!
//! # Five answers, and only one of them is *nothing moved for certain*
//!
//! [`Unpaired`] keeps apart what a person must not be told looks the same.
//! The daemon revoked it and wrote it down; the daemon revoked it and could
//! not write it down, so a restart brings it back; the daemon refused, in its
//! own words; nobody was at the door, so nothing was revoked; and a door that
//! was reached and never answered — where the daemon may or may not have
//! acted, and saying either would be a guess.

use alo_nearby::MachineId;

/// What asking the daemon to revoke a pairing came to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unpaired {
    /// The daemon took the pairing away at once and wrote the shorter list
    /// down.
    Revoked,
    /// The daemon took the pairing away at once and could not write that
    /// down: it is gone until this machine restarts, and back after.
    RevokedUntilARestart,
    /// The daemon refused, and this is what it said — carried rather than
    /// reworded.
    Refused {
        /// The daemon's own sentence, in the language the person reads.
        told: String,
    },
    /// Nobody was listening at the door, so nothing was asked and nothing
    /// was revoked.
    NobodyThere,
    /// The door was reached and no answer this request can earn came back:
    /// the daemon may or may not have revoked it.
    NotAnswered,
}

/// Somewhere the person's side can ask for a pairing to be revoked.
pub trait RevokingPairings: std::fmt::Debug {
    /// Ask for the pairing with this machine to be revoked, and hear what
    /// became of it.
    ///
    /// Infallible by design, for [`crate::Knocking::knock`]'s reason: every
    /// way the conversation fails is one of [`Unpaired`]'s cases, and
    /// [`crate::Changing`] decides what each one means to a person.
    fn revoke_pairing(&self, with: &MachineId) -> Unpaired;
}
