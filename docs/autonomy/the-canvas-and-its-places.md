# The canvas and its Places

**This plan is part of the Complete canvas milestone**, by the owner's ruling of
2026-10-04 — [ADR 0086](../decisions/0086-the-complete-canvas-is-one-current-milestone.md).
Every agreed canvas capability is in one current milestone, so **no task here is out
of scope for carrying a `[v0.5]` or `[v1]` label in an older document**. Order and
dependencies are unchanged: work may be sequenced, and nothing is excluded by a tier.

**And completion means one thing, in the ruling's own words:** *a canvas capability
is complete only when it works through the production path and meets its acceptance
criteria. A model, unused function, Figma state or isolated test is progress — not
completion.* This plan's territory produced that rule the hard way — a rule with no
caller, a mover called only by tests, a field a constant could replace without
seventeen tests noticing — so it is quoted at the top rather than left in a decision
record.


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

**Owner:** the **development PC** — the lane holding the parked pointer classifier, by
`handover/dev-pc/the-pointer-classifier`'s own README. The Mac holds the band it
reserves and the surface it is drawn into, so the two meet in `alo-shell` and say
which files before starting.

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

**Owner:** the **Mac**.

**Status:** in progress — **the set is complete and the recheck has a caller as
of 2026-10-02**; two named pieces are waiting on something other than work.

*This line read **the Dock and the status area are held; the panel is not** until
2026-10-02, and it was wrong in both directions: the panel had joined the set and
the status area had never been in it.* A status that names what is done is a
maintained count — nothing checks it, and the next reader plans around it. What
each piece is actually waiting on, so that nobody has to re-derive it:

| The check | Where it stands |
| --- | --- |
| The set, and 44 × 24 of the name kept clear | **Done, 2026-10-02.** All three are held and compared: the **Dock's band**, the **panel's reserved column**, and the **status area**, which joined when it could say where it is. Its band is the union of what was **painted** — every `Solid` and `Inked` carries its own area — rather than the room the indicator may grow into, which `Place` holds and which would reserve pixels nothing occupies. Nothing drawn is **no** band rather than a band of no size. *The wiring was unproven when first written: removing the status area from the rule's bounds left all nine hundred and fifty tests green, because the test that existed passes a rectangle straight to the rule and cannot notice whether this one is handed over. A rule that takes a list is tested by what is in the list, never by what the caller put there.* |
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

**Owner:** the **development PC**.

**Status:** **two clauses paid, one blocked on task 9, one waiting on an owner decision —
2026-10-02.** *Read `blocked on 4 only; 1 is done` until this edit, after task 4 landed the
same day. A status that outlived the thing it described, which this plan records in its own
margins three times. The desktop lane corrected it inside the change that pays the task,
rather than spending a branch on the line.*

**What is paid, measured rather than claimed.**

- *A put-aside window names its Place* — `Panel::put_aside` takes a `Place` and every
  `Preview` carries one. Landed with task 1.
- *Restoring one from elsewhere travels* — task 4 above, wired into
  `Server::set_window_minimized` on the way back, and reached in production since the
  desktop lane's `#412`: a click on a preview calls `bring_this_window_back`, which calls
  that function, which travels. Before `#412` the road had **no production caller at all**,
  so this clause was true of a model and of nothing a person could do.
- *The collapse state differs between two Places* — `alo_put_aside::TheCollapseChoice`, a
  `BTreeMap<Place, Chosen>` where absent means expanded.
- *A Place's group shows its windows individually* — `a_place_groups_its_windows`, with
  fourteen tests: two windows of one application on one Place stay two previews, one
  application spanning two Places stays in both, and each ordering rule the owner ruled on
  2026-10-03 is held by a test whose fixture disagrees with `Place` order. **The model only** —
  what the drawing waits on is below.

**What is not paid, and neither reason is this task's own work.**

**The collapse state *surviving a session* now has somewhere to go and nothing to put in it.**

*Read `blocked on task 9, which says Done and is not` until 2026-10-03.* That block was real
and is gone: `#428` wired the arrangement through `alo-desktop` and grew
`alo_arranging::Arrangement` to hold per-Place state, so the file this clause needed exists
and is written on a running machine. The lane that built it left the collapse choice here,
because it is `the_collapse_choice_per_place`'s business.

**What replaced the block: nothing in production can collapse the rail.**

```text
Panel::collapse / Panel::expand     defined, panel.rs:182 and :187
production callers                  none — the only hits are the methods' own bodies
"collaps" in crates/alo-shell/src   0   (the "chosen" hits are accent colours)
"collapse" in crates/alo-desktop    0
```

So persisting the choice would persist a state no person can set — the road that is missing is
the **input** one, and what gesture collapses the rail is written down nowhere this lane can
find.

**And underneath that, a scope question in work already landed.** `docs/features.md` promises
the panel and *a preview in the panel at the edge of the screen* at `[v0.01]` and promises a
**collapsed** rail nowhere — zero occurrences of either *collaps* or *rail* in that file —
while `docs/design/the-interface-in-the-file.md:50` puts *the minimized panel, collapsed and
expanded* at **v1**, and this task's acceptance asks for the collapse state to differ per Place
and survive a session. `#375` built the per-Place choice on 2026-09-30 under that acceptance.

Either the promise belongs in `features.md` at v0.01, or this clause and `#375` are ahead of
the release and the persistence waits. **One sentence from the owner settles it**, and this
lane is not settling it: *Scope is gated* binds building to `features.md`, and three documents
disagree about whether a collapsed rail is in this release.

Both findings are recorded with their commands in `docs/autonomy/putting-a-window-aside.md`
under *Two promises at v0.01 whose road is not there*, beside the same shape found at the other
end of Stop.

A settings file was the wrong instinct and is recorded as such. The collapse choice is
**remembered state, not a setting a person chose**, so `alo-arranging` is its home — and
ADR 0016 warns that inventing a second store is how a settings system becomes six.

**The grouping model is built to the ruling, 2026-10-03, and the drawing waits on a Place
having a name.**

`alo_put_aside::a_place_groups_its_windows` now answers `ThePanelInGroups`: the current Place
first when it holds anything, then the Places the World names **in the World's layout order**,
then the Places the World cannot name. `Headings::NotNeeded` for one represented Place and
`Headings::OnePerGroup` for several, so the drawer is handed the rule rather than counting
groups and copying it. Fourteen tests.

**The signature changed because the ruling forbade the old one.** It returned
`BTreeMap<Place, Vec<&Preview>>`, whose order is `Place`'s `Ord` over a `u64` — the order
Places were *made* in. The defence written for it was *two identical panels must produce one
layout*, which is a true property and was the wrong property: **deterministic and meaningless
are compatible.** The order now comes from a `World` handed in, because the World **is** the
layout, and this crate has no opinion about it.

**A case the ruling could not have anticipated, measured rather than assumed.**
`Server::the_world` is built from `the_frames_on_the_plane`, which reads the **mapped**
surfaces — and a put-aside window is hidden, so it is not mapped. **A Place whose windows have
all been put aside is absent from the World**, which means the World cannot order the very
Places this panel groups. Those come last, in the panel's own most-recent-first order: the
window somebody put away last is the one they are most likely to want back, and it is the only
ordering they can predict. Falling back to `Place`'s `Ord` there would have reintroduced
creation order through the back door.

**What the drawing waits on, and it is a missing concept rather than undone work.** The ruling
asks for *a heading per Place with its windows underneath*. A heading needs a Place's name, and
**a Place has no name**: `crates/alo-canvas/src/place.rs` says where the type is defined that
*a Place a person recognises is a name and a wallpaper on top of one of these, which is **not
this crate's business***, and nothing else in the tree provides one. So there is nothing to put
in the heading.

That is the second half of the caller rule's exception in
`docs/autonomy/putting-a-window-aside.md`, and it replaced the first half on the same day: the
grouping had no caller because the default was undecided, and now it has none because a
heading has nothing to say. Both are citable; neither is silence.

**Also owed, and not this clause's:** *Group by Place / Flat list* in the panel's own options
menu, with the choice remembered. The owner ruled it does not wait for Settings, and the panel
has no options menu yet — and *remembered* lands on the same missing road as the collapse
choice, which `alo-arranging` does not persist.

**One ordering test was strengthened after a mutation told on it.** Replacing World order with
`Place`'s `Ord` failed **one** test of the ordering pair rather than both: the second used two
following Places and expected `[reading, mail]` — Places 4 and 6, which *is* ascending, so it
passed against the sort it existed to forbid. Both now use three following Places in an order
that is neither sorted nor reverse-sorted, and the mutation fails both.


**This clause changes no geometry**, so the fixed-controls recovery recheck does not fire on
it: the panel's reserved column and rail are untouched, because grouping is an ordering over
previews and not a layout. If the owner rules that the panel groups by default, the rail's
height may change with headings and *then* the recheck fires — which is the right behaviour
and is the Mac lane's road working, not a new obligation.

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

### 8. A Place remembers time

**Status:** ready. **Owner:** **the Mac.** **Depends on:** 5, which is done — an
arrangement has to exist before a series of them can.

**Added 2026-10-04 by [ADR
0086](../decisions/0086-the-complete-canvas-is-one-current-milestone.md)**, which
put every agreed canvas capability into one current milestone. The promise was
`[v1.1]` — the furthest-out canvas promise there was — and had no task in any
plan, so no lane could have built it and nothing said so.

Drag a ribbon and the canvas is as it was on Tuesday.

**The word *snapshot* means two things here and taking the wrong one is the whole
risk.** `docs/features.md` says *from the snapshots undo already takes*, and the
snapshots undo takes are `alo-keeping-up`'s — **of the filesystem**. An
arrangement over time is a different subject with the same word, and reaching for
the filesystem's snapshots would be the two-vocabularies fault this plan's
territory has produced four times in two days: `alo-arranging` and `alo-displays`
both say *arrangement*, one the canvas a person left and one the monitors.

- **Acceptance:** arrange windows on a Place, change it, and reach a named earlier
  state through the production path — the layout that returns is the one that was
  held, and the person is never shown a state they cannot get back from. A test
  that round-trips a series in memory is **progress, not completion**, by the
  ruling's own words.
- **Constraint:** extends `alo-arranging`. A second store would be a second
  answer to *where was everything*, which is the fault this repository keeps
  finding.
- **Blocker to report rather than work around:** nothing holds more than one
  arrangement today, and whether a series is kept per Place or per session is a
  shape question this task answers rather than inherits.

### 9. Every screen is a view onto the canvas

**Status:** ready. **Owner:** **the Mac.** **Depends on:** the camera having one
home, which is the same thing `the-smallest-canvas-worth-showing.md` task 9's
remaining half turns on — so these two are done together or the second is done
twice.

**Added 2026-10-04 by [ADR
0086](../decisions/0086-the-complete-canvas-is-one-current-milestone.md).** It was
`[v0.5]` and had no task.

Two displays are two viewports at their own zoom, not two desktops.

**What exists and what does not, measured rather than assumed.** `alo-displays`
already models more than one display and `ScreenPlace` already gives each its own
room, so the display half is built. `alo-shell` holds **one** camera — and
`surfaces.rs`'s own note says *one home, not two* about the single duplicate that
already exists, which three mutators keep in step by hand. **A camera per viewport
is a change to where that state lives**, not a field beside it, and that is why
this task depends on the camera question rather than on the displays.

- **Acceptance:** two displays, each at its own zoom, each a view onto the same
  canvas — a frame moved on one appears moved on the other, and neither display
  is a second desktop with its own arrangement. Shown on a machine with two
  outputs, which is phase 8's ground for the second half.
- **Constraint:** no second camera copy. The one that exists is already named in
  `surfaces.rs` as a hazard; adding a third would make a restored window's size
  depend on how many mutators happened to be in step.

## The gaps, stated because the instruction says to state them

**Three promises were not moved and this lane did not move them — and the owner moved them on 2026-10-04.** *A Place remembers time*, *Every screen is a view onto the canvas* and *A panel out of view costs nothing* were at `[v1]` and `[v1.1]`, and this lane left them there because *the owner named five things completion requires and none of them is these* — moving them would have been a lane widening its own scope under cover of an instruction.

[ADR 0086](../decisions/0086-the-complete-canvas-is-one-current-milestone.md) settles it from the other direction: **every agreed canvas capability is one current milestone**, and all three are named in the ruling's own list. So they are `[v0.01]` now, with tasks 8 and 9 of this plan written for the first two and task 1 of `docs/autonomy/putting-a-window-aside.md` holding the third. **The restraint is kept on the record rather than deleted**, because it was right on the day: a lane may not move a tier, and the owner may.

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

**One promise was built below its own tier, and the tier has come down to meet it.** *Every canvas also answers as a list* was `[v1]` and built at v0.5, with six tests — recorded here rather than quietly re-tiered, which was the right half of the choice. ADR 0086 moves it to `[v0.01]`, so the record now reads as a **correction** rather than a widening: the work was done, and only the label was wrong.

**One acceptance condition needed scope the owner had not granted, and now has it.** Task 6's fourth check asks for a keyboard road to move a frame. *Keyboard access* is named in ADR 0086's milestone, so the clause is in scope — but **the promise it would rest on still does not exist**, and that is the blocker to report rather than work around: `docs/features.md`'s *each with a keyboard form* attaches to fit, fill and work-inside and **not to dragging**, at any tier, measured 2026-10-03. A capability in a milestone whose definition promises no road is a task that cannot state its own acceptance. Previously: there is none, and
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
