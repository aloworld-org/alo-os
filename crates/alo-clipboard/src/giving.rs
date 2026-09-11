//! The owner's half: the only place bytes come from.
//!
//! A clipboard holds an offer and a way back to whoever made it. This is that
//! way back — implemented by whatever talks to the application that owns the
//! selection, which on a real machine is the compositor writing into the pipe
//! Wayland's data device hands over, and in this crate's tests is a fixture.
//!
//! It is a trait rather than a field of bytes for the reason the whole design
//! rests on: **the broker never holds anybody's data.** There is nowhere in
//! [`crate::Clipboard`] to put a copy of what was copied, so *what is pasted is
//! what was copied* is not a promise this crate keeps carefully — it is the only
//! thing it can do, because the bytes do not exist here until an owner is asked
//! for them and there is no other door to ask through.
//!
//! # Why it is asked synchronously
//!
//! Wayland's transfer is a pipe, and a real compositor reads it without blocking
//! its own loop. What is here is the decision that surrounds that read — which
//! owner, which form, and whether it was allowed to be asked at all — and those
//! are answered before a byte moves. The wiring that turns this call into a
//! descriptor and a read belongs to `crates/alo-shell`, which is the desktop
//! lane's; an adapter there implements [`Gives`] and nothing about the rules
//! below moves.

use crate::kind::Kind;

/// Whatever can produce what an owner copied, in a form the owner offered.
///
/// Implemented by the compositor's side of a client's data source, and by
/// nothing else. [`crate::Clipboard`] holds one of these per offer and **drops
/// it the moment that offer is retired**, which is what makes a transfer
/// against a stale offer move nothing: the way back to the previous owner does
/// not exist any more, so there is nothing left to ask.
pub trait Gives {
    /// What was copied, in this form.
    ///
    /// Only ever called with a form the owner offered — [`crate::Clipboard`]
    /// checks that first and refuses in words otherwise — so an implementation
    /// never has to decide whether to convert, and must not.
    ///
    /// # Errors
    /// [`CouldNotGive`] when the owner cannot produce it after all: an
    /// application that stopped between the copy and the paste, or a transfer
    /// that failed part way.
    fn give(&mut self, form: &Kind) -> Result<Vec<u8>, CouldNotGive>;
}

/// Why an owner did not hand over what it offered.
///
/// One variant, and it is deliberately a fact about the far end rather than a
/// list of reasons: from the clipboard's side, an application that quit and an
/// application whose transfer broke are the same event and the same sentence.
/// `non_exhaustive` because the day a wiring can tell those apart is the day
/// this grows a variant, and doing that additively is the contract every public
/// surface here is held to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum CouldNotGive {
    /// The application that copied this did not produce it: it has stopped, or
    /// the transfer broke part way through.
    #[error("the application that copied this did not hand it over")]
    TheApplicationDidNot,
}
