//! What an agent may do to files, the real path it is checked against, and the
//! doing of it.
//!
//! `docs/features.md` promises six file verbs at v0.01 — list, read, find,
//! rename, move, archive — **over granted paths only**. This crate is the whole
//! of that promise on any machine: the declarations, the last question the
//! grants have to be asked before anything opens a file, and the `std::fs`
//! calls behind the six.
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
//! [`Did::of`] then asks one more question, and only of a change: **may this be
//! created?** The paths a call names have been asked about twice; the name a
//! rename invents, and the path a move or an archive puts something at, have
//! not been asked about at all. A grant covers where a file goes, not only
//! where it comes from.
//!
//! # And what a machine that can impose a boundary asks before any of it
//!
//! [`Reaching`] is the one question this crate answers for somebody else's
//! benefit: **which places does this execution have to be able to open?** On a
//! Linux machine that is what a turn is bounded to by the kernel (ADR 0013,
//! ADR 0015), and it is the paths the call named plus the folder above anything
//! it would create — narrower than the grants, and made of the call rather than
//! of them. [`reaching`] argues why, including the one case where it is wider
//! than a grant and why the grants are still the deciding answer.
//!
//! # What a person reads
//!
//! Nothing in this crate composes an English sentence for a screen. Every
//! string it can say is declared in [`words`] and answered through
//! `alo-strings` in the language the person in front of the machine reads:
//! [`Failed::said`] and [`RealError::said`] for what could not be done, and the
//! two refusals this crate words itself rendered where they are made, so that
//! what somebody was told is what the record keeps.
//!
//! **What the six verbs are, and what a person approves, is asked of the verb
//! itself** since item 9g: [`verbs`] hands each declaration the constants from
//! [`words`], so `alo_capability::Verb::purpose`, `Arg::purpose` and
//! `Call::sentence` answer in the reader's language from what the verb carries.
//! This crate no longer holds a second list saying the same thing.
//!
//! [`Failed`] and [`RealError`] therefore have no `Display`, which is the whole
//! of the guarantee: a sentence cannot reach anybody without something having
//! asked whether it was translated. A shell puts [`words::declare_into`] into
//! its vocabulary at startup, as it does [`declare_into`] into its verb list —
//! and a shell that forgets shows the key, marked, rather than English nobody
//! offered to translate.
//!
//! # What acts, and what only decides
//!
//! Two files touch a disk on purpose: [`resolving`], which asks where a path
//! really leads, and everything reached from [`Did::of`], which does the work.
//! Everything that *decides* — [`touching`], [`verbs`], and the shapes an
//! answer comes back in — is somewhere a test can reach without a filesystem,
//! and the deciding is tested that way.
//!
//! # The gap between the check and the open, and what is left of it
//!
//! A path resolved and then opened **by that name** can have a link swapped in
//! underneath it, at any component and not only the last. On Linux that gap is
//! closed: `opening.rs` resolves the whole path inside one call that refuses a
//! symbolic link at every component, so a file is reached rather than named —
//! and it moves a name with one call that refuses to replace anything, from
//! handles on the two folders rather than from their names. A filesystem that
//! cannot promise that refuses the move; it does not quietly go back to
//! replacing.
//!
//! On hosts without those calls it is `std`, which resolves the name a second
//! time, and `docs/quirks.md` says so.
//!
//! # And the link no path can reveal
//!
//! A **hard** link is a second *real* name for one file. Resolving does not
//! expose it and no comparison of paths could: the granted name genuinely is a
//! name for that file. So a hard link inside a granted folder is a way to widen
//! what an agent may read without widening any grant, and the boundary beneath
//! this one does not catch it either — it decides an open by where a directory
//! entry sits, and this entry sits in the granted folder, which
//! `alo-bounding`'s `a_hard_link_is_inside_every_boundary` measures rather than
//! assumes.
//!
//! **A turn can no longer make one**: the boundary watches `inode_link` and
//! refuses a link whose source or destination is outside the bound. That closes
//! the half where the turn is the one creating the second name, and it leaves
//! this half exactly where it was — a link made by anybody else, before the
//! turn began, is one no kernel hook can see the wrongness of.
//!
//! What a file *will* answer is **how many names it has**, asked of the open
//! handle. More than one and it is not read, wherever bytes are read: a person
//! is told the file has other names, that it was not read, and to copy it or
//! grant the folder the other name is in. It cannot say *where* the others are,
//! so this refuses a file whose second name is harmlessly beside the first —
//! the safe way to be wrong, and `docs/quirks.md` records the cost. On Windows
//! `std` cannot count names at all, and that gap is written down too.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod answer;
pub mod doing;
pub mod failed;
pub mod named;
pub mod reaching;
pub mod real;
pub mod resolving;
pub mod touching;
pub mod verbs;
pub mod words;

mod archiving;
mod changing;
mod crc;
mod looking;
mod opening;
mod walking;
mod zip;

#[cfg(test)]
mod testing;

pub use answer::{Answer, Archived, Listing, Search};
pub use doing::Did;
pub use failed::Failed;
pub use looking::MOST_READ;
pub use named::{Kind, Named, can_be_shown};
pub use reaching::Reaching;
pub use real::{Real, RealError};
pub use resolving::{OnThisMachine, Resolving};
pub use touching::Touching;
pub use verbs::{Declaring, declare_into, file_verbs};
pub use words::{Counted, EVERY_WORD, Word, WordsError, file_words};
