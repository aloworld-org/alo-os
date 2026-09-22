# A machine with no engine says so, instead of failing

**Date:** 2026-09-22
**Workstream:** v0.5 — documents and paper
**Task:** 9 of `docs/autonomy/v0-5-documents-and-paper-plan.md`,
*A machine with no engine says so, instead of failing*
**Contributor:** the third PC (`AGAI01`), at the owner's instruction. This plan
belongs to the development PC; this one task was assigned across because the
condition can be *produced* here — this machine has the pinned engine, so its
absence can be created and undone.
**Status:** ready for integration.

## What a person would notice

Nothing. No sentence a person reads changed, no verb changed, and no conversion
behaves differently. This is a change to what the test suite does on a machine
that cannot run the converter.

## The problem

Every run of `cargo test -p alo-converting` on the fleet's Mac failed the same
tests. The engine `alo-converting` drives is an x86_64 build pinned by digest;
the Mac is aarch64, so there is no release of it that machine could install.
The failures were addressed to nobody: a suite that is permanently red is one
people learn to read past, and the next real failure hides inside it.

## Nine, ten, and what the difference actually was

The plan recorded that the Mac reports **ten** failures and that a measurement
on x86_64 — `C:\dev\setup\which-tests-need-the-engine.sh`, which moves the
engine's directory aside, runs the suite and puts it back — found **nine**. It
said the difference was unresolved and might be something about the
architecture beyond a missing file, and told this worker to reconcile the two
rather than assume.

**It was neither machine. It was the measurement.** `cargo test` stops at the
first test *binary* that fails, so the original run ended inside
`converting_a_real_document.rs` and never started the targets after it. Re-run
on this machine on 2026-09-22 with `--no-fail-fast`
(`C:\dev\setup\which-tests-need-the-engine-all.sh`), the same condition produces
**ten**, and the two machines agree. The tenth is
`the_walk_from_a_file_arriving_to_one_that_cannot_be_opened_reads_as_the_table`
in `the_walk_through_documents_and_paper.rs`, which runs the engine at steps 4
and 5 of the walk. Its own module doc said it does not skip itself, which was
the reason to look at it.

This is recorded in the plan and in that file, because the next person to reach
for that script should know what it under-reports. It is not in
`docs/quirks.md`: that file is for hardware, application automation and pinned
engines behaving unlike their manuals, and `cargo test` doing exactly what it
documents is none of those.

## The decision this needed first

**ADR 0039 forbade it in as many words.** Under *What must happen before task 2
can be built* §2: *the conversion tests then **fail loudly** where it is absent,
and are never `#[ignore]`d and never skipped. A test that skips itself when the
engine is missing reports green on exactly the machines where nothing converts.*
Task 2's own entry in the plan repeats it: *a conversion test that skips itself
where the engine is missing is refused (the decision's own rule).*

That is not ambiguity to interpret around, so the code could not be written
until the decision was. `docs/decisions/0063-a-machine-that-cannot-run-the-engine-says-so-rather-than-failing.md`
sets out the new fact ADR 0039 did not have, what ADR 0039 was actually
protecting, three options with their costs, the decision and four terms that are
part of it, and the consequences. A note in ADR 0039 points at it, amending
those two sentences and nothing else; ADR 0039 keeps its status and everything
else it decided, and `crates/alo-opening/tests/converting_waits_on_its_decision.rs`,
which holds it in place, is unaffected.

**On its status line, which is the one judgement worth checking.** ADR 0063 is
recorded as *accepted, 2026-09-21, by the owner*, and what it rests on is stated
there: the owner wrote task 9 — the decision, the shape it takes and its two
constraints — argued it, and committed it in `b467e6a`. An ADR records a
decision; it is not the decision. If the owner intends this one to want a
separate acceptance of its own, the status line is the single thing to change
and the code is what it decides.

## What changed

**`crates/alo-converting/tests/asking/mod.rs`** (new) — whether this machine can
run the engine, asked by **running it**. `engine.rs`'s own argument list and
cleared environment, in a scratch home made and removed with the ask, with the
question *what are you* in place of what it is asked to produce. Four named
reasons, each carrying where the engine was looked for: nothing there that could
be started, started and never answered (killed and waited for), ran and ended
badly, ran and said nothing. Asked once per test binary through a `OnceLock`.

**Why it is not `test -x`.** The plan forbade that proxy, and the reason is
concrete rather than stylistic: the engine on a machine is a shell script that
starts a binary beside it, so on a machine of another architecture the
executable bit is set on a wrapper over a binary that cannot run. `alo-image`
already asks the image `test -x`, and this plan has shipped a defect from that
bit standing in for *the program runs*.

**Why the ask stops at "does it run".** A fuller ask — convert a document and
check a PDF came out — was considered and refused. An engine that starts and
then exports badly is a defect on a machine that *has* an engine, and ADR 0039
is right about those: it must fail loudly, in the test that converts, where the
failure names the conversion. An ask that converted first would turn that defect
into a skip, which is the silence the whole change exists to prevent. The narrow
ask is the correct one, not merely the cheap one, and it is term 2 of ADR 0063.

**`crates/alo-converting/tests/converting_a_real_document.rs`** — nine tests ask
before anything else and, where the engine cannot be run, print what was missing
and return. No assertion changed. Module doc rewritten: it said *these tests do
not skip themselves*, which is no longer true, and a doc that is wrong about a
rule is worse than no doc.

**`crates/alo-converting/tests/the_walk_through_documents_and_paper.rs`** — the
tenth, the same way, with the measurement that found it written down beside it.

**`crates/alo-converting/tests/a_machine_that_cannot_run_the_engine_says_so.rs`**
(new) — the ask's refusal paths and the list's own check:

- asking this machine answers what the engine is, or says why not — always runs,
  on every machine, so the ask itself is never an untested path;
- nothing there is a reason naming where it looked;
- a file that is not a program is a reason;
- **a wrapper whose binary cannot run is a reason** — the aarch64 shape written
  down, with the executable bit asserted set, so the case `test -x` gets wrong
  is in the suite rather than in an argument;
- a program that ends well and says nothing is not an engine;
- one that never answers is stopped and is a reason — and the program it started
  is gone, checked by a file the program would have written a second later;
- every reason names where the engine was looked for;
- **exactly these tests ask** — the sources read both ways against the measured
  ten, so a conversion test added without the ask fails, and an ask added to a
  test that does not need one fails too. Each must ask *before it does anything
  else*. Without this the narrow skip becomes a general one by drift, which is
  the thing ADR 0039 feared, and it is term 4 of ADR 0063.

**`docs/decisions/0063-…`** (new) and the amendment note in **ADR 0039**.

**The plan** — task 9 marked done with what was built and what was measured, and
**task 10 written**, because nothing followed task 9 and a plan that names no
next task sends the loop back at work already done.

## Decisions taken, that the task left open

- **The ask lives in `tests/`, not in `src/`.** It is a test concern, and
  shipping a public function in a product crate that only tests call would put
  it in a contract other people build against. It is a shared test module, the
  pattern this crate already uses for `making` and `reading_source`.
- **It reaches the engine's path through `alo_converting::engine::THE_ENGINE`**,
  so `engine.rs` remains the one file that knows the name — held by
  `converting_reaches_nothing.rs`, which still passes.
- **The skip's shape is `alo-in-use`'s**, as the plan required: a printed line
  and a return. Worth being plain about its limit — `cargo test` captures a
  passing test's output, so the ten lines appear under `--show-output` or
  `--nocapture` and not in a default run's summary. That is the shape the plan
  named and the shape this repository already uses in four crates; making skips
  visible in a default run is a change to how every crate reports, not something
  to invent inside one.
- **The tenth test skips too.** The plan's acceptance names nine and tells the
  worker to reconcile the list against a real run rather than assume. The
  reconciliation says ten, so ten ask, and the ninth-versus-tenth question is
  closed rather than carried.
- **Task 10 is the older `.doc`, `.xls` and `.ppt`** — named by ADR 0039's *What
  this does not decide*, and by task 2's own *Owed* line, and the exact shape
  task 7 already used for the OpenDocument three. It is written honestly as
  blocked on one real file of each, which is the same blocker task 2 carried and
  the owner cleared on 2026-09-16.

## Verification

Run on `AGAI01`, on the Linux side (`Ubuntu` under WSL2, x86_64), against the
serialized source copy at `/root/alo-trees/this-machine` synchronized from this
checkout, building in `/root/alo-builds/this-machine`, as
`docs/autonomy/SHARED_MAIN.md` requires. The engine on this machine is the
pinned release at `/opt/libreoffice26.2`, and it was moved aside and restored on
every exit path including an interrupt.

| Gate | Result |
|---|---|
| `cargo fmt --all` then `cargo fmt --all -- --check` | clean |
| `cargo clippy -p alo-converting --all-targets -- -D warnings` | clean |
| `cargo clippy -p alo-citing -p alo-reconciling -p alo-conforming -p alo-opening --all-targets -- -D warnings` | clean |
| `cargo test -p alo-converting` | 120 passed, 0 failed, across ten targets |
| `cargo test -p alo-citing` | passed — the new decision resolves, and so does every pointer at it |
| `cargo test -p alo-reconciling` | passed |
| `cargo test -p alo-conforming` | passed |
| `cargo test -p alo-opening` | passed — `converting_waits_on_its_decision` still holds ADR 0039 |

**The acceptance itself, measured twice:**

- **With the engine present:** `cargo test -p alo-converting` — 120 passed, 0
  failed, nothing skipped. Every one of the ten runs and passes.
- **With the engine moved aside:** the same crate — 120 passed, 0 failed, and
  exactly **ten** `skipped:` lines, each reading *this machine has nothing at
  `/opt/libreoffice26.2/program/soffice` that could be started (No such file or
  directory (os error 2)), so there is nothing on this machine to convert a
  document with*. Before this change the same condition produced ten failures.

**Not run, deliberately:** the full workspace suite. It takes the better part of
an hour on this machine and the supervisor runs it after this regardless; the
crates this change can reach were run here instead.

**Not measured:** a run on the Mac. This machine reproduced the condition rather
than the hardware, so what is held is *the engine cannot be run*, which covers
both a missing file and a binary for another architecture. The first run of the
suite on the Mac after this lands is what confirms the ten there are the ten
here; the wrapper refusal test is the aarch64 shape in the suite in the
meantime.

**No hardware acceptance is claimed.** Nothing here touches a certified machine,
and a green run on a machine with no engine is not evidence that anything
converts — ADR 0063 says so in its consequences.

## Proposed shared-document updates

For the integration owner; this report does not edit them.

**`CHANGELOG.md`** — under the current release:

> A machine that cannot run the document converter now says so and skips the
> ten tests that need it, instead of failing them on every run. The check runs
> the converter rather than looking for its file, because the file can be there
> on a machine where it cannot run. A converter that starts and then converts
> badly still fails loudly. Nothing a person sees or does has changed.

**`docs/autonomy/QUEUE.md`** — task 9 of the documents and paper plan is done;
task 10 (older `.doc`, `.xls`, `.ppt`) is added and is blocked on one real file
of each from the owner, the same way task 8 is blocked on a real Keynote and a
real Numbers document.

**`docs/autonomy/STATE.md`** — reference this report and
`docs/decisions/0063-a-machine-that-cannot-run-the-engine-says-so-rather-than-failing.md`,
whose status line records an owner decision taken in the plan rather than in a
separate answer, and is the one judgement in this change worth a second reader.

**`ROADMAP.md`** — no change. No exit gate moves.

## Limitations

- The ten skip lines are visible under `--show-output` or `--nocapture`, not in
  a default run's one-line summary. See *Decisions taken* above.
- `which-tests-need-the-engine.sh` is left as it is, on the third PC and outside
  this repository; `which-tests-need-the-engine-all.sh` beside it is the
  corrected run. What it under-reports is recorded in the plan and in the walk's
  module doc, which are inside the repository.
- Whether other crates' suites are red on the Mac for reasons nobody there can
  fix is unmeasured. It would be a task of its own, and ADR 0063 says so rather
  than guessing.

## Files

- `crates/alo-converting/tests/asking/mod.rs` (new)
- `crates/alo-converting/tests/a_machine_that_cannot_run_the_engine_says_so.rs` (new)
- `crates/alo-converting/tests/converting_a_real_document.rs`
- `crates/alo-converting/tests/the_walk_through_documents_and_paper.rs`
- `docs/decisions/0063-a-machine-that-cannot-run-the-engine-says-so-rather-than-failing.md` (new)
- `docs/decisions/0039-a-document-is-converted-by-an-engine-that-can-reach-nothing.md`
- `docs/autonomy/v0-5-documents-and-paper-plan.md`
- `docs/autonomy/updates/a-machine-with-no-engine-says-so.md` (this report)
