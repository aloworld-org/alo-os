//! An update applied, and the same machine afterwards.
//!
//! `alo-keeping-up` decides what an update is and what the base may be told;
//! this crate carries it out, and does nothing it did not decide. The mechanism
//! is the base's — ADR 0011: a new build is staged as a second deployment and
//! the machine boots into it at the next restart — and the base is rented and
//! unmodified. What is ours is three things around it.
//!
//! # What is here
//!
//! - **Applying an update the person chose** — [`apply`]. The base's status is
//!   read at that moment, `alo_keeping_up::Staging` decides the one instruction
//!   against it, and [`TheBase`] runs `bootc` with exactly those arguments and
//!   no shell. Every refusal ([`NotApplied`]) leaves the machine as it was.
//! - **What am I running, at any moment** — [`running`] and [`deployments`]
//!   ask the base each time and remember nothing.
//! - **The record saying the machine updated** — [`after_a_restart`], run once
//!   at each start, writes `alo_record::Happened::Updated` from which build to
//!   which, with no agent behind it, when a different build booted than the one
//!   last known ([`last_known`]).
//!
//! # What an update never touches
//!
//! The person's own files, their settings, their grants, their pairings, the
//! record and the file indexes all live under `/var` and `/etc`, which the
//! base carries from one deployment to the next and never replaces. Nothing
//! this crate hands the base names a path. That it holds on a real boot is not
//! argued here but measured: `tests/an_update_keeps_the_persons_things.rs`
//! writes each of those by name in a virtual machine, applies an update,
//! restarts, and finds every one byte for byte.
//!
//! # What is not here
//!
//! **No clock, no schedule and no check for an update.** Whether one is offered
//! is `alo_keeping_up::Offered`, heard during a check on the indicator; when the
//! person is asked is the shell's. Nothing here calls [`apply`] on its own.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod applying;
pub mod last_known;
pub mod refusing;
pub mod restarted;
pub mod status;
pub mod the_base;

pub use applying::apply;
pub use refusing::{NotAnswered, NotApplied, NotRead, NotRecorded};
pub use restarted::{THE_LAST_KNOWN_BUILD, after_a_restart};
pub use status::{THE_STATUS, deployments, running};
pub use the_base::{Base, THE_PROGRAM, TheBase};
