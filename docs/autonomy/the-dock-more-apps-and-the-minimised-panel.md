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

## The order

### 1 — Who is on the Dock

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

### 2 — Pressing an application

`WhatAPressOnTheDockDoes::GoTo` reaches `alo_dock::clicking` and something
happens. One window focuses or restores-then-focuses; nothing open launches;
several open the picker, which task 3 builds.

**Earns:** §2 of the ruling for the one- and none-window cases, and
*travels to it, never silently relocates*.

**Blocked on nothing.** The press is already routed and already classified; what
is missing is the arm that acts.

### 3 — The window picker

A surface of its own: every window of one application, **minimised ones marked
as minimised**, choosing one focuses or restores it. Reached from the Dock and,
by task 5, from *More apps*.

**Earns:** the third row of the click table, and *usable after the application
is selected* from inside the popover.

### 4 — The default pins reach a screen

Something produces `alo_dock::pinning::Discovery`. Files, Browser, Apps,
Settings, resolved against installed applications, written **once** and only
after discovery **succeeded**. An emptied list is never refilled.

**Depends on** the host's application-discovery integration, which is another
lane's and is the one hard dependency in this plan. `ThePins::what_to_pin`
already distinguishes `Completed`, `StillLooking` and `Failed` and already
refuses to write anything down for the last two — that half is finished and
tested.

### 5 — More apps becomes an overview

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

### 6 — Dismissal and the keyboard

Escape dismisses a picker, then the popover. Outside press already works. The
control closes it. Keyboard reaches the search field and the grid, focus scrolls
into view, and dismissing without choosing returns focus to the opener.

**The hard part is not Escape, it is taking Escape without stealing it.** The
seat must give the key to the Dock only while a Dock surface is open and to the
client otherwise. `crate::a_key_reaches_settings` is the precedent and says
plainly which of its lines no test in the crate can drive.

### 7 — Minimising recalculates eligibility

The link from `set_window_minimized` to the Dock. The compositor half is built
and contracted; what is missing is that anything recomputes the bar.

**Earns:** the four worked examples in §4 of the ruling, which are the
acceptance test for tasks 1 and 7 together.

### 8 — It survives a restart

Pins, their order, and a deliberately empty list, across a restart and an
update.

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
