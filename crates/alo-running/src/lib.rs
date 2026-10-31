//! A command the person chose, run inside the sealed box the kernel locks down.
//!
//! [ADR 0064](../../../docs/decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
//! gives the person three levels for running code, with a sealed box as the
//! default. `alo_capability::HowCodeRuns` is the choice; **this crate is the
//! box**.
//!
//! | | |
//! |---|---|
//! | [`AskedToRun`] | A command somebody asked to run, as asked, before anything judged it |
//! | [`WhereCommandsRun`] | Where commands run and where that is written down — the one road in |
//! | [`checked_and_written_down`] | Its first half: a request judged, and a refusal recorded |
//! | [`SomewhereToWrite`] | Somewhere a run can be written down, which a run cannot happen without |
//! | [`ARun`] | A request that has been checked, which is what the checking produces |
//! | [`WhatHappened`] | What a run did, in a form the record can take |
//! | [`NotRun`] | Why a command was not run, which is always a refusal |
//! | [`enter_the_box_and_become`] | What `alo-enter-the-box` does: admit itself, then become the command |
//!
//! # This is the one crate in the repository that starts a program
//!
//! [ADR 0117](../../../docs/decisions/0117-the-thing-that-starts-a-program-is-not-the-thing-that-bounds-a-turn.md)
//! put it here rather than in `alo-bounding`, and the reason is worth having at
//! the top of the file a reviewer opens first.
//!
//! `alo-bounding` holds the boundary for **agent turns**, which are one thread
//! of `alo-agentd` running one of six enumerated verbs. It guarantees that it
//! starts nothing, and the guarantee is a test — `a_turn_is_this_thread.rs`
//! reads its own source and fails on `Command`, `fork(` or any spelling of
//! `exec`. That guarantee is about the crate that enforces every file grant on
//! this machine, and it stays.
//!
//! What a person chose under ADR 0064 is a different thing. Law 2 has two
//! halves, and the second is *running code is a separate grant that only the
//! person gives*. So the `exec` lives here, in the crate that exists because
//! somebody chose it, and never in the crate that serves a closed verb list.
//!
//! **Law 2's first half is untouched.** Nothing here is a verb. The broker's
//! list does not gain `exec`, a shell, or anything else; this is reached
//! through the person's own grant and no agent can call it.
//!
//! # The record is kept here, and cannot be skipped
//!
//! `alo_capability::AlwaysOn` lists the three protections ADR 0064 keeps on at
//! every level. **This crate keeps the first of them**, and keeps it
//! structurally: [`WhereCommandsRun`] takes somewhere to write and there is no
//! constructor that does not, so a caller cannot reach a run holding nothing to
//! record it in. Every run and **every refusal** is written down, which is
//! `CLAUDE.md`'s gate in its own words.
//!
//! *This file said the opposite until 2026-10-16* — that the record was the
//! caller's. That was correct about ownership and wrong about where the promise
//! lives: a protection every level keeps is not one a caller may forget. The
//! shape is `alo_turn::Machine`'s, which takes an `alo_turn::Kept` for exactly
//! this reason, and [`SomewhereToWrite`] says why it is not that trait.
//!
//! **And a failure to write is a refusal.** A run whose record could not be
//! written does not happen, because a protection ADR 0064 calls always on is
//! not one that may be skipped because a disk was full.
//!
//! The other two are not this crate's and cannot be:
//!
//! - **The snapshot before each run** is taken *before* a run is asked for, so
//!   it cannot be taken from in here: one taken after the box was built would be
//!   a snapshot of a machine already changed. It is also **not built** — nothing
//!   takes a bracket, measured 2026-10-16, and ADR 0045 terms 8 and 9 are what
//!   unblocked it. *The folder it would go in is built as of 2026-10-17 —
//!   `alo-giving-a-folder` makes `/var/lib/alo-undo` — and nothing starts that
//!   unit yet either.*
//! - **The egress indicator** is `alo-egress`'s, and what this crate owes it is
//!   the bound: a box built with no approved destination cannot connect
//!   anywhere, which the kernel enforces on `socket_connect`.
//!
//! # What is still owed before a person can use this
//!
//! The surface. `docs/features.md` promises *running code at the level the
//! person picks*, and this is the mechanism for one of the three levels. The
//! setting a person chooses the level in, the surface they approve a command
//! on, and the wiring from the terminal are not built, and this crate makes
//! none of them look built: it is reached by a caller passing a level, and
//! nothing passes one yet.
//!
//! What is no longer owed is the record, which is why this paragraph is shorter
//! than it was.

#![cfg(target_os = "linux")]

mod a_run_inside_a_box;
mod carrying_what_it_says;
mod entering;
mod failing;
mod somewhere_to_write;
mod what_a_run_asks_for;
mod what_a_try_may_reach;
mod what_happened_in_the_box;
mod what_was_asked;

pub use a_run_inside_a_box::{THE_HELPER, WhereCommandsRun, checked_and_written_down};
pub use carrying_what_it_says::{Carrying, carried_until_it_ends};
pub use entering::{MEANING_ME, THE_PROCESSES, enter_the_box_and_become};
pub use failing::NotRun;
pub use somewhere_to_write::SomewhereToWrite;
pub use what_a_run_asks_for::ARun;
pub use what_a_try_may_reach::{
    NoPlacesForIt, THE_EMPTY_DEVICE, WHERE_PROGRAMS_ARE, what_a_try_may_reach,
    what_only_here_may_reach, where_the_libraries_are,
};
pub use what_happened_in_the_box::WhatHappened;
pub use what_was_asked::AskedToRun;
