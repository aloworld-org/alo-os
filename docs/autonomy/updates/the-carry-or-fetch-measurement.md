# The carry-or-fetch measurement ADR 0025 owes

**Date:** 2026-09-11
**Workstream:** v0.01 lane B — accounts and session entry
(`docs/autonomy/v0-01-lane-b-plan.md`, task 8)
**Contributor:** Claude Code worker, this checkout

## What the task was

ADR 0025 recommends carrying the model weights on the certified image and
fetching only where an image cannot — and says in as many words that this is a
recommendation with a measurement owed, owed **first**, before anybody puts
weights aboard. The measurement is two numbers and a sentence: the smallest
catalogued model that clears the verb-driving bar, what the update channel can
honestly carry, and the answer — carry, or fetch, or carry-here and
fetch-there — written in `docs/quirks.md` beside the numbers.

## What the measurement found

**The first number does not exist.** Off `data/catalogue.toml` rather than
from memory: twelve entries; the five anybody has run `alo-driving` against
(2026-09-04, recorded in `crates/alo-models/src/catalogue.rs`'s `MEASURED`
list and in `docs/quirks.md`'s models section) all graded `rarely`; the seven
others say `not-measured`, and `Driving::NotMeasured` refuses the bar on
purpose — an unmeasured model is not a candidate, it is a gap the ledger
already carries. So **no catalogued entry clears the verb-driving bar**, and
the plan's own constraint makes that finding the deliverable: the weights task
waits on the catalogue rather than on wishes.

**The channel half, honestly bounded.** ADR 0011 makes the OS a bootc OCI
image with content-addressed layers, so a carried weights layer travels once
per weights *change* rather than once per update, and rides inside the atomic
deployment `bootc rollback` restores — where a setup-time fetch sits outside
it, which is an argument *for* carrying, not only a cost. The sizes in
question (1.06–2.4 GB for the CPU-class entries, 4.4–4.9 GB for the 7B class)
are the order of what the image already moves in its pinned base and runtime
artefact. What is not claimed: no registry, update stream or mirror of ours is
running, so transfer time on the certified machine's network, hosting cost and
large-layer behaviour have **not been measured on real infrastructure** — the
entry states the structural argument as reasoning, never as a measurement.

**The sentence:** today, **no weights go aboard** — neither carried nor
fetched — because there is nothing to carry. When an entry clears the bar, the
answer the channel half supports is ADR 0025's own recommendation: carried on
the certified image, fetched at setup only where an image cannot, remembering
the ADR's caution that a machine which fetches at setup is not local by
default when it is offline at setup.

## What changed

- `docs/quirks.md` — new entry under *Models*: **The carry-or-fetch
  measurement ADR 0025 owes: the catalogue has nothing to weigh.** The full
  table of twelve entries (id, `download_bytes`, `drives_verbs`), the channel
  reasoning with its honest bound, and the sentence — beside the numbers,
  where ADR 0025 says the answer goes.
- `crates/alo-models/tests/the_carry_or_fetch_measurement.rs` — the entry
  parsed rather than admired, in the shape
  `the_unwatched_mutations_are_written_down.rs` set: every size and grade in
  the table is compared against `Catalogue::built_in()`, every catalogued
  entry must be in the table, the verdict must match what the catalogue says
  now, the channel and sentence paragraphs must exist and be answers rather
  than shrugs, and the plan must carry the follow-on task. The refusal paths
  are tested beside the legitimate one: a checker that has never been seen to
  fail is one nobody should believe, so every way the measurement can rot —
  an entry that clears the bar while the document says none does, a size or
  grade from memory, a model the catalogue does not offer, a dropped row, a
  verdict with no candidate under it, an empty table, an empty catalogue — is
  put in front of it and shown refused.
- `docs/autonomy/v0-01-lane-b-plan.md` — task 8 marked **Done, 2026-09-11**
  with the finding, and task 9 (*The grade the weights wait on*) written in
  the same change: measuring the three smallest unmeasured CPU-workable
  entries with `alo-driving`, grades only from runs actually made, next task
  written from the outcome.

## Decisions taken rather than asked

- **The finding is recorded in `docs/quirks.md`, and the ledger is proposed
  rather than edited.** The acceptance names `docs/quirks.md` as where the
  answer goes and it is there. `docs/autonomy/v0-01-evidence.md`'s entry *The
  local model is what the machine arrives ready to run* still reads *whether
  the weights ride on the certified image or are fetched at setup — a decision
  inside the work*; the integration owner may want to add one sentence
  pointing it at the quirks entry. Proposed wording: *"The carry-or-fetch
  measurement was made on 2026-09-11 (`docs/quirks.md`, the models section):
  no catalogued entry clears the verb-driving bar, so no weights go aboard
  yet, and the weights wait on the catalogue — lane B task 9."* Not applied
  here because the ledger is maintained beside the audit's own reading of it
  and consolidation is the integration owner's.
- **Task 9 is the catalogue measurement, not the weights-aboard task.** The
  acceptance asks for "the weights task that builds on the answer"; the answer
  is that the weights wait on the catalogue, so the task that builds on it is
  the grade — the weights-aboard task would be a task nobody can start, and
  writing it now would send a worker at wishes.
- **The verdict check is future-proof in both directions.** The test does not
  hardcode "nothing clears forever": if a catalogued entry is later measured
  `reliably`, the checker refuses the stale entry with *the measurement must
  be made again*, and once the entry is rewritten it requires the smallest
  clearing model to be named in the sentence. Both futures are exercised in
  the refusal test with fabricated catalogues.
- **The channel is argued structurally and bounded explicitly.** Inventing a
  bandwidth number for infrastructure that does not exist would be the exact
  failure ADR 0025 warns about — a promise published instead of a measurement.
  The entry says what is structural (layer semantics, rollback, size class)
  and names what is unmeasured.

## Verification

Platform: Windows 11, this checkout, product workspace.

- `cargo fmt --all` — clean (run before handoff; see handoff evidence).
- `cargo clippy --all-targets` with warnings denied — clean.
- `cargo test --workspace` — green.
- Each evidence line in `.kernel-loop/handoff.toml` was run on its own with
  `cargo test --package alo-models --test the_carry_or_fetch_measurement
  <name>` before being written down.

Executed checks: all of the above on this machine. Pending/physical: nothing —
this task is a measurement over the repository's own data and documents; the
runs behind the five existing grades were made 2026-09-04 and are not re-run
or re-claimed here.

## Limitations

- The channel half is reasoning with a stated bound, not a measurement on real
  infrastructure; the entry says so in as many words, and whoever stands up
  the registry owes the numbers.
- The grades this measurement reads are the catalogue's: measured in English,
  envelope included, on one box — the caveats `docs/quirks.md`'s *What
  `drives_verbs` measures* entry already carries. Nothing here re-runs or
  re-interprets them.
- Task 9 needs a machine with at least ten gigabytes free for the runtime,
  which the 6 GB measuring box was not; the plan says so beside the task.

## Proposed shared-document updates (integration owner's)

- `CHANGELOG.md`: "The carry-or-fetch measurement ADR 0025 owes was made: no
  catalogued model is measured driving the verbs reliably, so no weights go
  aboard the image yet — the finding, the update-channel reasoning and the
  answer are recorded in docs/quirks.md and held to the catalogue by a test."
- `docs/autonomy/v0-01-evidence.md`: the sentence proposed under *Decisions*
  above.

## Status

Ready for integration.
