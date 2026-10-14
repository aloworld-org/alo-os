//! The one thing this crate asks the machine to do, one method wide.
//!
//! `crate::machine` is the only implementation that ships and it is
//! `systemd-logind` over the system bus. This is the trait because the decision
//! — who may knock, whether there is anybody to open a session for, what comes
//! back — is arithmetic on numbers, and a decision that could only be shown
//! working on a machine with a running `logind` is a decision the workspace gate
//! never runs.
//!
//! # It is `&mut self`, and that is the design rather than a convenience
//!
//! What `CreateSession` answers with includes a descriptor, and **the session
//! lasts as long as somebody holds it**. So the implementation keeps it, and
//! keeping it is a change to the implementation. A trait taking `&self` would
//! be one whose honest implementation had to hide the descriptor behind
//! something shared — which is the same thing with the lifetime of the session
//! written somewhere less obvious than the signature.

use std::path::{Path, PathBuf};

use crate::refusing::NotOpened;

/// A session that is open, and what the machine said about it.
///
/// **The runtime path was answered and discarded until 2026-10-14.**
/// `CreateSession` replies with the person's own runtime directory -- the one
/// `logind` makes and takes away with their last session -- and the desktop
/// this component now starts needs exactly that to bind its socket in. It was
/// being deserialised into `_runtime` and dropped, which is a fact about the
/// machine being thrown away a line after it arrived.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheSessionOpened {
    /// The person's own runtime directory, as `logind` answered with it.
    runtime: PathBuf,
}

impl TheSessionOpened {
    /// Say that a session is open, with the runtime directory it answered with.
    #[must_use]
    pub const fn with(runtime: PathBuf) -> Self {
        Self { runtime }
    }

    /// The person's own runtime directory, which is theirs and not this
    /// machine's.
    #[must_use]
    pub fn runtime(&self) -> &Path {
        &self.runtime
    }
}

/// What can open a session on this machine.
pub trait Logind {
    /// Open a session for this account number, and keep it open.
    ///
    /// Answering `Ok` means the session exists **and this value is what holds
    /// it**: dropping the implementation ends it. That is how signing out is
    /// spelled, and it is why there is no `close`.
    ///
    /// # Errors
    ///
    /// [`NotOpened`], which is every way the machine can answer no: no bus to
    /// ask, a refusal from `logind` itself, or a reply this crate could not
    /// read.
    fn open_a_session_for(&mut self, person: u32) -> Result<TheSessionOpened, NotOpened>;
}
