//! **Telling somebody else what the indicator shows, at the moment it changes.**
//!
//! `alo-agentd` holds the indicator a provider call lights, and the desktop is
//! where a person looks. Until 2026-10-27 nothing carried one to the other, so
//! a question sent to a provider left the machine with nothing on the screen —
//! law 1's *visible at the moment it happens*, broken on every machine with a
//! provider (ADR 0134).
//!
//! # Told inside the call that shows it
//!
//! [`crate::Indicator`] calls its [`Telling`] **after the list changed and
//! before the call returns**, so the [`crate::Departing`] a caller needs to open
//! a connection exists only once whoever is told has been told. The service
//! that owns the indicator runs one thread and a provider call blocks it for
//! the length of the question, so a telling deferred to *the next round* would
//! arrive after the request had already gone.
//!
//! **What is told is the whole list**, never a change to it: the indicator's
//! own reason for being a list rather than a stream is that a list is still
//! right after a missed notification, and a listener that joins late reads it
//! once and is current.

use crate::indicator::Shown;

/// Somebody to tell whenever what the indicator shows changes.
///
/// **Telling cannot fail the egress.** It answers nothing, so a listener that
/// stopped reading cannot stop a departure the policy permitted: the record
/// still writes it down, and a listener that went is the listener's to
/// reconnect.
pub trait Telling: std::fmt::Debug + Send {
    /// What is showing now, oldest first, just after it changed.
    fn now_showing(&self, showing: &[Shown]);
}
