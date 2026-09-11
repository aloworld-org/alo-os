//! The daemon's door, as the person's side of the machine sees it.
//!
//! One method, and it carries **nothing**: the knock is
//! `alo_protocol::FromAPerson::Granted`, which has no field for a grant, a
//! path or a duration, so the most any implementation of this trait can cause
//! is the daemon reading a file this crate has already written and the daemon
//! already trusts on its own terms. That is the same argument
//! `alo-agentd/src/rereading.rs` makes from the other side of the socket.
//!
//! A trait for the reason `alo_files::Resolving` is one: the composition in
//! [`crate::Changing`] has to be testable against a door with a counter
//! behind it — *how many knocks, and what was on the disk when each one
//! arrived* — without a test having to stand a daemon up.
//! [`crate::TheDaemonsDoor`] is the only implementation that ships.

use crate::stood::Stood;

/// Somewhere the person's side can say *what is granted has changed*.
pub trait Knocking: std::fmt::Debug {
    /// Say it, and hear where the change now stands.
    ///
    /// Called only after the file has been replaced whole —
    /// [`crate::Changing`] is shaped so there is no other caller — and
    /// infallible by design: every way a knock can fail to land is
    /// [`Stood::AtTheNextSignIn`], because the change is already on the disk
    /// and a daemon that never heard reads it at the next start.
    fn knock(&self) -> Stood;
}
