//! What this person's Dock is pinned to, worked out once when the desktop
//! stands up.
//!
//! `crate::dock_windows::Server::what_the_dock_holds` has taken a
//! `&alo_dock::Holding` since it was written, and **every caller passed
//! `Holding::nothing()`** — so a fresh machine drew a Dock of whatever had a
//! window open and nothing else. Its own doc says so: *nothing in this shell
//! stores that yet*. This is what stores it.
//!
//! # The shell does not scan, and that is a rule rather than a layering taste
//!
//! Which application fills *Files* on this machine is a reading of the machine,
//! and `crates/alo-shell` is the crate forbidden to take one. So the answer
//! arrives as data — an `alo_dock::pinning::Discovery` handed to
//! [`crate::stand_the_desktop_up`] by the binary, exactly as the way to start
//! an application is. See `alo-desktop`'s `what_this_machine_offers`.
//!
//! # Worked out once, and the reason is the rule it would otherwise break
//!
//! `ThePins::what_to_pin` applies the defaults **only** when nothing has been
//! set. Asking it per frame would be harmless today and wrong the moment a
//! person removes a pin, because the computation would run again on the next
//! frame against the same `NeverSet` and put it back. So the answer is taken
//! at stand-up and kept.
//!
//! # What is not true yet, and must not be read as if it were
//!
//! **Nothing persists the result.** `WhatToPin::should_be_written_down` answers
//! `true` for a fresh machine and this crate has nowhere to write it, so the
//! defaults are worked out again on every boot. That is **indistinguishable
//! from correct until removal exists** and wrong on the day it does: a pin a
//! person takes off would come back when they next signed in. The persistence
//! is separate work and this paragraph is the record that it is owed.

use alo_dock::Holding;
use alo_dock::pinning::{Discovery, ThePins};

use crate::server::Server;

impl Server {
    /// Work out this person's pins from what the machine offers, and keep them.
    ///
    /// **`ThePins::NeverSet` is passed because nothing reads a saved list.** It
    /// is the honest value, not a placeholder: no file in this repository
    /// stores a person's pins, so *nobody has set them* is true of every
    /// machine. When a stored list exists it is read here and passed instead,
    /// and nothing else on this path changes.
    pub fn its_dock_is_filled_from(&mut self, discovery: &Discovery) {
        // **`should_be_written_down` is deliberately not consulted.** It
        // answers `true` here and there is nowhere to write, and a branch on it
        // that did nothing in either arm would read as though the write had
        // been considered and declined. See this module's header: it is owed,
        // not decided against.
        self.what_the_dock_is_pinned_to = ThePins::NeverSet.what_to_pin(discovery).holding();
    }

    /// What this person's Dock is pinned to.
    ///
    /// Empty on a machine whose discovery failed or has not run, which is the
    /// same list a machine with nothing installed gets — and the two are
    /// **not** the same state. `ThePins::what_to_pin` is where they are told
    /// apart, and it is told apart there precisely so that this accessor can be
    /// a plain list: what differs between them is whether the list is written
    /// down, and nothing downstream of here asks that question.
    #[must_use]
    pub fn what_the_dock_is_pinned_to(&self) -> &Holding {
        &self.what_the_dock_is_pinned_to
    }
}

#[cfg(test)]
#[path = "what_the_dock_is_filled_with_tests.rs"]
mod tests;
