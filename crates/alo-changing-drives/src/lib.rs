//! What a person reads when a drive they plugged in is opened or finished
//! with, through the privileged broker's two storage verbs.
//!
//! *USB drives and external storage that appear when plugged in*
//! (`docs/features.md`, v0.5). Mounting a drive is a change to the whole
//! machine, so ADR 0001 §2 puts it behind the broker, with no free-form
//! parameter: what crosses the door is a verb and the digest of what the disk
//! service reported, never a device name or a place to put it. Task 4 of
//! `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` built that road —
//! `alo_brokerd::Storage`, against `alo-drives` — and this crate is what that
//! road did not have: **a person in front of it.**
//!
//! | | |
//! |---|---|
//! | [`Change`] | Open it, or finish with it — and the broker verb each is |
//! | [`the_drive_called`], [`what_can_be_opened`], [`to_open`], [`to_finish_with`] | Which drive a person means, out of what is plugged in now |
//! | [`NotChanged`], [`changed_said`] | What a person reads, either way |
//! | [`words`] | Every sentence, with a note for whoever translates it |
//!
//! # Why this is a crate and not a file in `alo-brokerd`
//!
//! Because the broker has no person in front of it. It answers in one word from
//! a closed list — carried, not kept, refused — and adds no sentence to any
//! vocabulary; a daemon running as root that held a person's language would be
//! a daemon deciding what a person reads. So the wording lives where the asking
//! does, exactly as it does for the printers (`alo-changing-printers`) and the
//! network (`alo-changing-network`), and this crate is the third of those
//! three.
//!
//! # What it deliberately does not do
//!
//! **No second road to the door.** `alo_brokerd::Storage` carries the two
//! verbs out and `alo-broker`'s `asking` reaches the door; nothing here opens a
//! socket, and a surface hands its redeemed approval to the road that already
//! exists. **No grant.** Opening a drive grants nobody anything — this crate
//! cannot name a grant, because it depends on nothing that can make one, which
//! is the same argument `alo-drives` makes about itself. **No health.** How a
//! disk is doing is a read the disk service answers to anybody
//! (`alo_drives::Drives::now`), and a read answers inside a turn.
//!
//! **It is not yet reached from a turn, and the surface that will ask is the
//! desktop's.** *USB drives that appear when plugged in* is the desktop plan's;
//! what this crate owes it is the sentences and the choosing, so that a surface
//! is handed words rather than left to write English of its own. The walk from
//! a new printer to a recovered disk, and the table of exactly what a person
//! meets in order, is task 7 of the broker plan.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod choosing;
mod refusing;
mod wanted;
pub mod words;

pub use choosing::{the_drive_called, to_finish_with, to_open, what_can_be_opened};
pub use refusing::{NotChanged, changed_said};
pub use wanted::Change;
pub use words::{EVERY_WORD, WordsError, changing_drives_words, declare_into};
