//! The command a person types to put a question to the model on this machine.
//!
//! Thirteen crates in this workspace decide correctly about a question,
//! `alo-asking` is the one that puts one, and **nothing a person can type asked
//! anything**. On a machine whose first-class workload is a model the customer
//! owns, the only way to ask that model a question was the rented runtime's own
//! command — which means the rented name is what a person learns, types and
//! tells somebody else about, on a system sold on the model being theirs.
//!
//! This is the command that is ours. It reads the catalogue, asks the loopback
//! address `alo-modeld` is already listening on, and shows the answer with the
//! sentence that says where it came from.
//!
//! | | |
//! |---|---|
//! | [`which_model`] | which model on this machine answers, and the refusal when this machine cannot say |
//! | [`saying`] | the four crates' word lists, in one vocabulary, so nothing here says anything twice |
//! | `src/main.rs` | the process: the person's folder, their settings, the runtime, the question |
//!
//! # No policy here, and no words
//!
//! Every decision about a question is `alo-asking`'s, `alo-answering`'s or
//! `alo-models`'. This crate joins them up for one process and declares **no
//! strings of its own**: what a person reads is one of four crates' own
//! sentences in their own language, and [`saying`] is the table of which is
//! whose. A list here would be a second rendering of a moment three crates can
//! already describe, which is the failure `alo-asking`'s own word list was
//! written to refuse.
//!
//! The one decision that is this crate's is in [`which_model`], and it is a
//! decision only in the sense that it **refuses to be one**: it picks when there
//! is exactly one candidate and asks the person otherwise. If even that belongs
//! somewhere else, that is a finding about the split rather than licence to grow
//! it here.
//!
//! # Nothing leaves, so there is no indicator and no departure
//!
//! `Asking::to_this_machine` takes no `alo_egress::Indicator` and hands back no
//! departure, because there is nothing for either to be about: the address is
//! `127.0.0.1` and the answer is computed on this disk. A working day of these
//! is law 1's **zero inference egress**, and it is zero by the shape of the
//! call rather than by a rule somebody remembered to apply.
//!
//! # What it does not do, said here rather than discovered
//!
//! **It writes no record.** `alo_record::Entry::answered_here` is the entry a
//! local answer deserves and `alo_record::Record` is a list in memory with
//! `keep`, `everything`, `answering`, `len` and `is_empty` — **and nothing that
//! opens or flushes a file**. Every use of it outside its own crate is a test or
//! is held in a running service's memory, so an entry kept here would be
//! dropped when the process exited. Where a machine's record lives is a decision
//! nobody has made in this repository, and making it inside a command would be
//! the same mistake as spelling a portal's answers path inside a portal backend.
//!
//! **It asks this machine and nowhere else.** A person whose settings have their
//! questions answered by a provider still gets a local answer from this command,
//! and that is not the substitution
//! [ADR 0008](../../../docs/decisions/0008-where-inference-happens.md) forbids:
//! nothing is silent about typing a command named for the place it asks. What
//! that ADR rules out is a machine changing the place on somebody's behalf, and
//! there is no second door in this crate for it to change to.

pub mod saying;
pub mod which_model;

pub use saying::{NoVocabulary, what_it_can_say};
pub use which_model::{the_only_one, the_persons_model};
