//! A command the person chose, run inside the sealed box the kernel locks down.
//!
//! [ADR 0064](../../../docs/decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
//! gives the person three levels for running code, with a sealed box as the
//! default. `alo_capability::HowCodeRuns` is the choice; **this crate is the
//! box**.
//!
//! | | |
//! |---|---|
//! | [`ARun`] | A command and everywhere it may reach, checked before anything happens |
//! | [`ran`] | One command, run inside a box the kernel closes, start to finish |
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
//! # What this crate does not do, said plainly
//!
//! `alo_capability::AlwaysOn` lists the three protections ADR 0064 keeps on at
//! every level. This crate keeps **none** of them, and that is not a gap in it
//! — they are kept by whoever calls it, and two of them cannot be kept here:
//!
//! - **The record** is `alo-record`'s. [`WhatHappened`] exists so a caller can
//!   write one, and carries what ran, what it was allowed to reach and how it
//!   ended, because a caller that cannot say what ran cannot record it.
//! - **The snapshot before each run** is `alo-keeping`'s, taken *before* [`ran`]
//!   is called. It has to be: a snapshot taken from in here would be taken
//!   after the box was built and would be a snapshot of a machine already
//!   changed.
//! - **The egress indicator** is `alo-egress`'s, and what this crate owes it is
//!   the bound: a box built with no approved destination cannot connect
//!   anywhere, which the kernel enforces on `socket_connect`.
//!
//! So a caller that calls [`ran`] and writes nothing down has run a command in
//! a box and broken a promise. That sentence is here rather than in a commit
//! because this is the file somebody wiring it up will read.
//!
//! # What is still owed before a person can use this
//!
//! The surface. `docs/features.md` promises *running code at the level the
//! person picks*, and this is the mechanism for one of the three levels. The
//! setting a person chooses the level in, the surface they approve a command
//! on, and the wiring from the terminal are not built, and this crate makes
//! none of them look built: it is reached by a caller passing a level, and
//! nothing passes one yet.

#![cfg(target_os = "linux")]

mod a_run_inside_a_box;
mod entering;
mod failing;
mod what_a_run_asks_for;
mod what_happened_in_the_box;

pub use a_run_inside_a_box::{THE_HELPER, ran, ran_with_the_helper_at};
pub use entering::{MEANING_ME, THE_PROCESSES, enter_the_box_and_become};
pub use failing::NotRun;
pub use what_a_run_asks_for::ARun;
pub use what_happened_in_the_box::WhatHappened;
