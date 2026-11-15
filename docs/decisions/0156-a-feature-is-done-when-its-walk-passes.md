# ADR 0156 — a feature is done when its walk passes

**Status:** **accepted, 2026-11-14, by the owner**, relayed by the coordinator.
Offered the five working rules below, with their cost, the owner answered:
*"Yes please adopt all of them"*. The rules and the cost are the owner's
decision; the wording, and the account of which of them anything checks, is the
lane's.
**Date:** 2026-11-14

## What it changes for the person

**Fewer things that work in the tests and fail on their machine.** On
2026-11-13 and 2026-11-14 every fault that stood between a person and the second
demonstration was the same kind of fault: two pieces of alo, each finished and
each green, that did not fit together where one hands over to the other. A
person asked alo to open an application by its name and alo could not find it;
a model was told a verb but guessed what its arguments were called. Nothing any
test ran would have caught either, because each test checked one side against
its own idea of the other.

The cost is time: **each change takes about 20 to 30 per cent longer**, and the
owner accepted that cost in accepting the rules. What it buys is that *done*
means a person can do the thing on a booted machine, not that its pieces pass.

## Why

The evidence, every item of which can be re-read on GitHub:

1. **#1249** — *an installed application is reached by the name it goes by.*
   alo was asked for an application by name and looked for it by an id; each
   half was tested against its own notion of the other.
2. **#1251** — *a model is held to each verb's own argument names, and told
   them.* The model was offered a verb and guessed the argument names, because
   the names it was told and the names the verb reads were written in two
   places.
3. **A grant nothing could make** — the shape `CLAUDE.md` already records from
   2026-10-25, found again: the verb that checks a grant and the surface that
   should create it each passed, and no person could create one.
4. **#1245** landed on 2026-11-14 and **broke the image build from `main`** for
   hours, and nobody saw it until the next build after a merge. #1257 is the
   fix. The image is built only after a merge, so a change that breaks it is
   found only once it is already in `main`.

`CLAUDE.md` says it already: *our tests check that each end works, and nothing
checks that the two ends meet. `docs/walks/` is where the meeting is checked.*
These rules make that sentence the way work is done, rather than a thing a lane
remembers.

## The decision

1. **Walk first.** Before code, the steps a person takes are written as an
   automated walk: described in `docs/walks/` and run by
   `crates/alo-walking` in the **Walk** workflow on GitHub
   (`.github/workflows/walk.yml`, which boots the image under KVM). **A feature
   is done when its walk passes**, not when its pieces pass. Where a step
   cannot be walked yet — hardware a runner has not got, a sound nobody can
   hear on a runner — the walk says so as a step that is *not reached*, and the
   feature is reported as not yet walked rather than as done.
2. **Seam tests.** Every hand-over between two pieces — a line on a socket, a
   verb's argument, an application's id, a D-Bus call — has a test that runs
   **both real sides**: the real sender against the real receiver. A test of
   each side against its own copy of the other does not count, because the
   copy is exactly where the two disagreed in #1249 and #1251.
3. **One source for names.** Application ids, verb names and argument names
   are declared once and read everywhere they are needed — by the code that
   acts, the code that tells a model, and the tests. **A name typed by hand in
   a second place is a bug**, and is fixed by reading it from the first.
4. **The image is built on every pull request**, not only after a merge, so a
   change that breaks the build — #1245's fault — is refused before it lands.
   That build **is a check and never publishes**: it is not pushed, signed,
   pinned or handed to anybody, exactly like a walk image, so it does not
   break the rule that *no real image is built while any branch but `main`
   exists*. It is built by `.github/workflows/image-check.yml` (#1259), on the
   pull request and on the merge queue's own commit, and
   `crates/alo-image/tests/the_image_check_publishes_nothing.rs` holds it to
   publishing nothing.
5. **One lane owns one feature from start to finish.** A feature is not split
   across lanes — the lane that writes its walk writes both sides of its seams
   and lands it. A seam with an owner on each side is a seam nobody owns.
6. **Analyse before running** (added 2026-11-15). A walk takes about an hour,
   so it is the last place a fault should be found, not the first. Before a
   push or a walk:
   - **trace the request through the real code**, from the words a person
     types, to what the model is given, to the verb it should choose, to the
     service that carries it out, and to the exact line the walk waits for,
     quoting `file:line` at every hand-over. A line the walk waits for that no
     quoted code prints is a break found by reading;
   - **try the model's turn alone**, with its real instructions and verbs, at
     least three times, and record what it chose each time. That costs
     minutes, and it shows whether the model ever takes the road the walk
     expects;
   - **have a second reader review the diff**, handed the code rather than
     the writer's conclusion.

   The trace and the tries go in the pull request's body. *The owner,
   2026-11-15: "Can we pay attention during the writing so as we analyse well
   the code before we run it to catch those errors"*, after the VSCodium walk
   failed three times on `main`, at a different step each time, with nobody
   having followed the road through the code first.

## Which of these anything checks

Under `CLAUDE.md`'s *say whether a measurement was performed once or runs
forever*, honestly:

- **Rule 1 is checked only when a walk runs.** The Walk workflow runs when a
  person starts it; #1256, open as this is written, proposes running it after
  every merge. **Nothing checks that a feature has a walk at all**, or that a
  walk was written before the code: that part is a discipline.
- **Rule 2 is a discipline.** Each seam test runs on every gate once it is
  written, but nothing finds a seam that has none.
- **Rule 3 is a discipline, with partial checks.** Where a name is read from
  one place the compiler holds it; where #1251's check lands, a verb's argument
  names are held to the verb. Nothing searches for a hand-copied name in
  general.
- **Rule 4 is a check that runs on every pull request.** Its status, *the
  image builds*, was read on 2026-11-14 among the checks branch protection
  requires of `main`, beside `alo/gates-on-a-runner`, so a change whose image
  does not build cannot land. Which checks are required is the owner's
  setting, and stays so.
- **Rule 5 is a discipline nothing checks.** It runs when a lane chooses to
  follow it.
- **Rule 6 is a discipline nothing checks.** No gate reads a pull request's
  body, so the trace and the tries are there when a lane puts them there, and
  a reviewer who finds them missing should say so.

## What this does not change

The gate in `CLAUDE.md` stands whole: formatting, clippy, unit tests with the
refusal path, the capability guarantees as tests, and the integration test on
real hardware. A walk under KVM is a virtual machine; it is not the hardware
test, and passing one does not excuse the other.
