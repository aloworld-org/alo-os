//! What shows a proposal to the person at this machine.
//!
//! `alo_nearby::Receiving` answers a proposal only after a
//! [`alo_nearby::Surface`] has been handed the proposal and the code and
//! said it showed them; a surface with nobody in front of it answers `false`
//! and the proposal is refused rather than answered to an empty room. Two
//! surfaces ship, and the serving loop chooses between them by the one fact
//! it knows: whether a person's shell is connected on the person's door.
//!
//! # Shown means put where the person's door lists it
//!
//! Since the person's door learned to pair (`crate::pairing`), a proposal
//! that arrived is shown the way a change waiting for approval is shown: it
//! waits on the list the shell reads with `pairings`, with its code and its
//! enumerated list on it, and the person confirms it from that list with
//! `confirm-pairing`. So [`AtThePersonsDoor`] answers *shown* — the value is
//! on the door, and a shell that is connected reads it — and says so to the
//! service log. It is handed to the wire only while a shell is connected;
//! with none, the loop hands [`NobodyToShowItTo`] instead, and the proposal
//! is refused as *nobody to show it to*, which is the true word.
//!
//! What a shell then draws — the six digits beside the confirmation, the
//! list in words — is `alo-shell`'s and outside the local-network plan; what
//! this crate owes it is the value on the door.

use alo_nearby::{Surface, Waiting};

/// Nobody is in front of this machine to be shown a proposal.
#[derive(Debug, Clone, Copy, Default)]
pub struct NobodyToShowItTo;

impl Surface for NobodyToShowItTo {
    /// Shown to nobody, and said to the service log so that whoever stands
    /// the machine up knows a proposal reached it and why it was refused.
    fn show(&mut self, waiting: &Waiting) -> bool {
        eprintln!(
            "alo-agentd: a pairing was proposed by {} and no shell is connected to show it on; \
             it was refused",
            waiting.other()
        );
        false
    }
}

/// The person's shell is connected, and the proposal waits on its door.
#[derive(Debug, Clone, Copy, Default)]
pub struct AtThePersonsDoor;

impl Surface for AtThePersonsDoor {
    /// Shown by being put where the shell lists it, and said to the service
    /// log so that a proposal nobody confirmed can be found afterwards.
    fn show(&mut self, waiting: &Waiting) -> bool {
        eprintln!(
            "alo-agentd: a pairing was proposed by {} and waits on the person's door",
            waiting.other()
        );
        true
    }
}
