# Whose requantisation this catalogue may vouch for

**Date:** 2026-09-11
**Workstream:** v0.01, lane B (`docs/autonomy/v0-01-lane-b-plan.md`, task 14)
**Contributor:** Claude Code, in `C:\dev\alo-os-b`
**Status:** ready for integration

## What this task was

Task 12 refused a quantisation nobody can point at; task 13 refused a size
belonging to a file the entry does not claim. Both refusals land on the same two
entries — `eurollm-9b-instruct` and `teuken-7b-instruct`, the only catalogued
models whose publishers ship no quantised artefact at all — and both left the
question underneath them unanswered: **may a catalogue entry name a stranger's
requantisation, and what is this catalogue claiming when it does?**

The deliverable is the decision, and the rule a curator reads.

## What changed

### The decision

`docs/decisions/0026-whose-requantisation-this-catalogue-vouches-for.md`, in the
shape ADR 0024 and ADR 0025 used: the question, what is true today verified off
the tree, what may not be done whichever option is taken, four options with what
each costs, a recommendation and the consequences including the ones we would
rather not have.

**Accepted as Option C** under the owner's standing delegation of 2026-09-10 —
the same delegation ADR 0021, ADR 0023, ADR 0024 and ADR 0025 were taken under —
and the ADR says so in as many words, including that the owner did not review
these options one by one. That is the precedent this repository set for a
decision a worker takes; pretending an owner reviewed it would be the worse half
of it.

**What it decides:**

- **A third party's artefact may be named**, and naming one costs three
  statements — `by` (who made it), `sha256` (which file exactly) and `note`
  (what a reader needs to know about it).
- **What this catalogue borrows is a file, never a measurement.** ADR 0007 is
  untouched: `drives_verbs` remains a grade `alo-driving` earned on a machine we
  ran it on, and nothing a requantiser claims becomes one.
- **A grade belongs to the artefact it was earned against**, so an entry that
  names no artefact may not carry a grade at all.
- **When the three statements cannot be produced, the entry is carried and not
  omitted**: no quantisation, the publisher's own release, `not-measured` —
  exactly where both European entries stand.

### The rule, where a curator reads it

`crates/alo-models/data/catalogue.toml` grows **rule 6**, beside the rules 4 and
5 that sent the question here, and rule 5 gains one line saying rule 6 is what an
entry may do instead of falling back. The two European entries' own comments now
say that rule 6 is the road and that nobody has walked it yet.

### The rule, as arithmetic

`crates/alo-models/src/requantised.rs` is a new file — `Requantised`, its three
required fields and the five things that can be wrong with one — held apart from
`catalogue.rs` under law 4: the entry's own shape and the provenance of somebody
else's file are two reasons to change.

`Catalogue::parse` gains the refusals: a requantiser named beside no artefact, a
requantiser with no name, a requantisation attributed to the model's own
publisher, a pin that is not a `sha256`, a note that says nothing — **and a grade
on an entry that names no artefact**, which was reachable until this change.
`Model::graded_against()` is the reporting side of that last one.

`Model` also gains `#[serde(deny_unknown_fields)]`. This is not tidiness: a
provenance block spelled `[model.requantized]` would otherwise be dropped in
silence and the entry would load looking like a first-party artefact, which is
the exact confusion the decision exists to prevent.

## Decisions taken where the task left something open

- **Status: accepted rather than proposed.** The task required the rule to land
  in the same change as the ADR, and a rule written into a file from a
  recommendation nobody accepted would be the decision implementing itself while
  claiming not to. The standing delegation is the precedent; it is cited, and
  the ADR names whose judgement it was so it can be overturned in one line.
- **Three statements rather than a bare permission or a bare refusal.** A
  permission is Option B, which is authority for free. A refusal is Option A,
  which reads as caution and is really an eligibility rule tracking whether a
  publisher has a distribution team — the two European entries are ungradeable
  for a reason that has nothing to do with the models.
- **A `sha256` rather than a reproducible-recipe requirement.** A recipe cannot
  be verified from here: we would be restating the uploader's claim about their
  own build. A digest read off the repository's file list is checkable by any
  reader today and makes *the file we measured* and *the file a machine fetches*
  the same file or a failed fetch. What the fetch does with the pin belongs to
  the weights work, and the ADR says so rather than implying it is done.
- **Sub-table rather than flat fields.** `[model.requantised]` is one claim with
  three parts, and the loader refuses it part-missing the way rule 4 refuses half
  a quantisation. Flat optional fields would have made three independent
  half-claims possible.
- **The reporting rule lands in the data, not in a string.**
  `crates/alo-models/src/words.rs` says why this crate does not invent person-
  facing words for a panel nobody has drawn; `Model::graded_against()` and the
  entry's own `requantised` block are what a panel will read.
- **No entry was completed.** Naming a file for either European model moves a
  size, a memory figure, an `on_cpu` word and a licence question per uploader.
  Doing it inside the decision would have been the decision quietly choosing, so
  it is task 15, written into the plan in this change.

## Acceptance criteria and the tests behind them

`crates/alo-models/tests/whose_requantisation_this_catalogue_vouches_for.rs`:

| Criterion | Test |
| --- | --- |
| An ADR in the shape 0024 and 0025 used, answering all four questions | `the_decision_is_recorded_and_answers_what_it_was_asked` |
| The recommendation is written into the catalogue's own rules | `the_rule_is_written_into_the_catalogues_own_rules` |
| A third party's artefact is nameable when the entry says whose it is | `an_artefact_somebody_else_made_may_be_named_when_the_entry_says_whose_it_is` |
| Every way a borrowed file can be named without being vouched for is refused | `a_borrowed_artefact_that_is_not_stated_is_refused` |
| A grade is reported as the artefact's, never the publisher's | `a_grade_names_the_file_it_was_earned_against` |
| A decision, not an implementation: no entry, no size, no grade moved | `no_entry_was_completed_and_no_grade_moved_in_the_change_that_decided_this` |

`crates/alo-models/src/requantised.rs` holds the per-decision refusals:
`a_statement_that_says_all_three_things_holds`,
`every_way_a_borrowed_file_can_be_named_without_being_stated_is_refused` and
`a_digest_that_is_not_one_is_refused`.

The refusal paths are the bulk of it on purpose. A rule this file has already
broken twice in prose is only a rule if the entries it exists to refuse are
refused where a reader can see it happen.

## Verification

Run in WSL Ubuntu from `/mnt/c/dev/alo-os-b`, 2026-09-11:

- `cargo fmt --all` — clean.
- `cargo clippy --all-targets -p alo-models -p alo-driving -p alo-citing -p alo-image -p alo-saying -- -D warnings` — clean.
- `cargo test -p alo-models -p alo-citing -p alo-driving` — all passed.
  `alo-citing` is included because this change adds a decision file and cites it
  from rustdoc; `alo-driving` because it parses a catalogue fixture of its own.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-models` — clean.
- `cargo test` in `tools/kernel-loop` — 76 passed, which is the plan this change
  edits still parsing.

**Not run, deliberately:** the full workspace suite. The supervisor runs it after
this worker; two finished tasks have died at the deadline waiting on it.

**Nothing was measured against a model.** No runtime was started, no weights were
fetched, and no grade moved. This is a decision task and its constraint says so.

## The gate refused this once, and it was the build directory

The supervisor refused this task's first hand-over, twice on the same tree, with
five errors that all said the same thing:

```
error[E0609]: no field `requantised` on type `Model`
    = note: available fields are: `id`, `name`, `publisher`, `parameters_b`, `quantisation` ... and 8 others
error[E0599]: no method named `graded_against` found for struct `Model`
```

Thirteen fields and no `graded_against` is `Model` **as `main` has it** — not as
this tree has it. Both were in `crates/alo-models/src/catalogue.rs` when the gate
ran and had been for half an hour. Nothing was deleted, nothing was rewritten and
no test was dropped: a second worker took the tree as it stood and went looking
for what the compiler was actually reading.

**What was run, and what it said.** All of it in the lane's own gate directory,
`CARGO_TARGET_DIR=/root/target-claude`, on 2026-09-11:

- `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings` and
  `cargo test -p alo-models` — clean, and the six integration tests pass.
- The same tree on Windows (`cargo 1.97.1`) as well as WSL (`cargo 1.98.0`) —
  `cargo test -p alo-models` passes on both.
- `cargo build --workspace --all-targets` — **reproduced the gate's refusal
  exactly**, and did so immediately after a `cargo clean -p alo-models` that
  reported `Removed 171 files, 385.4MiB total`.
- Reading the unit the failing test was compiled against —
  `--extern alo_models=…/libalo_models-b78e1cf798cb1a8a.rlib` — its `.rmeta`
  contains no occurrence of `requantised`, and its dep-info names
  `src/catalogue.rs` and `data/catalogue.toml` and never `src/requantised.rs`.
  So the workspace build was handed an `alo-models` that predates this change,
  while the per-crate build of the same source in the same directory minutes
  earlier was not.
- Removing that package's artefacts by hand —
  `rm -rf …/debug/.fingerprint/alo-models-*` and
  `rm -f …/debug/deps/libalo_models-*` — and running
  `cargo build --workspace --all-targets` again: **`Finished dev profile in
  53.79s`, exit 0.**

**The conclusion, stated no further than the evidence goes.** The refusal was the
build directory and not the tree. The mechanism by which a freshly-run
`cargo clean -p` leaves a unit Cargo then compiles from sources it no longer has
is *not* established here and is not claimed — what is established is that the
clean did not drop the unit, that a manual purge of the same package's
fingerprints and artefacts did, and that the whole workspace builds afterwards.

**This is the same failure `tools/kernel-loop/src/gates.rs` already documents** —
*a cached unit of the callee that Cargo considered fresh and which predated the
method* — and it is the failure `forget_what_was_built_of` was added to prevent.
That function runs `cargo clean -p <crate>` for each crate a task touched, which
is exactly the step shown above to be insufficient. **Proposed as its own task,
not taken here:** have that function remove the package's `.fingerprint/<name>-*`
directories and `deps/lib<name>-*` artefacts directly rather than delegating to
`cargo clean -p`, with a test in `tools/kernel-loop` that a poisoned directory is
actually emptied. It is a change to the program that gates every lane and it does
not belong inside a catalogue task, which is why it is written down here instead
of made — and why it is written down at all: this has now refused or parked
finished work at least five times.

The lane's build directory has been left in the repaired state, so the next gate
run is about the work.

## Limitations, stated

- **The pin is stated and not yet verified at fetch time.** Nothing in this
  repository fetches a catalogued artefact yet; when something does, it checks
  the digest the way `image/Containerfile` already checks every pinned artefact.
  The ADR records this as an obligation it creates and does not discharge.
- **Naming an artefact does not make a model measurable here.** Task 9 measured
  that this lane's 5,926 MB guest cannot run a 7B model at four bits at a useful
  speed. Rule 6 removes the *decision* blocker on the two European entries; the
  *machine* blocker is unchanged and is an owner's decision about hardware or
  `C:\Users\SBW\.wslconfig`.
- **A pin ages.** A requantiser who re-uploads a better file makes our entry
  stale rather than wrong, and a curator has to notice. That is the correct
  direction for the error to fall and it is written into the ADR.

## Proposed shared-document updates

For the integration owner (`CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md`,
`docs/autonomy/STATE.md` are not edited here):

- **CHANGELOG.md**, under the unreleased v0.01 section:
  > **The catalogue says whose file it means.** A model whose publisher ships no
  > quantised weights may now be catalogued against somebody else's conversion —
  > and only when the entry names who made it, pins exactly which file it is, and
  > says what a reader needs to know about it. A grade is still a measurement alo
  > OS ran, and it now names the file it was run against; an entry that names no
  > file cannot carry a grade at all (ADR 0026).
- **QUEUE.md / STATE.md:** lane B task 14 done; task 15 — *The file the two
  European entries mean* — written and ready. No task in
  `docs/autonomy/v0-01-delivery-plan.md` matched this one, so nothing was marked
  there.
- **`docs/autonomy/v0-01-evidence.md`:** no promise changes state. The catalogue
  promises already carry their evidence; this decision narrows nothing and ticks
  nothing.

## Files

- `docs/decisions/0026-whose-requantisation-this-catalogue-vouches-for.md` (new)
- `crates/alo-models/src/requantised.rs` (new)
- `crates/alo-models/src/catalogue.rs`
- `crates/alo-models/src/lib.rs`
- `crates/alo-models/data/catalogue.toml`
- `crates/alo-models/tests/whose_requantisation_this_catalogue_vouches_for.rs` (new)
- `crates/alo-models/tests/sizes_an_entry_can_point_at.rs` — the rule count it
  reads moved from five to six
- `docs/autonomy/v0-01-lane-b-plan.md`
- `docs/autonomy/updates/whose-requantisation-this-catalogue-may-vouch-for.md`
