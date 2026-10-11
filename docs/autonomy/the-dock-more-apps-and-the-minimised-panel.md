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
| `Holding::showing` — who is on the bar | **built with the new rule**, and every caller of it is a test |
| anything on a *real* Dock | **nothing** — `alo-desktop` passes `on_the_dock: &[]` |
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

**Call site: there is none, and the sentence that stood here said there was.**

It read *already there — the shell passes `on_the_dock` from `Holding::showing`.
This is the one task whose production path exists, which is why it is first.*
Every word of that was wrong, and it was written in this file four hours before
it was checked. Measured by grep: **every caller of `Holding::showing` in this
repository is a test.** `crates/alo-desktop/src/main.rs:618` passes
`on_the_dock: &[]`, so **no application has ever appeared on a real Dock** — the
pictures in this repository are all of fixtures.

The comment at that literal is honest and names the seam exactly: *an `AppId`
per mapped surface, then a `Windows` built from them, then `Holding::showing`.*
All three parts exist — `Server::mapped_surfaces`,
`Server::minimized_surfaces`, `canvas_remembered::the_app_id_of` — and nothing
joins them.

**So task 1 is two tasks**, and splitting them is the point:

- **1a, the rule** — `belongs_on_the_dock` and `showing` applying it. Done, and
  mutation-tested four ways.
- **1b, the caller** — `alo-desktop` building an `alo_dock::Windows` from its
  mapped and minimised surfaces and calling `showing`. **Not done.** Until it
  is, every clause in this plan is about a fixture.

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

## What this plan deliberately does not do

It does not promise the **completion evidence** in §8 of the ruling as a single
demonstration. Eleven numbered scenarios across launching, minimising, searching,
screen-reader access and a restart cannot be earned by a probe that draws
frames: most of them need a running session with real applications. Each task
above carries the part of that evidence it can honestly produce, and the
remainder is named as needing a machine and a person.
