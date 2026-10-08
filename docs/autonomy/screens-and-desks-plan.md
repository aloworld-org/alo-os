# Screens and desks

**Owner:** the Mac. **Milestone:** v0.01, by the owner's direction of
2026-10-08. **Crates this plan owns:** `alo-displays`, and the display-facing
parts of `alo-shell` and `alo-desktop`.

**What the owner asked for**, 2026-10-08: *"I wanted us to reimagine using or
connecting monitors in a unique way and super user-friendly and intuitive way
that can be catchy"*, then *"users can name the monitors by any name like
'cat'"*, then *"I want all the discussed features of the monitors to be in the
next release after this one so you tasks today is to do all the features we have
agreed on"*.

The promises are in `docs/features.md` under **Screens and desks**, each with a
ledger entry in `evidence-it-boots-and-the-agent-acts.md`. The naming decision is
[ADR 0098](../decisions/0098-a-screen-carries-the-name-its-person-gave-it.md).

## What is already built, measured 2026-10-08

**This is not a plan to build a display model. One exists, and it is good.**
`alo-displays` already holds:

- **A separate arrangement per set of screens**, so an office desk and a home
  desk are remembered apart — `three_sets_of_screens_remembered_apart` proves it.
- **Nine sentences for a person**, written and translated, in `notes.rs` and
  `words.rs`, including *your screens are arranged the way you last left them*
  and *the arrangement you made at your other desk is still here for when you
  are back at it*.
- **The rule that there is no note for the ordinary morning**, because a system
  that announces every screen every time is one whose announcements nobody reads.
- **Windows that went away with a screen** — `windows_away`, `unplugged`,
  `plugged_in`, `resumed_to`.
- **A size a screen cannot draw, told to the person** — `Note::SizeRounded`.

**None of it has a caller.** `Server::the_screens()` is `None` on every running
machine. So this plan is mostly about reaching what exists, and the three new
things are the card in task 8, the name in task 6, and the showing in task 7.

## What this plan does not do

**It does not prove anything on two monitors.** This lane's machine has one
screen. Every task below ticks **the code** and none ticks **on the machine**;
that half is phase 8's, on a certified laptop with a second display attached, and
no amount of work here substitutes for it. `more-than-one-display-plan.md` has
said so from its first paragraph and this plan inherits it.

## The order, and why it is not negotiable

Tasks 1 to 4 are not features. **Every feature below reads something they
create**, and three of them are defects measured on 2026-10-08:

| | why nothing works without it |
|---|---|
| 1 hotplug | nothing **triggers** — a screen arriving mid-session is not noticed at all |
| 2 position | every display tells applications it is at `(0, 0)`, stacked on the others |
| 3 the arrangement survives | an arrangement built once is not an arrangement kept |
| 4 scale | a display's size reaches alo's own dock and panel and never an application's window |

## Tasks

### 1. A screen arriving or leaving is noticed

**Status:** ready. **Depends on:** nothing.

**This is sub-work, not scope, and the judgement is stated here because
`CLAUDE.md` requires the change to say which answer it got.** *Multi-monitor,
display scaling, hotplug* stays at `[v0.5]` and its `ROADMAP.md` exit gate is
untouched — **moving a milestone's exit gate changes when v0.01 is declared done,
and that is not a side effect worth producing while adding feature lines.** What
makes this task buildable now is that **it makes an existing promise true rather
than adding one**: *a desk comes back when you arrive at it* is promised at
`[v0.01]`, and arriving at a desk cannot be honoured by a machine that does not
notice you arrived. The same reasoning covers tasks 2 and 4, which are measured
defects rather than promises.

Nothing watches for a display appearing or disappearing. `discover_every_atomic_output`
is asked once, as the session starts.

- **Acceptance:** a display added or removed after the session has started is
  discovered, presented or retired, and the arrangement is asked for again — in
  a test that adds and removes one from a fake backend.
- **Constraint:** a session that never sees a change behaves exactly as it does
  now.

### 2. Every display is told where it actually is

**Status:** ready. **Depends on:** 1 for the re-ask, not for the fix.

`crates/alo-shell/src/presentation.rs:341` is the only call to
`change_current_state` in the crate and it always passes `Some((0, 0).into())`.
There is no `Space::map_output` anywhere. So on a two-display machine both
`wl_output` globals say *I am at the desk origin*.

- **Acceptance:** each output's advertised position is its corner in the
  arrangement, and a test reads the two back as different.
- **Constraint:** one display keeps reporting `(0, 0)`, because it is at the
  origin and that is every machine this lane can test on.
- **Note:** our own popups are unaffected — task 6 of the display plan
  constrains them against the shell's own rectangles — so this is a gap for
  applications, which is why it is worth saying rather than assuming it is the
  same bug.

### 3. The arrangement survives a screen arriving

**Status:** blocked on 1 and on **task 11 of `more-than-one-display-plan.md`**.

- **Acceptance:** plugging a screen in rebuilds the arrangement from the new set
  and keeps whatever the person arranged for a set they have arranged;
  unplugging returns to the smaller set's own arrangement rather than to a
  side-by-side default.
- **Constraint:** `Note::DidNotFit` is said when it is true and not otherwise.

### 4. The size a display is drawn at reaches an application's window

**Status:** ready. **Depends on:** nothing. **This blocks task 10.**

`display_scale` reaches `desktop_raster` — alo's own dock, panel and status area
— and nothing applies it to a client buffer. `presentation.rs:341` advertises
every output to every client as `Scale::Integer(1)`, and
`wp_fractional_scale_v1` is advertised nowhere.

Today `the_screens()` is `None` so every display answers 100 and the two agree.
**The moment an arrangement exists in production, a dense screen gets
double-size furniture beside unchanged application windows.**

- **Acceptance:** a display whose arrangement says 200 draws alo's furniture and
  an application's window at the same apparent size, asserted on both.
- **Constraint:** 100 must change nothing anywhere.

### 5. A desk comes back when you arrive at it

**Status:** blocked on 3.

- **Acceptance:** the arrangement a person left for *this* set of screens is in
  force on arrival, in a test that arranges one set, moves to another, and
  returns.
- **Constraint:** a set nobody has arranged gets the side-by-side default and
  says so, rather than borrowing another desk's arrangement.

### 6. A screen carries the name its person gave it

**Status:** blocked on **task 10 of `more-than-one-display-plan.md`** for the
seam. ADR 0098.

- **Acceptance:** a name a person typed is kept, shown everywhere a screen is
  named, survives a restart, and `cat` works. A default name is descriptive and
  never a bare number.
- **Constraint, and the one a later convenience will undo:** **nothing looks a
  screen up by the person's name.** Two screens both called `cat` are two
  screens. Tested in both directions, because ADR 0098's four rules all rest on
  tests that do not exist yet.

### 7. The machine says which desk it thinks you are at, once

**Status:** blocked on 5.

`Screens::notes()` returns nine sentences and nothing reads it.

- **Acceptance:** arriving at a known desk shows *as you left them* once; a new
  screen shows *new here*; a desk a person has not arranged shows *the other
  desk's arrangement is kept* **only when that is true of the moment**.
- **Constraint:** **nothing is said on the ordinary morning.** `notes.rs` holds
  that rule and the wiring must not break it — a test that the same screens
  plugged into the same machine show nothing at all.

### 8. Show me where

**Status:** blocked on 5 and 7. **The only task here with no engine behind it.**

`Note::NewHere` already says a new screen was put beside the others, so the
*guess* exists and the *correction* does not.

- **Acceptance:** a card on the new screen, pushed off an edge, writes the
  arrangement that the push describes — and the next frame lays out against it.
  Doing nothing keeps the guess and leaves the card offered where the notes are.
- **Constraint:** no timer and no delay anywhere in it. A reveal that depends on
  how fast somebody can move is the one thing a person with a tremor or a
  trackball cannot control, and Law 5's own text names this case.

### 9. Unplug and nothing is lost; plug back in and nothing moved

**Status:** blocked on 3.

- **Acceptance:** windows on a departing screen come home, the person is told how
  many and from where, an undo puts them back, and plugging the screen in returns
  them to where they were.
- **Constraint:** the return is **shown**. A frame that relocated itself silently
  is a person's arrangement edited without them, which the canvas plan forbids in
  as many words.

### 10. How large things are, per screen, in words

**Status:** **blocked on 4, and must not be built before it.**

- **Acceptance:** *smaller*, *just right*, *bigger* per screen, previewed on the
  screen being changed, with no percentage shown anywhere.
- **Constraint:** offering this while task 4 is open would hand a person a
  control that lies. `Note::SizeRounded` is said where a screen cannot draw what
  was asked.

## What closes this plan

Every task's code, plus **two screens on a certified laptop** with a person
naming one of them, arranging them by pushing a card, unplugging one and getting
their windows back. Until that, this plan ticks code and nothing else, and no
line of it may be read as *more than one display works*.
