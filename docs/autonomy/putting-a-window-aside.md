# Putting a window aside

**What this is.** The owner asked on 2026-09-30 for the panel at the right edge that
holds minimised windows, and moved the promise into the current release so it could be
built now. This is the order it gets built in.

**What it rests on.** `docs/design/the-windows-put-aside.md` is the design — what the
panel is, what already exists, and which of the owner's ten behaviour rules a crate can
decide without a display. `docs/design/one-plane-two-vocabularies.md` settles which
vocabulary the plane speaks. **Neither is restated here.** This file is *when*, and what
each increment has to earn before it counts as done.

**The one sentence it has to earn:** a window you put away is somewhere you can find it,
and bringing it back puts it where it was.

## What is already true, so nobody builds it twice

Measured by reading the tree rather than recalled:

- **Rule 1 is architectural already.** `crates/alo-canvas` is a plane that moves under a
  viewport that does not, and states the constraint: *nothing in the viewport layer may
  read the camera to correct itself. If it has to, it is in the wrong layer.* The panel
  is a viewport control by construction **if it never imports a `Camera`**.
- **`alo-dock` carries the window model** — `WindowId`, `AppId`, `HowItSits::PutAside`
  keeping its patch, restore-and-travel in `clicking.rs`, *Back returns a view* and peek
  in `travelling.rs`, per-window previews in `previews.rs`, and an edge state machine
  with no timers in `revealing.rs`.
- **Seven of the ten behaviour rules are decidable in a crate.** Four and seven are
  partly the compositor's; ten is split.

## How a task here is tested, decided once

Two rules taken from lanes that paid for them, so no task has to rediscover them.

**Hold the dependency, not the outcome.** `alo-canvas`'s own warning: a dock that
subtracted a pan to stay still *would pass a test that only checked where the dock ended
up*. A test asserting the panel is still at some x after a pan passes for a panel that is
in the wrong layer and compensating correctly. What is held is that the panel's position
**cannot** be a function of the camera — proved by the type not being able to see one.

**Enter by the road a person uses.** The frame-edge lane's bands were drawn, documented,
tested and **unreachable by any mouse**, because every test called the compositor
directly and none went through `pointer_button`. *A test that enters by the road a person
uses is the only one that can tell you the road exists.* For this panel that means a
press routed through the real input path, never `panel.peek_at(…)`.

---

## Tasks

### 1. The panel's own state, and its three presentations

**Status:** ready.

A new crate — the panel is a different reason to change from a dock, which is law 4 at
the scale of a crate — depending on `alo-dock` for the window model and `alo-canvas` for
the plane. **It must not depend on a `Camera`**, and the test for rule 1 is that it
cannot.

What it holds: the set of windows put aside, in the order they were put there, each
preview naming **one window** rather than one application. Three presentations —
expanded with named previews, collapsed to a rail of window icons, and empty, which is a
small edge handle rather than an empty column.

**Acceptance.** Three minimised windows of one application are three previews. Collapsing
changes only the presentation, proved by a test that asserts **no window state is touched
at all** rather than by inspection. The collapse choice persists per Place. The empty
state is its own thing and not a zero-length list, because *a panel holding nothing* and
*a panel that is not there* are different and a length cannot tell them apart.

### 2. Minimising, and what is saved

**Status:** blocked on task 1.

The window leaves the canvas; its patch, its Place and the zoom it was at are kept. The
other windows **do not move to fill the gap**.

**Acceptance.** A window put aside and restored with nothing else happening lands on the
same patch, byte-identical. Minimising never discards work — a property of the state
change, and tested as one.

### 3. Restoring, and the camera that travels to it

**Status:** blocked on tasks 1 and 2.

One click returns that window to its **saved** position and the canvas travels to show
it. Not to wherever the viewer happens to be.

**Acceptance.** The restore names the saved patch and not the current view — the test
distinguishes them by restoring from a view that does not contain the window. *Previous
view* returns a view rather than a window, which `travelling.rs` already decides.

### 4. When the saved place is taken

**Status:** blocked on task 3.

`docs/design/the-alo-dock.md` says the restored window *comes forward and nothing is
rearranged*; the owner's specification adds that the person is **shown** the collision and
offered a nearby position. Both refuse invisible stacking; the newer one decides more.

**Acceptance.** The intended placement is shown before anything moves, and the original
position is kept in History. Nothing is rearranged without the proposal being visible
first.

### 5. Peek

**Status:** blocked on task 1.

A larger readable view over the current canvas. Releasing or Escape removes it and **the
window stays minimised**.

**Acceptance.** Peek leaves the window's state and the camera both unchanged, asserted
separately. A press meant for the canvas does not reach the peek — tested through the
real input path, and using the shell's existing hit test rather than a second opinion
about where clients are.

### 6. The full-screen edge reveal

**Status:** blocked on task 1, and **on a decision**.

True full screen conceals the panel until the right edge reveals it, and the pointer must
be able to travel onto the panel without it disappearing.

**The decision first:** `alo-dock::revealing` already is this state machine, at the
bottom edge, with no timers and for a reason. **Whether it is generalised to an edge or
the panel gets its own is not decided, and writing the second one first is how there come
to be two.** That belongs to whoever owns `alo-dock`.

### 7. Alo working in a minimised window

**Status:** blocked on task 1.

The task alo was given, the scope it may change, progress, last confirmed action, **Stop
available immediately**, and *Requires you* when a decision is waiting.

**Acceptance.** With AI switched off the panel is fully functional and shows no empty
agent controls and no nagging, which is ADR 0009 in this surface. Deep teal appears only
with the alo mark and a word, never as a selection colour.

### 8. Privacy

**Status:** blocked on task 1.

A private window shows a neutral *Preview hidden* surface with a safe identifying name,
and still restores normally. Previews are made locally.

**Acceptance.** A minimised state is never permission for an agent to inspect, share or
act on the window — tested as a refusal, not assumed from the absence of a call.

---

## What this plan does not cover

**Compacting.** The promise names two things and says *the person picks*, and this plan
builds one of them. **A later ledger entry must not tick the promise when only the panel
exists.**

**The plane's reconciliation.** `docs/design/one-plane-two-vocabularies.md` decides that
the plane moves out of `alo-dock` into `alo-canvas`; that move belongs to the lane that
owns `alo-dock` and the panel is built against `alo-canvas` either way. A new surface
should not wait on a cleanup, and it should not learn the vocabulary that is leaving.

**The acceptance test the owner named**, which no task here earns:

> Open many windows, pan and zoom far away, minimise several, scroll the panel, restore
> one, and then return to the previous view. The canvas alone should move; the panel and
> Dock should remain exactly where the person expects them.

That is pixels under a moving camera. Every decision inside it is earned above; *remain
exactly where the person expects them* is the `On the machine.` half and law 3 ticks it
on a certified machine.

## One document this plan disagrees with

`docs/autonomy/the-smallest-canvas-worth-showing.md` lists *the minimized shelf* among
what it is not, and says of that list: **"Those are v1 and v1.1"**. That was true when it
was written and the tier claim is now stale — the owner moved this promise into the
current release on 2026-09-30.

**The exclusion itself still stands**: that plan did not build the shelf and closed at ten
of ten without it. Only the sentence about which tier it sits at is wrong, and that plan
belongs to another lane, so it is reported here rather than edited.
