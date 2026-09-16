//! The pairings a person's side read, asked the one question choosing a paired
//! machine needs.
//!
//! `alo_choosing::Choosing::answered_by_a_paired_machine` asks *may this
//! machine's models be asked, now?* of whoever holds the pairings, and refuses
//! before a byte is written when the answer is no. The daemon answers it over
//! its own list; Settings holds the list `alo_remembering::pairings_remembered`
//! read, and answers it here by asking `alo_nearby::Pairings::permits` — the
//! same rule, in the crate that owns it, with nothing decided on this side.

use std::time::SystemTime;

use alo_choosing::WhoMayBeAsked;
use alo_nearby::{MachineId, MayAskIts, Pairings};

/// The pairings Settings read, asked by `alo-choosing`.
pub(crate) struct PairedAsked<'a>(pub(crate) &'a Pairings);

impl WhoMayBeAsked for PairedAsked<'_> {
    fn may_ask_the_models_of(&self, machine: &str, now: SystemTime) -> bool {
        MachineId::read(machine)
            .is_ok_and(|machine| self.0.permits(&machine, MayAskIts::Models, now))
    }
}
