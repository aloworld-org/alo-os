//! **The fixed set**: the requests every change to what the model is told or
//! offered is tried against, and the road each is expected to take.
//!
//! Versioned by [`THE_VERSION`], which goes up whenever a request, a road or
//! the machine they are asked on changes, so two summaries are comparable only
//! when they name the same version.
//!
//! # Where every name comes from
//!
//! Rule 3 of ADR 0156: one source for names. **Nothing here is typed twice.**
//! The verbs and their arguments are the constants their declaring crates
//! declare them by; the applications are the ones a fresh machine installs
//! (`alo_software::Shipped`, `crates/alo-software/shipped.toml`), found by the
//! role they fill; the person's folders are the ones `alo-their-folders` makes;
//! and the VSCodium request is the VSCodium walk's own
//! (`alo_walking::writing_in_vscodium`), so the check and the walk ask the
//! same thing.
//!
//! # The machine they are asked on
//!
//! The one the walk boots: the person `alo`, at full access (the level the
//! first-start sheet arrives with), told where their home and folders are and
//! which applications are installed. **The applications are told by identifier
//! only**: a booted machine also tells each one's name from its desktop file,
//! and this crate has no single source for those, so the model here is told
//! less than on a machine, never more.

use std::path::Path;

use alo_instructing::WhatItMayReach;

use crate::a_road::ARequest;

/// The version of the set. Up by one for every change to a request, a road or
/// the machine.
pub const THE_VERSION: u32 = 1;

/// The person's home on the machine the walk boots: Fedora's atomic layout
/// puts homes under `/var/home`, measured on 2026-11-10
/// (`alo_instructing::the_persons_folders`).
pub const THE_HOME: &str = "/var/home/alo";

/// The GitHub account the publishing request names.
pub const THE_ACCOUNT: &str = "alo-demo";

/// The folder the publishing request names, under the person's Documents.
pub const THE_SITE: &str = "site";

/// The set could not be built from its sources.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("the set could not be built: {0}")]
pub struct NotTheSet(String);

/// The requests and the machine they are asked on.
#[derive(Debug, Clone)]
pub struct TheSet {
    requests: Vec<ARequest>,
    may_reach: WhatItMayReach,
}

impl TheSet {
    /// Every request, in order.
    #[must_use]
    pub fn requests(&self) -> &[ARequest] {
        &self.requests
    }

    /// What a turn on the machine is told it may reach.
    #[must_use]
    pub const fn may_reach(&self) -> &WhatItMayReach {
        &self.may_reach
    }

    /// Only the request called `named`, for a run that tries one.
    #[must_use]
    pub fn only(mut self, named: &str) -> Option<Self> {
        self.requests.retain(|request| request.named() == named);
        (!self.requests.is_empty()).then_some(self)
    }
}

/// The set, built from its sources.
///
/// # Errors
/// [`NotTheSet`], when a source it reads would not build.
pub fn the_set() -> Result<TheSet, NotTheSet> {
    let _ = Path::new(THE_HOME);
    Err(NotTheSet("not yet built".to_owned()))
}

#[cfg(test)]
#[path = "the_set_tests.rs"]
mod tests;
