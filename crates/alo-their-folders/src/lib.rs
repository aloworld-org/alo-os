//! The ordinary folders a person expects in their home, made when their
//! account is.
//!
//! A fresh alo machine gives a person a home folder with nothing in it — no
//! Documents, no Downloads, no Pictures, just the three hidden files every
//! Unix account has. Windows, macOS and every Linux desktop make these at the
//! moment an account is created, and a person arriving from any of them looks
//! for them by name. **This crate is that step**, and it exists because its
//! absence was found from the other end: the agent could not be granted a
//! folder, and when the screen for granting one was designed there was
//! nothing in the list to grant.
//!
//! `docs/features.md` puts it as a rule rather than a feature — *be ordinary
//! everywhere it is not the point* — and an empty home folder is not where
//! alo OS wants to be different from everybody else.
//!
//! # The folders are in the person's own language
//!
//! A person chooses their language on the first screen, before the account
//! exists, and it is theirs from the first sign-in. So a French person gets
//! *Documents*, a German one *Dokumente* and a Greek one a name in Greek
//! letters — **the names `xdg-user-dirs` gives them**, the package every Linux
//! desktop names these folders with, so that Dolphin and every other program
//! agree with alo OS about where a person's documents are
//! ([`catalogue`], ADR 0137). A folder that already has a name keeps it.
//!
//! That makes `crate::naming` the file to read first: a translated name
//! becomes a directory, and three kinds of translation produce a folder a
//! person cannot see, cannot open, or that is not where it says it is. Each is
//! refused there with a test against a name that breaks it.
//!
//! # And recorded, or other software makes a second set
//!
//! Making the folders is half of it. The file chooser, the camera application
//! and the browser all need to know *where do pictures go*, none of them can
//! ask the person, and left guessing they guess English — so on a Greek
//! machine they would quietly make a second `Pictures` beside the Greek one.
//! `crate::recording` writes the same small record every other Linux desktop
//! writes, in the same place, so they ask instead of guessing.
//!
//! # What it does not do
//!
//! It makes no grant. A folder existing and an agent being allowed to see it
//! are two different things, and the second is `alo-picking`'s — a person
//! standing in a folder and choosing it. This crate only means that when they
//! open that picker there is something in the list.
//!
//! # The shape
//!
//! - [`Folder`] — which folders a person gets, and why the others are absent;
//! - [`Named`] — one folder's name in their language, checked before it
//!   becomes a directory;
//! - [`made_for`] — making them, making what it can, and saying what it could
//!   not;
//! - [`Disk`] and [`OnThisDisk`] — the one road onto a real filesystem, so
//!   every rule above is tested without one;
//! - [`NotMade`] — every way it can fail, each with a sentence for whoever
//!   looks after the machine.

pub mod catalogue;
pub mod folders;
pub mod making;
pub mod naming;
pub mod on_this_disk;
pub mod recording;
pub mod refusing;
pub mod words;

#[cfg(test)]
mod testing;

pub use catalogue::Catalogue;
pub use folders::Folder;
pub use making::{Disk, Made, NamedDifferently, made_for};
pub use naming::Named;
pub use on_this_disk::OnThisDisk;
pub use recording::{AlreadyRecorded, THE_RECORD, the_record};
pub use refusing::NotMade;
pub use words::{EVERY_WORD, Word, WordsError, declare_into, their_folders_words};
