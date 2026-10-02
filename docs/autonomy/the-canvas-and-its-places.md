# The canvas and its Places

**What this is.** The owner put **the full canvas experience into v0.01** on
2026-09-30, with the instruction not to reduce Places to a minimal implementation
and not to defer the dependent panel behaviour. *The canvas owns Place identity;
all applications and windows participate in it.* Completion requires **cross-Place
movement and restoration, per-Place persistence, World navigation, fixed controls,
and window recovery.**

And the sentence this plan is held to before any of its tasks: **report actual
gaps openly rather than narrowing the agreed scope to close tasks.**

**What it is not.** A second canvas. `docs/autonomy/the-smallest-canvas-worth-showing.md`
closed on 2026-09-30 with ten tasks and **all of them are one Place** — the plane
and the camera, hit testing at any zoom, dragging, resizing, three roads of pan,
zoom and *Show all*, the canvas answering as a list, arrangement restored across a
session, and a walk that draws it. This plan is the level above that one and rests
on it entirely. Nothing here re-decides anything there.

## What exists, measured on 2026-09-30 rather than assumed

| | |
|---|---|
| One Place, complete | plane, camera, hit test, drag, resize, pan, zoom, *Show all*, list, restore, walk |
| A canvas `Place` type | **does not exist** |
| The World | **does not exist** — no type, no navigation, and no promise in `features.md` until today |
| Cross-Place anything | **does not exist**, because there is nothing to cross between |
| Which Place a window is on | **does not exist** — `alo_dock::Window` carries a patch and a `HowItSits` and nothing else |

**The word `Place` is already spent four times and none of them is the canvas:**

    Place   alo-agentd/src/place.rs      the directory a Unix socket goes in
    Place   alo-looking/src/place.rs     where this machine asks about an update
    Place   alo-dividing/src/place.rs    a half, a quarter, or a part of a screen
    APlace  alo-dock/src/places.rs       where an application icon sits on the bar

Measured by the desktop lane across the whole tree. **The canvas defines the
fifth, and it is the only surface entitled to** — which is why the panel refused
to invent a `PlaceId` of its own: reconciling two Place types afterwards is the
`one-plane-two-vocabularies` problem committed deliberately by people who had just
finished writing that problem down.

## Tasks

**These are stages rather than a list, and the heading above is `Tasks` because
that is the word `tools/kernel-loop/src/plan.rs` reads.** The first draft called
it *The stages* and the supervisor refused the plan outright — *read as 0 tasks,
so the loop would have nothing to do and would say the workstream was finished*.
A plan the loop cannot parse is a workstream that reports itself complete, which
is a worse failure than a plan nobody wrote.

Every one of these rests on the one before it. **A Place has to be a thing before
a window can say which one it is on; a window has to say which one before it can
be restored to a different one; and nothing can be persisted per Place until there
is a Place to persist it per.** So this is an order, not a menu, and a task taken
out of turn will be discovered to need the one before it.

---

### 1. A Place is a thing, and the canvas owns what one is

**Status:** **Done, 2026-09-30.** `alo_canvas::Place` in
`crates/alo-canvas/src/place.rs`; a `Place` on every `Frame`; `reached_by` folds
one Place's frames rather than every frame it is handed; and
`crates/alo-shell/src/canvas_place.rs` puts every toplevel on one when it is
created. Shown by `crates/alo-canvas/src/place_tests.rs`, the four Place cases in
`crates/alo-canvas/src/plane_tests.rs`, and
`crates/alo-shell/tests/every_window_is_on_a_place/mod.rs`.
**Depends on:** nothing.

**What it was discovered to be, against the warning at the foot of this plan.**
The plan said this task was *the one most likely to be discovered to be two*. It
was, and the seam is not where it was expected: not between crates, but between
**an identity** and **the roads that change it**. What landed is the identity, one
setter, and every reader that was silently ambiguous without it. What did not land
is any way for a person to change a window's Place — that is task 3's, by pointer
and by keyboard, and `Server::move_the_window_to` is deliberately named as the
primitive underneath both rather than as either of them.

**`alo-dock` was not touched and gained no dependency.** The survey expected the
Place to have to sit on `alo_dock::Window`; it does not, because `alo-shell` does
not use `alo_dock::Window` at all — the shell's windows are Smithay toplevels with
a `crate::window_number` identity. Two window vocabularies, and only one of them
is the compositor's. **The panel's half of this therefore remains open** and is
task 7's; nothing here has answered it.

The fifth meaning of a word already spent four times, and the only one entitled to
it: **a Place is the endless surface**, not a directory, not a screen division,
not a spot on the Dock. It lives in `alo-canvas`, which is the crate that already
owns the plane and the camera and depends on none of the four.

Every window and every application carries which Place it is on. **A patch alone
is ambiguous** — `(4200, 0)` exists on every surface — so *where is this window*
has two halves and has only ever had one.

- **Acceptance:** a `Place` identity in `alo-canvas`; every mapped window answers
  which Place it is on; two windows at the same patch on different Places are
  distinguishable, asserted by a test that would pass today and must not.
- **Constraint:** the four existing `Place` types are not touched and not renamed.
  This is a fifth meaning, and `names are for strangers` is served by the canvas's
  being the one a stranger means.

### 2. The World

**Status:** **Done, 2026-10-02** — arithmetic and gesture both.
`alo_canvas::World` in `crates/alo-canvas/src/world.rs` — Places laid out as
tiles, the span they reach, the Place under a point, and a camera that fits them
all. Shown by `crates/alo-canvas/src/world_tests.rs`.
**Depends on:** 1, which is done.

*Read `blocked on 1` until 2026-10-02, after task 1 landed as #351 on
2026-09-30 — a status that outlived the thing it described, hiding available
work from three lanes for two days. The laptop lane found it by reading `main`
against the plan rather than the plan alone.*

**What was owed is paid.** This status said *the gesture is not wired — nothing
in `alo-shell` answers a step out from `Zoom::FURTHEST_OUT` with the World*, which
it did for the hours between the two changes. `crates/alo-shell/src/canvas_the_world.rs`
now answers it on **both** roads a person has: the key, and Ctrl with the wheel.
Shown by `crates/alo-shell/tests/the_world_is_a_step_out/mod.rs`, seven cases
driven through the chord and the scroll rather than through the functions.

**And `Showing` had to become state, against what its own doc comment said.** It
read *returned rather than stored, because it is a consequence of the zoom* —
written before anything consumed it, and disproved by building the consumer. The
World and a Place are **different coordinate spaces**, so a camera at
`FURTHEST_OUT` over a Place and a camera over the World are both ordinary cameras
and nothing in a camera says which plane it is over. The comment was corrected
rather than worked around; what survives of it is the constraint that **a person
never sets it**.

**And the design decision, because the obvious implementation is wrong twice.**
The World is **not** a further-out view of the same plane. Lowering
`Zoom::FURTHEST_OUT` to make room would multiply `as_far_as_show_all_reaches` by
five — a test asserts that extent to the pixel — and it would assert that Places
share one coordinate space, which they cannot: a Place is *endless*, and `(4200,
0)` exists on every one of them. So the World is the level above, the tile a Place
occupies in it represents that Place rather than being its extent, and **no
existing number moved.**

Zoom out past a Place and **every Place the person has is seen at once**; zoom
into one and it fills the screen. `docs/decisions/0065`'s own diagram —
`World → Place → Object` — and the level of it that has never existed.

**The same gesture as moving across a Place**, which is the whole of why it is not
a switcher: there is no second way to navigate to learn, and `Zoom::STOPS` already
has a ladder to extend rather than a mode to add.

- **Acceptance:** zooming out past a Place's own extent shows every Place; zooming
  into one fills the screen with it; the gesture is the canvas's existing zoom on
  all three of its roads, with no new binding and no mode.
- **Constraint:** no second navigation model. If the World needs a switcher, this
  task has failed rather than found a requirement.

### 3. A frame moves between Places, by pointer and by keyboard

**Status:** **Done, 2026-10-02 — both roads.**

*Move to Place* ships as `Action::MoveTheWindowToTheNextPlace` on `⊞`+`Shift`+`.`,
and the pointer road is `crates/alo-shell/src/canvas_dragged_into_a_place.rs`: take
hold of a frame by its name, zoom out until the World appears, let go over a tile.
Shown by `crates/alo-shell/tests/move_to_place/mod.rs` and
`crates/alo-shell/tests/dragged_into_another_place/mod.rs`.

*This line read `blocked on 2 only` until 2026-10-02, after **both** roads had
landed. The Done mark was written twice, on the two branches that built them, and
lost both times resolving a conflict — which is the hazard this lane had already
recorded about carrying another task's status on a feature branch, met again by
the lane that recorded it. **A status edit belongs in a change of its own.*** *Read `blocked on 1` until 2026-10-02, after task 1 landed as #351 on 2026-09-30 — a status that outlived the thing it described, hiding available work from three lanes for two days.*

Dragged out to the World and dropped into another Place, **and the work goes with
it** — the promise that moved two tiers, from `[v1.1]`. **And by keyboard, with
*Move to Place*,** so neither road is the only road: a person who cannot drag can
still move a window between Places.

- **Acceptance:** a frame dragged to the World and into another Place is on that
  Place and not on the first, with its size and its content; the same by keyboard
  through *Move to Place*; neither is a copy and neither leaves anything behind.
- **Constraint:** a frame belongs to exactly one Place at every moment. There is no
  *between* — a frame in flight has a Place, and it is the one it started on until
  it has the one it ends on.

### 4. A restore travels; it does not relocate

**Status:** **Done, 2026-10-02.** `crates/alo-shell/src/canvas_a_restore_travels.rs`,
wired into `Server::set_window_minimized` on the way back only. Shown by
`crates/alo-shell/tests/a_restore_travels/mod.rs` — four cases driven through the
real restore rather than through the rule, because a test that called the rule
would pass just as well with nothing wired to it.
**Depends on:** 1, which landed as #351. **Separate from task 3 on purpose.**

*This status read `blocked on 1` until 2026-10-02, after task 1 landed as #351 on
2026-09-30 — a status that outlived the thing it described, hiding available work
from three lanes for two days.*

Restoring a minimised window **returns it to the Place it was already on**, and the
view travels there. **Nothing is relocated by a restore.** A window put aside on
Tuesday's Place comes back on Tuesday's Place, and the person is taken to it.

**This is its own task because the two were conflated while this plan was being
written, and the conflation is the easy mistake rather than an unlikely one.**
*Cross-Place restoration* and *a frame dragged into another Place* read as one
sentence at a glance and are two acts: one changes **where a window lives**, the
other changes **where the person is looking**. The owner settled both — transfer is
in v0.01, and a restore relocates nothing.

A task that fails this one silently does the more destructive thing: it moves
somebody's window to the Place they happened to be looking at.

- **Acceptance:** a put-aside window whose Place is not the one on screen comes
  back on **its own** Place, with the view travelling there; its Place is the same
  before and after, asserted directly rather than inferred from where it appears.
- **Constraint:** restoring never writes a window's Place. Only task 3 does.

### 5. Every Place is where it was left

**Status:** **Done, 2026-10-02.** `alo-arranging`'s file is a map of Places, each
with its own camera and windows; `FORMAT` is 2. Shown by
`crates/alo-arranging/src/arranging_tests.rs` and
`crates/alo-shell/tests/the_canvas_is_where_they_left_it/mod.rs`.
**Depends on:** 1, which is done — and the `FORMAT` version landed as #360, which
this reshape needed and which was free only while nothing wrote the file.

**Two things the reshape revealed rather than caused.** `camera_on` answers
`Option` now, and a test changed its claim rather than its fixture: *this Place
was never left anywhere* and *this Place was left at the origin* are different
answers, and version 1 could not tell them apart. A restore on a Place nobody had
arranged would have moved a person's view to the origin as though they had chosen
it.

**And `WhereTheyLeftIt::claimed` was keyed by application alone** — right while an
arrangement held one map, and with a place per Place it refuses an application on
every surface but the first, **silently**, because `put_back_where_it_was` answers
`false` for *already claimed* exactly as for *nothing remembered*. That is this
task's *per Place rather than per session* in the one field that would have
quietly denied it.

*This status read `blocked on 1` until 2026-10-02, after task 1 landed as #351 on
2026-09-30 — a status that outlived the thing it described, hiding available work
from three lanes for two days.*

Position, size, camera and the panel's own state, **per Place rather than per
session**. `crates/alo-arranging` already writes an arrangement and reads it back
through `At::checked`, `Size::checked` and `Zoom::of`; what it does not have is a
Place to key any of it by.

- **Acceptance:** two Places with different cameras and different frames both come
  back as they were after a session ends and starts; a frame whose application did
  not come back is absent rather than drawn empty, per Place.
- **Constraint:** extends `alo-arranging`. A second store would be a second answer
  to *where was everything*, which is the fault this repository keeps finding.

### 6a. The top controls, and the fourth member of the set

**Cited as task 6.** `alo-reconciling` reads a task number as digits followed by
`of` and finds a task by a `### <digits>.` heading under `## Tasks`, so `6a` is
not a pointer it can follow and the ledger points at the set instead. The
lettered heading stays because this belongs beside the controls it joins rather
than at the end of the plan.

**Status:** ready — **the owner made them a promise on 2026-10-02** and
`docs/features.md` carries them at `[v0.01]`. **Depends on:** nothing.

Until that decision the design drew them and `features.md` listed them nowhere,
so no lane could build them and three tasks arbitrated against a surface that
did not exist. The parked pointer classifier is the clearest case: `whose_area`
takes the Dock's area, the panel's reserved column **and the top controls'**, and
could not land in any crate because one of its three inputs was unbuildable.

- **Outcome:** a band along the top holding the active window's controls and the
  way back to the canvas, which gives way to a full-screen window, is reached by
  pointer or keyboard, **stays while it is being used**, and can be kept visible
  by a person who would rather it never hid.
- **It is the fourth fixed control**, and joins the set by being pushed rather
  than by a signature changing — `canvas_fixed_controls`'s own header says so,
  and `FixedControlsDrawn` gained its third field on 2026-10-02 exactly that
  way. So a frame keeps 44 × 24 of its name clear of this too, and the recheck
  and the mover cover it with no change of their own.
- **The top-right corner is the panel's**, by the owner's ruling in
  [the regions note](../design/the-regions-a-pointer-can-be-in.md). The top
  controls' region stops before the panel. Two surfaces claiming one corner was
  the fault that ruling settled, and a classifier that re-derived it would
  reopen it.
- **No reveal or hide timers**, for the reason the Dock's reveal already gives:
  a timer makes reaching a surface depend on how fast somebody can move, which
  is the one thing a person with a tremor or a trackball cannot control.
- **Verification:** the classifier arbitrating three real surfaces, and a frame
  whose name is under the top controls named by the recheck — the same seam test
  the status area got, which is the only kind that can notice whether a control
  is handed over at all.
- **Owner:** the lane holding the parked classifier, which is the panel lane by
  `handover/dev-pc/the-pointer-classifier`'s own README. **Not this lane's**, and
  recorded here rather than taken.

### 6. The fixed controls, and a window that can always be got back

**Status:** in progress — the recheck has a caller as of 2026-10-02; two named
pieces are waiting on something other than work.

*This line read **the Dock and the status area are held; the panel is not** until
2026-10-02, and it was wrong in both directions: the panel had joined the set and
the status area had never been in it.* A status that names what is done is a
maintained count — nothing checks it, and the next reader plans around it. What
each piece is actually waiting on, so that nobody has to re-derive it:

| The check | Where it stands |
| --- | --- |
| The set, and 44 × 24 of the name kept clear | The **Dock's band** and the **panel's reserved column** are held and compared. The **status area** is not, and cannot be: `EgressStatusPicture` carries rows and solids and **no rectangle**, so there is nothing to hand over. It joins when it can say where it is. |
| Rechecked when the display, the scale, the Dock's bounds or the panel's state changes | **Done.** The draw records the controls and, when they differ from the frame before, asks for the frames they now hide and brings them back. Before this the detector and the mover were both written, both tested and **called by tests alone.** |
| A recovery that moves a frame shows the move and records where it was | **The record is done** — `Recovery::BroughtBack` carries where the frame was as well as where it is. **The showing is not**, and it is not a line of work: `alo_notifying::arriving::from_alo_os` is the mechanism and **alo OS has no production notification anywhere in this tree**, so the first one is its own task, with its own externalized words in every shipped language. |
| A keyboard road to move a frame | Needs scope the owner has not granted; recorded at the foot of this plan rather than invented. |

The Dock, the status area and the **expanded** minimized-window panel are a
**set**, and a frame keeps a usable part of its name outside every one of them —
44 × 24 logical pixels, scaled with the accessibility settings. *One exposed pixel
is technically reachable and practically lost.*

Recovery is rechecked when the **display, the scale, the Dock's bounds or the
panel's state** changes, and **a recovery that moves a frame shows the move and
records where it was** — a frame that relocated itself silently is a person's
arrangement edited without them.

**This is task 8 of the closed canvas plan, carried here whole rather than
restated.** It is `Status: Open` there and stays open; what changes is that its
missing half now has a plan to belong to.

**The owner's list said *Dock-position*, and on this tree the Dock's position
cannot change.** ADR 0076 fixes it to the bottom edge — the decision is in its own
title. What does vary is the Dock's **bounds**: the owner decided separately that
it grows with its icons, expanding to the margins and the panel's reserved area
before the glyph shrinks. So the trigger is the region the Dock occupies, not
which edge it is on, and this task says *bounds* for that reason.

**Recorded rather than silently reinterpreted**, because the two readings are not
the same promise: if the owner does mean the edge can move, then ADR 0076 is what
has to change and this task is waiting on it rather than on me. The laptop lane
has separately put it to the owner that ADR 0076 removed a choice from the person
and that law 5 — *a change that takes a choice away from the person is a bug,
whatever the reason given* — arrived a week before it and is cited nowhere in it.
**That question is upstream of this clause and is not this plan's to settle.**

- **Acceptance:** the four checks task 8 already names — *Show all* reaches every
  frame in the supported zoom range; fixed controls cannot cover every usable drag
  handle; display, scale, Dock-position and panel-state changes preserve recovery;
  and **keyboard users can find and move a frame without reaching its name band.**
- **What is held today:** the first. **Nothing else, and this line said otherwise.**

  It read *the Dock's and status area's share of the second, landed 2026-09-30*.
  **That work is not landed.** `2f8eaaaf` — the usable-handle rule, the set of
  fixed controls, eight unit tests and five fixture tests — sits on a local branch
  `task/mac/a-usable-handle`, gated nine of nine and never pushed. It was written,
  it passed, the owner redirected this lane to the scope move, and the plan was
  then written describing it as landed.

  **Which is this plan's own subject, committed by its author, in the document
  written to stop it.** Every instance we have catalogued needed somebody to have
  been mistaken about work they could not see; this one was mistaken about work it
  had done itself, four hours earlier, on the machine it was writing from.

  **And it must not simply be pushed now.** `CLAUDE.md` gained two rules on
  2026-09-30 — *the design is followed, not approximated* and *nothing is built to
  one screen size*, with **a figure taken from a frame becomes a proportion or a
  named rule, never a constant**. The unlanded work defines
  `A_USABLE_HANDLE: (f64, f64) = (44.0, 24.0)`, scaled only by the text setting and
  not derived from the display.

  **The laptop lane measured the design file twice, and the second measurement is
  the one to act on.** The first said *29, 40, 42, 44 or 52, and never 24*, and
  this line repeated it. **24 is in that file 328 times.** What the measurement
  actually covered was every node named as a hit area, a focus area or a target:
  29, 40, 42, 44 square, one 52 × 48, one 183 × 44. So the sharper statement, and
  the true one: **24 is a glyph dimension in this design and never a target
  dimension — and a handle is a target.** The smallest target anybody drew is 29.

  **Which turns the question from a conflict into a suspected conflation.**
  `A_USABLE_HANDLE` may be one constant doing two jobs: 44 behaving as a target
  and 24 as a glyph. That is the same fault already found in `alo-dock`, where
  `measures::ICON = 48` is both the picture and the thing you press. If it is the
  same fault, the resolution that satisfies **both** the owner's 44 × 24
  instruction and *a figure taken from a frame becomes a proportion or a named
  rule* is a glyph and a target named separately — 24 surviving as the first, the
  second derived rather than written down.

  **Named here, not resolved by this lane.** The owner gave the 44 × 24 minimum on
  2026-09-30 and the rules arrived the same day; whether the pair is one measure or
  two is theirs to say, and the laptop lane offered its reading explicitly as a
  candidate rather than a decision.
- **Owed, and the owner has granted part of it.** The fourth check needs a keyboard
  road to move a frame. *Move to Place* is now promised and is task 3's — so
  **moving a frame between Places by keyboard is in scope**. What is still not
  promised is moving a frame **within** one Place by keyboard, which is the case
  this check is mostly about: a frame whose name is under a control needs to be
  moved a few hundred units, not sent to another Place. Reported rather than
  assumed to be covered.

### 7. The minimized-window panel, per Place

**Status:** blocked on 4 only; 1 is done — and it is the desktop lane's. *Read `blocked on 1` until 2026-10-02, after task 1 landed as #351 on 2026-09-30 — a status that outlived the thing it described, hiding available work from three lanes for two days.*

Which Place a put-aside window belongs to; restoring across Places; the collapse
choice remembered **per Place**, expanded on one surface and collapsed on another;
and grouping by Place **without hiding individual windows behind an application
icon**.

**Explicitly not deferred**, by the owner's instruction. It is listed last because
it depends on 1 and 3, not because it is optional.

- **Acceptance:** a put-aside window names its Place; restoring one from elsewhere
  travels; the collapse state differs between two Places and survives a session;
  a Place's group shows its windows individually.
- **Constraint:** the panel does not define Place identity. It consumes stage 1's.

## The gaps, stated because the instruction says to state them

**Three promises were not moved and this lane did not move them.** *A Place
remembers time*, *Every screen is a view onto the canvas* and *A panel out of view
costs nothing* remain at `[v1]` and `[v1.1]`. The owner named five things
completion requires and none of them is these. Moving them would have been this
lane widening its own scope under cover of an instruction.

**This paragraph said four, and *Tidy this canvas* was the fourth.** It moved to
`[v0.01]` later the same day, and the reason it had to is a fault of this lane's
own making rather than a new decision: *Frames, dragged and resized like a design
canvas* was moved to `[v0.01]` in this very change **with *several can be taken at
once, guides and snapping line them up* already in it**. So multi-select and
snapping went into the current release while alignment and distribution — the
adjacent operation on the same selection — stayed at `[v1]`. **One operation at
two tiers, and the sentence above defended it on the grounds that the owner had
not named it.** That defence was sound about a promise this lane had not touched
and unsound the moment it had moved the other half. The laptop lane found it while
reconciling a collision in `features.md`; the argument that settles it is this
one, not theirs.

**One promise is built below its own tier.** *Every canvas also answers as a list*
is `[v1]` and was built at v0.5, with tests. Recorded rather than quietly
re-tiered.

**One acceptance condition still needs scope the owner has not granted.** Task 6's
fourth check asks for a keyboard road to move a frame. There is none, and
`features.md:422`'s keyboard forms are *fit*, *fill* and *work inside* — not drag.

**And the honest shape of task 1.** *The canvas owns Place identity* is written
here as one task because it is one decision, but it touches `alo-canvas`,
`alo-shell`, `alo-dock` and `alo-arranging`. It is the largest single thing in
this plan and the one most likely to be discovered to be two.

## Three things measured while starting task 1, which change what later costs

**Task 5 breaks every arrangement file, and there is no version number to say so.**
`alo-arranging`'s `TheFile` carries `#[serde(default, deny_unknown_fields)]` and
three keys — `looking-at`, `zoom`, `windows` — **none of which is a version**.
`arranging_tests.rs:94` asserts the refusal deliberately: *an unknown key is a file
written by something else, or by a later version of this one. Either way it is not
read half-way.* So the per-Place key task 5 needs makes every new file unreadable
to every older binary, and there is no field to negotiate the transition through.

**And task 5 is not a key addition at all**, which is the part that was assumed.
Its acceptance says *two Places with different cameras*; `TheFile` holds **one**
camera beside a flat `BTreeMap<app_id, Written>`. Two cameras means the file
becomes a map of per-Place records — a reshape, not a field.

**What makes both affordable is that this is free today and will not be tomorrow:**
nothing in this repository writes that file to disk. `Arrangement::written()` has
no production caller, `read` none outside the crate's own tests, and the file has
no name and no path anywhere in the tree. **The format can be changed now at the
cost of editing tests, and after the first machine saves one it cannot.** That is
a reason to do task 5's reshape early rather than in its turn, and it is the only
place in this plan where the order should probably not be obeyed.

**`alo-dock` does not depend on `alo-canvas`, and task 7 is where it starts to.**
`alo_dock::Window` is where the panel would read a Place from, and a `Place` on it
is a new edge between two crates that have never met — deliberately, since
`alo-dock` is a decision crate with no geometry it did not define. `alo-put-aside`
is the crate that already imports both and bridges them, and is the cheaper seam.
Recorded now because it is an architectural change, not a field.

**A third `1_000_000` exists.** `alo_canvas::plane::FURTHEST` is the plane's bound;
`crates/alo-shell/src/window_placement.rs:87` writes `(-1_000_000..=1_000_000)`
out again, unconnected to it, for the same rule. Two constants for one fact, and
the standing rule says a figure becomes a named rule rather than a constant. Not
this task's to fix, and named so it is not discovered a third time.
