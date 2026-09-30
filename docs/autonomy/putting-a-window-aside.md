# Putting a window aside

**What this is.** The owner asked on 2026-09-30 for the panel at the right edge that
holds minimised windows, and moved the promise into the current release so it could be
built now. This is the order it gets built in.

**And it is part of something larger, decided later the same day.** The owner put the
**full canvas experience** into this release: Places are not to be reduced to a minimal
implementation, the panel behaviour that depends on them is not to be deferred, and
**the canvas owns Place identity, with every application and window participating in
it**. Completion needs cross-Place movement and restoration, per-Place persistence,
World navigation, fixed controls and window recovery.

So **this plan is an input to the canvas plan rather than a finished piece**, and the
lane that owns the canvas is writing that. What is here stays true — the panel's own
state, minimising, restoring, peek — and everything in it that touches a Place waits on
the canvas naming one. **Nothing here is marked done to close a task**: where a clause
is owed it says so, which is the owner's instruction in their words — *report actual
gaps openly rather than narrowing the agreed scope to close tasks.*

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

**Status:** **blocked on Place identity, 2026-09-30.** Built except for one clause of
its own acceptance, which stays in this task rather than moving to a later one.

**This line said `Done` for one revision and that was the fault in miniature.** The
prose three paragraphs below said the task was not wholly done, while the status a
reader scans said it was — and the status is what gets read. A task whose clause has
moved elsewhere becomes completable, the acceptance quietly gets smaller, and nothing in
the document says anything was removed. So the clause stays here and the status carries
the block, because **the status line is the state and the prose is a comment.**

**What is built:** `crates/alo-put-aside` — `Panel` with the windows put aside most
recent first, `Preview` naming one window, and `Chosen` (two cases, what the person
picked) against `HowItShows` (three, what is drawn). Six tests in
`crates/alo-put-aside/tests/the_panel_holds_windows_and_not_applications.rs`.

**What is owed:** the collapse choice keyed **per Place**. See below.

**Rule 1 is held by the dependency not existing.** `Cargo.toml` does not list
`alo-canvas`, so nothing in the crate can take a `Camera` at all — which is stronger
than a test of where the panel ended up, for the reason that crate's own header gives: a
surface in the wrong layer that compensates correctly passes a test of its position.

**This task is not wholly done, and the gap is named rather than closed.** It said the
collapse choice *persists per Place*. **The choice is held and remembered; it is not yet
keyed by Place**, because there is no canvas Place to key it by — measured across the
whole tree, and the word is spent four times over on other things: a socket's directory
in `alo-agentd`, an update source in `alo-looking`, a region of a screen in
`alo-dividing`, and where an icon sits on the bar in `alo-dock`. None is the endless
surface `docs/features.md` defines.

**It is blocked on Place identity, which the canvas owns.** On 2026-09-30 the owner put
the full canvas experience into this release, ruled that Places are not to be reduced to
a minimal implementation and that the panel behaviour depending on them is not to be
deferred, and said the canvas owns Place identity with every application and window
participating in it. So the fifth meaning of the word is the canvas lane's to define,
and a `PlaceId` invented here would be the surface least entitled to define it doing so —
and would have to be reconciled later, which is the two-vocabularies fault repeated on
purpose after both lanes had learned it.

**So this task cannot be marked done, and that is the point of leaving the clause in
it.** Per the owner's instruction to report actual gaps rather than narrow the scope to
close a task: **per-Place persistence is this task's, it is owed, and the task stays
open until it exists.**

**The same work appears in the canvas lane's plan as its task 7** — *the minimised-window
panel, per Place* — under the constraint that the panel consumes Place identity rather
than defining it. Two plans, one piece of work, and **neither closes until the Place half
is built**. Named here so a reader of this plan finds the other.

**And the empty case is a state rather than a count.** A person who collapses the panel
and then restores their last window sees the handle, and gets the rail back when they
put something aside again — emptiness wins over the choice without overwriting it,
because *the panel is holding nothing* and *there is no panel here* are different and a
length is the same number for both.

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

**Status:** ready.

**Not blocked on task 1, and the difference matters.** Task 1 is blocked on a *clause* of
its own acceptance, not on its work: the panel, the previews and the presentations are
built and this task needs only those. **A task blocked on a dependency and a task blocked
on one of its own clauses are different states**, and reading the first as the second
would stall this whole plan behind a Place that has not been designed yet.

The window leaves the canvas; its patch, its Place and the zoom it was at are kept. The
other windows **do not move to fill the gap**.

**Acceptance.** A window put aside and restored with nothing else happening lands on the
same patch, byte-identical. Minimising never discards work — a property of the state
change, and tested as one.

**The Place half of this task is blocked on Place identity and stays in this task**,
exactly as task 1's clause does: a patch alone is ambiguous, because `(4200, 0)` exists
on every surface, so *which surface* is part of what minimising has to save and this
task does not close without it. The zoom is available — `alo_canvas::Zoom` is thousandths
held as an integer — and **holding a zoom a caller hands in is not reading the camera**,
which is the distinction rule 1 turns on: a panel that took a `Camera` could correct
itself, a panel that remembers a number cannot.

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

**Everything a Place is.** Four of the panel's behaviours need one and none of them can
be built here:

- **minimising saves which Place**, because a patch alone is ambiguous — `(4200, 0)`
  exists on every surface, so restoring needs the surface as well as the coordinates;
- **restoring across Places**, which is the owner's *cross-Place restoration*: bringing
  back a window whose Place is not the one being looked at has to travel to that Place
  rather than drop the window on this one;

  **This is not `docs/features.md:426`, and the owner separated them on 2026-09-30.**
  That promise — *a frame can be dragged out of one Place and into another, and the work
  goes with it* — is a person **relocating** a window's home, and it stays `[v1.1]`
  along with *a Place remembers time* at `:429`. Restoring a window that was put aside
  **moves nothing**: the window kept its Place all along and the camera travels to it.
  One changes where a window lives and the other changes where the person is looking,
  and the panel needs only the second;
- **the collapse choice persisting per Place**;
- **grouping by Place**, *without hiding individual windows behind an app icon*.

**These are owed, not deferred.** They belong to the canvas plan and the canvas defines
the Place they key on. Task 2 below builds what it can without one and says which clause
it could not.

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
