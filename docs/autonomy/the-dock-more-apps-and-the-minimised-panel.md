# The Dock, More apps and the minimised panel

**What this is.** The owner ruled on 2026-10-11 that these are **three distinct,
connected surfaces**, and asked for the design contract, the feature
documentation and this plan to be updated *first*, then for the production
integration to be completed. The ruling is
[ADR 0103](../decisions/0103-the-dock-more-apps-and-the-minimised-panel-are-three-connected-surfaces.md);
it supersedes two rules by name and is not restated here.

**The one sentence it has to earn:** a person can see which applications they
have, reach any window they put away, and find the Dock exactly as they left it
after a restart.

**What it rests on.** `docs/design/the-alo-dock.md` has the membership rule and
what a click does. `docs/design/the-windows-put-aside.md` has the panel's
measurements and what a card shows.
`docs/design/the-docks-visual-specification.md` has the Dock's own geometry and
says plainly which parts of the popover are **not** settled. None of them is
restated here. **This file is the order, and what each task has to earn.**

## The honest starting point, measured rather than recalled

Drawing exists and behaviour does not. That asymmetry is the whole plan.

| what | state on 2026-10-11 |
|---|---|
| the Dock's band, icons, indicators, surface, four edges | **built and drawn** |
| the *More apps* control, drawn permanently and last | **built and drawn** |
| a list-shaped popover with a heading and rows | **built and drawn**, and is a staging post |
| its placement clear of the panel's column | **built and tested**, ADR 0102 |
| the panel's three presentations | **built and drawn** |
| `alo_dock::clicking` — what a click means | **built, and nothing calls it** |
| `alo_dock::pinning` — what a fresh machine pins | **built, and nothing produces its `Discovery`** |
| `Holding::showing` — who is on the bar | **built with the new rule**, and reached in production through `Server::what_the_dock_holds` |
| the Dock seeing minimised windows | **built** — the builder reads `minimized_surfaces` as well as `mapped_surfaces` |
| pressing an icon | **built** for one window, open or put aside; a picker and a launcher are named variants that do nothing |
| a window picker, a search field, a grid | **do not exist in any form** |
| any keyboard route into any of it | **does not exist** |
| Escape dismissing anything of the Dock's | **does not exist** |

**The repeated fault this plan exists to stop** is in the third column of rows
six to eight: a complete, tested mechanism with no production caller. It has
happened to `fit`, `clicking`, `menu`, `Places::at` and `pinning` in this
repository already. **So no task below is done when its crate is green.** Each
names the production call site that must reach it.

## How a task here is tested, decided once

Three things, and a task that cannot say all three is not ready to start:

1. **The rule, in the crate that owns it**, with the refusal path tested as
   carefully as the happy path.
2. **The production path**, through `alo-shell`, asserting the thing a person
   would see — never through the helper the rule lives in. A test that exercises
   the rule and not the caller is the fault above wearing a green tick.
3. **A picture**, from `desktop_check`, for anything that changes what is drawn.

And each new guard is **mutation-tested**: plant the fault it describes and
watch it fail with the numbers printed.

## Tasks

### 1. Who is on the Dock

**Status: done, 2026-10-11.** The rule is `OnTheDock::belongs_on_the_dock` —
pinned by the person, or more windows than are put aside — and eleven tests in
`holding_membership_tests.rs` walk all seven rows of the owner's table in one
test rather than naming six of them in six names. `Holding::showing` is
`in_the_overview` with the rule applied by `retain`, so the bar and the overview
cannot drift about which applications exist. The shell draws from it.

**What has never run:** the pinned half. Every caller passes `Holding::nothing()`
and no file in this repository saves a pinned list, so the clause *pinned by the
person* has been exercised only in unit tests. It becomes true with task 4.

`Holding::showing` learns the membership table. It already counts an
application's windows and how many are put aside, so the rule is readable off
data it has: an unpinned application with `windows == put_aside` is not shown.

**Earns:** the table in `the-alo-dock.md`, every row; that *not minimised* is
about state and never about the camera; one icon per application; pinned
positions kept on opening; **focusing never reorders**.

**Call site: `Server::what_the_dock_holds`, and it was already there.**

Two sentences stood here before this one and both were wrong, in opposite
directions, which is worth keeping because the second was wrong for a reason
that costs this repository time repeatedly.

The first said the call site *already existed* — written without checking. The
second said *every caller of `Holding::showing` in this repository is a test*
and that *no application has ever appeared on a real Dock*. That was checked,
and the check was `grep … | head -20`: twenty lines of `alo-dock`'s own tests
came back and the list was cut before `crates/alo-shell/src/dock_windows.rs:140`
ever printed. **A truncated search and an empty one look the same.** The
`on_the_dock: &[]` at `crates/alo-desktop/src/main.rs:618` is real and is
overridden on the way to the frame by `what_only_the_server_knows`, which is
exactly what a reader checking only that literal would conclude wrongly.

What *was* true is narrower and is task 1b:

- **1a, the rule** — `belongs_on_the_dock` and `showing` applying it to the
  whole Dock. Done, mutation-tested four ways.
- **1b, what the builder could see** — `Server::the_windows_the_dock_sees` read
  `mapped_surfaces` only, and marked every window `OnTheCanvas`. Its own header
  said so: *minimised windows are not here … the Dock learns about those through
  `alo-put-aside`, which is a different road and not this one.* A true
  description of the code, and it made three answers wrong while looking right:
  `how_many_put_aside` was permanently zero on every machine, so a screen reader
  was told about no window a person had put away; an application whose windows
  were all minimised was not *open*, so it drew no running indicator, which the
  ruling requires; and the membership rule could not fire, because such an
  application was already absent **by exclusion rather than by decision** — the
  right answer for the wrong reason, which stops being the right answer the
  moment the rule changes. Done: the builder reads `minimized_surfaces` too.

**Both halves are now verified against a real application**, not a fixture —
see the evidence section below.

**Watch for:** the ordering clause is the easy one to leave untested, because a
Dock drawn once looks right whatever the order rule is. It needs two draws and a
focus between them — `focusing_an_application_does_not_move_its_icon` is that
test, and it exists because `Windows` keeps two orders and `showing` must walk
the right one.

### 2. Pressing an application

**Status: done, 2026-10-12.** A press focuses one window, brings a put-aside one
back and then focuses it, or opens the window list where there are several.
Seven tests in `a_press_on_the_dock_tests.rs` hold the table, and the decision
is a pure function so *which slot a press reached* and *what that slot does* are
both checkable without a display.

**Launching landed on 2026-10-12** (#635). A press on an idle icon records an
ask, and `an_application_is_started` sends it off the input path: the bus starts
the application, so nothing in this repository reads an `Exec` line or runs a
command. A machine with no session bus reports every ask as not started rather
than claiming one worked. **What is still owed is a machine**: no application
has been started on real hardware, because that needs a session bus and a
D-Bus-activatable application and the gate VM has neither.

**One thing is still not done, and it is not a guess.** The canvas does not
travel: `the_windows_the_dock_sees` gives every window a one-by-one patch at the
origin, so `TheView::already_shows` compares placeholders, and travelling on
that would be motion decided by a placeholder. What is owed is the window's real
patch reaching that builder.

`WhatAPressOnTheDockDoes::GoTo` reaches `alo_dock::clicking` and something
happens. One window focuses or restores-then-focuses; nothing open launches;
several open the picker, which task 3 builds.

**Done for the cases a compositor can carry out, and each of the rest is a
named variant rather than a silence.** `what_a_press_on_an_icon_does` turns
`alo_dock::clicking`'s answer into `WhatAnIconDoes`:

| answer | state |
|---|---|
| one window on the canvas | **focused** |
| one window put aside | **brought back, then focused** |
| several windows | `OpenThePicker` — task 3, nothing drawn |
| nothing open | `Launch` — nothing in this workspace can start an application |

Three faults were designed out rather than left to a caller. **Bringing a
window back is part of the answer**, not the applier's idea: focusing a hidden
window puts the keyboard somewhere a person cannot see, and the variant names
the order. **The id is looked up at the press**, never remembered from the draw,
because ids are the window list's own indices and a window closing renumbers the
rest. And `the_windows_and_what_they_are` is one iteration that both the builder
and the lookup read, so a lookup cannot come to disagree with the builder and
focus **somebody else's window**.

**What is deliberately not carried across is `travels`.** The Dock's window
builder gives every window a one-by-one patch at the origin because it cannot
see canvas geometry, so `TheView::already_shows` compares placeholders and
`BringForward` against `TravelTo` is decided by one. Travelling on that would be
motion chosen at random, which is worse than not travelling. What is owed is
each window's real patch reaching that builder.

**The press itself is read and not measured, and the split exists to say so.**
`Server::the_docks_slots_were_drawn` is called from `crate::direct_desktop` and
nowhere else, so the slots a press is classified against only exist on the lane
that needs a login seat and real devices; the nested lane draws a Dock and
cannot press it, because `submit_with_desktop` is handed no `Server` to record
into. So the decision is a pure function over `alo_dock::Windows`, tested in six
cases and mutation-tested three ways, and the part that needs a machine is three
lines underneath it. The same shape `crate::a_key_reaches_settings` describes
for its own lane.

### 3. The window picker

**Status: done, 2026-10-11.** `alo_dock::picking::ThePicker` is the model,
the popover draws it, minimised rows are marked as minimised, and choosing one
restores and focuses that exact window. Reached from an icon with several windows
and from the overview, and Escape goes back to the overview rather than out.

The list scrolls, which it did not until 2026-10-11: every row was drawn at its
content offset and the remainder clipped away, so twenty windows showed about
three and the rest were unreachable.

A surface of its own: every window of one application, **minimised ones marked
as minimised**, choosing one focuses or restores it. Reached from the Dock and,
by task 5, from *More apps*.

**Earns:** the third row of the click table, and *usable after the application
is selected* from inside the popover.

### 4. The default pins reach a screen

**Status: open, not started — blocked on another lane.** Nothing in this
repository can answer *which applications are installed*, so Files, Browser, Apps
and Settings are a specification and not a list of identifiers anything can look
up. Named as a dependency on 2026-10-11 rather than discovered on the day — see
*What is owed to another lane, and from when*.

**It is the most expensive block on this plan.** Task 8 depends on it, the pinned
half of task 1 has never run because of it, and task 2's launching needs the same
answer.

Something produces `alo_dock::pinning::Discovery`. Files, Browser, Apps,
Settings, resolved against installed applications, written **once** and only
after discovery **succeeded**. An emptied list is never refilled.

**Depends on** the host's application-discovery integration, which is another
lane's and is the one hard dependency in this plan. `ThePins::what_to_pin`
already distinguishes `Completed`, `StillLooking` and `Failed` and already
refuses to write anything down for the last two — that half is finished and
tested.

### 5. More apps becomes an overview

**Status: done, except the search field — 2026-10-11.** The overview carries
every pinned application and every open one, including those whose windows are
all minimised and which are therefore not on the bar — the one case the two lists
are guaranteed to differ on. Fifteen tests in `more_apps_tests.rs`. Its bounds are
the owner's and hold by construction: a compile-time assertion ties the fixed
parts plus the list's cap to 536, so a popover that grew past it would not
compile. Sixty applications scroll inside the same viewport.

**The search field takes no typing.** It is drawn, it is a well with a prompt, and
nothing filters. So a person with sixty applications scrolls rather than searches,
which is the slower half of the promise and the half the owner named first.

Contents first, because they are decidable without a pixel: pinned in saved
order, then every open application **including minimised-only ones**, each once,
installed-but-not-open excluded.

**Then the grid**, and **not before the Figma file carries it.** The owner's
note is explicit that the list-style reference is historical input. Building a
grid off a list frame would be measuring the wrong thing carefully.

**Earns:** the contents rule, which is checkable today; the search field,
filtering with stable order and a clear empty result; the bounded height that
scrolls rather than stretching — **the list currently draws 644 of 768 pixels
tall, which is the sidebar the ruling forbids.**

### 6. Dismissal and the keyboard

**Status: done, except three things — 2026-10-11.** Escape, the four arrows,
Enter and the number pad's Enter are taken, each by its evdev code, and only while
a surface is open and only on a press. Focus lives inside the open-surface state,
so there is no value in which something is focused and nothing is open. The
focused item is ringed with the measurement `the-first-start.md` read off the
design, and it scrolls into view in the grid and in the window list.

**Not done, and each named rather than left in the sentence above:** the search
field takes no typing (task 5's half); dismissing does not return focus to the
opener, because the bar's own slots hold no keyboard focus for it to return to;
and the menu key does not open the actions, because `alo_dock::menu` is a model
nothing opens.

Escape dismisses a picker, then the popover. Outside press already works. The
control closes it. Keyboard reaches the search field and the grid, focus scrolls
into view, and dismissing without choosing returns focus to the opener.

**The hard part is not Escape, it is taking Escape without stealing it.** The
seat must give the key to the Dock only while a Dock surface is open and to the
client otherwise. `crate::a_key_reaches_settings` is the precedent and says
plainly which of its lines no test in the crate can drive.

**Done, except the search field — 2026-10-11.** Escape, the four arrows, Enter
and the number pad's Enter are taken, each by its evdev code, and only while a
surface is open and only on a press. Focus is held inside the open-surface state
rather than beside it, so there is no state in which something is focused and
nothing is open. The focused item is ringed with the measurement
`docs/design/the-first-start.md` read off the design, and it scrolls into view in
the grid and in the window list.

Three things this task named are **not** done, and each is written here rather
than left in the sentence above:

1. **The search field takes no typing.** Task 5's remaining half.
2. **Dismissing does not return focus to the opener.** The Dock's own slots hold
   no keyboard focus at all yet, so there is no opener to return it to — this
   waits on the bar itself becoming keyboard-reachable, which is the design's
   *everything the Dock shows on hover is reachable by keyboard focus*.
3. **The menu key does not open the actions.** `alo_dock::menu` is the model and
   nothing opens it.

**And one gap found while doing it, which is larger than this task.** Neither
surface takes a **pointer** press: `crate::where_the_dock_is::TheDocksPlaces` is
the hit test and it covers the band, not the popover above it. So pressing a
cell in *More apps* or a row in a window list does nothing, and the keyboard is
today the only way to choose from either. The picker's list also could not
scroll at all — every row was drawn at its content offset and the rest clipped
away — which was fixed here because *focus scrolls into view* cannot be true
without it. The pointer's hit test is the next task on this plan.

### 7. Minimising recalculates eligibility

**Status: done, 2026-10-11.** `crate::dock_windows` builds the Dock's window
list from `mapped_surfaces` **chained with** `minimized_surfaces`, so minimising
one window of an application recalculates that application's eligibility on the
next frame without anything having to remember to.

That chaining is the whole task, and it was missing: until 2026-10-11 the builder
read only the mapped half, so `how_many_put_aside` was permanently zero on every
machine and a screen reader was told no window was put aside whatever a person
had done.

The link from `set_window_minimized` to the Dock. The compositor half is built
and contracted; what is missing is that anything recomputes the bar.

**Earns:** the four worked examples in §4 of the ruling, which are the
acceptance test for tasks 1 and 7 together.

### 8. It survives a restart

**Status: open, not started — depends on task 4.** There is no pinned list to
persist. When one exists, what has to survive is the pins, their order, and a
list a person deliberately emptied — that last one being the clause a naive
implementation breaks, because an empty list and no list look alike on disk.

Pins, their order, and a deliberately empty list, across a restart and an
update.

### 9. A press reaches inside the two surfaces

**Status: green, not landed — 2026-10-12.** Nine of nine on
`25d760615e78bd84df5a4d7eda4ab8b41ff33eb7`, awaiting the merge queue. Both
surfaces take a
pointer press, which neither did on 2026-10-11: `TheDocksPlaces` covers the band
and nothing covered the popover above it, so for one day the keyboard was the only
way to choose from either.

**Added 2026-10-11, while task 6 was being built**, which is why it is last and
not third: nothing had noticed. `crate::where_the_dock_is::TheDocksPlaces` is
the Dock's hit test and it classifies a point against the **band** — the slots,
the overflow control, the divider. Nothing classifies a point against the
popover the control opens. So a cell in *More apps* and a row in a window list
are drawn, are the things the owner's §2 and §3 say a person chooses from, and
do nothing when pressed.

What it needs: the popover's rectangle recorded at the draw, as the band's slots
already are; a reverse lookup from a point to an index, beside
`TheOverview::where_a_cell_sits` and `ThePickersPanel::where_a_row_sits` in
`alo-dock` where both can be tested without a display; and the act, which is
already written — `Server::the_keyboard_takes_what_is_focused` does exactly what
a press must do, and a press should call the same thing rather than a second copy
of it.

**Earns:** the parts of §8's evidence that say *click*. Today every one of them
would have to be read as *press a key*.

**Written 2026-10-12, not yet built.** All three parts exist and none has
compiled: the machine runs one Cargo operation at a time and another branch's
gate has held the slot since this was written. So this is a statement about what
is in the tree and not a claim that it works.

What it does, and the one decision in it worth disagreeing with:

- **`alo-dock` answers the point.** `TheOverview::which_cell_is_at` and
  `ThePickersPanel::which_row_is_at`, both taking **viewport** coordinates and
  adding their own scroll. That last part is deliberate and is
  `ThePicker::which_window`'s own reason: a caller that had to add the scroll
  could forget to, and the fault would be silent and only for somebody with
  enough applications to scroll. Both refuse the gaps between items, the group
  headings and the empty end of a short row — `alo_dock::places`' rule for the
  bar, held for the surface: *a gap that reached its nearest neighbour would
  mean a person aiming at the space between two applications opening one of
  them.*
- **The draw records what a press needs**, in
  `crate::where_the_docks_surface_is::TheSurfaceAsDrawn`: the popover, the
  list's viewport inside it, and which face with its layout. The **layout** and
  not a list of rectangles, because the layout already answers *which item is at
  this point* and answers it at the scroll it was drawn at; a list recorded here
  would be a second answer that could disagree about a scrolled list.
- **One performer for both ways in.** Enter and a press both call
  `Server::choose_from_the_open_surface`. `crate::window_control_input` records
  what the other choice costs: migrating only the keyboard once left the button
  broken while a keyboard test passed — *a fix down the path nobody uses*.
- **The surface is asked before the band**, because it is drawn in front of it.
  A press on a cell that also lands over the bar must open that application
  rather than whatever slot is underneath.
- **Dismissal is not decided here.** A press outside answers *not mine*, and
  `crate::pointer` closes whatever is open on any press the Dock does not take —
  as it has since before this surface could be pressed. The first draft of this
  closed it here too, which was one rule in two places.

The tests are **round trips**: every cell and row is fed back the point the
layout drew it at and must answer its own index, walked exactly as the draw
walks it — content offset less the scroll — because an asymmetry between the two
directions is the fault a round trip exists to find. Plus the gaps, the
headings, the short last row, all four sides and both diagonals of *outside*,
and a scrolled surface with the **unscrolled one as a control**, so a test that
passed because the scroll happened to be zero cannot look like a test that
passed because the scroll was read.

**What it still does not do:** the search field takes no typing, which is task
5's remaining half and is now the only way a person with sixty applications
cannot find one quickly.

### 10. The Dock holds two kinds of thing

**Status: open, not started — 2026-10-12.** The owner's ruling: the saved list
distinguishes an **application entry** from a **native-action entry**, and each
is carried out by its own handler. Settings is the first of the second kind, and
it is removable and reorderable exactly as an application is.

What exists is the intention and not the distinction. The stored list carries
application identifiers and only those, so there is nowhere for a native action
to live and no way for a reader to tell the two apart.
`alo_dock::pinning::ARole::Settings` names the role; nothing stores what fills
it.

**The clause a naive implementation breaks:** *native describes how it runs, not
how long it stays.* The easy way to make Settings always work is to make it
always present, and that takes a choice away — which law 5 forbids whatever the
reason given for it.

**Earns:** the fresh defaults becoming *Files, Browser, Settings* rather than two
applications and a gap. Depends on task 4 for the defaults and task 8 for the
persistence; this is the shape they have to carry.

### 11. The Dock's own controls can be hidden, and the way back is not on the Dock

**Status: open, blocked on a surface nobody has built — 2026-10-12.** The
composer, *Show all*, *History* and *More apps* become visible-by-default rather
than unconditional, a person may hide any of them, and the choice persists. Each
may sit where its own layout needs rather than pretending to be an application
pin.

**The order is the owner's and it is the whole of the block.** Before any of them
can be removed there has to be a discoverable way back that does **not** live on
the Dock, reachable by pointer, by keyboard and by a screen reader — a canvas
action or its equal. **A person who empties their Dock must still be able to
restore it**, and a control whose only route to itself is itself is a control a
person can lose for good. Offering the removal first would be shipping a trap
and calling it a setting.

Nothing in this repository opens a canvas action or any other surface that could
be that route. **So the hiding is not the first task here — the way back is**,
and until it exists this task cannot honestly start.

The ruling also requires that the design and the persistence contract are
updated **together** before this is called complete.

**Earns:** nothing on its own. It is the clause that makes every other
customisation safe to offer.

## What is owed to another lane, and from when

- **Application discovery** — task 4 cannot start without it. Named here on
  2026-10-11 so it is not discovered on the day.
- **The Figma file** — task 5's grid waits on it. The owner's instruction is
  that the revised behaviour *must also be reflected in Figma*, which makes this
  a design task somebody owns rather than a blocker nobody listed.

## What has been earned against a real application, and what has not

`crates/alo-shell/examples/a_real_application.rs` starts real Wayland clients
against a live compositor and then walks the owner's §4 example. Run 2026-10-11
with two `weston-flower` windows — two windows of one application, which is the
case the whole rule turns on:

```
nothing put aside: unpinned on the Dock, pinned on the Dock with 0 put aside
1 of 2 put aside:  unpinned on the Dock, pinned on the Dock with 1 put aside
2 of 2 put aside:  unpinned gone,        pinned on the Dock with 2 put aside
one brought back:  unpinned on the Dock, pinned on the Dock with 1 put aside
```

That is items **3, 4, 7 and 8** of the owner's evidence list. The walk fails the
run with exit 1 and names the step: reverting the builder to report minimised
surfaces as on the canvas produces `<- NOT WHAT THE RULE SAYS` against three of
the four lines.

**What it is not.** A nested compositor under a headless parent is not a screen.
This says what the shell computed. And the run is **by hand, not on every
build** — nothing in the nine gates starts a Wayland client, so this is a
measurement somebody took and not a check that runs again. The gate that would
run it does not exist, and inventing one in passing would be worse than saying
so.

## What this plan deliberately does not do

It does not promise the **completion evidence** in §8 of the ruling as a single
demonstration. Eleven numbered scenarios across launching, minimising, searching,
screen-reader access and a restart cannot be earned by a probe that draws
frames: most of them need a running session with real applications. Each task
above carries the part of that evidence it can honestly produce, and the
remainder is named as needing a machine and a person.
