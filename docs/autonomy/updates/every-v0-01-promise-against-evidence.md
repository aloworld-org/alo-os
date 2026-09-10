# Every v0.01 promise, against executable evidence

**Date:** 2026-09-10 · **Workstream:** v0.01 delivery plan, task 11 (phase 8's
first half) · **Contributor:** Claude Code, `C:\dev\alo-os-claude`

## What changed

Two things, and the second is what makes the first stay true.

**`docs/autonomy/v0-01-evidence.md`** — the ledger. All **forty-one** `[v0.01]`
lines in `docs/features.md`, one entry each, quoting the promise and answering
with the test or the report that shows it and what is still owed on it.

**`crates/alo-reconciling`** — the check. It parses `docs/features.md` and the
ledger together and fails the gate when they disagree:

- a promise no entry is about — **the failure that actually happened**, six
  times, one at a time, over the roadmap's seven readings;
- an entry about a promise the definition no longer makes, which is a rewording
  nobody carried through to the evidence;
- an entry whose quotation fits two promises, which would reconcile whichever it
  reached first and leave the other reading as done;
- a promise answered twice;
- **a promise with no evidence and nothing owed** — the finding this task exists
  to produce;
- a named test that is not there, or a source file offered as a test that tests
  nothing;
- something that is not evidence at all: an ADR, `ROADMAP.md`, a file in another
  repository;
- what is owed said as a shrug (`not yet`, `todo`);
- and the one that would otherwise be silent — a ledger with no entries in it,
  because a heading moved and the audit passed over a document it never found.

A user-readable description: *alo OS now checks its own release promises. Every
v0.01 promise in the feature list has to name the test or the report that shows
it, or say what is still missing and why — and a promise added without either
fails the build in the change that adds it.*

## What the audit found

Forty-one promises: **two are shown with nothing owed**, **thirty-three are shown
in part** with the rest named, and **six have no evidence at all.** The six are
the finding, and they are four different kinds of thing:

| The promise | What it is |
|---|---|
| *Copy, cut and paste* | No line anywhere: no crate, no clipboard handling in `alo-shell`, no test |
| *The GPU works on first boot* | No line anywhere: nothing in `image/` installs or verifies a driver stack |
| *And it never nags* | No line anywhere, and nothing that would notice when a surface starts asking |
| *Anything an agent verb can do, a person can do by hand* | A standing rule nothing checks |
| *The agents point at the local model by default* | Cannot get a line without a decision — see below |
| *Boots on one certified machine* | Scheduled: task 12, and it needs a machine |

**The one that needs the owner.** `docs/features.md` promises at v0.01 that *the
agents point at the local model by default — sovereignty is the default
configuration, not an option to find*. ADR 0016 settled the opposite shape:
`alo-choosing` is deliberately unable to produce a choice nobody made, because
*a default is a choice, made by whoever set it*, and a machine nobody has
configured has no answer at all. Either the promise means **setup offers the
local model first**, which is a surface nobody has built and which the ADR
permits, or it means **a default in the settings**, which the ADR refuses. I did
not narrow the promise and did not contradict the ADR: the tension is written
into the ledger and into this report, and the decision is the owner's. No code
in this change depends on the answer.

## Decisions taken, and why

**The evidence lives in its own file, not in `docs/features.md`.** The
definition is the customer-facing sentence and the scope gate; hanging a test
path off each line would make it a build artefact and put pressure on the
wording every time a test moved. A separate ledger keeps the definition
untouched — this change does not edit `docs/features.md` at all — and the
reconciler is what stops the two drifting.

**It lives under `docs/autonomy/`, beside the plan.** `ROADMAP.md` is the
release-progress document and has one writer; a second progress document in
`docs/` would compete with it. The ledger is not a verdict and says so in its
first paragraph.

**A promise is named by a quotation, not by a number or a heading.** Line
numbers move and headings get renamed; a quotation that has drifted from what it
quotes is exactly what the audit should catch. Requiring the quotation to match
**exactly one** promise is what makes rewording a promise force somebody to look
at its evidence again — which is the only moment anybody knows whether it still
holds. The hazard is real rather than theoretical: `docs/features.md` says
*neither is the other's fallback* twice, about two different pairs of things,
and the ledger has to quote enough to tell them apart. That case is the fixture
in `an_entry_that_fits_two_promises_is_refused`.

**Evidence is a test or a report, and nothing else.** A unit test beside the
code counts — several v0.01 promises are held by one — so the question asked of
a `.rs` path is whether the file contains a `#[test]`, not which directory it is
in. An ADR is where an argument was settled rather than proof anything was
built; `ROADMAP.md` cannot be its own evidence; another repository's file cannot
be run from here. All three are refused by name, with a test for each.

**Both halves of every entry.** Nearly every v0.01 promise is partly kept —
code finished, machine never asked — and recording one as *shown* would repeat
the failure `ROADMAP.md` already made once, in a line ticked outright whose own
footnote said the hardware was still owed. *Still owed* has a floor beneath it
(40 characters, `owed::AN_ANSWER`) so that `not yet` is refused; nothing
mechanical can judge whether a sentence is honest, and the floor is not
pretending to.

**It says nothing to a person and declares no strings.** The reader of a finding
is whoever is reconciling the release, in the repository, with both documents
open — the same argument `alo_saying::NotCollected` makes for keeping its
English. `alo-saying` does not collect this crate, and there is nothing here to
externalise.

**It reads no disk.** `reconcile` is handed both documents' text and a closure
that reads a file by its repository-relative path, which is what lets every
refusal be shown happening against a fixture. The integration test is what puts
the real repository behind it.

## The audit caught one of its own errors

On its first real run the reconciler refused the ledger: an entry offered
`crates/alo-driving/src/lib.rs` as showing *the catalogue says whether a model
can drive the verbs*, and that file tests nothing. It was replaced with
`exercises.rs` and `measured.rs`, which do. That is the failure mode this crate
exists for, caught within a minute of the ledger being written and by the
machine rather than by a reader.

## Acceptance, against the plan

> Every `[v0.01]` line in `docs/features.md` names the test or the report that
> shows it, or is named as owed. A promise with neither is the finding, and it
> is written down before anything else is.

- **Every line names evidence or is owed** —
  `every_v0_01_promise_is_reconciled_against_evidence_that_runs`, which runs the
  reconciliation over the real `docs/features.md`, the real ledger and the real
  files on the disk it is checked out on.
- **A promise with neither is refused** —
  `a_promise_with_no_evidence_and_nothing_owed_is_refused`.
- **A promise nobody reconciled is the finding, by name** —
  `a_promise_the_ledger_says_nothing_about_is_the_finding`.
- **The findings are written down before anything else** — the six are in the
  ledger's closing section and in this report, and nothing in this change acts on
  any of them.

## Refusal paths tested

Every finding the crate can produce is put in front of it against a fixture, and
a sound ledger is reconciled first so the refusals mean something:
`a_ledger_that_adds_up_is_reconciled_and_counted`,
`a_promise_the_ledger_says_nothing_about_is_the_finding`,
`a_promise_with_no_evidence_and_nothing_owed_is_refused`,
`evidence_that_cannot_be_run_is_refused`,
`something_that_is_not_evidence_is_refused`,
`an_entry_that_fits_two_promises_is_refused`,
`a_promise_answered_twice_is_refused`, `a_shrug_is_not_what_is_still_owed`,
`a_ledger_with_no_entries_is_checking_nothing` — plus the parser's own refusals
in `promise.rs`, `entry.rs`, `ledger.rs`, `evidence.rs` and `owed.rs`, including
a heading that moved leaving nothing to reconcile and a path quoted inside *what
is owed* being counted as showing nothing.

## Verification

Windows 11, `C:\dev\alo-os-claude`, all executed:

- `cargo fmt --all` — clean.
- `cargo clippy --all-targets --workspace -- -D warnings` — clean, zero
  warnings.
- `cargo test --workspace` — all green; `alo-reconciling` contributes 15 unit
  tests and 10 integration tests.
- `cargo test` in `tools/kernel-loop` (its own workspace, because this change
  edits the plan that tool parses) — 49 passed.
- Each evidence test run on its own with `--exact`.

Not run, and not claimed: nothing here touches hardware, a kernel, a bus or a
compositor, so there is no integration test on real hardware to owe. The audit
itself reads files.

## Limitations

- **The reconciler does not judge whether a test proves its promise.** Nothing
  mechanical can, and this crate says so in its own documentation. What it
  removes is a promise nobody reconciled and a pointer at a file that has gone.
- **It audits v0.01 only.** The same shape would serve v0.5 and v1 and is
  deliberately not built today: the tier is one constant in `promise.rs`, and
  widening it would demand forty more entries nobody has the knowledge for yet.
- **The ledger's judgement is mine.** Where I could not find a test I wrote the
  promise down as owed rather than reaching for the nearest file; the entries
  are meant to be read and argued with, and a follow-up report is how a
  correction arrives.

## Proposed updates to the shared documents

I did not edit `CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` or
`docs/autonomy/STATE.md`. Proposed:

- **CHANGELOG.md** — *alo OS now checks its own release promises: every v0.01
  promise in the feature list names the test or the report that shows it, or
  says what is still missing, and a promise added with neither fails the build.*
- **ROADMAP.md** — the six promises above are the honest state of v0.01's
  first-band claims; three of them (*copy, cut and paste*, *the GPU works on
  first boot*, *it never nags*) have no item anywhere in the roadmap either and
  need one, and *the agents point at the local model by default* needs an owner
  decision against ADR 0016 before it can have one.
- **QUEUE.md / STATE.md** — task 11 of `v0-01-delivery-plan.md` is done and
  marked in that file; task 14 (*every verb's by-hand answer, and a check that it
  has one*) was written there from these findings, because tasks 12 and 13 are
  both unstartable and a plan whose remaining tasks are all blocked reads to the
  loop as the workstream being finished.

## Status

**Ready for integration.** Not committed and not pushed; the supervisor gates
and publishes. `.kernel-loop/handoff.toml` names the task, this report, the
commit, the files and the evidence.
