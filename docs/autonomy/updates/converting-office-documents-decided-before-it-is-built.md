# Converting office documents, decided before it is built

**Date:** 2026-09-14
**Workstream:** v0.5 — documents and paper (`docs/autonomy/v0-5-documents-and-paper-plan.md`, task 2: *`.docx`, `.xlsx`, `.pptx` — opened, and what the conversion cost*)
**Contributor:** Claude Code worker in `C:\dev\alo-os`, for the repository owner
**Status:** ready for integration, **as a decision**. The code for task 2 is
**not written** and waits on ADR 0039 and the two prerequisites it names. This
report does not claim the formats open.

## What changed, for a person

Nothing a person can use yet, and this report says so plainly. Opening a Word,
Excel or PowerPoint file on alo OS needs a converter that runs on the machine,
and choosing how it runs decides three things people are promised: that nothing
leaves the machine without them seeing it, that the agent never runs a program
of its own choosing, and that it only ever reaches files a person granted. This
change writes that choice down for the owner to answer. Its recommendation: a
small converting service that has no network and cannot see any folder, handed
only the one document it converts. The copy is a PDF, written beside the
original and never over it, and it names what the copy lost: a missing font, a
date that is now fixed, a linked picture that was not fetched.

## Why this task became a decision

Task 2's acceptance says the three formats are opened "through the rented
converter running on this machine, with a test per format against a real file".
The first line of that code has to choose things that are not a worker's to
choose. Checked in the repository and on the gate machine on 2026-09-14:

1. **Nothing in the product starts a program.** `alo-bounding/src/lib.rs`
   (*what runs in the cgroup, and why nothing is started*) says a turn's work is
   one thread, because *a program alo OS starts on an agent's behalf is one
   review away from a program an agent named*. The only rented engine, the model
   runtime, is started by systemd (`image/usr/lib/systemd/system/alo-modeld.service`).
2. **Whether a process forked from a turn's thread stays inside the turn's
   kernel boundary is unmeasured.** If it does not, the converter reads with the
   person's whole authority. That would be a grant widened in fact, which ADR
   0013 forbids and which a worker may never do.
3. **Office documents link to pictures, data and templates by address**, and an
   office engine fetches them when it converts. That is egress the person never
   saw (law 1).
4. **The image pins no converter** (`image/Containerfile`; `alo-image` checks the
   model runtime alone), and `alo-saying`'s list of everything we rent has none.
5. **The gate machine has no converter.** `which soffice` found nothing in WSL
   Ubuntu 24.04. Installing a package there is shared maintenance that
   `docs/autonomy/SHARED_MAIN.md` reserves for an explicit idle handoff. A test
   that skips when the engine is missing is the gate weakened.
6. **No machine the loop runs on has an office suite**, so no worker can make
   the "real file rather than a synthesised one" the acceptance asks for.

Reasons 1 and 2 put the obvious code at odds with how law 2 and the
kernel-enforced grant are kept. Reasons 5 and 6 are outside a worker's
authority. So, as the task instructions direct, the decision is the work.

## What changed, in the repository

- **`docs/decisions/0039-a-document-is-converted-by-an-engine-that-can-reach-nothing.md`**
  (new, *proposed*). It sets out:
  - **Three options**, each with what it costs and what it keeps:
    - **A**, a converting service of our own. It is socket-activated, runs as
      its own login with no network, and is handed descriptors opened by the
      verb's thread inside the turn.
    - **B**, the verb starting the engine inside the turn.
    - **C**, the person's own office application through an adapter.
  - **The rejections:** a converter of our own, and uploading anything.
  - **The recommendation, A**, and what is decided with it:
    - the copy is a PDF;
    - the verb is `convert_document(file, into)`, a change with grants over both;
    - the copy is created with `O_EXCL` and never overwrites;
    - what the copy lost is found by inventorying both documents, never from the
      engine's log, as a closed set (font substituted, field fixed at its value,
      macros not carried, linked content not fetched, comments and tracked
      changes not shown);
    - `Carried::Everything` is reachable only after both inventories complete,
      and an inventory that cannot complete refuses the copy, so there is no
      *not checked* variant;
    - it rents an inflater and an XML reader, used only in the service's crate
      and capped against compression bombs;
    - every conversion and refusal is recorded;
    - the code goes in a new crate, `alo-converting`.
  - **The three prerequisites:** the owner's answer; the pinned engine on the
    gate machine through an idle handoff; three real documents with their
    provenance.
- **`crates/alo-opening/tests/converting_waits_on_its_decision.rs`** (new, 5 tests).
  It uses the same shape as `alo-bounding`'s tests for ADR 0029 and ADR 0030:
  - The ADR exists once under 0039 and still stands.
  - Task 2 of the plan names the ADR, and while it is proposed the task is
    `blocked` and not marked done. This is the refusal path: the loop is never
    offered a task that waits on the owner.
  - The ADR carries three options, three costs, a recommendation, *nothing is
    uploaded*, and the three prerequisites.
  - While it is proposed, nothing converts: no `crates/alo-converting`, no
    shipped source naming the verb, and no converter named in the image recipe.
  - A self-check confirms that everything the file reads by is still where it
    looks.
- **`docs/autonomy/v0-5-documents-and-paper-plan.md`**: task 2's status is now
  `blocked` on ADR 0039, with a paragraph saying why and what it waits on. It is
  **not** marked done. Task 4 depends on task 2 and so waits with it. Task 3
  (printing) is unaffected and is the loop's next task in this plan.

## Decisions made in this task, and why

1. **The ADR route rather than B or C in code.** B contradicts the kernel
   boundary as `alo-bounding` states it, and rests on an unmeasured kernel
   behaviour. C narrows *the documents people are actually sent open*. A is
   buildable only with a pinned image component and an engine on the gate
   machine that a worker cannot install. No option could be finished and gated
   today.
2. **Status `blocked`, not `scheduled`.** The task waits on a decision first and
   on arrangements second.
3. **The test lives in `alo-opening`**, the crate whose `ThisMachine::converts`
   the conversion registers into. It reads files and adds no dependency, so task
   1's `deciding_never_leaves_the_machine` still holds unchanged.
4. **No measurement of the engine was made.** Downloading and running it in a
   scratch folder was possible, but the ADR's argument does not rest on its
   behaviour, and a half-measured engine in a proposal reads as settled. What
   the ADR states about it (it converts the three formats headless, and it
   resolves linked content) is upstream's documented behaviour. It is marked for
   measurement when the gate machine has the engine.

## Acceptance criteria and where each is shown

This task's deliverable is the decision. The plan's code criteria are **not met**
and are listed as pending.

| Criterion | Shown by |
|---|---|
| The decision exists, under its number, and stands | `converting_waits_on_its_decision::the_decision_exists_once_under_its_number` |
| The plan points at it and does not offer task 2 while it is proposed (refusal path) | `converting_waits_on_its_decision::the_plan_points_at_the_decision_and_steps_over_the_task_while_it_waits` |
| Options, costs, recommendation, no upload, prerequisites | `converting_waits_on_its_decision::the_decision_sets_out_the_options_their_costs_and_a_recommendation` |
| No converter is built ahead of the owner (refusal path) | `converting_waits_on_its_decision::nothing_converts_while_the_decision_is_proposed` |
| The checks cannot go quiet | `converting_waits_on_its_decision::the_checks_would_notice_what_they_read_going_missing` |
| **Pending:** each format opened through the converter on this machine against a real file | not started; waits on ADR 0039 and its prerequisites |
| **Pending:** what the conversion could not carry, by name; *lost nothing* distinct from *not checked* | not started; shape decided in ADR 0039 |
| **Pending:** the copy is written where the person chose, never over the original | not started; shape decided in ADR 0039 |
| **Pending:** converting is an `alo_capability` verb under a grant on the file's folder | not started; shape decided in ADR 0039 |

## Verification

Platform: Windows Server 2022 host. Gates run in WSL Ubuntu 24.04 as root with
`CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-88e6ebddb0cab76e`.

| Check | Command | Result |
|---|---|---|
| Format | `cargo fmt --all`, then `cargo fmt --all --check` | clean |
| Lint | `cargo clippy -p alo-opening --all-targets -- -D warnings` | 0 warnings |
| Tests | `cargo test -p alo-opening` | 38 unit; 6, 5, 2, 6, 4 integration; 1 doctest; 0 failed |
| Citations | `cargo test -p alo-citing` | 21 + 10 passed (the new ADR and the new test's citations resolve) |
| The loop reads the plan | `cargo test every_plan_this_repository_drives_holds_only_tasks` in `tools/kernel-loop` | passed |

**Not run:** the whole workspace suite (the supervisor runs it). **Not
measured:** the converter's behaviour, and where a process forked from a
threaded control group lands. Both are named in the ADR as unmeasured.

## Remaining limitations

- Every code criterion of task 2 is outstanding (above).
- Task 4 (*"I can't open this file", said properly*) depends on task 2 and waits
  with it.

## Proposed shared-document updates (for the integration owner)

- **CHANGELOG.md:** "How alo OS will open Word, Excel and PowerPoint files is
  now a written proposal for the owner. A converter on the machine that has no
  network and can see no folder makes a PDF copy beside the original, and says
  what the copy lost. Nothing opens yet."
- **ROADMAP.md:** no box changes. `.docx`, `.xlsx`, `.pptx` *open* stays
  unticked.
- **QUEUE.md / STATE.md:** documents-and-paper task 2 is blocked on ADR 0039
  (proposed). It needs the owner's answer, the pinned engine installed on the
  gate machine in an idle handoff, and three real documents from the owner.
  Task 3 (printing) is next in this plan; task 4 waits on task 2.
