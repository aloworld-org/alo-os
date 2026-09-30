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

**Status:** ready. **Depends on:** nothing.

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

**Status:** blocked on 1.

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

**Status:** blocked on 1, 2.

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

**Status:** blocked on 1. **Separate from task 3 on purpose.**

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

**Status:** blocked on 1.

Position, size, camera and the panel's own state, **per Place rather than per
session**. `crates/alo-arranging` already writes an arrangement and reads it back
through `At::checked`, `Size::checked` and `Zoom::of`; what it does not have is a
Place to key any of it by.

- **Acceptance:** two Places with different cameras and different frames both come
  back as they were after a session ends and starts; a frame whose application did
  not come back is absent rather than drawn empty, per Place.
- **Constraint:** extends `alo-arranging`. A second store would be a second answer
  to *where was everything*, which is the fault this repository keeps finding.

### 6. The fixed controls, and a window that can always be got back

**Status:** in progress — the Dock and the status area are held; the panel is not.

The Dock, the status area and the **expanded** minimized-window panel are a
**set**, and a frame keeps a usable part of its name outside every one of them —
44 × 24 logical pixels, scaled with the accessibility settings. *One exposed pixel
is technically reachable and practically lost.*

Recovery is rechecked when the **display, the scale, the Dock's position or the
panel's state** changes, and **a recovery that moves a frame shows the move and
records where it was** — a frame that relocated itself silently is a person's
arrangement edited without them.

**This is task 8 of the closed canvas plan, carried here whole rather than
restated.** It is `Status: Open` there and stays open; what changes is that its
missing half now has a plan to belong to.

- **Acceptance:** the four checks task 8 already names — *Show all* reaches every
  frame in the supported zoom range; fixed controls cannot cover every usable drag
  handle; display, scale, Dock-position and panel-state changes preserve recovery;
  and **keyboard users can find and move a frame without reaching its name band.**
- **What is held today:** the first, and the Dock's and status area's share of the
  second, landed 2026-09-30.
- **Owed, and the owner has granted part of it.** The fourth check needs a keyboard
  road to move a frame. *Move to Place* is now promised and is task 3's — so
  **moving a frame between Places by keyboard is in scope**. What is still not
  promised is moving a frame **within** one Place by keyboard, which is the case
  this check is mostly about: a frame whose name is under a control needs to be
  moved a few hundred units, not sent to another Place. Reported rather than
  assumed to be covered.

### 7. The minimized-window panel, per Place

**Status:** blocked on 1, 4 — and it is the desktop lane's.

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

**Four promises were not moved and this lane did not move them.** *Tidy this
canvas*, *A Place remembers time*, *Every screen is a view onto the canvas* and *A
panel out of view costs nothing* remain at `[v1]` and `[v1.1]`. The owner named
five things completion requires and none of them is these. Moving them would have
been this lane widening its own scope under cover of an instruction.

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
