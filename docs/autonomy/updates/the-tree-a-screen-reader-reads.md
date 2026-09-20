# The tree a screen reader reads

**What this is:** `docs/autonomy/v0-5-the-shell-plan.md` task 12, in two
changes. The first is the tree — *the shell exposes every surface's role, name
and state to AT-SPI as `alo-access` decides them, and a test reads the exposed
tree over the bus for each surface the shell draws*, and *the approval surface
is exposed as the sentence, then two answers, nothing preselected* — and is
what the body of this document is about. The second is the rest of the
acceptance: **high contrast, the magnifier, and the shell's own half of
keyboard-only operation**, added at the end under *What the second change
did*, with what it found and what it still does not claim.

## Nothing is decided here

The role, the name and the state of every control are `alo_access::Surface`'s,
unchanged. The numbers are at-spi2's — `AtspiRole` and `AtspiStateType`, part of
a published interface (ADR 0011: rented, configured, never patched) — and
`crates/alo-shell/src/access_roles.rs` is the one place the two meet, so a role
decided there and a number sent on the bus cannot drift apart anywhere else.
**No reader of our own** is written here or anywhere: what this adds is a tree,
and Orca is what speaks it.

Three files, one reason to change each: the numbers (`access_roles.rs`), the
shape (`access_nodes.rs`), and the serving (`access_bus.rs`).

## What crosses the bus

| | |
|---|---|
| Interface | `org.a11y.atspi.Accessible`, and `org.a11y.atspi.Application` on the root |
| Answered | `GetRole`, `GetState`, `GetInterfaces`, `GetChildren`, `GetChildAtIndex`, `GetAttributes`; `Name`, `Description`, `ChildCount`, `Parent` |
| Never answered | `Component` — nothing is asked or told where a thing is on the screen; `Text` — a name is not a document a caret moves through |
| Never sent | `IS_DEFAULT`, `FOCUSED`, `CHECKED` |
| Emitted | nothing at all: no signal, no subscription |

Names cross the bus **already said**, in the person's own language: each is
`alo_strings::Strings::say` of the word `alo-access` named the control with, so
a machine used in Latvian publishes a tree in Latvian, and what a reader hears
and what a sighted person sees are one sentence rather than two that can drift.

## How it was measured

`crates/alo-shell/tests/the_tree_a_reader_finds.rs` starts **a session bus of
its own**, and on it **at-spi2's own bus launcher** — the same program a
signed-in session starts — which brings up the accessibility bus and its
registry. The tree is served on that bus and **embedded through the registry**
with `org.a11y.atspi.Socket.Embed`, the way every toolkit's bridge joins one.
It is then read back with **`alo-adapters`**, the agent's own reader, which
knows nothing about this crate and asks the questions a screen reader asks.

That the agent's reader is the one used is the point rather than a convenience:
`alo_access::tree` argues that a blind person and an agent read **one**
description of this machine, and this measures the claim instead of asserting
it. It also holds the role numbers to the only other place in the workspace
that has them. If a program the test needs is missing it fails rather than
skipping.

Five tests on the bus, twenty beside them:

- every surface `alo-access` names is read back with the kind, the name and the
  state it decided;
- the password field crosses as a **password field**, and offers no text to
  anything that asks;
- **nothing reads as already chosen** — not the default, not focused, not on —
  on any surface, the approval above all (ADR 0001);
- what is **focusable** on the bus is exactly where the keyboard stops
  (`alo_access::focus_order`), so a control a reader offers and the keyboard
  cannot reach is a thing that cannot be built;
- the approval surface is read as the sentence, then its two answers, in that
  order.

## Two decisions worth the words

**A surface that is not up is described and not shown.** Every surface is in the
tree — a person can be told this machine has a recovery screen — and only the
ones that are up carry `SHOWING`. A tree that claimed all nine at once would
have a reader announcing screens nobody is looking at.

**A surface with no container gets a filler with no name.** Where `alo-access`
decided a Window, a Dialogue, a List or a status strip first, that control *is*
the surface and the rest hang inside it. Where it decided no container — the
window controls are two buttons and no box around them — the controls hang in a
`FILLER`, which is what that role is for. Inventing a name for a box nobody
decided would be this crate wording something.

## Findings

**The machine itself has no name.** The thing a reader announces as the
application — the shell — is published with **no name**, because no crate words
one: `alo-access` names controls, not machines, and nothing else in the
workspace has a sentence for *this machine*. A reader therefore announces the
application as nothing. It is a word to be decided, not one to be written in a
crate that draws, and it is recorded in
`docs/autonomy/v0-5-access-and-language-plan.md`.

**Nothing starts a session yet.** The tree is served and read on a real bus by a
test, and no shipped program brings up a session that would serve it in front of
a person — which is true of every other surface this crate draws, and is the
shell plan's standing state rather than something this task left out.

**Neither the magnifier nor high contrast is drawn yet**, and keyboard-only
operation has a half still in this crate. The shell draws from exactly two
palette doors — `Palette::of(scheme)` for the panels and `DesktopPalette::of`
for the desktop and dock — so high contrast is a change at those two plus the
look each surface carries; it is not started here, and task 12 stays open for
it, for the magnifier and for the focus ring. **Closed by the second change,
below** — and the count was wrong: there are **seven** palettes in this crate,
not two.

**No screen reader has read this tree.** Orca has never been run against it: the
reader in the measurement is the agent's, asking the same questions on the same
bus. A person hearing it is not the same as a test reading it, and the plan is
explicit that those differ.

## What the second change did

The rest of task 12: **high contrast**, **the magnifier**, and **the shell's
own half of keyboard-only operation**. Three subjects, three files, and one
test file that walks the fourth.

### High contrast: one door, seven palettes

What high contrast *is* stays `alo_access::HighContrast`'s — a second palette
held to WCAG 2.2 AAA over every pair this crate draws, with the agent's
terracotta taken deep enough to read rather than replaced. What this change
adds is the door: `crates/alo-shell/src/access_contrast.rs` turns that palette
into the four colour roles this crate paints with, and **every look now carries
which palette it is drawn in**, the way it already carried light or dark, the
text size and the reading direction. `DesktopLook::of` asks
`alo_access::TurnedOn` for it, which is the only road: no crate here can decide
that a screen is drawn in high contrast.

The first note above said the shell draws from *exactly two palette doors*.
That was wrong, and finding out how wrong is the reason this is written down:
there are **seven** — the approval panel's (the recovery screen shares it), the
sign-in screen's three colours, the egress indicator's row, the record window's
(Settings is laid out in it), the desktop's, the lock screen's own two, and the
agent's mark. Every one of them is now built from the door.

| Surface | Held by |
|---|---|
| Sign-in | the same layout, every flat colour in the AAA palette, the ground changed |
| The approval surface | the panel and both answers where they were, every flat colour in it |
| The egress indicator | rows unchanged, the mark still the agent's, the design's terracotta gone |
| The mark itself | the arrow and the edge still stand apart by lightness, in the other palette |
| Every pair this crate draws | measured again through the colours this crate paints, at AAA |

**The accent in high contrast is not the person's own**, and that is
`alo-access`' decision rather than this crate's: at AAA a chosen accent is not
guaranteed to be readable, and that crate decided the role carries the agent's
colour. A person's accent is still *checked* on the way through — an accent
nobody may choose is refused before any palette is built — and then not drawn.

### The magnifier: the frame, not the surfaces

`crates/alo-shell/src/access_magnifier.rs` magnifies **the prepared frame**.
Asking each window to draw itself larger would be a second way of laying the
screen out, and a client that declined would be a hole in it; magnifying the
frame means everything is magnified, the egress indicator law 1 promises
included, with nothing that can be left out. Nearest, never smoothed: a
magnified pixel is the pixel it came from, because smoothing softens exactly the
edges — a letter's stem, the caret — that somebody using a magnifier is trying
to make out. The view is clamped inside the screen, so a pointer at a corner
shows the corner rather than a band of nothing, and **no pixel is invented**.
`PreparedScanout::magnified` is the step a presenter takes between preparing a
frame and submitting it; `magnifying(&TurnedOn)` answers whether to take it.

### Keyboard-only operation, and the two surfaces that disagreed

`crates/alo-shell/tests/every_road_a_keyboard_takes.rs` walks the keys a person
presses on the surfaces this crate draws against `alo-access`' own answers. It
found two that did something else:

- **the approval surface ignored Escape**, where `alo_access::leaving` says it
  **declines** — and the reason is that crate's: a key pressed to get out of the
  way must never approve, and must not leave the proposal waiting behind
  somebody's belief that they have dealt with it;
- **the sign-in screen ignored Escape**, where that crate says it **clears what
  was typed** — there is nowhere further back to go from the screen a machine
  starts at, and somebody who cannot see the password field should be able to
  start again with one key rather than counting erasures.

Both now do what that crate decided. Neither was findable by either crate's own
tests: one held a model nobody had implemented, the other held an
implementation nobody had compared to the model.

**Focus is visible before any key is pressed** on every surface that has a
focus, and that is how this crate was already built rather than something the
setting turns on: there is no state here in which focus is drawn only after a
first keypress, so `Setting::FocusAlwaysVisible` asks for what is already true.
A test says so, so that a change which made focus appear later would fail.
**Nothing is drawn as chosen on the approval surface or the recovery screen**,
which is ADR 0001's rule and the recovery screen's own — and it is why no
*focus ring drawn on the first stop* was added: a ring on an answer nobody has
moved to is a preselected answer.

### What this still does not claim

**No screen reader has read the tree.** Orca has never been run against it. The
reader in the measurement is the agent's, asking the same questions on the same
bus, and a person hearing it is not the same as a test reading it.

**Two of `alo-access`' Tab stops are not walked here.** The desktop's *ask the
agent* and the dock's launcher are surfaces **this crate does not draw at all** —
there is no launcher and no agent overlay in it — so there is no key of this
crate's that reaches them. The walk names the four surfaces it has no key of its
own for and fails when one of them is drawn without being walked, so the lane
that draws a launcher is told to widen it in the same change.

**Nothing brings up a session.** As with every other surface this crate draws,
the magnifier is a step a presenter would take and no shipped program is the
presenter yet. That is the shell plan's standing state, not something this
change left out.

