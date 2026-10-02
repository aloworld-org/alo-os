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

> **What the third hop found, 2026-10-02, before touching anything.**
>
> Two hops are landed: `TheRoom` names the value at the boundary (`#387`) and
> reaches the background and the picture (`#390`). The third hop is
> `dock_raster` and the desktop raster path, and measuring it first turned up
> something the first two did not.
>
> **Within one `desktop_raster::picture` call, one surface is converted by the
> display's scale and two are not:**
>
> ```text
> dock_raster::picture(dock, look, size, ..)            size unchanged
> panel_raster::picture(.., size, ..)                   size unchanged
> division_raster::picture(.., i32::from(display_scale), ..)   converted
> ```
>
> And in the direct path, `size` is the output mode's own resolution —
> `direct_target`'s `size()` returns `self.output.output.mode.size()`. The
> Dock's measures come from `Measure::of(TextScale)`, so **nothing in the Dock's
> layout sees the display's scale at all.**
>
> **What that does NOT mean, corrected the same day it was written.** I first
> wrote that the Dock would appear at half size on a two-times display. It
> cannot, today: `alo-desktop` passes `display_scale: 100` as a **documented
> placeholder**, so nothing anywhere receives another value. Its comment already
> names the consequence and the cause — *this binary holds no `Screens` and no
> `alo_displays::Scale`, so it has nothing true to put here... until this binary
> reads the displays, a two-times screen draws a division at half the room it
> owns.*
>
> So the Dock not receiving the scale is a **latent** question, not a live
> fault, and the live blocker is upstream of it: the binary does not read the
> displays. **A finding is only worth what its reach is, and I stated this one's
> reach wrongly before checking what feeds it.**
>
> **And the canvas zoom is a different number entirely, which is what prompted
> the check.** `alo_canvas::Zoom` is thousandths and moves the plane a person
> works on; `display_scale` is hundredths and is how dense their screen is.
> **Zooming the canvas must not resize the Dock, and does not** —
> `alo-dock`'s `on_the_canvas` says so in its own words, *Zoom is not here... a
> zoom level would be a second way to say the same thing, and two ways to say it
> would disagree*, and `dock_raster` has no zoom in it. Two things called scale,
> one of which the Dock is right to ignore.
>
> **And my own test could not have seen it.**
> `the_dock_band_and_the_panel_column_do_not_move_with_the_displays_scale`
> varies the scale and finds the band unchanged. I read that as *the band is in
> laid-out units*. It is equally consistent with *the scale never reaches the
> Dock at all*, which is what the lines above show. **A test that varies an
> input a function never receives proves the function ignores it, and says
> nothing about which space its answer is in.** That is the same shape as the
> three name collisions this fleet hit today, in a test rather than a grep.
>
> **This reopens a question I closed.** `#379` removed a conversion from the
> drag-handle floor on the reading that the Dock's band is logical. If the band
> is in framebuffer pixels because `size` is, the floor needs converting after
> all — and the owner's ruling that the floor is *44 × 24 logical, scaled by the
> person's text size and nothing else* is about what the floor **is**, not about
> what the rectangles it meets are.
>
> **The decision the hop is actually waiting on:** is `size`, in the direct
> path, meant to be the framebuffer or the room? One answer makes the Dock's
> layout the bug; the other makes `#379` the bug. Both cannot be right, and a
> lane choosing between them is the thing that has gone wrong twice today
> already.
>
> *Nothing is built on this until it is answered. The two landed hops are
> unaffected: `TheRoom` is constructed from pixels and a scale by its only
> constructor, and says what it is wherever it travels. What is unresolved is
> what the **other** path's `size` was ever meant to be.*


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

### Not landable yet — the pointer classifier, and not for the reason given twice

**This entry said `Ready` under this machine and it is the panel lane's task.**
The material is this lane's; the task is not. `handover/README.md` on
`handover/dev-pc/the-pointer-classifier` records the handover and, in advance,
the error of taking it back:

> Handed to the desktop lane on 2026-10-01 at its request, with the dev-PC lane
> claiming the task back from it earlier the same day for exactly the reason it
> should not have: *I have material for it*. **Having material is a reason to
> hand the material over, not to hold the task.**

Acting on this entry would have been that same reclaim a second time, against a
document written to prevent it. It was caught by measuring before building
rather than by anybody remembering.

**The work is genuinely unbuilt**, which was also worth checking rather than
assuming either way. Two of the parked file's test names appear in `main` —
`a_panel_mirrored_to_the_left_needs_no_change_here` and
`a_screen_that_is_not_the_frame_the_design_was_drawn_on` — and on that evidence
alone it looked absorbed by `#373`. It is not: **ten of thirteen names are
absent**, and the two shared ones are concepts the panel lane reused for a
different module, which answers *which preview* rather than *whose area*. A
strong inference from two matching names would have retired a real task.

**One line of the old entry was stale in the other direction.** *`Revealing`
gets a home there* — it has one: `crates/alo-dock/src/revealing.rs`, exported
from that crate's `lib.rs`, and `alo-put-aside` already talks to it from
`the_region_the_panel_claims.rs`. The handover README gives *`Revealing` has no
owner* as the whole reason the files are out of a crate, so **the stated reason
for parking them has expired** and whoever takes it should know that before
reading the rest.

What survives unchanged: `no_figure_from_the_design_reaches_the_code` must be
carried across **in the same change**, because a source check that stops
applying because its source moved is the rule quietly ceasing to exist.

**And the destination in the old heading was wrong — into `alo-shell`, beside
`panel_raster.rs`, where the files already are.** The panel lane checked where
the task said to put them before putting them there, and the crate forbids it in
its own words. Measured rather than argued:

```text
the_region_the_panel_claims.rs   "No coordinates, and therefore no screen size",
                                 satisfied "by the value not being reachable"
                          :90    "Nothing else in this crate takes a position"
panel_raster.rs:3                `alo-put-aside` "holds no geometry at all"
alo-put-aside/Cargo.toml         no `smithay`; a paragraph on why `alo-canvas`
                                 is the only geometry-adjacent dependency
Point< or Rectangle< anywhere    none
```

`whose_area(at: Point<i32, Physical>, surfaces: &TheSurfaces)` takes a position
and three rectangles. Moving it there would add `smithay` to that crate and make
it take coordinates. **So the move as named would not be a relocation — it would
be the deletion of an invariant, by a commit whose message says it is tidying.**

The rule it would delete is the reason the geometry is where it is:
`alo-put-aside` decides *what is in the panel*, and the one conversion from a
design's figures to a display's pixels happens in the crate that knows the
display. One conversion in one place, rather than two that agree today.

*This entry's **ownership** was corrected earlier the same day and its
destination was not, because that correction was asking whose task it was and
never asked where the task went. **A record can be wrong in more than one way at
once, and fixing one of them makes the rest look checked.***

---

**And `alo-shell` is not the answer either. The task cannot complete in any
crate, because one of the three surfaces it arbitrates between does not exist
and is promised at no tier.** Measured by the panel lane and confirmed here:

```text
whose_area(at, surfaces)   needs reserved_for_the_panel, the_docks_area,
                           the_top_controls_area
the panel's reserved       exists — panel_raster produces it
the dock's area            exists — canvas_fixed_controls::bounds
the top controls' area     top_controls|TopControls in crates/alo-shell/src: 0
docs/features.md           "top control" 0, "top edge" 0, "return to canvas" 0
the design                 draws them — the-regions-a-pointer-can-be-in.md:90,
                           "the full 1440"
```

Both files also compile with **every item dead**, which is a gate failure under
`-D warnings` before any of the above is reached.

So giving the classifier a caller means either **building the top controls** —
a promise that needs a line in `features.md` with a tier, which is the owner's
and not a lane's — or **passing it an empty rectangle for a surface that does
not exist**, which is the arbitration answering about nothing while looking
complete.

**The handover was parked for want of two surfaces and a geometry, not for want
of a home.** That is what its README meant by *`Revealing` has no owner*, which
both lanes read as a question about which crate.

The files stay on `handover/dev-pc/the-pointer-classifier` at `29f28ba8`, where
they are findable and where they remain correct.

**A question for the owner, not a lane's to answer:** the design draws the top
controls and `features.md` does not carry them at any tier. `CLAUDE.md` says
nothing is built that is not in `features.md` with a tier. So either they are a
promise that is missing its line, or they are not a promise — and until that is
said, three tasks arbitrate against a surface nobody may build.

*Three passes over this entry: ownership, then destination, then whether it can
land at all. Each was thorough about the question it asked, and each left the
one underneath it looking checked. **The hazard is not a careless reading — it
is a careful one that answered a narrower question than the entry needed.***

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
  [the-executable-plan.md](the-executable-plan.md).
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
