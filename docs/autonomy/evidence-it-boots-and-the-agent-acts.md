# Every promise of *it boots and the agent acts*, against the evidence

*Named `v0.01 — every promise, against the evidence` until 2026-10-02. **A release code is not a subject** — `v0.01` says *when*, never *what*, which is what `CLAUDE.md`'s *names are for strangers* forbids and what the owner asked be cleared out of this repository. **Both evidence documents had the same subject**, so the release was the only thing telling them apart; this one is named for the release's own descriptive title rather than its number. The release this belongs to is in `ROADMAP.md`, which is where release codes live.*

`docs/features.md` is the definition of what alo OS is. This is the other half
of it: **for every `[v0.01]` line, the test or the report that shows it, and
what is still owed.** It is not a release verdict — `ROADMAP.md`'s exit gate is
— and nothing here ticks anything. It answers one question, promise by promise:
*what in this repository would show somebody that this is true, and what would
not?*

## Why it is a file a test reads

The last audit of these promises was done by reading, seven times, and
`ROADMAP.md` records what that cost: **six v0.01 promises with no item and no
line, found one at a time, and twice the reading believed it had found the
last.** A promise is not missed through carelessness. It is missed because a
long document is read in a different order every time and the eye stops where
the last reading stopped.

So `crates/alo-reconciling` reads this file and `docs/features.md` together and
fails the gate when they disagree: a promise no entry is about, an entry about a
promise the definition no longer makes, a named test that is not there, a file
offered as a test that tests nothing, and a promise with no evidence and nothing
owed. **A promise added to `docs/features.md` and not reconciled fails in the
change that adds it** — which is the only moment anybody has the knowledge to
reconcile it.

## What counts as evidence here

Two things and no third: **a test in this repository**, named by its path, or **a
task report** under `docs/autonomy/updates/`. An ADR is where an argument was
settled, not evidence that anything was built; `ROADMAP.md` is the release's
account of itself and cannot be its own evidence; a file in another repository
cannot be run from here. All three are refused by name.

Nothing here judges whether a test *proves* its promise — no test can judge
that, and pretending otherwise would be this audit lying in a new way. The
reader is the check on that, and this file is written to be read.

## How to read an entry

Most entries have both halves, and that is the honest shape of v0.01: the code
is finished and the machine has never been asked. **Both halves are written
down**, because a promise recorded as shown while a third of it is owed is how a
release convinces itself it is finished — a failure `ROADMAP.md` already made
once, in a line ticked outright whose own footnote said the hardware was still
owed.

*Still owed* is deliberately not "not yet". It names the missing thing and why
it is not done, and a sentence too short to do that is refused.

## Every v0.01 promise, one at a time

### Every goal is a canvas

**Shown by:** `crates/alo-shell/tests/one_plane_under_one_viewport.rs`,
`crates/alo-shell/tests/client_lifecycle.rs`,
`crates/alo-shell/tests/the_canvas_walked.rs`,
`crates/alo-shell/tests/the_nested_fixtures.rs`,
`docs/autonomy/updates/the-canvas-walked.md`

The plane and the camera, the viewport that does not move under them, a press
reaching the right surface at three zooms and two pans, dragging, resizing,
panning on three roads, zoom and *Show all*, and ten steps of a walk read back off
a real parent's frames.

**Still owed:** **everything above one Place.** The plan that is closed is one
endless surface, and this promise says *a Place*. There is no canvas `Place` type
in this repository — the word is spent four times on other things — so *where the
person put them* is currently answerable only within a single surface. See task 1
of the canvas plan.

### Tidy this canvas

**Shown by:** nothing yet.

**Still owed:** all of it. Alignment and distribution of a selection, and the
same asked of alo as a proposal shown before anything moves. **The propose-then-
approve half already exists** — `crates/alo-put-aside/src/proposing.rs` shows a
placement before anything moves — and what does not exist is a proposal that
carries a **set** rather than one frame, which is the hard part of this promise
rather than the alignment arithmetic. Reached v0.01 on 2026-09-30 because the
frames promise had already moved with *several can be taken at once* in it,
leaving one operation at two tiers. Where
the work is: task 3 of `docs/autonomy/the-canvas-and-its-places.md` gives it the
selection to act on; nothing yet names the arrangement proposal itself.

### A Place remembers time

**Shown by:** nothing yet.

**Still owed:** all of it. Dragging a ribbon to see the canvas as it was on Tuesday
needs snapshots of an arrangement over time, and what exists is one arrangement —
`alo-arranging` holds the layout a person left, not a series of them. **The
nearest thing is not undo's snapshots either**: `alo-keeping-up` snapshots the
filesystem, which is a different subject with the same word, and reaching for it
would be the two-vocabularies fault this ledger has recorded three times. Reached
this release on 2026-10-04 by [ADR
0086](../decisions/0086-the-complete-canvas-is-one-current-milestone.md); it was
`[v1.1]`, the furthest-out canvas promise there was. Where the work is: **task 8 of `docs/autonomy/the-canvas-and-its-places.md`**, written the same day for this promise, which had no task in any plan before it.

### Every screen is a view onto the canvas

**Shown by:** nothing yet.

**Still owed:** all of it, and the shape of what is missing is known.
`alo-shell` holds **one** camera — `surfaces.rs`'s own note says *one home, not
two* about the single copy that already exists — so two displays at their own zoom
means a camera per viewport, which is a change to where that state lives rather
than an addition beside it. `alo-displays` already models more than one display
and `ScreenPlace` already gives each its own room, so the display half is there
and the canvas half is not. Reached this release on 2026-10-04 by [ADR
0086](../decisions/0086-the-complete-canvas-is-one-current-milestone.md). Where the work is: **task 9 of `docs/autonomy/the-canvas-and-its-places.md`**, which depends on the camera having one home — the same thing **task 9 of `docs/autonomy/the-smallest-canvas-worth-showing.md`** turns on, so the two are done together or the second is done twice.

### A panel out of view costs nothing

**Shown by:** nothing yet for the costing.

**Still owed:** **the costing, which is the promise.** What is built is that a
panel out of view *draws* nothing; what is promised is that a Place holding forty
frames is not forty programs running, and nothing measures or bounds that. A still
picture instead of a live surface is the mechanism, and no part of the tree yet
answers *what does an unreached frame cost*. Reached this release on 2026-10-04 by
[ADR 0086](../decisions/0086-the-complete-canvas-is-one-current-milestone.md).
Where the work is: **task 1 of `docs/autonomy/putting-a-window-aside.md`**, the development PC's lane, which holds the panel's three presentations — the still picture is one of them. The *measurement* of what an unreached frame costs is named by nothing, and that is the blocker to report rather than work around.

### Every canvas also answers as a list

**Shown by:** `crates/alo-shell/tests/every_frame_answers_as_a_list/mod.rs`,
reached through `crates/alo-shell/tests/client_lifecycle.rs` — six cases: the list
is in the plane's own order, a frame put aside still appears, the names are the
ones a reader would say, and the keyboard reaches each in turn.

**Still owed:** **the machine half only.** This is the one promise in this
milestone that is built: task 7 of
`docs/autonomy/the-smallest-canvas-worth-showing.md`, *Done, 2026-09-29*. It was
`[v1]` and built at v0.5, so its arrival in this release on 2026-10-04 by [ADR
0086](../decisions/0086-the-complete-canvas-is-one-current-milestone.md) is a
**correction of a promise built below its own tier** rather than new scope. What a
suite cannot show is a screen reader reading it on a machine, which is phase 8's
ground for every accessibility promise.

### Give it to alo

**Shown by:** nothing yet for the canvas half.

**Still owed:** **the selection, and the whole-goal handover.** *Alo working in a
window you put aside, and Stop* already reached this release and is the
cancellation half; what this promise adds is that **anything selected** can be
handed over, and that a whole goal can be. Neither exists: the canvas has no
selection a person can act on across frames, and `alo-admitting` and
`alo-reported` — the validated agent-status road — are an **island**, a tested
pair that nothing in the repository depends on, so a report crossing from the
agent reaches no production caller. Reached this release on 2026-10-04 by [ADR
0086](../decisions/0086-the-complete-canvas-is-one-current-milestone.md). Where
the work is: task 3 of `docs/autonomy/the-canvas-and-its-places.md` gives the
selection to act on; the island needs a dependent before the handover can report
anything.

### When the machine moves a window, the person is told

**Shown by:** `crates/alo-shell/tests/a_frame_is_never_lost_while_dragging/mod.rs`, reached through `crates/alo-shell/tests/client_lifecycle.rs` — a frame the panel's column came to cover is brought back and **where it was is kept**, which is the record half. The recheck that produces it has had a production caller since #403.

**Still owed:** **the sentence.** `alo_notifying::arriving::from_alo_os` has **no
production caller anywhere in this tree**, so this is alo OS's first notification
of its own, and its words are externalised in every shipped language from the
first commit — which is the real cost rather than the call site. Added to this
release on 2026-10-04 by [ADR
0086](../decisions/0086-the-complete-canvas-is-one-current-milestone.md), which
names recovery notices as part of the complete canvas; **no promise carried them
before**, so three tasks had arbitrated against a telling that was not in the
definition. Where the work is: **task 8 of `docs/autonomy/the-smallest-canvas-worth-showing.md`**, whose third acceptance clause is this telling and whose other half — the record — is paid.

### The World, and moving between Places

**Shown by:** `crates/alo-canvas/src/world_tests.rs`,
`crates/alo-shell/tests/the_world_is_a_step_out/mod.rs`,
`crates/alo-shell/tests/dragged_into_another_place/mod.rs`,
`crates/alo-shell/tests/move_to_place/mod.rs`

Places laid out as tiles, the span they reach, the Place under a point, and a
camera that fits them all. **Navigation is in production on both roads** — the
step out into the World is reached by the key and by the wheel, and stepping into
the Place under a point goes back in. Moving a frame between Places is built both
ways too: *Move to the next Place* on a chord, and the pointer road where a person
takes hold of a frame by its name, zooms out until the World appears, and lets go
over a tile.

**Still owed:** **the layout is derived rather than held, so where a Place sits
is not yet something a person can rely on or change.** `World`'s own header says
why that matters and asks for the opposite:

> The order Places were made in is not the order they are laid out in, and this
> type holds the layout rather than deriving it: where a Place sits in the World
> is a thing a person may come to rely on, so it is data rather than a function
> of a `BTreeMap`'s ordering.

The type does hold it. **Its only production constructor computes it, and
`canvas_the_world`'s own header states the property it then derives away** —
*where a Place sits in the World is something a person will come to rely on*.
`Server::the_world` collects the Places that have frames, sorts them — which is
`Place`'s `Ord`, and `Place` is a `u64`, so that is **the order they were made
in** — and lays them out at `nth` viewport widths across. Nothing stores a
`World`; the server's own state has no field for one; and `World::with` is called
by nothing but that fold. So a person cannot move a Place in the World.

**And it is worse than an arbitrary order: it is an unstable one.** The Places
come from `the_frames_on_the_plane`, which comes from the mapped surfaces — live
windows. That is right for *drawing* a World, and the header says so
deliberately: a Place nobody has put anything on is not drawn as an empty tile.
For **ordering** it means a Place leaves the World the moment its last window
closes, and **every later Place shifts one tile left**. So the order is not
creation order; it is the creation order of whichever Places currently hold
frames, and it changes under a person as they close things.

**That has a consequence for the panel's grouping, ruled on 2026-10-03, and the
ruling forbids it twice over.** The ruling says to put the current Place first
and order the others **in World order**, explicitly so the order is not
inherited accidentally from a `BTreeMap` — and separately that **new activity
must not reorder rows under the pointer**. Built against `the_world()` as it
stands today, World order *is* creation order, which fails the first clause by a
longer road; and because it is computed from live windows, closing a window on
an earlier Place reorders the panel's rows underneath the person, which fails the
second outright. A person who learns that one Place sits to the right of another
is wrong as soon as they close the last window on an earlier one.

So the grouping should **take the ordering from a `World` handed in** rather than
deriving one, so that whoever holds the layout owns the order and this fold's
fault stays visible at the one place that builds a World — and say in its status,
citably, that a person cannot arrange the World yet, so what a caller can hand in
today is still creation order. The grouping is then correct and the **meaning** of
its ordering is what waits.

Reached v0.01 on 2026-09-30 when the owner put the full canvas experience into
the current release. This entry read *there is no World — no type, no
navigation* until 2026-10-03, which was true when written and had been false
since task 2 landed on 2026-10-02. It is the kind of staleness that costs
something: **a ledger saying a type does not exist is what a lane checks before
building one**, so the risk was a lane declining to build on `World` or writing a
second one beside it.

Where the work is: task 2 of
`docs/autonomy/the-canvas-and-its-places.md` holds the World itself, and the
layout a person can arrange is the part this entry now names as owed.

### Frames, dragged and resized like a design canvas

**Shown by:** `crates/alo-shell/tests/client_lifecycle.rs`,
`crates/alo-shell/tests/the_frame_in_numbers.rs`,
`docs/autonomy/updates/the-canvas-walked.md`

The name moves a frame at three zooms and two pans and a press in its content does
not; all eight edges and corners resize with the application told its size during
the drag; the four double-headed arrows say which. ADR 0071 settled the shape.

**Still owed:** *several can be taken at once*, *guides and snapping line them
up*, *fit the Place to the screen* as distinct from *Show all*, *fill the screen
with what is selected*, and *double-click to work inside*. **And each of those is
promised with a keyboard form**, which is a larger gap than it reads: there is no
keyboard road to move or resize a frame at all, and the three keyboard forms this
repository has are zoom in, zoom out and *Show all*.

### A frame arrives the shape its work is

**Shown by:** nothing yet.

**Still owed:** all of it. An application declares no opening shape anywhere, and
*remembered per Place once the person changes it* needs both a Place to remember
per and a store to remember in. The arranging crate is the store and has no Place. The increment is task 5 of `docs/autonomy/the-canvas-and-its-places.md`.

### A frame can be dragged out of one Place and into another

**Shown by:** nothing yet.

**Still owed:** all of it, and it is two roads rather than one — through the World
by pointer, and by keyboard with *Move to Place*, ruled by the owner on
2026-09-30. **And task 4 is the thing this promise is not:**
restoring a minimised window returns it to the Place it was already on and
relocates nothing, and the two are separate tasks because they read as one
sentence and are two acts. The increment is task 3 of `docs/autonomy/the-canvas-and-its-places.md`.

### The habits people arrive with still work

**Shown by:** `crates/alo-shell/tests/every_road_a_keyboard_takes.rs`,
`crates/alo-shell/tests/client_lifecycle.rs`

The three canvas actions that exist — zoom in, zoom out and *Show all* — and the
window ones a person arrives with: next window, previous window, close.

**Still owed:** *switch desktops becomes move between Places*, which needs Places.
And *cycle frames* is currently *cycle windows* — the same ring, not yet a ring
per Place.

### Every application lives in a Place

**Shown by:** nothing yet.

**Still owed:** all of it. A dock window carries a patch and a how-it-sits and
**nothing that says which surface it is on**, so an application does not live
in a Place today because there is no Place for it to live in. This is the promise
that makes *one world rather than a modern half and an old half* true, and it
rests entirely on task 1. The increment is task 1 of `docs/autonomy/the-canvas-and-its-places.md`.

### The colours come from a source this repository can read

**Shown by:** `crates/alo-appearance/tests/a_palette_with_one_source.rs`,
`docs/autonomy/updates/a-palette-with-one-source.md`

**Still owed:** the promise says the source *generates* both the shell's
constants and the custom properties the web client needs, and nothing in this
repository generates the second — `alo-workplace`'s stylesheet is a fourth copy
of the palette that nothing checks, in a repository this test cannot read.

### Compositor: Wayland via Smithay

**Shown by:** `crates/alo-shell/tests/client_lifecycle.rs`,
`crates/alo-shell/tests/output_metadata/mod.rs`,
`crates/alo-shell/tests/input/mod.rs`, `crates/alo-shell/tests/pointer/mod.rs`

**Still owed:** every one of those runs against a nested session, which is a
development fixture and proves a client talks to us rather than that a machine
boots to us. No certified machine has ever displayed this compositor.

### Sign-in with an alo identity, and a local account that needs no tenant

**Shown by:** `crates/alo-accounts/tests/a_person_signs_in.rs`,
`docs/autonomy/updates/the-local-account-that-needs-no-tenant.md`

**Still owed:** two things. The alo identity half — a tenant, an account that
is not this machine's — is not built at all. And nothing on the image shows a
person a place to type a password: that is task 13 of the delivery plan, blocked
on `docs/decisions/0024-what-a-person-signs-in-at.md` being accepted.

### The agent overlay: one key, from anywhere

**Shown by:** `crates/alo-overlay/tests/one_key_summons_the_agent.rs`,
`crates/alo-overlay/tests/what_the_overlay_shows_at_rest.rs`,
`crates/alo-context/tests/from_an_invocation_to_a_change.rs`,
`docs/autonomy/updates/agent-overlay-summoning-seam.md`

**Still owed:** nothing draws it. The chord resolves, the request is made once
and the state at rest is derived from the daemon's own answers, and no pixel of
any of that has ever been on a screen.

### Launcher and window management: open, focus, close, tile

**Shown by:** `crates/alo-applications/tests/from_a_call_to_a_window.rs`,
`crates/alo-shell/tests/one_layout_decider.rs`,
`crates/alo-shell/tests/window_close/mod.rs`,
`crates/alo-shell/tests/window_activation/mod.rs`

**Still owed:** the launcher. `alo_shortcuts::Action::Launcher` is a chord with
nothing behind it — no surface lists the installed applications and nothing
starts one from a person's own choice rather than from a verb.

**What changed on 2026-09-26:** tile named the shell's window-tiling Wayland
tests until the v0.5 shell plan's task 16 took the half-output tile out and
replaced it with a division between two windows. The test that stood there is
gone with the mechanism it tested, and what is named in its place holds the
thing that replaced it: that there is exactly one layout decider in this
compositor, and that it is the dividing crate.

### Copy, cut and paste

**Shown by:**
`crates/alo-clipboard/tests/copy_cut_and_paste_across_applications.rs`,
`crates/alo-clipboard/tests/the_clipboard_is_not_a_turns_context.rs`,
`docs/autonomy/updates/the-clipboard-before-there-is-anything-to-draw.md`

**Still owed:** the compositor. `crates/alo-clipboard` is the selection — an
owner, the forms it offers, and a transfer somebody asked for, with every
refusal decided — and **nothing wires it to `wl_data_device`**, so no two
applications on a real machine have ever moved anything between them through it.
That wiring is `crates/alo-shell`'s and is the desktop lane's, and until it
exists images and files are offered forms measured against a fixture rather than
against a drawing program and a file manager. The three v0.5 lines around this
one — the clipboard portal for sandboxed applications, screenshots to the
clipboard — stay where the definition puts them, and *clipboard history* stays
at v1, which is why nothing here remembers anything.

The paragraphs below are the audit's own, left as it wrote them: a finding
rewritten by whoever closed it is a finding nobody can check.

**Was owed, 2026-09-11, before task 20:** all of it. Nothing in this repository
implemented a clipboard — no crate, no Wayland data-device handling in
`alo-shell`, no test. This was a promise with no line at all, and it was the
seventh of the kind the roadmap's audit kept finding one at a time.

**Read against the repository, 2026-09-11: this one needs no screen, no decision
and no machine, and the increment is task 20 of
`docs/autonomy/the-executable-plan.md`.** The audit that found it sorted it with
*the GPU works on first boot* and *boots on one certified machine* into work
waiting on hardware, in one pass while it was finding six things at once, and
that sorting was never examined. It is wrong. A clipboard is a **protocol before
it is a surface**: one client owns the selection and says which types it can
give; another asks for one of those types and is handed a pipe; the compositor
brokers and holds nothing of its own. Every one of those is a value, and every
refusal in it — a type that was never offered, an offer left over from an owner
who has since given the selection up, a paste with nothing behind it — is
decidable with no pixels, exactly as `crates/alo-overlay`, `crates/alo-approving`,
`crates/alo-indicator` and `crates/alo-recounting` decided their surfaces without
drawing one.

Nothing gates it either. **No agent verb touches the clipboard**: the ten in
`docs/contracts/agent-verbs.md` and `docs/by-hand.md` are six about files and
four about applications, so ADR 0001's grant model is not in this promise's way —
copy and paste is a person moving their own text between their own windows. ADR
0005's portal is the *sandboxed application's* route to the same thing and
`docs/features.md` schedules it at v0.5, so what v0.01 promises is the native
Wayland selection between clients of our own compositor. `smithay` 0.7 already
carries the protocol the compositor half would wire in, and `crates/alo-shell`
enables the `wayland_frontend` feature that holds it.

What is **not** closed by the increment, and is named so nobody reads task 20 as
the promise: the compositor wiring is `crates/alo-shell`'s and that crate is the
desktop lane's; images and files as offered types will be measured against real
clients rather than a fixture; and the three v0.5 lines around this one — the
clipboard portal, screenshots to the clipboard — stay where the definition puts
them.

### Notifications, with do-not-disturb

**Shown by:** `crates/alo-notifying/tests/do_not_disturb_holds_every_notification.rs`,
`crates/alo-notifying/tests/nothing_is_shown_while_the_machine_is_locked.rs`,
`crates/alo-notifying/tests/a_notification_cannot_answer_an_approval.rs`,
`crates/alo-notifying/tests/the_agent_never_reads_a_notification.rs`,
`crates/alo-notifying/tests/what_a_person_missed_is_kept_here_and_never_synced.rs`,
`docs/autonomy/updates/notifications-and-do-not-disturb.md`

Carried here with the promise when it moved tier on 2026-10-02. It is the most
completely built promise in its group: do-not-disturb holds *every*
notification, nothing shows on a locked machine, a notification cannot answer an
approval, and the agent never reads one — four refusals, each tested.

**Still owed:** **a notification on a screen, and the caller that would put one
there.** `arrives(notification, seat, quiet, missed)` is the only way to a
`Shown`, because `Shown::of` is `pub(crate)` on purpose, and it is `arrives`
that asks whether the seat is locked and whether quiet holds. It has no caller
outside its own crate; nothing in production holds a `Seat<Notification>`; and
`alo-desktop` passes an empty slice into every frame. So the rules are built and
nothing walks them, and do-not-disturb has nothing to hold back.

The clearest live consequence is already in the tree and commented as such:
`direct_desktop` moves a window out from under the fixed controls when they grow
and drops the result, because the telling is not built. The move happens and is
visible; the sentence that would explain it needs this promise, and its count
must go through `alo-strings`' plural machinery rather than an English format
string, because several of the twenty-four languages have three or four plural
categories.

Reached v0.01 on 2026-10-02 by the owner's decision, so that a promise already
at v0.01 could be kept rather than silently half-kept. Where the work is: task 6
of `docs/autonomy/the-session-and-the-displays-plan.md` for the rules and
do-not-disturb, and task 11 of `docs/autonomy/the-shell-plan.md` for drawing
them on a frame.

### The canvas is where they left it

**Shown by:** nothing yet.

**Still owed:** **the whole of the restart, which is the whole of the promise.** The
in-memory half is built and used in production — `alo-shell/src/canvas_remembered.rs` holds
an `alo_arranging::Arrangement`, puts a frame back where it was, and can produce
`the_arrangement_now()` — so this is not an unbuilt road but an unconnected one.

Nothing reaches a disk. `Arrangement::written()` and `Arrangement::read()` have **no callers
anywhere**; `the_arrangement_now()` has four and they are all in one test file;
`WhereTheyLeftIt` is constructed **only in tests**; and no file path for a canvas arrangement
exists. So a person who arranges their windows and signs out finds them gone.

Reached v0.01 on 2026-10-03 by the owner's decision, and the promise had been listed in
`docs/features.md` nowhere before that — it lived only in the plan, which read `Done` while
none of this was true. The owner ruled *do not weaken the promise* and reopened the task
rather than restating it.

**Not the same thing as monitor arrangement persistence**, which `alo-displays` does keep and
test. The names are close enough that checking the wrong one answers yes.

Where the work is: task 9 of
`docs/autonomy/the-smallest-canvas-worth-showing.md`.

### Alo working in a window you put aside, and Stop

**Shown by:** `crates/alo-put-aside/tests/the_panel_is_whole_with_no_agent_at_all.rs`,
`crates/alo-put-aside/tests/the_types_that_carry_a_guarantee_have_one_way_in.rs`,
`crates/alo-admitting/src/tests.rs`,
`crates/alo-reported/src/tests.rs`

The report's model and every refusal it makes are built: a task with no name, a blank
last-confirmed action and a stop with no reason are each refused by name, the whole report
arrives at once so it cannot be drawn half-built, and the panel is complete on a machine with
no agent at all. Stop is held by there being nothing that could withhold it — no flag, no
state in which a report exists and stopping does not. And the crossing from the agent service
was built on 2026-10-02: a neutral contract, a coordinator that checks a report against what
the person actually handed over, and a lost connection that becomes *status unavailable*
rather than staying *working*. Built, and reachable from nothing outside the two crates
themselves — which the owed half below states exactly, because built and in service are the
two words this document exists to keep apart and it had no word for the state between
them.

**Still owed:** **the drawing, and everything Stop does when pressed.** No surface draws any
of it — `Preview::alo` has one production caller, a filter, and the panel's raster reads no
part of the report, so no variant of the report type has ever been drawn. And nothing
cancels: there is no control, no cancellation road, no withdrawal of a run's authority, and
none of the three row wordings exists in any language.

**And no producer, which is the half this entry was missing.** No production code constructs
`alo_admitting::Reports`, nothing outside the pair calls
`alo_admitting::Reports::handed_over`, and no report has ever been admitted outside these
crates' own tests. So the road has no beginning as well as no end: nothing can send a report
in, and nothing can draw one out.

Both symbols are written out in full on purpose. Bare `handed_over` appears in thirty-eight
other files — `alo-broker` hands over an approving key, an unrelated thing with the same
English in it — so the short form is a true claim whose obvious check returns a
contradiction, and a reader who checks it concludes the entry is wrong rather than that
their search was too broad.

**The two crates are an island — a real edge between them and no way in.** Measured on
2026-10-03, and stated by direction because a sweeping version of it is false twice over:
`alo-admitting` is declared as a dependency by **nobody**; `alo-reported` is declared by
**`alo-admitting`**, in code, so it is not unreferenced; and the only trace outside the pair
is two doc comments in `alo-put-aside`, whose own manifest mentions neither crate. Both are
workspace members, so both compile, both pass clippy at `-D warnings`, and both run their
tests on every pass of these gates. Every signal a reader can reach says healthy, and what
those signals are healthy about is a model nothing can call.

So what the tests under *Shown by* demonstrate is **a model exercised by its own tests** —
they pass, they are real, and this entry previously let that read as a crossing in service.
They are listed as evidence and they stay listed: the fault was the word *landed*, not the
tests. **What this needs is a caller, not a removal.**

Wiring it is a design decision rather than a gap to fill on inference. A producer means
choosing where in the shell a person hands a window to alo, which is shared ground, and a
consumer means the panel's raster, which another lane owns. So the promise that reached
v0.01 on the owner's decision is one **no machine can currently walk in either direction**
— which belongs in front of the owner, beside the same shape found from the drawing side,
rather than inside a commit of mine.

Reached v0.01 on 2026-10-03 by the owner's decision, with the design ruled the same day. It
had been listed in `docs/features.md` nowhere, and its two nearest neighbours are `[v1]` and
promise a great deal more; they keep their tier, so this promise names only the surface that
was ruled on.

Where the work is: task 7 of `docs/autonomy/putting-a-window-aside.md`.

### Keyboard shortcuts, and a person can change them

**Shown by:** `crates/alo-shortcuts/src/changes.rs`,
`crates/alo-shortcuts/tests/what_this_crate_says.rs`,
`crates/alo-shell/tests/shortcut_dispatch/mod.rs`,
`crates/alo-shell/tests/shortcut_dispatch/layout.rs`

**Still owed:** a person changes them through a settings surface that does not
exist. Today a change is a value in a crate, and the file it would be written to
is not read by anything a person can reach.

### Window management: move, resize, snap, tile, minimise, maximise, close

**Shown by:** `crates/alo-shell/tests/window_move/mod.rs`,
`crates/alo-shell/tests/window_resize/mod.rs`,
`crates/alo-shell/src/window_dividing_tests.rs`,
`crates/alo-shell/tests/window_minimize/mod.rs`,
`crates/alo-shell/tests/window_maximize/mod.rs`,
`crates/alo-shell/tests/window_close/mod.rs`

**Still owed:** snap is a shortcut action dispatched to a layout and has no
pointer gesture behind it — dragging a window to an edge does nothing. **And it
now needs two windows:** a division divides between windows, so a chord with one
window open is refused by name, where the half would have put the only window on
half a display with nothing beside it. All of it is still measured in a nested
session rather than on a machine.

**What changed on 2026-09-26:** snap and tile were one mechanism — the shell's
window-tiling module, which computed half an output — and the v0.5 shell plan's
task 16 removed it, because a division becoming session state would have meant a
window's place decided twice, once by a tree of shares and once by a half. A
chord now divides the display between the focused window and the next one. The
Wayland tests that stood here went with the mechanism, and what is named in
their place holds what a chord does now, including what it refuses.

### The alo Dock

**Shown by:** `crates/alo-dock/src/layout.rs`,
`crates/alo-dock/tests/what_this_crate_says.rs`,
`docs/autonomy/updates/the-dock-fixed-to-the-bottom-edge.md`

**Still owed:** **nothing draws a dock.** The labels giving way to icons are
arithmetic that no compositor asks for yet, and everything the Dock is for —
showing what you can open, bringing what is already open into focus, what a click
does, favourites and an overflow area — has no code at all.

*This entry read **The dock, and the person decides where it goes**, and cited
`along.rs` for the two orientations. [ADR
0076](../decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
withdrew that promise and fixed the Dock to the bottom edge; `along.rs` is
deleted. **The owner reversed that decision the next day and the edge choice is
a promise again**, now its own entry above — so the Settings surface is owed
after all, and `along.rs` is the deletion that has to be answered rather than a
file nobody needs. What the Dock's settings section also asks — whether it gives
way when a window needs the room — is its own promise at v0.5.*

### The top controls

**Shown by:** nothing yet.

**Still owed:** all of it. There is no top band in this repository: nothing lays
one out, nothing holds the active window's controls, and no surface offers the
way back to the canvas. The pieces it will join already exist and are the reason
it can be added by being pushed rather than by a signature changing —
`FixedControlsDrawn` carries `dock_band`, `panel_reserved` and `status_area`, so
the never-lost rule, the recheck and the mover cover a fourth control with no
change of their own.

**Two halves of this promise rest on roads that do not exist, and they are
different roads.** *It stays while it is being used* needs the covered-surface
reveal, which is built in `alo-dock` and called by nothing. *A person who would
rather it never hid may keep it visible* is a **setting** — a thing a person
chose, not the state their work is in — and ADR 0038's per-crate settings file
is where it belongs; nothing names one for this surface yet.

Reached v0.01 on 2026-10-02 by the owner's decision. It had been drawn in the
regions note and listed in `docs/features.md` nowhere, which is why it is owed
rather than merely unbuilt: three tasks were arbitrating against a surface no
lane was permitted to build, and the pointer classifier could not land in any
crate because one of its three inputs was unbuildable. Where the work is: task
6 of `docs/autonomy/the-canvas-and-its-places.md`, which is the fixed controls as
a set; the top controls are its fourth member and are written up there as 6a.
**Cited as 6 rather than 6a because 6a is not a pointer this repository can
follow** — `alo-reconciling` reads a task number as digits followed by `of`, and
finds a task by a `### <digits>.` heading under `## Tasks`, so a promise citing
6a would point nowhere.

### Switching between windows, and between applications

**Shown by:** `crates/alo-shell/tests/window_switch/mod.rs`,
`crates/alo-shell/tests/shortcut_dispatch/mod.rs`

**Still owed:** there is no switcher surface — nothing shows a person what they
are switching to — and, as with everything in the compositor, no machine has
displayed it.

### list, read, find, rename, move, archive

**Shown by:** `crates/alo-files/tests/doing.rs`,
`crates/alo-files/tests/on_this_machine.rs`,
`crates/alo-files/tests/what_one_execution_reaches.rs`,
`crates/alo-files/tests/a_file_with_another_name_is_not_read.rs`

**Still owed:** the six verbs have never run on a certified machine, which is
the half of law 3 only hardware gives.

### open, focus, arrange, close

**Shown by:** `crates/alo-applications/tests/from_a_call_to_a_window.rs`,
`crates/alo-applications/tests/what_this_crate_says.rs`

**Still owed:** the four verbs reach a window through a port that a real
compositor does not implement yet, so no application on any machine has been
opened, focused, arranged or closed by an agent.

### focused window, selection, open document

**Shown by:** `crates/alo-context/tests/from_an_invocation_to_a_change.rs`,
`crates/alo-context/tests/what_this_crate_says.rs`

**Still owed:** the context offered at invocation comes from a fixture rather
than from a compositor: nothing reads a real focused window, a real selection or
a real open document, because no surface has ever invoked a turn.

### Grants: pick a folder, see what is granted, revoke it, and it expires

**Shown by:** `crates/alo-picking/tests/a_person_picks_a_folder.rs`,
`crates/alo-remembering/tests/the_grants_a_machine_keeps.rs`,
`crates/alo-capability/tests/what_this_crate_says.rs`,
`crates/alo-granted/tests/the_grants_a_person_can_see.rs`,
`crates/alo-changing/tests/a_persons_change_reaches_the_daemon.rs`,
`docs/autonomy/updates/native-folder-selection.md`,
`docs/autonomy/updates/a-grant-made-now-reaches-the-daemon-now.md`,
`docs/autonomy/updates/the-grants-a-person-can-see.md`,
`docs/autonomy/updates/a-persons-change-reaches-the-file-the-daemon-re-reads.md`

**Still owed:** the list has never been drawn. `crates/alo-granted` is *see
what is granted* and *revoke it by hand* as a value — derived from the
machine's own kept grants, with revocation through the same
`alo_capability::Grants::revoke` the daemon enforces, an expired grant never
shown as live, and *nothing granted* a sentence — and `crates/alo-changing`
is the person's half of a change made through either: written whole to the
file the daemon re-reads, the knock after the write and never before, and a
machine with no daemon keeping the change for the next sign-in. Putting any
of it on a screen is the compositor's, in `crates/alo-shell`, which no
machine has displayed.

### Every execution recorded with its origin, approval and grant

**Shown by:** `crates/alo-keeping/tests/on_this_machine.rs`,
`crates/alo-recounting/tests/afterwards_ask_what_it_did.rs`,
`docs/autonomy/updates/afterwards-ask-what-it-did.md`

**Still owed:** the record is written and read back on a development machine
only; no record has ever been written by a daemon running on a certified one.

### It runs on the machine you already own

**Shown by:** `crates/alo-models/src/catalogue.rs`,
`crates/alo-driving/tests/against_a_model_on_this_machine.rs`,
`crates/alo-driving/tests/from_a_prompt_to_what_a_machine_offers.rs`

**Still owed:** the catalogue's memory figures are a table until a model has run
on the certified laptop, and seven of the twelve entries have never been
measured anywhere because the measuring box has less memory than they want.

### It works well on a CPU and it works well on a GPU

**Shown by:** `crates/alo-models/src/catalogue.rs`,
`crates/alo-driving/tests/against_a_model_on_this_machine.rs`

**Still owed:** the GPU half entirely. No machine with a card has run any of
this, so *works well* is measured on one side of a promise that says it is
measured on both.

### The catalogue says whether a model can drive the verbs

**Shown by:** `crates/alo-driving/tests/from_a_prompt_to_what_a_machine_offers.rs`,
`crates/alo-driving/src/exercises.rs`, `crates/alo-driving/src/measured.rs`,
`crates/alo-models/src/driving.rs`

**Still owed:** seven of the twelve entries say `not-measured`, which is an
honest answer and not a grade — the catalogue can carry the judgement for
entries nobody has put a question to.

### And it is measured by us, not claimed by the publisher

**Shown by:** `crates/alo-driving/tests/against_a_model_on_this_machine.rs`

**Still owed:** the five measured grades were taken on a development box rather
than on a certified machine, and the measurement has never been repeated on the
hardware the catalogue's figures describe.

### A machine is only offered agent work it can actually do

**Shown by:** `crates/alo-models/src/refusing.rs`,
`crates/alo-driving/tests/from_a_prompt_to_what_a_machine_offers.rs`

**Still owed:** the refusal and its three alternatives have no setup screen to
appear on, so no person has ever been shown the choice this promise is about.

### The GPU works on first boot

**Still owed:** all of it. Nothing in `image/` installs or verifies a driver
stack, no test asks a machine what card it has, and no machine with a card has
booted this image. It is a v0.01 promise with no line anywhere in the
repository, and it is the second such finding of this audit.

**Read against the repository, 2026-09-11: it waits on a machine, and on an
image that carries a stack to accelerate.** `docs/hardware.md` already defines
the promise in four clauses — the display comes up at native resolution with no
configuration, the card is available to the model runtime with no driver
installation and no CUDA or ROCm archaeology, a model runs in one command, and an
upgrade cannot break that stack because the runtime is versioned with the drivers
it needs. Three of the four are answers a machine gives and nothing else does;
the fourth is about what an image contains, and `image/Containerfile` adds two
binaries, two units, two directories and one description to a pinned base and
**carries no model runtime and no weights at all**
(`docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md` found
the same gap from the other side). So there is nothing on this image for a card
to accelerate, and a check that the image names a driver stack would be a check
with nothing to hold: which stack is pinned for the certified workstation is an
engine decision under ADR 0011's *configured, never patched*, and the certified
workstation itself is `docs/hardware.md`'s *24 GB VRAM or more* with no model
named in the table. The machine is task 12 of
`docs/autonomy/the-executable-plan.md`, which is scheduled and needs hardware
nobody has plugged in. **Nothing here is reachable by this lane**, and saying so
with the reading behind it is what this entry is for.

### A model runs in one command

**Shown by:** `crates/alo-models/src/runtime.rs`,
`crates/alo-models/src/ollama.rs`, `crates/alo-models/src/weights.rs`

**Still owed:** the runtime adapter has never been run against a real model
service on any machine — the tests put its own protocol to a socket, which is
the shape of the exchange rather than the thing working.

### The release carries no model, and a person brings their own

**Shown by:** `crates/alo-image/src/weights.rs`,
`crates/alo-image/src/checking.rs`,
`crates/alo-image/tests/what_the_image_owes_the_daemons.rs`,
`crates/alo-models/src/choosing.rs`,
`docs/autonomy/updates/the-release-carries-no-model.md`

**This entry replaced one for *the local model is what the machine arrives ready
to run*,** which was this promise from 2026-09-11 until 2026-10-06. The owner
decided the release ships no weights: a model we picked was a model picked for
them, it was sized for the one 16 GB laptop the hardware document certifies
first, and everybody downloaded 4.87 GiB including the people who would never
run it. ADR 0025's argument for carrying them was sound and is not called wrong
— what changed is which half of it governs, and *nobody chooses for the person*
is the half that does.

**What is shown.** The recipe lands no weights, read off the real recipe rather
than off a fixture, and a recipe that puts them back fails — caught by where a
copy lands rather than by a build stage's name, so weights returned under a
different name or in the store's new place are caught too. The store the runtime
serves from moved into the service's own state directory, and a second rule asks
whether that place is one the machine may write: the read-only half of a bootc
machine is where the weights used to land, so leaving the store there would have
left every sentence about bringing your own weights reading correctly while the
thing itself could not happen.

**What a machine with no model does** was built and tested before this change,
which is what made the decision safe to take. It says so and names the
alternatives in a fixed order, and no method in that crate answers with a place
or with a model — so nothing chooses for the person.

**Still owed:** the sentence a person reads when they first meet a machine with
no model. *Out of the box, this machine answers nothing*, and that is the promise
rather than a gap in it — but nothing in this repository draws a screen that
offers the four roads, so the words exist without a surface. The decision this
promise rests on is
`docs/decisions/0095-the-release-carries-no-model-and-a-person-brings-their-own.md`,
and the road back to answering out of the box is our own API, which is **future**
and is marked so in the feature list. When it arrives, answering that way means
inference leaving the machine — which Law 1 covers, and which no sentence may
describe as zero egress.

### Run a model we never catalogued.

**Shown by:** `crates/alo-models/tests/a_model_we_never_catalogued.rs`,
`crates/alo-models/tests/a_brought_file_is_one_the_runtime_answers_to.rs`,
`crates/alo-models/tests/the_pinned_runtime_accepts_what_alo_os_sends.rs`,
`crates/alo-choosing/src/choosing.rs`,
`crates/alo-driving/tests/against_a_file_brought_to_this_machine.rs`,
`crates/alo-image/src/weights.rs`,
`docs/autonomy/updates/a-brought-file-is-one-the-runtime-answers-to.md`,
`docs/autonomy/updates/the-pinned-runtime-and-what-alo-os-sends-it.md`,
`docs/autonomy/updates/the-release-carries-no-model.md`

**This entry moved here from the next release on 2026-10-06**, with the promise.
The tier moved rather than the scope gate being crossed: with no weights
shipped, this is how a person gets a local model at all, so a promise of
bringing your own that waited for a later release would have been a promise with
nothing behind it.

**What is shown.** A brought file is one the runtime answers to rather than one
we recognise, and the pinned runtime accepts what alo OS sends it — tested
against the runtime itself. A person points alo OS at a file where it already
is; the file is measured rather than described, its grade is *not measured*
until somebody measures it, and every refusal names the path. Bringing a file is
not choosing it, for the same reason adding a provider is not. A grade a
measurement earns is written beside the person's own weights and never into the
catalogue alo OS ships, which is not theirs. The catalogue recommends and does
not gate.

**What moving it added** is the store. The runtime used to serve from the
read-only half of a bootc machine, which is right for weights that arrive with
the image and impossible for weights a person brings; it now serves from the
model service's own state directory, and a rule asks whether that place is one
the machine may write.

**Still owed:** somebody's own weights on their own machine. Everything above is
tested here, and no person has yet brought a file to a booted alo OS and been
answered from it. There is also **no surface**: the door exists and nothing in
this repository draws the screen that opens it.

### Add your own provider in Settings

**Shown by:** `crates/alo-choosing/tests/the_three_choices.rs`,
`crates/alo-choosing/tests/a_persons_choice_reaches_the_machine.rs`,
`crates/alo-models/src/secret.rs`,
`crates/alo-secrets/tests/a_real_keyring_answers.rs`,
`crates/alo-asking/tests/a_key_reaches_one_provider_only.rs`,
`docs/autonomy/updates/a-persons-choice-written-where-the-machine-reads-it.md`

**Still owed:** the Settings surface — the place in the shell where a name, an
address and a key are typed, which is the compositor lane's and has no pixels
anywhere yet.

**No longer owed, 2026-09-11: adding one by hand.** This entry used to read *a
provider is added by writing the person's own settings file by hand*, and that
was the true state of the repository: `alo-choosing` read the file and nothing
wrote it, so every choice ADR 0016 gives the person was one no surface could
carry out. `alo_choosing::Choosing` is the written change — a provider added, a
model chosen, weights brought, a language picked — landing in the person's own
file whole or not at all, and read back through the door `alo-agentd` reads it
through. What a surface has left to do is show it. The key is still the
keyring's and is still not in the file: there is nowhere in the shape to put one
and the writer cannot invent one, which
`crates/alo-choosing/src/writing.rs`'s round trip refuses by name.

### Setup's fourth choice

**Shown by:** `crates/alo-record/tests/a_machine_with_no_agent.rs`,
`crates/alo-capability/tests/what_this_crate_says.rs`

**Still owed:** there is no setup, so there is no fourth choice to make. What is
built is the machine that results from having made it — the agent's reach gone
at once, nothing further recorded — rather than the moment of choosing.

### A person never learns the name of anything we rented

**Shown by:** `crates/alo-saying/src/rented.rs`,
`crates/alo-saying/src/translated.rs`,
`crates/alo-saying/tests/what_this_machine_can_say.rs`,
`docs/autonomy/updates/a-translators-line-held-to-the-same-rule.md`

**Still owed:** a surface that composed a sentence of its own would pass
through nothing — the checks read the English declarations, the vocabulary
they build, and, since 2026-09-11, every translated line at the moment a
machine loads the file: a line naming a rented component is left out with the
same list and the same argument, the rest of the file is kept, and the refusal
names the file, the key and the language. What keeps a surface from composing
is construction rather than a check — every surface crate's sentence types are
sealed against being made from text — and nothing verifies that mechanically
across the workspace.

### And it is enforced rather than remembered

**Shown by:** `crates/alo-saying/src/rented.rs`,
`crates/alo-saying/src/translated.rs`,
`crates/alo-saying/tests/what_this_machine_can_say.rs`

### Anything an agent verb can do, a person can do by hand

**Shown by:** `crates/alo-by-hand/tests/every_verb_can_be_done_by_hand.rs`,
`docs/autonomy/updates/every-verbs-by-hand-answer.md`

**Still owed:** the check holds every verb to *naming* a plain way that the
definition promises; it cannot hold one to a plain way that is **built**, and
today none of them is. Six of the ten verbs ship at v0.01 and their surface —
the file manager, the search, the text editor, the terminal — is v0.5, so a
v0.01 machine whose agent is unavailable can do none of the six by hand;
`docs/by-hand.md` says so in its own words and `ROADMAP.md`'s v0.5 exit gate is
where it comes due. One verb, `archive_folder`, has no promised plain way at all
and is recorded as owed: nothing in `docs/features.md` promises **making** an
archive by hand, and the proposed line is in the report rather than added here,
because the scope gate is the owner's.

### And it holds however the agent became unavailable

**Shown by:** `crates/alo-answering/tests/from_a_question_that_failed_to_what_left.rs`,
`crates/alo-answering/src/failed.rs`, `crates/alo-asking/src/ran_out.rs`

**Still owed:** the six ways the agent becomes unavailable are refusals a crate
produces; nothing shows one to anybody, so *the machine loses convenience and
never capability* has never been true of a screen.

### Running out of credit is its own answer, not an error

**Shown by:** `crates/alo-asking/src/ran_out.rs`,
`crates/alo-answering/src/wrong.rs`,
`crates/alo-asking/tests/a_key_reaches_one_provider_only.rs`

**Still owed:** *it says it once* is now a mechanism rather than a hope —
`crates/alo-telling` is the memory, and a repeat of one unavailability produces
no value a surface could word. What is owed is the surface itself: no screen has
ever shown the sentence, once or otherwise.

### And it never nags

**Shown by:**
`crates/alo-telling/tests/a_machine_that_cannot_reach_a_model_says_so_once.rs`,
`crates/alo-telling/src/telling.rs`,
`docs/autonomy/updates/a-machine-that-cannot-reach-a-model-says-so-once.md`

**Still owed:** the memory is a session's and nothing holds one yet, because
nothing in this repository runs a session with an agent in it. So the promise is
kept by the only thing that could break it — a shell adopting `alo-telling`
cannot say the same thing twice unasked — and not yet by a machine anybody has
watched all afternoon. The other half is a surface, which is task 3's overlay
and does not exist.

### Or use an API instead

**Shown by:** `crates/alo-asking/tests/from_a_question_to_what_left.rs`,
`crates/alo-asking/tests/a_key_reaches_one_provider_only.rs`,
`crates/alo-models/src/source.rs`

**Still owed:** every provider exchange in the tests is against a socket this
repository stands up. No question has been put to a real provider's API from a
machine, with a real key, and the paired-machine place named in the same
sentence is a v0.5 line elsewhere in the definition.

### is a bound around it and never a choice inside it

**Shown by:** `crates/alo-choosing/tests/on_this_machine.rs`,
`crates/alo-choosing/tests/the_three_choices.rs`,
`crates/alo-agentd/tests/what_a_machine_says_about_itself.rs`,
`docs/autonomy/updates/an-administrator-set-that-rule.md`

**Still owed:** two files and two owners are real and no person has seen either.
The refusal naming the rule has no surface to appear on, and nothing writes the
machine description on a machine an administrator manages.

### Where the answer came from is said where the answer appears

**Shown by:** `crates/alo-asking/src/answer.rs`,
`crates/alo-asking/tests/a_day_that_only_looks_like_it_never_left.rs`,
`docs/autonomy/updates/an-operating-system-that-does-not-claim-what-it-cannot-know.md`

**Still owed:** *where the answer appears* is a screen, and there is none. The
provenance line is carried beside every answer in the code and has never been
put in front of a person.

### Never a silent fallback, in either direction

**Shown by:** `crates/alo-answering/tests/from_a_question_that_failed_to_what_left.rs`,
`crates/alo-answering/src/wrong.rs`, `crates/alo-answering/src/failed.rs`

### A model on this machine and a provider you added are both ordinary

**Shown by:** `crates/alo-choosing/tests/the_three_choices.rs`,
`crates/alo-models/src/source.rs`, `crates/alo-answering/src/failed.rs`

**Still owed:** the promise is about how the two are *presented* as much as how
they behave, and nothing presents anything: no setup screen, no settings panel,
no place where one could be shown as the good option and the other as the
degraded one.

### Model lifecycle: pull, list, serve, unload, remove

**Shown by:** `crates/alo-models/src/runtime.rs`,
`crates/alo-models/src/costing.rs`, `crates/alo-models/src/brought.rs`

**Still owed:** the five operations have never been run against a real model
service, so *disk accounted honestly* is arithmetic over figures nothing has
checked against a disk.

### every network egress an agent causes, visible at the moment it happens

**Shown by:** `crates/alo-egress/tests/what_this_crate_says.rs`,
`crates/alo-indicator/tests/the_egress_indicator_on_a_screen.rs`,
`crates/alo-asking/tests/from_a_question_to_what_left.rs`,
`docs/autonomy/updates/the-egress-indicator-on-a-screen.md`

**Still owed:** the indicator is decided, worded and kept in step with the
machine, and it has never been drawn. *Visible at the moment it happens* is a
claim about something a person can see, and nobody has seen it.

### No telemetry

**Shown by:** `crates/alo-egress/src/errand.rs`,
`crates/alo-egress/src/itself.rs`,
`crates/alo-asking/tests/a_day_that_never_left.rs`

**Still owed:** the policy is an enumerated list of why alo OS itself reaches
the network, and the promise's proof is a measurement at a network boundary on a
machine that has run a working day. No machine has.

### Boots on one certified machine, firmware to sign-in

**Still owed:** all of it, and it is scheduled rather than missing — task 12 of
`docs/autonomy/the-executable-plan.md`, which needs a machine nobody has plugged
in. *To sign-in* is owed twice over: there is nothing to sign in at until
`docs/decisions/0024-what-a-person-signs-in-at.md` is accepted and task 13 of
`docs/autonomy/the-executable-plan.md` is built.

**Read against the repository, 2026-09-11: not reachable, and it is the one of
the four that is honestly waiting rather than unexamined.** Both halves were
checked again. The firmware half is a machine: `crates/alo-image` holds what the
image owes the daemons and `image/Containerfile`'s own first paragraph says an
image that builds is not an image that boots. The sign-in half is a binary that
does not exist — `crates/alo-shell` has no `src/main.rs` and no `[[bin]]`, which
is the finding task 10 stopped on — and building one means choosing between the
options ADR 0024 sets out. So this promise waits on a machine **and** on a
decision, and no increment in between is available to this lane.

### Image built as an OCI container image

**Shown by:** `crates/alo-image/tests/what_the_image_owes_the_daemons.rs`,
`docs/autonomy/updates/what-a-person-signs-in-at.md`

**Still owed:** the image is built and booted in a VM, which is not a machine,
and it carries no shell to boot to — `crates/alo-shell` has no binary, which is
the finding task 10 stopped on.

### Compact, and minimised — two things, and the person picks

**Still owed:** all of it, and it is scheduled rather than missing — task 1 of
`docs/autonomy/putting-a-window-aside.md`, with tasks 2 to 8 behind it. **This
entry has no *Shown by* half, and that is what it is for.** The promise moved into
this release on 2026-09-30, when the owner asked for the minimised-windows panel
to be built now. Nothing has been written for it yet, so an entry claiming
evidence would be the failure this ledger exists to catch, one day old.

What is decided rather than built is in
`docs/design/the-windows-put-aside.md`: the panel is a viewport control and never
a child of the canvas, which `crates/alo-canvas` already enforces by refusing a
viewport surface the camera; each preview names one window rather than one
application; and seven of the owner's ten behaviour rules are decidable in a
crate with no display. Which vocabulary it speaks for the plane is settled in
`docs/design/one-plane-two-vocabularies.md`.

**Compacting is the other half of the same promise and is equally unbuilt.** A
later entry must not tick this one when only the panel exists: the line names two
things and says *the person picks*, so half of it answered is a promise owed.

### Full screen

**Shown by:** `crates/alo-shell/tests/a_window_that_fills_the_screen/mod.rs`,
`crates/alo-shell/tests/support/wm_capabilities.rs`

**Still owed:** **somebody seeing it on a display.** The mechanism landed on
2026-10-01 in `7c236e0` and the entry that stood here — *all of it*, `Mode` has
no case that fills the screen, no `fullscreen_request`, a client answered with
silence — was true until that morning and is kept above in this sentence rather
than deleted, because what a ledger entry said is how a reader judges what it
says now.

What is built and runs: `window_mode::Mode::FillingTheScreen`;
`fullscreen_request` and `unfullscreen_request` on the `XdgShellHandler`;
`Fullscreen` advertised in `wm_capabilities`; and the shell answering
`a_window_is_filling_the_screen()`, which `desktop_raster` asks before drawing
the Dock and the panel, so both give way whatever the person chose about the
Dock hiding. Four tests in
`crates/alo-shell/tests/a_window_that_fills_the_screen/mod.rs`, three of them
driving a real client through the protocol rather than the shell's own entry,
because the request is the thing that was missing: a client asking is answered;
`Fullscreen` is sent without `Maximized`; leaving returns to normal rather than
to maximised; and the shell knows, through the trusted entry.

**Why that is not a tick.** Every one of those runs under a nested compositor.
A nested session shows that a client talks to us and cannot show that a person
sees a window take a real display — the rule this plan holds itself to. The
owed half is the same owed half as tasks 38 and 39: a machine with a real
display.

### Where the Dock goes

**Still owed:** **all of it, and the code was taken out rather than never
written.** `alo-dock` has no edge at all: `crates/alo-dock/src/changes.rs` says
so in its own words — *there is no `edge` field and no `displays` field* — and
its tests pin the absence, one asserting that a `dock.toml` naming an edge loads
with the edge **ignored** rather than refused, so that a file written by an
earlier release still reads. ADR 0076's remedy removed the field on 2026-09-29;
the owner reversed the decision on 2026-09-30 and the field has not come back.

**So the honest state is a promise restored in the record and removed from the
code**, which is the reverse of the usual gap and worth naming as such: the
tests that would change when this is built are the two in `changes.rs` that
currently assert an edge is ignored. A reader checking whether this is built
will find tests passing, and they pass *because* it is not.

The default is not the question. Bottom remains the default and is what the
layout does today; what is owed is that a person may choose left, right or top,
and that the band works in both orientations rather than being a horizontal bar
turned sideways.

**Where the work is:** task 11 of `docs/autonomy/the-smallest-canvas-worth-showing.md`,
written on 2026-10-01 by the owner's direction and **Open, blocked on design**.
It carries the four edges, orientation-aware layout, labels, overflow, the
reveal paths and shared-edge collision, and it names the five states each of the
left, right and top designs needs. *This entry cited only the ADR until that
task existed, because a plan carried no task for the promise — which the owner
then directed into this plan.*

### Reaching the Dock over a full-screen window

**Still owed:** the whole interaction, because the situation it happens in
cannot be created — the entry above is why. The rules for it exist and are
tested: `alo-dock`'s revealing module holds *which regions keep a surface open*,
with the pointer, the keyboard, a drag and an open menu each holding it by
itself, and twelve tests including one that reads its own source to hold that
nothing in it consults a clock. **They are deliberately not cited above.**
Nothing calls them, on any machine, so they are reusable groundwork and not
evidence that a person can reach the Dock over a full-screen window — which is
the distinction this ledger exists to keep. Where the work is: task 6 of
`docs/autonomy/putting-a-window-aside.md`.

### ★ **The strip above every setup screen reads `alo OS`.**

**Shown by:** `crates/alo-setting-up/src/the_strip.rs`, where the text is a
constant and three tests hold it: that it is the product's name, that it holds
no digit anywhere, and that it is one name rather than the two halves the six
old labels used. Putting the old counter back fails all three.

**Still owed:** nothing draws it. The strip is a value with no surface reading
it, so what is shown is that the rule exists and cannot be edited quietly — not
that a person booting this machine sees it. The drawing is the compositor
lane's. Where the work is: the first-start screens in
`docs/design/the-first-start.md`, which carry the measurements a surface needs.

### **The running canvas has no strip and must not gain one.**

**Shown by:** `crates/alo-setting-up/src/the_strip.rs`. Its screen type has two
variants and only the setup sheet returns a strip; giving the canvas one fails
the test named for it. The distinction is measured rather than asserted — 8 of
the design file's 77 frames have no strip and all 8 are canvas states, counted
off the refreshed snapshot.

**Still owed:** the same thing the entry above owes. No compositor consults
that type, so nothing on a running machine can get this right or wrong yet.

### ★ **Setup shows no overall progress counter.**

**Shown by:** `crates/alo-setting-up/src/the_strip.rs`, whose counting test
refuses a digit anywhere in the strip — deliberately stricter than the
decision, so that neither the old counter nor a version number can arrive
without arguing with it.

**Still owed:** the rule covers the strip and not the rest of a screen. Nothing
in this repository would catch a counter added somewhere else on a setup sheet,
because there is no setup sheet to add one to. **That is a real limit of this
evidence** and not a formality: the promise is about a screen, and what is
tested is a string.

### **Real installation progress stays beside its own operation.**

**Still owed:** all of it. The progress a person waits on belongs to the
installer, which is another lane's crate, and no screen exists on either side
of that boundary. What exists is the design record: the measured track, its
explanation and the two error-and-recovery screens are named with their frame
ids in `docs/design/the-first-start.md`, so that whoever removes the counter
cannot remove these by accident. **No test holds this**, and recording it as
shown because the counter rule is tested would be the mistake this ledger
exists to prevent — the two are opposite halves of one decision and only one
of them is checked. Waiting on
`docs/decisions/0094-the-strip-above-a-setup-screen-is-the-products-name.md`,
whose fifth clause is this promise.

### ★ **`Optional` sits immediately above the heading, on ten screens.**

**Still owed:** all of it. The word's position and text were read individually
off all ten screens in the design file on 2026-10-07 and written into
`docs/design/the-first-start.md` with the frame ids, which is evidence about
the design rather than about this repository. **Nothing here implements an
optional screen**, so there is no eyebrow to place and no test that could fail.
Waiting on
`docs/decisions/0094-the-strip-above-a-setup-screen-is-the-products-name.md`,
which records where the word went and why it moved.

### **Each optional screen keeps its own way to decline.**

**Still owed:** all of it, and it is the half most likely to be lost. Six
different words decline across the ten screens, and one of them — the `Not now`
on file import — is 160 wide where the three new ones are 184. A build that
tidied those into one action or one width would be reasonable-looking and
wrong. The list is in `docs/design/the-first-start.md`; nothing enforces it.
Waiting on
`docs/decisions/0094-the-strip-above-a-setup-screen-is-the-products-name.md`,
which holds the rule against normalising them.

### ★ **One selection border: 2 logical pixels of navy, plus the word.**

**Shown by:** `crates/alo-appearance/src/selecting.rs`. The two edge widths are
constants, the marking colour is navy and no other colour is reachable through
it, and focus and selection are held independently so that committing answers
from the chosen half alone. Flattening the edge to 1px, marking a selection in
teal and making focus commit a choice each fail the test named for them.

**Still owed:** nothing draws a card, so the measure is not yet a border on a
screen. The word `Selected` that must accompany the colour is the design's and
is not in this module — a surface that drew the edge and omitted the word would
satisfy every test here and break the promise. **Named rather than left
implicit**, because colour alone is the accessibility failure this rule exists
to prevent.

### A desk comes back when you arrive at it

**Shown by:** nothing yet, and more is built than for any other line here.

**Still owed:** the caller and the surface. `alo-displays` already keeps a
separate arrangement per **set** of screens and
`crates/alo-displays/tests/three_sets_of_screens_remembered_apart.rs` proves it,
so the remembering is real and tested. What is owed is that nothing in production
builds a `Screens` at all, so the arrangement is never reached, and nothing shows
a person which desk they are at. Reached this release on 2026-10-08 by the
owner's direction. Where the work is: **task 17 of `docs/autonomy/more-than-one-display-plan.md`**, which depends on **task 11 of
`docs/autonomy/more-than-one-display-plan.md`**.

### You name your screens, and the name is yours

**Shown by:** nothing yet.

**Still owed:** all of it. There is nowhere for a name to live: a screen's name
is a person's setting, the shell shows and never measures, and the seam that
carries a person's display settings to the shell does not exist yet. Settled by
[ADR 0098](../decisions/0098-a-screen-carries-the-name-its-person-gave-it.md).
Reached this release on 2026-10-08 by the owner's direction. Where the work is:
**task 18 of `docs/autonomy/more-than-one-display-plan.md`**, which depends on **task
10 of `docs/autonomy/more-than-one-display-plan.md`** for the seam.

### Screens are named, not numbered, by default

**Shown by:** nothing yet.

**Still owed:** all of it, and one part of it is not yet decidable. A default
name wants the screen's own make and model, which `OutputMetadata` carries, and
which side it is on, which needs the arrangement. **A screen that says nothing
about itself has no make to use**, and `alo-displays` already has a sentence for
being remembered by its socket instead. Reached this release on 2026-10-08 by the
owner's direction. Where the work is: **task 18 of `docs/autonomy/more-than-one-display-plan.md`**.

### alo OS says which desk it thinks you are at, once

**Shown by:** nothing yet. The sentences themselves are built, translated and
tested inside `alo-displays`, which is why this entry is about the showing
rather than the words.

**Still owed:** anything that shows one. `Screens::notes()` returns them and
nothing reads it, because no production code builds a `Screens`. `notes.rs`'s own
rule — *there is no note for the ordinary morning* — is written and must survive
the wiring. Reached this release on 2026-10-08 by the owner's direction. Where
the work is: **task 19 of `docs/autonomy/more-than-one-display-plan.md`**.

### Show me where

**Shown by:** nothing yet.

**Still owed:** all of it, and it is the only line here with no engine behind it.
`alo-displays` already *guesses* — `Note::NewHere` says a new screen was put
beside the others — so the guess half exists and the correction half does not.
The card, the push gesture, and writing the result back as the person's
arrangement are all new. Reached this release on 2026-10-08 by the owner's
direction. Where the work is: **task 20 of `docs/autonomy/more-than-one-display-plan.md`**.

### Unplug and nothing is lost; plug back in and nothing moved

**Shown by:** nothing yet.

**Still owed:** the showing and the undo. `Attached::unplugged`,
`Attached::plugged_in` and `Screens::windows_away` are built and tested, so
**which** windows came home is answerable; nothing tells the person, nothing
offers an undo, and nothing puts them back on return. Reached this release on
2026-10-08 by the owner's direction. Where the work is: **task 21 of `docs/autonomy/more-than-one-display-plan.md`**.

### How large things are, per screen, in words

**Shown by:** nothing yet, and it must not be offered yet.

**Still owed:** all of it, and **it is deliberately blocked rather than merely
unstarted.** A display's size reaches `desktop_raster` — alo's own dock, panel
and status area — and nothing applies it to an application's window, measured
2026-10-08 at `crates/alo-shell/src/presentation.rs:341`, which advertises every
output to every client as one pixel per pixel. Offering *smaller, just right,
bigger* before that is fixed would give a person a double-size dock beside
normal-size windows and call it a setting. `alo-displays` already has the
sentence for a size it cannot draw. Reached this release on 2026-10-08 by the
owner's direction. Where the work is: **task 16 of `docs/autonomy/more-than-one-display-plan.md`** for the defect, then **task 10** for
the control, in that order and not the other.


## What this audit found

**The audit in figures: 71 promises, 2 shown whole, 51 shown in part, 18 with no
evidence at all.**

*This line is the ledger's own count of itself and it is checked.*
`crates/alo-reconciling/tests/every_v0_01_promise_is_reconciled.rs` reads these
four numbers and compares each against what the reconciliation computes, so a
promise entering or leaving a release fails the audit until this sentence is
brought with it. **Whoever changes which promises are in v0.01 changes this
line in the same commit.**

**It was not checked until 2026-10-07, and it had drifted.** The sentence said
*53 promises, 2 shown whole, 41 shown in part, 10 with no evidence at all* while
the reconciliation computed **63, 2, 46 and 15** — three of the four numbers
wrong, the first by ten promises. The claim above was true of the count of
wholly owed promises, which was asserted and is what kept that one honest; the
other three were prose that read like measurements. **A description of a
document that the document cannot contradict is the fault this crate was written
to end**, and it had made its way into this crate's own ledger. The test now
reads the sentence and fails on all four.

*Written in digits on 2026-09-30 because the figure had nowhere to live. The
current count used to exist only as a literal in that test file, while the
paragraph below — the ledger's own account of itself — still said six, and the
correction under it said five. The test's failure message claimed to be quoting
this document and was quoting nothing: three appeared nowhere here. Two lanes
had to remember a number that lived in neither of the places a reader would
look, and getting it wrong failed the workspace's tests for all three.*

***And the first run of the new check found the ledger already wrong by one.***
*It was written as 41 promises and 36 shown in part, from the paragraph below;
the audit counts **42 and 37**. A promise had entered v0.01 and been reconciled
as shown in part with nothing recording it. The old literal could not have
caught it — it asserted only the number with no evidence, which had not moved,
so three of the four figures were unchecked and one of them had already
drifted.*

The paragraph below is the audit as first written, kept as history. Where its
numbers differ from the line above, the line above is the current one and the
notes that follow record what moved.

Forty-one promises. **Two are shown with nothing owed on them**, thirty-three
are shown in part with the rest named above, and **six have no evidence at
all**. The six are the finding, and they are not one kind of thing:

***And seven more arrived the same day, when the owner put the full canvas
experience into v0.01.*** *Three are shown in part — the closed canvas plan built
one Place and its tests are named above — and **four name nothing**: the World, a
frame arriving the shape its work is, a frame dragged between Places, and every
application living in a Place. Each names the task that is its increment in*
`docs/autonomy/the-canvas-and-its-places.md`*. That is 44 promises to 51 and five
with no evidence to nine, and this line moved with them because the figures above
are checked.*

- **All seven that name nothing**: *copy, cut and paste*, *the
  GPU works on first boot*, *it never nags*, and the four the canvas brought on
  2026-09-30 — *the World*, *a frame arrives the shape its work is*, *a frame
  dragged out of one Place and into another*, and *every application lives in a
  Place*. Each is a v0.01 promise with no crate, no test and no report — the same
  kind the roadmap's audit found six of, one at a time, over seven readings.
  **The difference between the first three and the last four is that the four
  name the task that is their increment** in
  `docs/autonomy/the-canvas-and-its-places.md`, and the three point nowhere.
- **One is a standing rule nothing checks**: *anything an agent verb can do, a
  person can do by hand*. It is the check on every verb anybody proposes, and no
  test walks the verbs asking it.
- **One cannot get a line without a decision**: *the agents point at the local
  model by default*. ADR 0016 refuses a default that nobody chose, and nothing
  here may narrow the promise to fit or contradict the ADR.
- **One is scheduled and needs hardware**: *boots on one certified machine*,
  which is task 12 of the delivery plan.

They are written down before anything else is done about them, which is this
task's acceptance. What follows belongs to whoever owns the scope: four of them
are work nobody has scheduled, one is a question only the owner can answer, and
one is waiting for a machine.

### One of the six closed, 2026-09-11

*Anything an agent verb can do, a person can do by hand* now has a line, so the
count above stands at **five with no evidence at all** and the standing rule is
no longer one of them. `docs/by-hand.md` answers for each of the ten verbs alo OS
declares and `crates/alo-by-hand` holds the document to them: a verb added with
nothing said about it fails the gate in the change that adds it. The entry above
carries what the check cannot reach, and it found one thing worth reading twice —
**six of the ten verbs ship at v0.01 and their plain way arrives at v0.5.** Task
14 of `docs/autonomy/the-executable-plan.md` and
`docs/autonomy/updates/every-verbs-by-hand-answer.md` are the work; the paragraphs
above are left as the audit wrote them, because a finding rewritten by whoever
closed it is a finding nobody can check.

### A second of the six closed, 2026-09-11

*And it never nags* now has a line, so the count stands at **four with no
evidence at all**, and it is the last of the six this lane could close without a
screen, a decision or a machine. `crates/alo-telling` is the memory nothing had:
the same unavailability told once is told once, and telling it again takes the
source changing, the reason changing, or the person asking again themselves.
Task 15 of `docs/autonomy/the-executable-plan.md` and
`docs/autonomy/updates/a-machine-that-cannot-reach-a-model-says-so-once.md` are
the work.

What is left of the six is what nobody here can close alone: *copy, cut and
paste* and *the GPU works on first boot* are unscheduled work, *the agents point
at the local model by default* waits on a decision ADR 0016 will not let this
lane make, and *boots on one certified machine* waits on a machine. The
paragraphs above are left as the audit wrote them, for the reason the first
closure gave: a finding rewritten by whoever closed it is a finding nobody can
check.

### The third is sent to a decision rather than closed, 2026-09-11

**The count stands at four**, deliberately. *The agents point at the local model
by default* now has
`docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md` behind
it — the options, a recommendation, and what each would cost `alo-choosing`,
`alo-image` and the setup flow — and **a proposed decision is not evidence that
anything was built**, which is what this crate refuses an ADR for in the first
place. The entry above says what is owed under it, including the fact the wording
had hidden from every reading until now: the image carries no model runtime and
no weights, so the promise is unbuilt in `image/` rather than blocked on a
settings key. Task 16 of `docs/autonomy/the-executable-plan.md` and
`docs/autonomy/updates/a-default-nobody-chose.md` are the work. The entry closes
when a machine arrives with a model on it, and not before.

### The four were read one at a time, and one of them was mis-sorted, 2026-09-11

**The count stays at four.** Nothing was closed here and nothing was ticked; what
changed is that each of the four now carries the reading behind its verdict,
under the promise it is about, so the next person inherits an argument rather
than a sorting. Task 19 of `docs/autonomy/the-executable-plan.md` is the work and
`docs/autonomy/updates/the-four-promises-with-no-evidence.md` is the report.

**One of the four was in the wrong pile.** *Copy, cut and paste* was sorted into
*needs a machine* by the audit that found it, in the same pass that found five
other things, and the sorting was carried unexamined ever since. A clipboard is a
protocol before it is a surface — an owner, the types it offers, and a transfer
somebody asks for — and every refusal in it is decidable with no screen, no
machine and no decision. It is now task 20 of
`docs/autonomy/the-executable-plan.md`, with its own acceptance. The other three
are genuinely waiting: *the GPU works on first boot* on a machine with a card and
on an image that carries something for it to accelerate, *the agents point at the
local model by default* on the owner accepting a proposed decision, and *boots on
one certified machine* on both a machine and a decision.

**And the ledger is now held to saying where the work is.** A promise with no
evidence at all must name the decision it waits on or the task that is the
increment — with the plan beside the number, because three plans in this
repository number their tasks from one — and the pointer is followed to a file on
the disk. `crates/alo-reconciling/src/waiting.rs` is the rule and
`a_promise_with_no_evidence_and_nowhere_to_go_is_refused` is it refusing. Being
owed is not the finding; being owed and pointing nowhere is, because that is the
entry whose reasoning is derived again from scratch every time somebody opens
this file — which is the seven-times-over reading this ledger exists to end,
arriving from the other end.

### The mis-sorted one is closed, and three are left, 2026-09-11

*Copy, cut and paste* now has a line, so the count stands at **three with no
evidence at all** — and it is the one the previous reading found in the wrong
pile rather than one anybody had scheduled. `crates/alo-clipboard` is the
selection this repository never had: an owner says which forms it can give,
somebody asks for one of them, and the broker holds the offer and the way back
to the owner and holds nothing else. Task 20 of
`docs/autonomy/the-executable-plan.md` and
`docs/autonomy/updates/the-clipboard-before-there-is-anything-to-draw.md` are the
work.

**What the increment does not close is written into the entry above**, so nobody
reads a crate as the promise: no two applications on a machine have moved
anything through it, because the `wl_data_device` wiring is `crates/alo-shell`'s
and is the desktop lane's.

The three that are left are the three the previous reading said were genuinely
waiting, and nothing about them has changed: *the GPU works on first boot* waits
on a machine with a card **and** on an image carrying something to accelerate,
*the agents point at the local model by default* waits on the owner accepting
`docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md`, and
*boots on one certified machine* waits on both a machine and
`docs/decisions/0024-what-a-person-signs-in-at.md`. **None of the three can be
closed by this lane**, which is a fact about scope rather than about effort: two
need hardware nobody here can plug in, and the third needs a decision only the
owner can make.
