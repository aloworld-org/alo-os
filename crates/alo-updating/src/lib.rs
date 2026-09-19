//! An update applied, and the same machine afterwards.
//!
//! `alo-keeping-up` decides what an update is and what the base may be told;
//! this crate carries it out, and does nothing it did not decide. The mechanism
//! is the base's — ADR 0011: a new build is staged as a second deployment and
//! the machine boots into it at the next restart — and the base is rented and
//! unmodified. What is ours is what surrounds it.
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
//! - **Why the base refused, when it did** — [`refused_for_its_signature`]
//!   tells the signature policy's refusal apart from every other way preparing
//!   an update can fail, so that *this machine could not confirm the new
//!   version came from alo OS* is its own sentence rather than the same line a
//!   stalled download reads as. It reads what the base said, because the base
//!   gives no other sign, and what it does not recognise it does not guess at.
//! - **Yesterday's machine** — [`yesterday()`] names the build before, when it
//!   was replaced and whether it is still on the disk, and decides whether going
//!   back can be offered, so a return that cannot be done says so first.
//! - **Going back, because the person approved it** — [`go_back`] reads the
//!   machine now, notes the build chosen ([`AcrossRestarts`]), and runs the
//!   base's one instruction; [`after_a_restart`] writes
//!   `alo_record::Happened::RolledBack` at the first start on it.
//! - **Whether what the record says an agent did can be put back** —
//!   [`what_was_done`] reads one entry onto `alo_keeping_up`'s closed table and
//!   [`putting_back()`] answers with the undo or with the reason there is none.
//!   The seam between the record and the crate that decides, in one file, and
//!   nothing in it puts anything back.
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
//! # What going back does not carry
//!
//! Going back never touches `/var`: the person's files, their settings, the
//! grants, the pairings, the record and the indexes are exactly as they were,
//! measured in `tests/back_to_yesterdays_machine.rs`. **`/etc` is the base's
//! per build**, and going back starts the earlier build with the copy of `/etc`
//! it had (`docs/quirks.md`): accounts, passwords and whole-machine
//! configuration changed since the update stay with the newer build. The same
//! test measures that too, and the sentence the person approves says it.
//!
//! # What is not here
//!
//! **No clock, no schedule and no check for an update.** Whether one is offered
//! is `alo_keeping_up::Offered`, heard during a check on the indicator; when the
//! person is asked is the shell's. Nothing here calls [`apply`] on its own.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod across_restarts;
pub mod applying;
pub mod genuine;
pub mod going_back;
pub mod last_known;
mod one_build;
pub mod putting_back;
pub mod refusing;
pub mod restarted;
pub mod status;
pub mod the_base;
pub mod yesterday;

pub use across_restarts::{AcrossRestarts, THE_BUILD_TO_GO_BACK_TO, THE_LAST_KNOWN_BUILD};
pub use applying::apply;
pub use genuine::refused_for_its_signature;
pub use going_back::go_back;
pub use putting_back::{putting_back, what_this_machine_kept, what_was_done};
pub use refusing::{NotAnswered, NotApplied, NotGoneBack, NotRead, NotRecorded};
pub use restarted::after_a_restart;
pub use status::{THE_STATUS, deployments, running};
pub use the_base::{Base, THE_PROGRAM, TheBase};
pub use yesterday::{Yesterday, yesterday};
