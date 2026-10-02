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

## What none of these tasks has

**The panel is drawn and nothing drives it.** Two measurements a few hours apart, the second
superseding the first in the good direction.

**Superseded, kept for provenance:** *nothing consumes this crate*, measured on `9d1f6571` by
the lane that was about to build against it, and true then. `#343` made it false.

**Current, measured on `bd66424e`:** `alo-shell` depends on `alo-put-aside`, `panel_raster.rs`
draws the panel, and `desktop_raster.rs` carries a `&Panel` through. **Every reference is
read-only.** Workspace-wide, outside this crate, there is not one call to `putting_aside::`,
`restoring_into_a_taken_place::`, `peek_at(`, `alo_is_now(` or `bring_back(`. The two non-zero
hits are both inside the shell lane's own raster test fixture, which builds a panel to lay out
rather than to operate.

So the panel on a screen is **always empty** — and that lane's own `panel_raster.rs` says what
that draws: *an empty panel has a rail of no height and no slots*.

### Measured again on 2026-10-02, and one of the two halves has gone

**Peeking is reachable.** A pointer now travels from `libinput_routing` through
`direct_seat` and `direct_pointer` to `Desk::dispatch`, which asks
`Server::where_the_pointer_is_on_the_panel` for a classification and hands it to
`TheDesktop::the_pointer_is_now`; `alo-desktop` holds the `Panel` and calls `peek_at`.

**Putting a window aside is not.** `Server::put_this_window_aside` is written and its only
callers are its integration tests, because the gesture that would call it does not exist. So a
person can now point at a panel that still has nothing in it, and **that one road is what every
remaining task in this plan is waiting on.**

### The order the remaining work goes in, and why

Written here rather than on one machine, because this fleet has twice paid for knowledge living
where the other lanes could not see it — a parked pointer classifier nobody knew existed, and a
gate-provenance finding two lanes rediscovered five days late.

1. **The gesture that puts a window aside.** One road, and it unblocks the evidence clause of
   every task below. It goes first for that reason and not because it is the smallest.
2. **`surface_areas.rs` and `panel_region.rs` into this crate**, handed over by the dev-PC lane
   with its header corrected after 378 grid points were measured changing answer on a reordered
   match. **The `no_figure_from_the_design_reaches_the_code` check moves with them** — a source
   check that stops applying because the source moved is the rule quietly ceasing to exist.
3. **The remaining shell callers**: `ask_for`, `alo_is_now`, `what_an_agent_may_do`. The last
   returns only refusals, so its caller is wherever an agent asks about a put-aside window —
   **check that road exists before writing it** rather than inventing one.

**Task 4 stays blocked on History, which is `[v1]`** — outside this release, so it is not work
being skipped. Its status line says so, which is what a supervisor reads.

### The gesture is not missing. It exists and it does something else.

The sharper finding, from the lane that owns the shell, after being asked whether the drawing
was step one or a gap. `alo-shell/src/window_minimize.rs` holds `set_window_minimized`, wired to
XDG's `minimize_request` at `surfaces.rs:380`, and its own header says what it is: **trusted
visibility transitions**. It **hides a buffered root** — hidden roots keep their buffers,
placement and cycling position, receive no scene hits, and revealing neither activates nor
raises.

**That is what minimise meant before this product had a panel.** `docs/features.md` distinguishes
the two and says the person picks: *compacting* leaves a live tile on the canvas, *minimising*
puts a preview in the panel at the edge. The shell implements **neither**. It implements hiding.

**So the fault is not an absent road, it is a road that arrives somewhere else** — and the danger
is that it reads as done from outside. A compositor that answers `minimize_request`, and a plan
that says minimising puts a preview in the panel, **can both be true sentences about different
mechanisms.** Neither document is wrong, and two mechanisms under one name is how this stops
being reviewable.

### What `#343` evidences, which is one clause of one task

**Task 1's *an empty panel is its own state rather than a list of length zero*.**
`HowItShows::AnEdgeHandle` is drawn by something real and the reserved column exists on a
surface. That is all of it.

Not task 1's previews or its collapsed rail, because nothing puts a window aside. Not task 3's
travel, nor task 5's peek, nor task 7's *alo working*, nor task 8's *`Preview hidden`* — **each
needs a call that does not exist.**

**And the region contract is now *drawn against* rather than *reached by*.** `panel_raster.rs`
imports `WhichEdge` and reads it to place the reserved column, and **nothing in this repository
names an edge as a result** — which is that file doing the job it was written for. It is weaker
than a pointer reaching it and it is not nothing. It becomes *reached by* when the classifier
lands, and that is a sentence to write then rather than now.

### The rest of this section was written when nothing consumed the crate

It stands, because the tests are unchanged: **every test in this crate still enters by calling a
function directly**, which is what the rule below forbids.

### That is forbidden by the rule at the top of this document

*Enter by the road a person uses*, written here before task 1 was built, quoting the frame-edge
lane’s scar: their bands were drawn, documented, tested and **unreachable by any mouse**. It
ends *for this panel that means a press routed through the real input path, never
`panel.peek_at(…)`.*

`tests/a_peek_leaves_the_window_and_the_canvas_alone.rs` opens with `peek_at(&panel, …)` three
times. **The forbidden road, named by that exact function, in a rule this lane wrote down and
relayed to others as a lesson already paid for.**

### Two tasks said `done` and neither was

Task 3 and task 8. Both were judged by whether their **acceptance clause** was inside this lane,
and both acceptance clauses are met. But a task is its whole sentence: task 3 says *the canvas
travels to show it* and task 8 says *a private window shows a neutral `Preview hidden` surface*,
and **showing is drawing**. Reading the acceptance and calling the task done is how the
acceptance quietly becomes the task — the same shrinkage this plan refuses when a clause is
moved to a later task, arriving instead by reading a narrower line than the one that was agreed.

### Why this lane was strict about it elsewhere and blind to it here

Two other modules in this repository are built, tested and reached by nothing, and this lane
insisted on saying so about both: `alo-dock::revealing`, whose owner declined to cite its twelve
passing tests, and this plan’s own region contract. **The standard was applied wherever it had
been handed over and nowhere it had to be noticed.**

The difference is size. One module with no caller looks like an omission. Twelve files, sixty-five
tests and ten merged pull requests look like progress, and **a plan that is wrong in every task
reads as a plan that is going well.**

### What unblocks it, and it is one thing rather than four

Tasks 3, 5, 7 and 8 all wait on the same work: **a person being able to put a window aside, and
the panel driven by it in `alo-shell`** — the drawing landed in `#343` and the gesture is next in
that lane, using that crate’s existing hit test rather than a second opinion about where
clients are — which is what task 5’s acceptance already demanded, written against precisely this
fault. It is lane B’s, it is in scope since `#322`, and this lane will not reach into
`alo-shell` to do it.

**Until then the honest status of all eight is the same: model built, no evidence.** Each task
above names the frame its drawing clause will be measured against, because a clause that names
its frame cannot later be evidenced against a description of the frame.

---

## Tasks

### 1. The panel's own state, and its three presentations

**Status:** **model built and its owed clause paid; blocked on no window reaching the panel,
2026-10-02.**

**The collapse choice is keyed per Place.** `crates/alo-put-aside/src/the_collapse_choice_per_place.rs`
holds `TheCollapseChoice`, a map from `alo_canvas::Place` to `Chosen` where **absent means
expanded** — not a default invented for a missing entry, but the same answer a fresh panel
gives, because `Chosen` carries `#[default]` on `Expanded`. Expanding *removes* the entry
rather than storing the default, so *never collapsed* and *expanded again* are one state rather
than two a person cannot tell apart.

`Panel::chosen`, `collapse`, `expand` and `showing` all take a Place now. Eight tests in
`tests/the_collapse_choice_is_remembered_for_each_place.rs`, every one of them using **at
least two Places** — a suite in which every assertion is about one Place cannot tell a
per-Place choice from a single field, which is the same reason the travel tests use Place 7 and
look at Place 2.

**Watched failing**: `on` changed to ignore its argument and answer the first entry, which is
exactly the old single-field behaviour. It compiled, and five of the eight failed.

**This clause was owed for a day and the reason it is paid now is that the canvas lane landed
`Place`.** It was never this crate's to define — the owner ruled on 2026-09-30 that the canvas
owns Place identity, so a `PlaceId` invented here would have been the surface least entitled to
define it doing so. The clause stayed in this task rather than being moved or quietly dropped,
which is what *the status line is the state* was for.

**What the canvas lane's task 7 still needs from this.** That plan carries *the minimised-window
panel, per Place* under the constraint that the panel consumes Place identity rather than
defining it. This is that half. Neither plan closes until a window can actually reach the panel.
Since `#343` the empty panel is drawn by something real, so *an empty panel is its own state
rather than a list of length zero* is evidenced. Nothing puts a window aside, so the previews and
the rail are not — see *What none of these tasks has* above. Its
drawing clauses will be measured against **Minimized panel / 02 Expanded previews**, **07
Collapsed rail** and **12 Empty handle**. And one clause of its own acceptance is owed, which
stays in this task rather than moving to a later one.

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

**What was owed and is now built:** the collapse choice keyed **per Place**, in
`src/the_collapse_choice_per_place.rs` with eight tests. The paragraphs below are kept as the
record of why it was blocked for a day and who it was blocked on — **read them as history, not
as the current state**, which the status line above carries. A reader who scans the prose and
not the status is the reader this task has already misled once.

**Rule 1 is held by the dependency not existing.** `Cargo.toml` does not list
`alo-canvas`, so nothing in the crate can take a `Camera` at all — which is stronger
than a test of where the panel ended up, for the reason that crate's own header gives: a
surface in the wrong layer that compensates correctly passes a test of its position.

**History, 2026-09-30 to 2026-10-02 — this task was not wholly done and the gap was named
rather than closed.** It said the collapse choice *persists per Place*. **The choice was held
and remembered but not keyed by Place**, because there was no canvas Place to key it by —
measured across the
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

**Status:** **model built, no evidence; the Place clause is built, 2026-10-01.** Nothing
draws or routes this, so the gesture is untested by the road a person uses. The transition is built and the
zoom is saved; one clause of this task's own acceptance is not, and it stays here.

**What is built:** `crates/alo-put-aside/src/putting_aside.rs` — `put_aside` and
`bring_back`, moving a window between the canvas and the panel.
`crates/alo-put-aside/src/where_it_goes_back.rs` — the view a window returns to, patch and
zoom as one value so a caller cannot take the patch and forget the zoom. Fourteen tests in
the crate across three files.

The panel accepts the preview **before** the window's state changes: if the order were
reversed and the panel refused, the window would be marked put aside with nothing holding
its preview — a window a person can neither see nor get back, which is the one outcome
this surface exists to prevent.

**What is owed:** the Place. That clause is below, and so is the zoom's — which is settled
now, and kept because the first answer written here was wrong in a way worth reading.

**This task was not blocked on task 1, and the difference mattered.** Task 1 is blocked
on a *clause* of its own acceptance, not on its work: the panel, the previews and the
presentations were built and this task needed only those. **A task blocked on a
dependency and a task blocked on one of its own clauses are different states**, and
reading the first as the second would have stalled this whole plan behind a Place that
has not been designed yet.

The window leaves the canvas; its patch, its Place and the zoom it was at are kept. The
other windows **do not move to fill the gap**.

**Acceptance.** A window put aside and restored with nothing else happening lands on the
same patch **at the same zoom**, byte-identical. Minimising never discards work — a
property of the state change, and tested as one.

The zoom is asserted separately from the patch, and against `Zoom::LIFE_SIZE` as well: the
fixture uses 1_750 thousandths precisely because life size is the default, so a test that
saved it would pass for a panel that stored no zoom and built a default on the way out.

**The Place half of this task is blocked on Place identity and stays in this task**,
exactly as task 1's clause does: a patch alone is ambiguous, because `(4200, 0)` exists
on every surface, so *which surface* is part of what minimising has to save and this
task does not close without it.

**The zoom is done, and the first answer written here was wrong.** Kept rather than
deleted, because the mistake is one this repository is prone to and the correction is the
useful part.

What this section said: `alo_canvas::Zoom` is the right type, **holding a zoom a caller
hands in is not reading the camera** — a panel that took a `Camera` could correct itself, a
panel that remembers a number cannot — but `Zoom` is exported beside `Camera`, so taking
the dependency would put a `Camera` in reach. **The clean resolution is a small split in
`alo-canvas`**, it said, `Zoom` somewhere `Camera` is not.

**That resolution does not exist. Rust's privacy boundary is the crate, not the module.**
`alo-canvas` declares `pub mod camera`, so `alo_canvas::camera::Camera` is nameable from
any crate that depends on `alo-canvas`, whatever module `Zoom` is exported from. The split
would have bought a tidier import and no boundary at all — **and a boundary somebody
believes in and does not have is worse than none, because it is the one they stop
checking.** Two other lanes measured that crate's `lib.rs` instead of reasoning about it,
which is why the error lasted hours rather than weeks.

**So the rule is held by a test, and that is not the weaker option here — it is the only
one of the two that holds anything.** `crates/alo-put-aside/tests/the_panel_never_reaches_the_camera.rs`
reads the crate's own source for `Camera` and `camera::`, the idiom
`alo-adapting/tests/nothing_here_can_send_anything.rs` already uses to forbid itself a
network. It was watched refusing a real `use alo_canvas::Camera;` before it was trusted,
and it names both offending lines.

What a test here must **not** do is check a position: `alo-canvas`'s own header says a
surface in the wrong layer that compensates correctly passes a test of where it ended up.
The check is on whether anything can ask, not on whether the answer came out right.

**A third option was considered and rejected by the canvas lane, and the reason is the
better half of this entry.** The panel could have stored thousandths as a raw integer and
depended on nothing, holding the rule by genuine absence — but it would then hold a zoom
nobody validated. `Zoom::of` is what refuses 0 and 50_000, and `alo-arranging` already
rebuilds every restored value through its checked constructor for exactly that reason. **A
panel holding an unchecked number is a worse trade than a panel holding a checked type
beside a check that it holds nothing else.**

A local zoom type stays rejected on its own merits: a second vocabulary for one idea, which
is the `one-plane-two-vocabularies` fault repeated on purpose.

### 3. Restoring, and the camera that travels to it

**Status:** **model built, no evidence, 2026-09-30.** **This said `done` and that was wrong** —
see *What none of these tasks has* above. *The canvas travels to show it* is drawing, nothing
draws it, and the travel decision has never been taken by a person clicking anything. Its
drawing clause will be measured against **Minimized panel / 05 Restored in place**.

One click returns that window to its **saved** position and the canvas travels to show
it. Not to wherever the viewer happens to be.

**Acceptance.** The restore names the saved patch and not the current view — the test
distinguishes them by restoring from a view that does not contain the window. *Previous
view* returns a view rather than a window, which `travelling.rs` already decides.

**Built:** `crates/alo-put-aside/src/restoring.rs` and six tests in
`crates/alo-put-aside/tests/restoring_goes_to_the_saved_view.rs`.

**This file decides whether a travel is needed and what it would be; it does not perform
one.** The caller has the camera and moves it. That is not squeamishness about a
dependency: a version of this file that took a `Camera` and moved it would be a viewport
surface driving the plane, and it would pass any test that only checked where the window
ended up. What it takes instead is a `TheView` — the part of the plane being shown — and a
view is not a camera, because it says what is visible rather than who decided.

`Travel` is **a named enum rather than an `Option`**. *No travel is needed* and *no answer*
are different, and `Option<WhereItGoesBack>` spells them the same way — a caller that forgot
`None` would silently do nothing in the one case where doing nothing is correct, and be
right by accident until the day it was not.

**Each of the four ways to get this wrong was watched failing before the tests were
trusted:** never travelling, always travelling, answering with the current view, and
dropping the saved zoom. Each is caught by a test that names it, rather than by the suite
going red somewhere. A test written after the implementation agrees with the implementation
until somebody makes it disagree.

`already_shows` is *wholly inside* and stays strict, with a test whose only job is to say so.
The cost of the strict reading is a travel for a window that is nearly there; the cost of
the loose one is a click that appears to do nothing, and a person whose click changes
nothing concludes the click was lost rather than that the window was already visible.

### 4. When the saved place is taken

**Status:** **model built, no evidence, and blocked on History, which is `[v1]`, 2026-09-30.**
*The intended placement is shown* is drawing, and nothing draws it. The proposal is built and the
original position is carried; the clause that names History cannot be finished inside this
release and stays here.

`docs/design/the-alo-dock.md` says the restored window *comes forward and nothing is
rearranged*; the owner's specification adds that the person is **shown** the collision and
offered a nearby position. Both refuse invisible stacking; the newer one decides more.

**Acceptance.** The intended placement is shown before anything moves, and the original
position is kept in History. Nothing is rearranged without the proposal being visible
first.

**Built:** `crates/alo-put-aside/src/proposing.rs` (`Proposal`),
`crates/alo-put-aside/src/shown.rs` (`Shown`),
`crates/alo-put-aside/src/restoring_into_a_taken_place.rs` (`ask_for`, `accept`,
`dragged_to`, `Placed`), and ten tests in
`crates/alo-put-aside/tests/a_taken_place_is_shown_before_anything_moves.rs`.

**Nothing moves before the proposal is visible, and that decided the whole shape.** A
collision cannot be resolved inside a restore: by the time a function has brought the window
back and is choosing where to put it, the rearranging has begun and all that is left is how
much. So the collision is checked **before the panel is touched**, and a taken place returns
having changed nothing at all — the panel still holds the preview, the window still says it is
put aside. The test for it compares every window whole, because the subject is an absence.

**The refusal it guards is a proposal accepted without having been shown**, which is invisible
stacking wearing a confirmation dialog's paperwork. `Shown` is what stops it: there is no way
to make one without naming the view it was drawn in, this crate cannot draw, and `accept`
takes it **by value** — one showing authorises one placement, because an approval is never a
session.

**The History clause is not buildable in this release and is not being pretended away.**
History is `[v1]` in `docs/features.md`; the panel is `[v0.01]` only because the owner moved
that line on 2026-09-30, and the move was recorded as a tier move rather than a gate crossing.
`CLAUDE.md` binds building to what that file says.

So what is built is that **the original position is never lost**: `Placed` carries where the
window went *and* where it was, on both roads, so History has something true to read when it
exists. Keeping a value is not building a surface, and that reading is stated rather than
assumed — if the clause means *this task waits for History*, the work stands and only this
status line changes.

**It is told what is in the way rather than finding out.** `alo-dock`'s
`Patch::wholly_inside` is private and *overlaps* is a different question, so there is no public
predicate to ask. Asked of the lane that owns that crate; computing geometry about another
crate's type here would be a second opinion that agrees today. `ask_for` therefore takes the
occupying window, exactly as `restoring` takes a view rather than a camera — one line of wiring
when the predicate lands, and no stub in the meantime.

**Five mutations were watched failing before the tests were trusted:** restore-then-propose,
never propose, accept at the saved place, a drag that lands on the offer, and `was_at`
overwritten with where the window ended up. Each is caught by a test that names it.

**And one test here cannot fail, which its own doc now says.** The check that these types
cannot derive `Default` is a tripwire rather than a test: `Patch` has no `Default`, so the
derive is a compile error today. It stays because `Camera` already derives `Default`, so
`Patch` gaining one is an ordinary change — and on that day every guarantee-carrying type in
this crate becomes silently defaultable with `Zoom::default()` being `LIFE_SIZE`. **A check
that cannot fail lends unearned credit to the checks beside it**, which is why it is labelled
instead of quietly kept. Found because two mutations did not compile and therefore reported
nothing, and a mutation that produces no output is not a mutation that passed.

**A law 4 split was found by a test rather than by reading.** `Shown` began inside
`proposing.rs`; the one-way-in check counts constructors per file and reported two, which was
right — `Proposal` changes when what is offered changes, `Shown` changes when what counts as
having been seen changes. Two reasons to change, so two files.

### 5. Peek

**Status:** **built and reachable on a running machine; blocked on nothing putting a window
aside, 2026-10-02.**

**The pointer road exists.** A person moving the pointer over a preview now gets a peek, end
to end: `libinput_routing` translates the event, `direct_seat` and `direct_pointer` settle the
position, `Desk::dispatch` asks `Server::where_the_pointer_is_on_the_panel` for a
classification, and `TheDesktop::the_pointer_is_now` hands it to `alo-desktop`, which holds
the `Panel` and calls `alo_put_aside::peeking_at_a_preview::peek_at`. Three jobs, one crate
each: the shell classifies coordinates because it laid the panel out, `alo-desktop` owns the
state, and `alo-put-aside` owns the rule.

Seventeen tests across the two shell files. The ones that matter are
`the_same_count_in_a_different_order_names_no_window` — a reordered panel names no window
rather than the one now in that slot — and
`the_panels_own_region_without_a_preview_leaves_the_peek_alone`, which is the clause a person
would feel: crossing the gap between two previews must not drop the peek.

**What remains, and it is one thing rather than this task's whole surface.** Nothing *puts* a
window aside. `Server::put_this_window_aside` is written and has only its integration tests,
because the gesture that would call it does not exist — so the panel a person can now point at
has nothing in it. **That is the next change, and it is this task's last clause**: peeking is
reachable, filling the panel is not.

**Two earlier versions of this status were each true when written and wrong within hours**, and
both are kept because the pattern is the lesson rather than the dates: the first said *blocked
on the panel not being drawn*, which `#343` retired; the second said *blocked on there being no
input road*, which this change retired. A status is a measurement with a timestamp, not a
standing fact.

**The previous wording was stale in one clause and right in the other, and the correction is
narrower than it first looked.** *Blocked on the panel not being drawn* stopped being true in
`#343`: `panel_raster` lays it out, `desktop_raster` calls it every frame, and `alo-desktop`
holds a real `Panel` and hands it in. So that half sat stale for a day and hid work.

But *or routed at all* is still true, and it is **bigger than a hit test**. Measured on `main`
at `ad229c9c` rather than inferred:

- `Server::put_this_window_aside` has **six callers and all six are integration tests**;
- `alo-desktop`, which owns the `Panel`, has **no pointer or keyboard handling at all**;
- so the panel a person sees is drawn from a `Panel::new()` that nothing can add to.

**So no amount of work in `alo-put-aside` or in the shell's own files finishes this task.** What
it waits on is an input road in the crate that holds the panel — a gesture reaching
`put_this_window_aside`, and a pointer reaching `the_pointer_is_now_over_the_panel`. Named here
so the next lane does not rediscover it, and **not claimed as unblocked**: another lane read the
old status, concluded the task was available, and told the owner so. It was half available, and
the half that was missing is the half nobody had measured.

Its drawing clause will be measured against **Minimized panel / 04 Peek**.

A larger readable view over the current canvas. Releasing or Escape removes it and **the
window stays minimised**.

**Acceptance.** Peek leaves the window's state and the camera both unchanged, asserted
separately. A press meant for the canvas does not reach the peek — tested through the
real input path, and using the shell's existing hit test rather than a second opinion
about where clients are.

**Built:** `crates/alo-put-aside/src/peeking_at_a_preview.rs` and seven tests in
`crates/alo-put-aside/tests/a_peek_leaves_the_window_and_the_canvas_alone.rs`.

**The state machine is `alo_dock::Peeking` and no second one was written.** Checked for
specificity on an axis its header does not advertise, because that is exactly what went wrong
with `revealing`: it holds one window, which is right for a peek rather than a limit; it does
not ask how the peek began and says so loudly, because the keyboard road must end somewhere
identical; and **it holds no view at all**, which is how *the canvas stays exactly where it is*
is kept.

**Both halves of *unchanged* are held, and only one of them by a test.** The window's state is
this crate's own data and is asserted directly. The camera is held by the shape — `peek_at` and
`stop_peeking` take no view and return none, so no caller can be handed a reason to move.

A test comparing a view across those calls **could not fail**, and this lane spent today
learning what that costs. So the camera clause is guarded by a source check instead: the peek
module must never construct a `Travel`. That can fail, and it was watched failing.

**The finding, which is the reason this module exists at all.**
`alo_dock::PeekEnded::ByTravellingThere` names the ordinary road for a window on the canvas. For
a window that is **put aside**, the ordinary road is a *restore* — and since task 4 a restore
can come back with a proposal, because the saved place may be taken.

So a reader who follows that variant's name and performs a travel **skips the collision
proposal**, putting the window back invisibly behind whatever is there: the one thing task 4
forbids, reachable through a variant name. No rename was asked for — the variant's meaning,
*the person chose it, do the ordinary thing*, is general, and renaming another lane's public
enum over vocabulary is churn. What was built instead is `chosen`, which goes through
`ask_for`, with a test that says so — **so the wrong reading fails in this crate rather than in
the shell.**

**Four mutations watched failing:** choosing that ignores the collision, peeking at a window the
panel does not hold, peeking that un-minimises, and a `Travel` built inside the peek module. One
earlier attempt did not compile and therefore proved nothing, which is the second time tonight
that a mutation producing no output looked like a pass.

**What is owed is the input-path evidence**, and it needs `alo-shell`, which is another lane's.
The clause stays here rather than moving somewhere it could be ticked. Its wording names the
fault to avoid in advance — *rather than a second opinion about where clients are* — which is
the same reason task 4's overlap predicate was asked of `alo-dock` instead of written here.

### 6. The full-screen edge reveal

**Status:** **model built, no evidence, and blocked on there being no input road to the panel
on a running machine, 2026-10-01.**

**The full-screen half is stale: full screen landed in `#352`**, and
`crates/alo-shell/src/window_full_screen.rs` is in `main`. That clause sat blocked on something
that had shipped, which is the same fault corrected in task 5 above and in the lane table
earlier the same day — a record outliving what it described, this time hiding work rather than
misattributing it.

**The replacement reason is measured and is not a formality.** `alo-desktop` holds the `Panel`
and has no pointer or keyboard handling, and `Server::put_this_window_aside` has six callers all
of which are integration tests. So the edge reveal has nothing to reveal *from* on a running
machine, and this task's integration evidence waits on that road rather than on full screen.
See task 5 for the measurement.
The generalisation has landed, the decision was made and acted on, and **the region contract is
built with no evidence** (see above, and the entry in `docs/autonomy/v0-01-evidence.md`).

**This was the only status line here that said *no evidence* from the start**, and it was right
about itself while seven others were wrong about themselves. What made the difference is that
another lane had already said it about `revealing` — so this lane applied the standard where it
had been handed it, and nowhere else. Its drawing clauses will be measured against **Dock 11
Full screen edge** and the **Reveal** variants for Top, Bottom and Right, with the regions
recorded in `docs/design/the-regions-a-pointer-can-be-in.md`.

**The blocker neither lane had measured.** This task completes by *revealing the panel over a
truly full-screen window*. That cannot be run, and neither can the Dock's half of it, because
the shell cannot put a window into full screen:

- `alo_shell::window_mode::Mode` has three cases — `Normal`, `Maximized`, `InAShare`. None of
  them fills the screen, and `Maximized` is not it: a maximised window sits in the work area
  the Dock was already laid out beside, which is the arrangement this task's reveal exists to
  escape.
- The `XdgShellHandler` implementation in `alo-shell/src/surfaces.rs` answers `move`,
  `maximize`, `minimize`, `unmaximize`, `resize` and `reposition`. **`fullscreen_request` and
  `unfullscreen_request` are absent**, so Smithay's empty defaults run and a client asking to
  fill the screen is answered with silence rather than a refusal.
- `alo_dock::HowItSits::FillingTheScreen` exists in the model and occurs nowhere in `alo-shell`
  but one doc comment — seven occurrences of the word across every crate, and five of them are
  a menu item.

**The shell is honest about it, and the first draft of this paragraph was not.** That draft said
the compositor advertises a capability it does not implement. It does not:
`alo-shell/tests/support/wm_capabilities.rs` pins the advertised set to exactly `Maximize` and
`Minimize`, and asserts that adding `Fullscreen` to it **fails**. A test written against this
very mistake caught it before it was published, which is the second time in a week that the
record was about to be corrected in the wrong direction.

**What it means for the split.** Rules 3, 4 and 6 landed in `#332` and the region contract is
still this lane's. Neither produces this task's evidence, because the situation the machine
exists for cannot be created. **Full screen in `alo-shell` is the prerequisite**, it belongs to
lane B with the rest of the shell wiring, and **it is in no plan** — measured across `docs/`,
where the word occurs in this task, one design note and one menu item.

**`alo_dock::menu` already offers `What::FullScreen`.** Deciding a menu's contents before the
capability exists is not a fault, but nobody is to read that item as evidence that it does.

**`alo-dock::revealing` was generalised by lane B and it is done** — `#332`, *what holds a
revealed surface open is a set, not the last event*. It is the redesign rather than the rename
both lanes first read it as: `Revealing` now holds **which regions keep it open** — `ThePointer`,
`TheKeyboard`, `ADrag`, `AMenu` — and is revealed while any of them does, so rules 3 and 4 fall
out of set semantics instead of being cases. Rule 6 arrived as `FocusGoes` and `Dismissed`, so a
person who tabbed to the surface and pressed Escape lands back in their work rather than
nowhere. `ThePointer::OnTheSurface` already documents the continuous region, *including its
previews, its controls and the path between them, which the caller classifies*.

True full screen conceals the panel until the right edge reveals it, and the pointer must
be able to travel onto the panel without it disappearing.

**Decided by the owner on 2026-09-30: reuse the existing machine.** `alo-dock::revealing`
already is this state machine, at the bottom edge, with no timers and for a reason. It is
generalised rather than copied — writing the second one first is how there come to be two —
and **each surface gets its own instance**, so revealing or dismissing one does not affect
the other. ADR 0076 is untouched: the Dock stays at the bottom and the panel reveals from
the right, because **the edge becomes a parameter of the reveal rather than a setting of the
Dock.** Nobody is to read that parameter later as a preference to offer a person.

**The seven interaction rules, and which of them the existing machine does not have.** The
full spec and the region contract land with this task's own branch; recorded here now
because a decision written down late is a decision re-litigated.

1. Entering the activation strip reveals the surface. *Already true.*
2. Moving from the strip onto the panel keeps it open. *Already true.*
3. Pointer presence **or** keyboard focus keeps it open, and leaving one cannot dismiss it
   while the other remains inside. **Not true, and the current file does the opposite in
   both directions:** `the_pointer_is` conceals on `Elsewhere` whatever the keyboard is
   doing, and `the_keyboard` conceals on `IsElsewhere` whatever the pointer is doing. Tab
   into the panel, move the mouse away, lose the panel.
4. An active drag or an open panel menu also keeps it open. **Not represented at all.**
5. Leaving all those interaction regions conceals it. *Already true, and becomes the empty
   set.*
6. Keyboard dismissal returns focus to the full-screen window. **Not represented** —
   `Revealing` has never had an opinion about where focus goes.
7. No reveal or hide timers. *Already true, and nothing here ever had a clock.*

**So this is a logic change and not a rename, and the first report of it said otherwise.**
The error is worth keeping. `revealing.rs` is genuinely general about **geometry** — its own
header says *three places rather than a coordinate*, and that is true — and specific about
**how many things can hold it open**, which is one, the most recent. A module that
advertises the axis it is general on reads as general. Two lanes read that header and drew
the same wrong conclusion.

**Hit-testing:** the panel's previews, controls, menus and the connecting pointer path are
**one continuous interaction region**. Coordinate classification belongs to the caller and
the reveal machine consumes a classification, never a position.

**Completion needs integration evidence, and tests of the shared machine are explicitly
insufficient.** Reveal the right panel over a truly full-screen window, move onto a preview,
click it successfully, then the whole interaction again through the keyboard, and check the
bottom Dock still behaves. **Zero consumers means the module is reusable groundwork, not
evidence that edge reveal works in the shell** — the rename and the actual wiring are
separate work, and this task stays open until the complete interaction passes.

**The split.** The generalisation, rules 3, 4 and 6, and the shell wiring belong to lane B,
which owns `alo-dock` and works in `alo-shell`. The panel's side of the region contract is
this lane's.

**The region contract is built, and it is evidence of nothing yet.**
`crates/alo-put-aside/src/the_region_the_panel_claims.rs`, with eight tests in
`crates/alo-put-aside/tests/one_pointer_position_reveals_one_surface.rs`.

**Nothing on a machine reaches it**, because the situation the reveal exists for cannot be
created until `alo-shell` has full screen. So the ledger entry for this promise says *no
evidence*, and these tests are not cited as any — the lane that owns `revealing` set that rule
for its own module and it applies here unchanged: **a crate with passing tests and no caller is
reusable groundwork, and citing it would make the record claim a person can reach something
nobody can.**

What it holds:

- **The previews, the panel's controls, its menus and the path between them are one region.**
  The path is the part that is easy to omit, and omitting it is the flicker — a pointer
  travelling from a preview to a control crosses it, and answering *elsewhere* there conceals
  the panel under a pointer that never left it.
- A region is **not confined to the panel's outline**. The design has a peek region in the
  middle of the screen, nowhere near an edge, so a menu opening away from the panel is ordinary.
- **The asking strip and the surface are different answers**, because the strip reveals from
  concealed and the surface only keeps what is revealed.
- **The edge is data.** One frame among a hundred and fifty-six mirrors the shelf to the
  opposite edge, and nobody has established whether that is a stray or a right-to-left variant.
  Taking the edge as data costs nothing if it is a stray and is the difference between a rename
  and a rewrite if it is not — the correction `revealing` itself already went through.
- **No numbers at all.** No strip width, no rail width, no panel height. The standing rule that
  nothing is built to one screen size is satisfied here **by the values not being reachable**
  rather than by conversions done correctly, and every figure stays with whoever draws — the
  only place that knows the display's size and scale.

**And it contains no priority order, which is a refusal rather than an omission.** The owner
settled the shared corner: the panel owns the reserved area including the top-right corner, the
top controls and the Dock stop before it, and *one pointer position cannot reveal two surfaces*.
That is held as `at_most_one_surface_claims_it`, which **reports whether the rule holds and does
not pick a winner** — a version returning *which surface wins* would be the order this file
refuses to contain, and a classifier resolving overlap by the sequence of its branches would
decide geometry by accident.

The invariant is checked over a grid at **both** panel widths, because the reserved area follows
the panel's current width and the expanded one is where the top region is smallest and the
corner nearest. A rule about overlap tested where the surfaces are far apart is tested where it
cannot fail.

**One of those tests asserts that the design as drawn fails the invariant**, because it does:
the top controls region is the full screen width and overlaps the shelf's region at the
top-right corner. That is not a hypothetical mistake — it is the state the frames are in, which
the owner's decision supersedes, and an invariant that could not catch it would be checking
nothing. Four mutations were watched failing: the top controls reaching into the reserved area,
the Dock running to the screen edge, the panel not owning the corner, and the at-most-one rule
loosened to at-most-two.

### 7. Alo working in a minimised window

**Status:** **model built, no evidence, and blocked on there being no input road to the panel on
a running machine, 2026-10-01.**

**The panel is drawn**, and has been since `#343` — `panel_raster` lays it out, `desktop_raster`
calls it every frame, `alo-desktop` hands it a real `Panel`. The old reason was stale for a day.
What replaces it is measured in task 5 and is larger: nothing on a running machine can put a
window into that panel, because the crate holding it has no input handling, so there is no
minimised window for alo to be working in.

**The deep-teal clause was never `alo-appearance`'s and this line said it was** — that crate's
own `lib.rs` states ADR 0010's second half is *true of screens rather than of colours* and
*belongs where the drawing happens*, and that nothing in it can enforce it. The attribution was
this lane's mistake, repeated to the owner by another lane on this lane's word. Its drawing
clause will be measured against **Minimized panel / 11 alo working**. The
report is built and *no empty agent controls* holds in the types; the colour clause is about
drawing and stays here.

The task alo was given, the scope it may change, progress, last confirmed action, **Stop
available immediately**, and *Requires you* when a decision is waiting.

**Acceptance.** With AI switched off the panel is fully functional and shows no empty
agent controls and no nagging, which is ADR 0009 in this surface. Deep teal appears only
with the alo mark and a word, never as a selection colour.

**Built:** five small files, one responsibility each —
`crates/alo-put-aside/src/what_alo_is_doing.rs` (is there anything to draw),
`alo_at_work.rs` (the report), `the_scope_alo_may_change.rs`, `how_far_alo_has_got.rs`,
`requires_you.rs` — plus `Preview::alo`/`alo_is_now`, `Panel::alo_is_now` and
`Panel::waiting_on_the_person`, and nine tests in
`crates/alo-put-aside/tests/the_panel_is_whole_with_no_agent_at_all.rs`.

**ADR 0009 is the whole design, in its own words:** *the agent's surfaces disappear rather
than nag* — absent rather than present-but-disabled, because **a greyed-out feature is an
advertisement.** So the work was to make *present but hollow* **unrepresentable** rather than
merely avoided. The failure being guarded is not a crash: it is a panel that works and offers
an empty agent section, which passes any test that only asks whether the panel functions.

- `WhatAloIsDoing::Nothing` is the **value a preview is born with**, so a machine that
  declined AI never depends on a caller passing an argument to say so. There is no *switched
  off* variant, deliberately: both cases draw nothing, and the first thing somebody would do
  with a `SwitchedOff` case is write a message for it — **that message is the nagging.**
- An enum and not `Option<AtWork>`, because `None` reads as *not yet* and invites
  `if let Some(..) else { draw_the_placeholder() }` — idiomatic Rust that breaks ADR 0009.
- **Stop is not a field.** *Stop available immediately* is held by there being nothing that
  could withhold it: no `can_stop`, no state where the report exists and stopping does not. A
  boolean would be a promise a caller could set to `false`, and it is the one promise a person
  cannot check for themselves from a window they cannot see.
- Every field refuses its own hollow version: a blank task, a blank last-confirmed line, an
  empty change scope, *Requires you* with no question, a progress past the end. Reading and
  changing nothing are **different answers** rather than a list that happens to be empty, and
  absent progress differs from zero — *nobody can say how far* and *begun and got nowhere*
  draw differently, which is also why `Progress` has no `Default`.
- `waiting_on_the_person` finds the stopped task **in the panel's own order**. A panel that
  moved a waiting row to the top would move a preview out from under a pointer travelling
  towards it, which is the reveal machine's fault arriving by way of sorting.

**Three mutations watched failing:** a preview born holding a placeholder report, a blank task
name accepted, and an empty change scope accepted. Each caught by the test that names it.

**The deep-teal clause is not this lane's and is not dropped.** *Deep teal appears only with
the alo mark and a word, never as a selection colour* is about drawing; this crate holds no
pixels and `alo-appearance` owns the palette. The half that can be held here is held: a test
reads this crate's own source and fails if it names any colour at all, because a colour here
would be a second opinion about that palette and a selection colour here would be the exact
misuse the clause forbids. The rest stays in this task, with the task open.

### 8. Privacy

**Status:** **model built, no evidence, 2026-09-30.** **This said `done` and that was wrong.**
The refusal is tested as a refusal and holds. But *a private window shows a neutral `Preview
hidden` surface* is drawing, and nothing draws it — measured against **Minimized panel / 08
Private preview** when something does. The acceptance clause is inside this lane; the sentence
above it is not, and calling the task done read the acceptance and not the task.

**Every acceptance clause of this one is inside this lane and all of them hold**, which is why
it was the tempting one to close. It is still the task with the least owed to anybody else.

A private window shows a neutral *Preview hidden* surface with a safe identifying name,
and still restores normally. Previews are made locally.

**Acceptance.** A minimised state is never permission for an agent to inspect, share or
act on the window — tested as a refusal, not assumed from the absence of a call.

**Built:** `crates/alo-put-aside/src/a_safe_name.rs`, `whether_it_is_private.rs`,
`what_a_preview_is_headed_with.rs`, `what_an_agent_may_do.rs`, and eight tests in
`crates/alo-put-aside/tests/a_private_window_put_aside_stays_private.rs`.

**The acceptance's last clause decided the shape of everything.** *Assumed from the absence of
a call* is the test it forbids: assert that nothing reached the window and it passes on a
machine where the agent is not running, then goes on passing after somebody adds the call —
absence of evidence measured as evidence, which is ADR 0080's family.

So there is a **door**, it is **asked**, and it **says no**. `what_an_agent_may_do` returns
`NotPermitted` rather than a `Result`: **the return type has no success case**, so no caller
can obtain permission here whatever it passes. Nine calls in the tests, three things an agent
might want against two reasons it might give, each declined by name.

**Both reasons are refused, and by different names.** *It is put aside* is the one a
well-meaning caller actually reaches for — the window is out of the way, so surely reading it
harms nobody — and a person put a window aside to stop dealing with it, which is not handing it
over. *The person granted this* is refused too, because **this is not where grants are
checked**: a crate saying yes to a grant it never saw would be guessing, and a second place
deciding permission would be a second answer to *may this happen* with an agent finding
whichever is weaker. The two refusals differ so a caller can tell *not on those grounds* from
*not here*.

**A private window's title is now unreachable, not merely undrawn.** `Preview::called()` is
gone, replaced by `headline()`, which yields the safe name for a private window and does not
return the title at all. A public accessor returning the title is the whole of what a leak
needs, and documentation beside it stops nobody. **The same correction as `Zoom` and `Camera`,
made before a peer had to measure it** — a rule held by a value being out of reach beats one
held by a caller not asking for it.

`Privacy::Private` **carries** its `SafeName` rather than sitting beside one, so *private with
no safe name* does not exist — a `bool` plus an `Option` has that fourth state and the only
thing a surface could do with it is fall back to the title. A blank safe name is refused too,
because three rows reading *Preview hidden* identify nothing and a person with three private
windows away has to be able to pick one.

**Privacy is an argument, not a field with a default.** It is passed to every constructor that
needs it, because a default of *ordinary* is the wrong default for privacy: the caller who
forgets gets a leaked title, and forgetting is the commonest thing a caller does. Omitting it
does not compile.

*Preview hidden* is a **case and not a message**. The words belong to whoever draws and are
externalised there, because this crate cannot know the person's language and English shipped
here would be the hardcoded-English bug the standing rules name.

**Previews are made locally**, held by a test reading this crate's own source for any road off
the machine and its manifest for any dependency that could carry one. A preview that could
travel is a private window travelling.

**Four mutations watched failing:** the headline falling back to the title for a private
window, *it is put aside* accepted as a reason, a blank safe name accepted, and a road off the
machine added to the crate.

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

### And three designed frames that have no task here at all

Found on 2026-09-30 by mapping the Figma page's twelve frames against these eight tasks.
Eight of the twelve land on a task and *03 Location cue* is the Place, recorded above. **These
three land on nothing:**

- **06 Drag preview** — dragging a window out of the panel, with the collision shown before the
  drop;
- **09 Multi-select** — choosing several previews at once;
- **10 Arrange together** — the arrangement proposed for them, shown before it happens.

`docs/design/the-windows-put-aside.md` lists the first two under what is missing, in almost
those words, and **this plan never turned either into a task or into an entry here.** So the
gap was neither built nor recorded, which is the worse of the two ways to leave something out:
a deferred thing is visible and an unrecorded one is indistinguishable from a thing nobody
wanted.

**Recorded rather than made into tasks.** Whether they are in this release is the owner's, and
a lane adding three tasks to its own plan because it found three frames would be deciding scope
by drawing up work. The owner's instruction is the one this entry follows — *report actual gaps
openly rather than narrowing the agreed scope to close tasks* — and the gap is reported in the
direction that does not flatter the plan.

Task 4's proposal is the nearest thing already built: it shows an intended placement before
anything moves, for **one** window. *Arrange together* is that for several, and the hard part is
not the arrangement but that the same *nothing moves before it is visible* rule has to hold
across a set, which a proposal carrying one window cannot express.

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
