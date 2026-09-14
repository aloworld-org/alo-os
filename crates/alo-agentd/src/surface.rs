//! What shows a proposal to the person at this machine, until a shell does.
//!
//! `alo_nearby::Receiving` answers a proposal only after a
//! [`alo_nearby::Surface`] has been handed the proposal and the code and
//! said it showed them; a surface with nobody in front of it answers `false`
//! and the proposal is refused rather than answered to an empty room. The
//! surface that really shows one is `alo-shell`'s and is outside the
//! local-network plan, so the process is handed this one: it shows nothing,
//! says so to the service log, and answers `false`.
//!
//! Which means a proposal to a machine running this daemon is refused as
//! *nobody to show it to* until a shell implements the trait — the true
//! answer, said on the wire in the proposing person's own language, rather
//! than a proposal accepted on somebody's behalf. The tests in
//! `crate::serving` hand in a surface of their own to hold the rest of the
//! road.

use alo_nearby::{Surface, Waiting};

/// Nobody is in front of this machine to be shown a proposal.
#[derive(Debug, Clone, Copy, Default)]
pub struct NobodyToShowItTo;

impl Surface for NobodyToShowItTo {
    /// Shown to nobody, and said to the service log so that whoever stands
    /// the machine up knows a proposal reached it and why it was refused.
    fn show(&mut self, waiting: &Waiting) -> bool {
        eprintln!(
            "alo-agentd: a pairing was proposed by {} and there is no surface here to show it on; \
             it was refused",
            waiting.other()
        );
        false
    }
}
