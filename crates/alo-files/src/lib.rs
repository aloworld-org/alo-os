//! What an agent may do to files, and the real path it is checked against.
//!
//! `docs/features.md` promises six file verbs at v0.01 — list, read, find,
//! rename, move, archive — **over granted paths only**. This crate is the
//! portable half of that promise: the declarations, and the last question the
//! grants have to be asked before anything opens a file.
//!
//! # The last question
//!
//! [`alo_capability`] decides reach **lexically**: it compares paths component
//! by component and touches no disk, so that a grant means the same thing
//! whether or not the file exists and without a syscall per question. Its
//! `path` module says plainly what that leaves undone, and leaves it here:
//!
//! > a symbolic link inside a granted folder can point outside it, so whatever
//! > executes a verb resolves the real path first and asks about *that*.
//!
//! This crate is *that*. [`Touching`] takes an [`alo_capability::Authorised`] —
//! a call that has already passed every check the deciding crate can make —
//! resolves every path it names, and asks the grants again about where those
//! paths really lead. Only then does it hand back the token an executor must
//! hold. So `/home/anna/Invoices/march.pdf` is refused when it is a link to
//! `/etc/shadow`, and it is refused *by the grants*, in their own words, into
//! the same record as every other refusal.
//!
//! It is the same shape as [`alo_capability::Authorised`] and
//! `alo_egress::Departing`, and for the same reason: a guarantee carried by a
//! type is one that stays true when somebody who has not read this file writes
//! the next verb.
//!
//! # The order the questions are asked in, and why it is that order
//!
//! [`Touching::of`] asks the grants about the path **as it was written**,
//! before it looks for it on the disk. That is not belt and braces; it is what
//! stops the file half becoming a way to ask whether a file exists. A refusal
//! about a path nobody granted says only that it was not granted, and the
//! machine is never touched on its behalf — so an agent cannot learn that
//! `/home/anna/.ssh/id_ed25519` is there by being told it is missing.
//!
//! Every path a call names is asked about, not only the ones the verb said its
//! grant covers. A verb that forgot to require a grant over one of its paths is
//! a mistake somebody will make, and it is not one that should reach a disk.
//!
//! # What this half does not do
//!
//! **It does not open anything.** The `std::fs` calls — reading the folder,
//! moving the file, writing the archive — are item 6a in
//! `docs/autonomy/QUEUE.md`, and they take a [`Touching`] rather than a path.
//! What is here is complete: nothing is stubbed, and nothing half-built ships.
//!
//! **It cannot close the gap between the check and the open.** A path resolved
//! and then opened by name can have a link swapped in underneath it, and a hard
//! link is a real name for a file that also lives somewhere else — neither is
//! visible to any path-based check. Both are written down in `docs/quirks.md`,
//! and closing them belongs to the code that opens the file.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod real;
pub mod resolving;
pub mod touching;
pub mod verbs;

pub use real::{Real, RealError};
pub use resolving::{OnThisMachine, Resolving};
pub use touching::Touching;
pub use verbs::{Declaring, declare_into, file_verbs};
