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
//! Task 16 of `docs/autonomy/the-executable-plan.md` met one edge of this and
//! fixed exactly one pointer — `docs/autonomy/evidence-it-boots-and-the-agent-acts.md` is held to the
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
//! # The other family of document this repository points into
//!
//! An ADR is not the only pointer a reader believes without opening. `ROADMAP.md`
//! offers task reports as the evidence for its boxes, and the reports cite each
//! other, and a filename looks like a fact for the same reason a number does. So
//! [`reports`] holds those citations to the same two things: that the document
//! exists, and that the citation can be followed by something other than a person
//! who already knows where the reports live.
//!
//! It is here rather than in `alo-reconciling` because it is the same question
//! this crate already asks — *does this pointer land* — and because
//! `alo-reconciling` asks a different one. That crate's `evidence.rs` does check
//! that a report offered as evidence is there, but only for the entries of a
//! ledger; and its `the_gate.rs` reads report names out of prose deliberately
//! **by filename alone**, because its job there is to match a report against the
//! task it belongs to rather than to resolve a path. Neither is wrong and neither
//! was ever asked whether the repository's citations can be followed, which is how
//! 46 of them came not to be.
//!
//! # A citation that names a line is not read as a pointer at all
//!
//! `sources.rs`'s `is_a_path` requires a span to end with one of `.rs`, `.toml`,
//! `.sh`, `.bpf.c`. A citation written `crates/…/transactions.rs:41` ends with
//! `:41`, so it fails that test and is never read as a path.
//! **A line number makes a citation less checked, not more.**
//! It is the one check that would catch the file being gone, and the suffix
//! is what disqualifies it.
//!
//! Measured on 2026-10-03: **38 such citations across 23 documents** — the
//! shapes counted were paths under `crates/`, `tools/`, `docs/` or `scripts/`
//! with a source or document extension, searched in `docs/**/*.md` only, so
//! the number is a floor rather than a total.
//! **Exactly one names a file that no longer exists**, and it is invisible
//! here for the reason above.
//!
//! # Why that one is not a bug to fix, and why no fourth direction is added
//!
//! That citation is inside a pasted halt record — a transcript of what a tool
//! said at a moment, ending *preserved at halt*.
//! **Repairing a line number inside quoted tool output forges the transcript.**
//! So stripping the suffix to make these visible would surface one finding
//! whose only available action is the wrong one, and
//! **a check whose only finding must not be acted on is worse than no check**,
//! because the next person acts on it.
//!
//! The reason generalises past that one line. A citation naming a line falls
//! into one of four kinds, and only the first is a pointer a check may hold:
//!
//! | | |
//! |---|---|
//! | A live pointer, meant to be followed now | checkable |
//! | A record — pasted compiler output, a panic, a halt log | **must not be touched** |
//! | A dated measurement, true on its date | re-dating it is a new claim |
//! | One whose symbol is gone, not moved | unrepairable by any line number |
//!
//! Six of the 38 are records. A paragraph headed *Measured 2026-09-30* is one
//! too: the number was true then, so replacing it silently re-dates the claim
//! to today, where it may not hold. **Nothing mechanical can tell these apart**
//! — which is the same boundary this crate already draws when it reads a number
//! and stops, rather than judging whether `ADR 0001 §3` is about granting a
//! folder.
//!
//! # The obvious remedy does not verify itself here
//!
//! Citing by a quoted phrase instead of a line, and proving the phrase unique
//! with a count, is the remedy that would need no check — the command that
//! finds the phrase is the command that proves it unique.
//! **In this repository's prose it does not work**, because the documents are
//! hard wrapped and any phrase long enough to be unique spans a line break,
//! so a plain count returns **zero for a phrase that is present**. The share of
//! lines in the 60-to-90 character band, which is what a hard wrap looks like:
//! 76% of `ROADMAP.md`, 57% of the lane charter, 39% of `docs/features.md`.
//!
//! Whitespace has to be normalised on **both** sides before matching, needle
//! included. And the two requirements pull against each other: a phrase short
//! enough not to wrap is short enough not to be unique.
//! **That is a limitation of the remedy rather than a detail of using it**,
//! and the failure value is a zero, which reads as *stale*.
//!
//! A bare identifier cannot wrap, so a zero on one is a true zero — which is
//! what makes the fourth kind above decidable: a symbol that `grep` cannot find
//! anywhere is gone, and no line number repairs it.
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
pub mod reports;
pub mod sources;

pub use citation::{Citation, Named};
pub use citing::cited_in;
pub use decisions::{Decision, THE_DECISIONS, among};
pub use finding::Finding;
pub use holding::{Held, held};
pub use naming::named_in;
