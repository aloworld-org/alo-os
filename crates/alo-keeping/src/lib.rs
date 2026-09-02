//! Where the record is written, and how long what an agent did is kept.
//!
//! `alo-record` is
//! [ADR 0001](../../../docs/decisions/0001-the-capability-model.md) §7 as
//! working code: every execution and every refusal, kept with what ran, under
//! whose authority, from which approval and against which grant. It holds all
//! of that in memory and says, beside the type, why it stops there:
//!
//! > How long a record is kept, and by what, is a decision for whatever writes
//! > it to a disk, made once and in the open, rather than a method anything
//! > holding this type can reach for.
//!
//! This crate is *whatever writes it to a disk*. A record that lives only in
//! memory answers questions until the machine is next turned off, and §7 asks a
//! question that outlives a session.
//!
//! | | |
//! |---|---|
//! | [`Keeping`] | How long what happened is kept. The decision, made once and where somebody can see it |
//! | [`Writing`] | The record, open to be added to — and the only thing that can shorten one |
//! | [`Reading`] | A record read back: where it starts, what happened, and what could not be read |
//! | [`Head`] | The first line: what shape the file is in, and where the record now begins |
//! | [`Damage`] | What could not be read, which is never stepped over |
//! | [`Pruned`] | What a shortening did |
//! | [`NotKept`] | Why what happened is not being written down |
//!
//! # Why this is not part of `alo-record`
//!
//! Because `alo-record` promises that **nothing takes an entry out** — there is
//! no `remove`, no `edit`, no `forget` — and something, somewhere, has to be
//! able to. Putting the two in one crate would leave that promise true of a
//! type and false of the crate around it, and the promise is the kind a
//! security reviewer checks by reading the file list.
//!
//! So the crate that can shorten a record is a separate one, it is small, and
//! everything in it is about making that hard to do quietly:
//!
//! - what goes is decided by a **rule and a moment**, and there is no way to
//!   name an entry, an agent or a day to remove;
//! - shortening is a method on the **writer**, so nothing that is not already
//!   holding the record open can do it;
//! - it **leaves a mark that cannot age out**, because the mark is in the first
//!   line and pruning only walks the rest;
//! - it **refuses a record it cannot read all of**, rather than rewriting the
//!   evidence that something was wrong.
//!
//! It is the same argument `alo-record` made for not being part of
//! `alo-capability`, one crate further along: the thing that decides, the thing
//! that remembers and the thing that can forget are three jobs, and the last
//! one is the one to keep in a file somebody can read in an afternoon.
//!
//! # The file
//!
//! One line saying what the file is, then one line of JSON per entry, appended
//! and never rewritten except by a shortening. [`writing`] has the reasoning
//! and [`Head`] has the format number, which is a public surface: at v1 a
//! record is exported to a security team's own console, and a record written by
//! a newer alo OS is refused rather than appended to.
//!
//! # What a person reads
//!
//! Nothing here composes an English sentence for a screen. Every string this
//! crate can say is declared in [`words`] and answered through `alo-strings` in
//! the language the person in front of the machine reads, so [`NotKept`],
//! [`Damage`], [`Head`] and [`Keeping`] have no `Display` between them. A shell
//! puts [`declare_into`] into its vocabulary at startup; one that forgets shows
//! the key, marked, rather than English nobody offered to translate.
//!
//! # Nothing here reads the clock
//!
//! As in `alo-capability`, `alo-record` and `alo-appearance`, and for the same
//! reason: [`Writing::prune`] takes the moment it is running at, so what a rule
//! removes is arithmetic a test can do rather than something that has to be
//! waited for — and the daemon and a settings panel cannot disagree about when
//! *now* is.
//!
//! # What is not here
//!
//! **Where the file lives, and when a shortening runs.** Both are the daemon's:
//! a path under `/var/lib` that a package decides, and a moment that something
//! with a timer picks. This crate is handed a path and a moment. `alo-agentd`
//! does not exist yet, and queue item 4b is what is owed when it does.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod damage;
pub mod failing;
pub mod head;
pub mod keeping;
pub mod pruning;
pub mod reading;
pub mod words;
pub mod writing;

#[cfg(test)]
mod testing;

pub use damage::Damage;
pub use failing::NotKept;
pub use head::{Head, THE_FORMAT};
pub use keeping::{Keeping, KeepingError};
pub use pruning::Pruned;
pub use reading::Reading;
pub use words::{Word, WordsError, declare_into, keeping_words};
pub use writing::Writing;
