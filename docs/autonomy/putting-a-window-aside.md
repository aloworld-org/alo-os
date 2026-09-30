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

**Status:** **blocked on Place identity, 2026-09-30.** The transition is built and the
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

**Status:** done, 2026-09-30.

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

**Status:** **blocked on History, which is `[v1]`, 2026-09-30.** The proposal is built and the
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

**Status:** **blocked on the input-path evidence, which needs `alo-shell`, 2026-09-30.** The
peek is built and both *unchanged* clauses hold; the clause that needs the shell's own hit test
stays here.

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

**Status:** **blocked on the region contract here and on integration evidence in the shell,
2026-09-30.** The generalisation has landed; the decision was made and acted on.

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

### 7. Alo working in a minimised window

**Status:** **blocked on the deep-teal clause, which is `alo-appearance`'s, 2026-09-30.** The
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

**Status:** done, 2026-09-30. **Every clause of this one is inside this lane, and all of them
hold.**

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
