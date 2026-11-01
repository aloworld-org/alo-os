//! Putting a folder back as it was, from a place this machine kept before a
//! command a person typed.
//!
//! `docs/features.md` promises ★ *scrollback you can move along, not only
//! read*: **in a session you asked to be able to undo, every command is a place
//! you can return to: scroll up, choose one, and your files are as they were
//! before it ran.** Moving along it was built on 2026-10-15. The place being
//! kept was built on 2026-10-22
//! ([ADR 0130](../../docs/decisions/0130-what-takes-the-bracket-before-a-persons-own-command.md)).
//! **This is *and your files are as they were*** — which until now did not
//! exist anywhere in this repository: a search of every crate found `snapshot`,
//! `delete`, `create`, `list` and `show`, and nothing that put a file back.
//!
//! # What it is, honestly, and what it is not yet
//!
//! [ADR 0127](../../docs/decisions/0127-a-persons-own-commands-are-bracketed-in-a-session-they-asked-for.md)
//! term 2 narrowed the promise, and the narrowing is carried into
//! `docs/features.md` by the change that added this crate: **the folder you
//! picked, now — or all of them, at your next sign-in.** The whole home is the
//! second road and waits for a sign-in, because nothing can swap a directory
//! somebody is living in; a privileged helper that did it anyway was refused
//! outright, as a new privileged path whose job would be deleting a person's
//! home.
//!
//! **And the choosing is by name, not yet by scrolling.** The promise's *scroll
//! up, choose one* needs the terminal to know which place belongs to which
//! command, and it deliberately does not: ADR 0130 says alo never learns what a
//! person typed, so a place carries a moment and no command. Pairing a row of
//! scrollback to a place is a surface this crate does not build and is the next
//! piece of that line. What a person has today is the list and a name, in their
//! own terminal, which is where law 2 says they are trusted with a shell.
//!
//! # Nothing privileged, and nothing of an agent's
//!
//! Every path here runs as the person, in their own process, over their own
//! files. `docs/contracts/kept-undo-folder.md` gives a person's directory
//! inside the undo folder to that person, which is what makes that possible —
//! and ADR 0045's second term is the other half: **there is no agent verb that
//! undoes, and none that proposes an undo.** This crate is reached from a
//! person's own command and from nowhere else, and it declares no verb.
//!
//! | | |
//! |---|---|
//! | [`places`] | The places kept, read off the disk, newest first — and what would not read |
//! | [`putting_back`] | The act, the plan a person approves, and every refusal |
//! | [`said`] | Each refusal, in the person's own language |
//! | `removing` | Taking away a half-made copy, including one that cannot be read |
//! | [`when`] | A moment, written the one way that reads the same everywhere |
//! | [`words`] | Every sentence this crate can say |
//!
//! # The act is Linux; the words are not
//!
//! The one call in here that has to be atomic is `renameat2` with
//! `RENAME_EXCHANGE`, and the copy is the base's own `cp --reflink`. Both are
//! this operating system's, and so is the filesystem the whole feature rests on
//! — so [`putting_back`], [`said`] and the private remover are compiled only
//! there.
//!
//! **[`words`] is not**, deliberately. `alo_saying::everything_this_machine_can_say`
//! assembles one vocabulary that has to come out the same on every host, so a
//! crate whose words are behind a target gate has to stand outside it — and a
//! crate outside it is one whose sentences no translator's file is ever checked
//! against. `alo-agentd` is apart for exactly that reason and the cost is real.
//! There is no such reason here: a sentence is a sentence anywhere. What is
//! Linux is the folder being swapped, not the words about it.
//!
//! # Why the command is not in this crate
//!
//! `alo-back` is its own crate, as `alo-before` is. The command needs
//! `alo-saying` to load translations, `alo-saying` depends on every crate whose
//! words it collects, and a crate cannot be both — so the choice is between
//! collecting these words and having the command here. The words win.

pub mod places;
pub mod when;
pub mod words;

#[cfg(target_os = "linux")]
pub mod putting_back;
#[cfg(target_os = "linux")]
mod removing;
#[cfg(target_os = "linux")]
pub mod said;

#[cfg(test)]
pub mod testing;

pub use places::{APlaceKept, Places, where_this_person_keeps};
#[cfg(target_os = "linux")]
pub use putting_back::{
    NoneWentBack, NotGoingBack, WhatGoesBack, go_back, go_back_together, what_would_go_back,
};
pub use words::{EVERY_WORD, WordsError, as_it_was_words, declare_into};
