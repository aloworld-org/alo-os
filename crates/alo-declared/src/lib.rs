//! Every verb alo OS ships, on one list — and that list held to the workspace
//! that has the crates.
//!
//! Two workspace tests ask a question of *every verb this machine ships*:
//! `alo-by-hand` asks whether a person can do the same thing without the agent
//! (ADR 0009), and `alo-software` asks whether any of them reaches the terminal
//! (ADR 0043). Neither can read the verbs out of thin air. Each has to call
//! every crate that declares any, and until now each kept its own copy of the
//! list — a hand-kept array of crate names beside a hand-kept run of
//! `declare_into` calls, in two files, neither of which any task ever means to
//! touch.
//!
//! # What that cost, on one day
//!
//! Three times on 2026-09-17 a lane was refused for a change it never made:
//!
//! - `alo-software`'s terminal test spelled out the crates that declare verbs.
//!   It broke when `alo-converting` was added, and again for `alo-capturing`.
//! - `alo-by-hand`'s test kept the same names and its own copy of the calls. It
//!   broke for `alo-capturing` too.
//! - `alo-saying`'s two lists of the crates that declare *words* carried a
//!   written length, so two lanes each adding a crate merged cleanly and then
//!   failed to compile — five times on 2026-09-16 alone.
//!
//! Every one of them broke every lane on every machine until somebody fixed a
//! list their own task never touched. So there is one list, here, and both tests
//! are handed it.
//!
//! # Why the list cannot write itself, said plainly
//!
//! The obvious fix is to have nothing written down at all: walk the workspace,
//! find the crates with verbs in them, call each one. It cannot be done, and the
//! reason is not effort. **A test cannot call a crate it does not depend on.**
//! `alo_files::declare_into` is a path the compiler resolves against this
//! crate's own `Cargo.toml`, so the calls are Cargo dependencies, and a
//! dependency is written down by whoever adds it. Nothing derived at build time
//! reaches into the dependency graph to add one.
//!
//! What *is* derived is the only half that matters: **whether the list agrees
//! with the workspace.** [`held`] walks this workspace's own member list through
//! [`alo_by_hand::whoever_declares_verbs`], and a crate that declares verbs and
//! is not on [`WHO_DECLARES_THEM`] is a finding naming it — and naming the one
//! file to add it in, because whoever reads that finding has just written the
//! crate and has two lines to write.
//!
//! So this is not a list that maintains itself. It is a list that cannot drift
//! from the workspace without saying so, in one place instead of three, and the
//! second half of that — that the names and the calls beside them cannot drift
//! from each other either — follows from deriving every view from the entries
//! in [`shipped`]. Its tests also run each declaration and require real verbs.
//!
//! # It says nothing to a person
//!
//! Nothing here reaches a screen. The reader of a [`Finding`] is whoever is
//! adding a crate, in the repository, with `Cargo.toml` open — so the findings
//! keep their English and their `Display`, exactly as `alo_collected::Finding`
//! and `alo_by_hand::Finding` do for the same reason. There are no strings to
//! externalise here, and `alo-saying` does not collect this crate.
//!
//! # It reads no disk
//!
//! [`held`] is handed the manifest's text and a way to read a file by its
//! repository-relative path. The test in `tests/` is what puts the real
//! repository behind it. That is what lets every refusal below be shown
//! happening against a fixture: **a check that has never been seen to refuse
//! anything passes on the day it stops looking, in exactly the same colour.**

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod finding;
pub mod holding;
pub mod not_declared;
pub mod shipped;

pub use finding::Finding;
pub use holding::{Held, held};
pub use not_declared::NotDeclared;
pub use shipped::{WHERE_THE_LIST_IS, WHO_DECLARES_THEM, every_verb_this_machine_ships};
