//! A keyring this machine does not have, which refuses honestly instead of
//! stopping everything else.
//!
//! # What this is for
//!
//! `alo-portald` opened the Secret Service at start-up and **exited** when it
//! could not. On a booted machine on 2026-10-14 that looked like this:
//!
//! ```text
//! alo-portald: the keyring on that bus could not be opened: Unavailable
//! alo-portald.service: Scheduled restart job, restart counter is at 78.
//! ```
//!
//! The image installs no Secret Service at all — nothing in `image/Containerfile`
//! provides one — so there was never a keyring to open and the service could
//! only ever crash-loop.
//!
//! **And the keyring answers one portal of several.**
//! [`crate::serving::Backend`] serves the file chooser, opening a URI and the
//! rest; only `org.freedesktop.portal.Secret` needs a keyring. So one optional
//! capability's missing dependency took down **every** portal, and a machine
//! with no Secret Service could not open a file dialog — which has nothing to
//! do with secrets.
//!
//! That is why `Ctrl`+`Alt`+`T` opened nothing on that machine: the chord asks
//! the portal to start a terminal, and the portal was not there.
//!
//! # Why a type rather than an `Option`
//!
//! [`NotKept::Unavailable`] already means *the keyring is not there, is locked,
//! or refused* — the vocabulary for this answer was in the trait before this
//! file existed. So a machine without a keyring is one that answers
//! `Unavailable` to a secret and serves everything else, and that is one
//! implementation rather than a `None` tested at each use.
//!
//! An application asking for its secret is told no, in the word the contract
//! already has. It is not told that the portal is missing, because it is not.

use std::io::Write;
use std::time::SystemTime;

use alo_capability::Grants;

use crate::judging::Allowed;
use crate::keeping_secrets::{KeepsSecrets, NotKept};
use crate::request::Request;

/// The keyring on a machine that has none.
///
/// Every request for a secret is refused with [`NotKept::Unavailable`], and
/// every other portal is unaffected.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoKeyringHere;

impl KeepsSecrets for NoKeyringHere {
    /// Refuse, because there is no keyring on this machine to ask.
    ///
    /// **The grants are not consulted and nothing is written.** A refusal that
    /// read the grants first would make this machine's answer depend on
    /// permissions it cannot act on either way, and an application would learn
    /// something about its grants from a machine that cannot keep secrets at
    /// all.
    fn hand_over(
        &self,
        _request: &Request,
        _grants: &Grants,
        _now: SystemTime,
        _into: &mut dyn Write,
    ) -> Result<Allowed, NotKept> {
        Err(NotKept::Unavailable)
    }
}

#[cfg(test)]
#[path = "no_keyring_here_tests.rs"]
mod tests;
