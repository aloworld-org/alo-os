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
//! dropped when the process exited.
//!
//! *Two sentences stood here saying that where a machine's record lives is a
//! decision nobody has made in this repository. **That was false when it was
//! written and it is corrected rather than deleted**, because it is the reason
//! this paragraph reads as settled when it is not.* Measured 2026-10-18:
//! `docs/contracts/record-file.md` is a published contract that names
//! `/var/lib/alo-letting-go/record.jsonl` and `/var/lib/alo-forgetting/record.jsonl`,
//! [ADR 0029](../../../docs/decisions/0029-what-the-kernel-writes-down-about-a-turn.md)
//! fixes the shape as one additive line per entry synced per entry, and **14
//! crates name `record.jsonl`** — `alo-keeping` in nine files. What is true is
//! narrower and is the only thing this paragraph may claim: **this command has
//! no road to that file**, which is a missing caller and not a missing decision.
//!
//! **It asks this machine and nowhere else, and for most people that answers
//! nothing.** A person whose settings name a provider gets no answer from this
//! command at all.
//!
//! The paragraph here used to argue that
//! [ADR 0008](../../../docs/decisions/0008-where-inference-happens.md) permits
//! this, because *nothing is silent about typing a command named for the place
//! it asks*. **That argument does not survive reading the ADR, and it does not
//! survive the command's own name.** ADR 0008: *it assumed everybody runs a
//! model. Most people will not* — and *a hosted API is a supported first-class
//! option and never a silent fallback*. This command is called `alo-ask`, which
//! names no place;
//! [ADR 0095](../../../docs/decisions/0095-the-release-carries-no-model-and-a-person-brings-their-own.md)
//! ships no weights, so **the ordinary state of a fresh machine is one where
//! this command cannot answer**. What ADR 0008 forbids is a *silent* change of
//! place, which is a reason to say where an answer came from — not a reason to
//! have one door.
//!
//! Every transport part of the other road already exists:
//! `alo_choosing::Settings::provider`, `alo_choosing::what_answers_here` and
//! `alo_asking::Asking::to_a_provider`.
//!
//! **And it still cannot be called from here, for a reason worth more than this
//! crate.** Law 1 wants an egress *visible at the moment it happens and
//! afterwards in a record*. `alo_egress::Indicator` is a `Vec` in process
//! memory; `docs/contracts/native-egress-indicator.md` says in as many words
//! that it is *only the drawing surface* and adds **no protocol, agent
//! capability, D-Bus interface or stored format**; and `docs/contracts/record-file.md`
//! names services that append, not a road a command may use. So a command that
//! asked a provider would show the departure **to itself** and drop it on exit
//! — a law 1 violation wearing the very API that exists to prevent one, which
//! compiles and type-checks and shows nobody anything.
//!
//! Law 1 is satisfiable today only inside the compositor and the agent daemon.
//! This is the first person-facing command that would cause egress and the
//! first to find that out; the next one meets the same wall. **How a process
//! that is neither of those reports an egress and keeps an entry is a public
//! surface and needs an ADR**, which is recorded as task 11 of
//! `docs/autonomy/the-terminal-in-full-plan.md` — written there as work and a
//! named block, rather than left here as a justification.

pub mod beside_an_answer;
pub mod saying;
pub mod which_model;

pub use beside_an_answer::beside_an_answer;
pub use saying::{NoVocabulary, what_it_can_say};
pub use which_model::{the_only_one, the_persons_model};
