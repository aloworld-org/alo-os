//! Every decision this repository points at, held to being one somebody
//! actually wrote — and every decision held to saying what it is.
//!
//! `CLAUDE.md` makes the ADRs binding: *read the ADR before proposing an
//! alternative*. So this repository points at them constantly — `ADR 0001 §3` in
//! a crate's rustdoc, `ADR 0009` inside a finding's own sentence, `ADR 0016` as a
//! promise's evidence, four hundred times over. **Nothing checked that any of
//! those pointers landed.**
//!
//! An ADR reference is the one kind of pointer a reader believes without
//! opening, because the number looks like a fact. A number with no file behind it
//! is therefore not a broken link: it is a sentence that has borrowed the
//! authority of a decision nobody made, and it reads exactly like one that has
//! not.
//!
//! Task 16 of `docs/autonomy/v0-01-delivery-plan.md` met one edge of this and
//! fixed exactly one pointer — `docs/autonomy/v0-01-evidence.md` is held to the
//! decision it names, because *waits on a decision* sending a reader to an ADR
//! nobody wrote reads exactly like an answer. This crate is the same answer for
//! every other citation in the repository, and it is the fourth in the family
//! after `alo-reconciling` (promises against evidence), `alo-by-hand` (verbs
//! against their plain way) and `alo-collected` (words against the one
//! vocabulary).
//!
//! # The citations are read out of the text, never out of a list kept here
//!
//! [`cited_in`] and [`named_in`] read the repository's own sentences, so a
//! citation added anywhere is a citation this check sees, with nothing to update.
//! **Nothing here holds a decision's number or a file's name** — not the
//! twenty-five that exist and not the neighbouring repository's four. Comparing
//! two lists written by hand would only ever prove that two lists agree.
//!
//! # Three directions, because a pointer fails in more than one
//!
//! [`held`] answers all three:
//!
//! | | |
//! |---|---|
//! | A number, or a filename, that no decision answers for | [`Finding::ADecisionNobodyWrote`], [`Finding::AFileNobodyWrote`] |
//! | Two files claiming one number | [`Finding::TwoDecisionsOneNumber`] |
//! | A decision whose own file does not say what its status is | [`Finding::ADecisionWithNoStatus`] |
//!
//! The third is not about a pointer landing, and it is here because of what
//! happens *after* one lands: an ADR nobody recorded as accepted or proposed is
//! one every reader will read as settled. The citation resolves, the file opens,
//! and the weight of a decision is taken by a recommendation still waiting on the
//! owner — which is exactly what `0025-the-default-is-what-a-machine-arrives-able-to-do.md`
//! is.
//!
//! # It judges the pointer, never the argument
//!
//! Whether `ADR 0001 §3` is really about granting a folder is a reader's job and
//! nothing mechanical reaches it. The same goes for a section: this check reads
//! the number and stops there. `ADR 0019 §3` is a numbered heading, `ADR 0020 §2`
//! is not one at all, and `ADR 0004 §policy` is not a number — so resolving a
//! section would mean either imposing a document convention this check has no
//! standing to decide, or producing findings nobody could act on. That is a
//! decision rather than an oversight, and it is written here so the next reader
//! does not have to infer it.
//!
//! # A neighbouring repository's decisions
//!
//! `alo-workplace` has ADRs of its own and this repository cites four of them.
//! Refusing those would be this check demanding a true sentence be deleted, so a
//! citation is read as this repository's **unless the line names the repository
//! it points into, before the number**. That makes the rule checkable and it
//! makes a demand on writing at the same time: keep the repository's name with
//! the number it qualifies, because a reader cannot scroll up for it either.
//!
//! # It says nothing to a person
//!
//! Nothing here reaches a screen. The reader of a [`Finding`] is whoever wrote
//! the citation, in the repository, with the line open — so the findings keep
//! their English and their `Display`, exactly as `alo_reconciling::Finding`,
//! `alo_by_hand::Finding` and `alo_collected::Finding` do for the same reason.
//! There are no strings to externalise here, and **`alo-saying` does not collect
//! this crate** — which `alo-collected` is what proves, since a crate with no
//! `src/words.rs` is not a crate that declares words.
//!
//! # It reads no disk
//!
//! [`held`] is handed the decisions, the files and the neighbours. The test in
//! `tests/` is what puts the real repository behind it. That is what lets every
//! refusal be shown happening against a fixture: **a check that has never been
//! seen to refuse anything passes on the day it stops looking, in exactly the
//! same colour.**

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod citation;
pub mod citing;
pub mod decisions;
pub mod finding;
pub mod holding;
pub mod naming;

pub use citation::{Citation, Named};
pub use citing::cited_in;
pub use decisions::{Decision, THE_DECISIONS, among};
pub use finding::Finding;
pub use holding::{Held, held};
pub use naming::named_in;
