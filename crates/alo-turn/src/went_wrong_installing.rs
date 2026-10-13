//! The two ways an installation fails that are faults in alo OS.
//!
//! Its own file because it is its own fact. Everything else that stops an
//! installation is `alo_software::NotDone` — a place an organisation does not
//! permit, an application already here, a signature that did not check, a tool
//! that did not answer — and every one of those is a refusal or the world, said
//! in words a person reads. These two are neither:
//!
//! - an authority for another verb, or one that came from no approval, reached
//!   the installing executor — which is this machine routing a call to the
//!   wrong place;
//! - the line raised on the indicator did not describe the errand about to be
//!   carried out — which is this machine failing to show what it was doing, one
//!   step after deciding it;
//! - an installation reached the executor on a machine with no tool to install
//!   with — which cannot be arranged from outside, because the constructor that
//!   offers the verb is the one that takes the tool, and is here so that the
//!   rule is enforced rather than remembered.
//!
//! `alo-software` keeps both in English on purpose, and says why: *its reader
//! is fixing that executor*. So this type keeps them in English too, and the
//! sentence the person reads is [`crate::words::COULD_NOT_INSTALL_HERE`],
//! which carries none of it. That is `NOT_BOUNDED`'s rule, met a second time:
//! what a person can act on is that nothing happened and that the machine is
//! what has to be looked at.
//!
//! # Why they are one type and not two refusals
//!
//! Because they are one fact to whoever reads a record: *alo OS contradicted
//! itself and installed nothing*. Two variants of [`crate::NotDone`] would be
//! two sentences a translator has to render for a case neither of them can
//! reach from outside, and a reader choosing between them would be choosing
//! between two descriptions of the same morning.

use alo_software::{NotAnInstallation, NotShown};

/// An installation alo OS could not carry out through a fault of its own.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WentWrongInstalling {
    /// An authority that is not an approved installation reached the installing
    /// executor.
    #[error("the installing executor was handed an authority that is not an installation: {0}")]
    NotAnInstallation(#[from] NotAnInstallation),
    /// The indicator was showing something other than the errand about to run.
    #[error("the installation was not the errand on the indicator: {0}")]
    NotShown(#[from] NotShown),
    /// This machine offered an installation it has nothing to install with.
    ///
    /// No data, because there is nothing to say beyond the fact: a machine
    /// built by `Machine::carrying_out_file_verbs` does not offer the verb, so
    /// an authority for it arriving at the executor means the two lists came
    /// apart somewhere above.
    #[error(
        "an installation reached the executor on a machine with no tool, so the verbs offered \
         and the verbs this machine can carry out have come apart"
    )]
    NothingToInstallWith,
}
