//! The two ways looking at the catalogue fails that are faults in alo OS.
//!
//! `crate::went_wrong_installing` one verb along, and the same argument holds
//! with one difference worth stating: **this read has no refusals that are not
//! faults.** An installation can be stopped by the world — a place an
//! organisation does not permit, a signature that did not check, a tool that
//! did not answer — and every one of those is `alo_software::NotDone`, said in
//! words a person reads. Looking at the catalogue cannot be stopped by the
//! world at all. It parses a string compiled into this binary.
//!
//! So everything that can go wrong here is alo OS contradicting itself:
//!
//! - an authority for another verb, or one that came from an **approval**,
//!   reached the looking executor — which is this machine routing a call to the
//!   wrong place, or treating a question as a change;
//! - the catalogue this crate ships could not be read — which
//!   `alo-software`'s own suite refuses four ways, so a build whose tests ran
//!   cannot do it.
//!
//! Both are kept in English, for `alo-software`'s stated reason: *its reader is
//! fixing that executor.* The sentence the person reads is
//! [`crate::words::COULD_NOT_LOOK_AT_THE_CATALOGUE`] and carries none of it,
//! which is `NOT_BOUNDED`'s rule met again — what a person can act on is that
//! nothing was answered and that the machine is what has to be looked at.
//!
//! # One type rather than two refusals
//!
//! `went_wrong_installing`'s reason, unchanged: they are one fact to whoever
//! reads a record — *alo OS contradicted itself and answered nothing* — and two
//! variants of [`crate::NotDone`] would be two sentences a translator has to
//! render for cases neither of them can reach from outside.

use alo_software::{NotALook, NotCatalogued};

/// A look at the catalogue alo OS could not answer through a fault of its own.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WentWrongLooking {
    /// An authority that is not a look at the catalogue reached the looking
    /// executor.
    #[error("the looking executor was handed an authority that is not a look: {0}")]
    NotALook(#[from] NotALook),
    /// The catalogue this machine ships could not be read.
    ///
    /// **Not a refusal and not the world.** `catalogue.toml` is
    /// `include_str!`'d into this binary and checked against `shipped.toml`, so
    /// this is a file in this repository being wrong — which the crate's own
    /// tests make impossible to ship.
    #[error("the catalogue this machine ships could not be read: {0}")]
    NotCatalogued(#[from] NotCatalogued),
}
