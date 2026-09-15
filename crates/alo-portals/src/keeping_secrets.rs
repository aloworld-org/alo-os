//! The keyring the Secret portal is answered from, as the backend sees it.
//!
//! `alo-secrets` owns the keyring and already depends on this crate, for
//! [`Request`] and its judgement. The backend therefore reaches the keyring
//! through this trait, which `alo_secrets::TheKeyring` implements, rather than
//! naming that crate — a dependency the other way would be a cycle.
//!
//! **The keyring judges, not the backend.** [`KeepsSecrets::hand_over`] takes
//! the request and the grants, and the implementation answers with what the
//! grants allowed or `alo-capability`'s refusal, as
//! `TheKeyring::for_the_application` already does. The backend only records and
//! responds.
//!
//! **The secret is written, never returned.** The bytes go from the keyring
//! straight into what the application handed over, so there is no value here
//! holding a secret for anybody to log.

use std::io::Write;
use std::time::SystemTime;

use alo_capability::Grants;

use crate::judging::Allowed;
use crate::refused::Refused;
use crate::request::Request;

/// A keyring that hands an application its own portal secret.
pub trait KeepsSecrets: Send + Sync {
    /// Write the portal secret of the application `request` is from into
    /// `into`, if `grants` allow the request at `now`.
    ///
    /// # Errors
    /// [`NotKept`] — the grants' refusal, a keyring that would not answer, or
    /// a secret that could not be written.
    fn hand_over(
        &self,
        request: &Request,
        grants: &Grants,
        now: SystemTime,
        into: &mut dyn Write,
    ) -> Result<Allowed, NotKept>;
}

/// Why no secret was handed over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotKept {
    /// The grants refused the request, so the keyring was not asked.
    Refused(Refused),
    /// The keyring is not there, is locked, or refused.
    Unavailable,
    /// The secret could not be written to what the application handed over.
    NotWritten,
}
