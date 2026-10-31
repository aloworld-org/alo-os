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
        let _owed = self.its_dock_is_filled_from_what_they_kept(&ThePins::NeverSet, discovery);
    }

    /// Work out this person's pins from what they kept and what the machine
    /// offers, keep them, and **say what is owed to the disk**.
    ///
    /// [`Some`] is *write exactly this down*, and it is returned rather than
    /// written because `crates/alo-shell` keeps no file of its own: the binary
    /// that read the list is the one that writes it, through the crate that
    /// owns `dock.toml`. [`None`] is *write nothing*, which is the answer for a
    /// person who already has a list, for a scan that failed and for one that
    /// has not finished — the owner's fourth clause, carried as a value rather
    /// than as a caller's judgement.
    ///
    /// **The decision is `WhatToPin`'s and not this function's.** A caller
    /// holding a list has to decide whether to persist it, and the case that
    /// goes wrong is the empty one; so the type answers and this passes the
    /// answer on.
    pub fn its_dock_is_filled_from_what_they_kept(
        &mut self,
        kept: &ThePins,
        discovery: &Discovery,
    ) -> Option<Vec<alo_dock::ADockEntry>> {
        let what = kept.what_to_pin(discovery);
        self.what_the_dock_is_pinned_to = what.holding();
        self.the_dock_is_settled = what.is_settled();
        what.should_be_written_down()
            .then(|| what.to_draw().to_vec())
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
