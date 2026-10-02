# The queues — one active task per machine, two or three behind it

**One shared order, four queues.** `ROADMAP.md` says what alo OS must deliver
and in what sequence; `docs/features.md` is the only list of what gets built;
the plans under `docs/autonomy/` hold the task text. **This file says only who
is doing what next.** It is not a second roadmap and it must never become one:
three separate roadmaps would drift, duplicate work and leave dependencies
unresolved.

*Written 2026-10-02, after a day in which the fleet delivered 14 merges against
a five-day average of 40, with two gaps of 5h18 and 2h11 in which every machine
was idle and the merge queue was empty. The protocol was not the only cause.
**Nothing told a machine what to start next.***

## The working rule

**When you submit a task, start the next ready task in your queue.** If a task
is blocked, record the exact dependency here and take another. Return to a
failed check promptly. Ask the owner only when a decision exceeds your
authority — a tier, a contract, an ownership row, or a design that does not
exist.

**Submitted is not done.** A task is done when its acceptance is met and the
change has landed, never when the writing about it is finished.

## What a task carries

| Field | Means |
|---|---|
| **Outcome** | the behaviour a person receives |
| **Acceptance** | how we know it works |
| **Owner** | one machine, responsible for completion |
| **Depends on** | tasks or decisions it genuinely needs |
| **Scope** | components and shared contracts it touches |
| **Verification** | the tests or design checks required |
| **State** | ready, working, submitted, blocked, done |
| **Evidence** | branch, pull request, verification result |

Two or three ready tasks behind the active one, and no more. A long queue is a
stale queue, and the plans already hold everything further out.

## How a landing works now

    commit → push → open the pull request → enqueue → start the next task

CI checks the head and the queue commit and posts the required status itself.
**No lane types a status, holds the queue, waits for a turn, or needs `main` to
be still.** A lane's own full gate is diagnostic; run what is useful for the
crates you touched and let CI own validation. See
[SHARED_MAIN.md](SHARED_MAIN.md), whose opening records what was retired.

**Open the pull request at the first coherent push.** Unfinished delivery needs
a visible object: on 2026-10-01 a branch sat 13 hours unlandable while its lane
measured its own hardware four times, and the reason nothing had landed was that
no pull request had ever been opened.

---

## A — the development PC, lane B checkout (`alo-os-lane-b`)

Crates: `alo-appearance`, `alo-dock`, `alo-shortcuts`, `alo-choosing`,
`alo-changing`, `alo-kept`, `alo-dividing`, `alo-desktops`, `alo-keyboards`.
Full gate 136 s.

**First responsibility, 2026-10-02: completed.** The CI-required-check
transition is made and verified end to end on `#363`. **This machine stops
being the fleet's gatekeeper** — it no longer gates another lane's queue commit,
because nothing needs it to.

### Active — a frame is never lost

- **Outcome:** a person cannot drag or resize a frame to where they can no
  longer reach it, and the fixed controls cannot swallow one.
- **Acceptance:** task 8 of
  [the-smallest-canvas-worth-showing.md](the-smallest-canvas-worth-showing.md),
  whose status says the current tests are *evidence of progress, not
  completion*.
- **Depends on:** nothing.
- **Scope:** `alo-canvas`'s never-lost rule and `alo-shell`'s fixed-control
  bounds. **The rule had no caller at all until the Dock's band was handed to
  it** — every rectangle it had seen was invented by a test.
- **Verification:** the rule driven by a real drag rather than a constructed
  rectangle.
- **State:** ready.

### Ready — the display scale reaches hit-testing, handles and exclusion

- **Outcome:** a dense screen draws the shell at the right size, not half of it.
- **Acceptance:** the owner's direction of 2026-10-01 — tests at **1.25×, 1.5×
  and 2×** over layout, pointer hit-testing, frame handles and fixed-control
  exclusion.
- **Depends on:** the test fixture being able to set a scale. `Fixture::new()`
  and `keyboard()` take no arguments and `render(size, fail, time)` has no
  scale, so **those tests cannot be written yet.** That capability is the first
  half of this task.
- **Scope:** `alo-shell`'s test harness; no product behaviour.
- **Verification:** each test watched failing with the scale withheld, as the
  division's was.
- **State:** ready. *The division's half landed as `#362`/`#363`.*

### Ready — the logical-coordinate boundary

- **Outcome:** a reader of the shell can tell which values are logical and which
  are physical.
- **Acceptance:** the owner's direction — *introduce an explicit
  logical-coordinate distinction at the appropriate boundary; audit consumers by
  what the values actually represent; do not mechanically replace all 93
  usages.*
- **Depends on:** the scale tests above, so the audit follows evidence. **`Physical`
  is smithay's own marker type and is not renamed** — engines are configured,
  never patched.
- **Scope:** `alo-shell`, `alo-displays`' one conversion, and whatever ours is
  called.
- **Verification:** the mislabel it fixes has already produced two wrong
  diagnoses in one hour and one owner ruling against a premise that was false.
- **State:** ready.

### Blocked — the Dock at any edge

Task 11 of the canvas plan. **Blocked on the left, right and top designs, which
do not exist.** Five states each, and the side variants need the alo Bar as a
horizontal composer beside the Dock rather than the composer rotated. No code
moves this.

---

## B — the desktop PC, panel lane (`task/panel/...`)

Crates: `alo-put-aside`, `alo-canvas`'s region work, and
`.github/workflows/gate.yml` with the gate automation supporting it. Full gate
909 s.

### Active — the dispatcher

- **Outcome:** a machine that finishes a task starts the next one without a
  person prompting it.
- **Acceptance:** a durable dispatcher over this file's queues. It claims work
  so two machines cannot take one task; it records one outcome per return —
  *submitted*, *changes needed*, *blocked*, *no eligible work*, *session
  failed*; it persists task identity so a restarted session resumes rather than
  duplicates; it limits retries; and a watchdog distinguishes *worker idle while
  eligible work exists* from *a twenty-minute compiler job*, by inspecting
  sessions and CI runs rather than silence.
- **Depends on:** nothing. **This is the second of the two failures**: the first
  was the delivery path, now fixed; this is that nothing restarts a machine.
- **Scope:** tooling, outside the product. **Version-controlled**, unlike the
  three lanes' gate scripts, each of which was found on 2026-10-01 not to
  enforce the rule it printed.
- **Verification:** a session ended deliberately with eligible work queued, and
  the next task observed starting.
- **State:** ready.

### Ready — the four unwired shell callers

`ask_for`, `peek_at`/`stop_peeking`, `alo_is_now`, `what_an_agent_may_do`.
Taken under *work a task needs is part of that task*; none changes a signature
another lane depends on, so splitting them breaks nothing.

### Ready — the pointer classifier into `alo-put-aside`

`surface_areas.rs` and `panel_region.rs`, handed over on
`handover/dev-pc/the-pointer-classifier`, with
`no_figure_from_the_design_reaches_the_code` carried across **in the same
change** — a source check that stops applying because the source moved is the
rule quietly ceasing to exist. `Revealing` gets a home there.

### Ready — three stale statuses in the put-aside plan

Tasks 5, 6 and 7 read *blocked on the panel not being drawn* and *blocked on
there being no full screen*. The panel has been drawn since `#343`; full screen
landed as `#352`. **Three available tasks reading as blocked.**

### Also — retire `alo/this-queue-commit-needs-a-lane`

It exists to tell a person to go and type the other status. Nothing requires
either now. It is the last of the old protocol inside `gate.yml`, which is this
machine's file.

---

## C — the Mac (`task/mac/...`)

Crates: `alo-canvas`, `alo-arranging`, `alo-portals`, `alo-granted`,
`alo-applications`, `alo-secrets`, `alo-access`, `alo-conforming`,
`alo-formats`, `alo-adapting`, `alo-hosted`, `alo-capturing`, `alo-in-use`, and
the devices and media crates. Full gate **57–97 minutes**, unpredictably: 3.9 GB
of RAM on an 8 GB host, two of six cores usable, and 2999 s of one 5344 s run
was `cargo doc` over 104 crates.

**This machine does not run the full gate to land.** It runs focused checks for
what it touched and lets CI validate. Its hardware stopped being a fleet problem
on 2026-10-02.

### Active — a frame moves between Places

- **Outcome:** a person drags a frame out to the World and drops it into another
  Place, and the work goes with it.
- **Acceptance:** task 3 of
  [the-canvas-and-its-places.md](the-canvas-and-its-places.md).
- **Depends on:** tasks 1 and 2, both done — **task 2 landed the World on
  2026-10-02 and task 1 on 2026-09-30.**
- **Scope:** `alo-canvas`.
- **State:** ready. *This read `blocked on 1, 2` while both were finished.*

### Ready — a restore travels, it does not relocate

Task 4. Returns a put-aside window to **the Place it was already on**, the view
travelling there. Separate from task 3 on purpose.

### Ready — every Place is where it was left

Task 5. Position, size, camera and the panel's own state, **per Place rather
than per session**, over `alo-arranging`.

### Ready — the ADR about what controls are called

The second half of task 8 of the access-and-language plan is blocked on it, and
**this machine is the one that understands why it matters.** It reaches 24
languages and `docs/contracts/person-settings.md`. *A blocker that is a decision
is an ADR to write, not a wait.*

---

## D — the third PC, installer lane

Crates: `alo-installer`, `alo-installing`, `image/`, `alo-image`, and
`.github/workflows/` except `gate.yml`.

**No branch from this lane since 2026-09-29.** It holds the only pure-code task
left in v0.01.

### Ready — the image carries the compositor a machine boots to

- **Outcome:** a machine installed from the image reaches the sign-in screen
  rather than a console.
- **Acceptance:** task 40 of
  [v0-01-delivery-plan.md](v0-01-delivery-plan.md).
- **Scope:** a build stage on the base's own distribution — the current one is
  static musl and **cannot link Wayland at all** — the five development
  packages, two binaries now that `alo-desktop` has split off, the unit with
  `ALO_DISPLAY`, `ALO_PERSON` and `ALO_KEYBOARD`, and `alo-image` holding it to
  *whatever holds the privilege holds nothing else*.
- **Depends on:** nothing buildable. Its last clause — *a machine installed from
  that image reaches the sign-in screen* — needs a real display, like tasks 38,
  39 and 12.
- **Verification:** `alo-image`'s own checks; the display half named as owed
  rather than ticked.
- **State:** ready, unstarted.

### Ready — what the install leaves behind can be watched

Task 21 of the installer plan. *Depends on nothing*, and it blocks the snapshot
loop paying off and every later walk that watches an installed system.

---

## Reporting

Four lines, and no request for the next instruction:

    Landed:  PR #…
    Working: task …
    Blocked: none / the exact dependency
    Next:    task …
