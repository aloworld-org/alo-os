# The tree a screen reader reads

**What this is:** the first half of `docs/autonomy/v0-5-the-shell-plan.md` task
12 — *the shell exposes every surface's role, name and state to AT-SPI as
`alo-access` decides them, and a test reads the exposed tree over the bus for
each surface the shell draws*, and *the approval surface is exposed as the
sentence, then two answers, nothing preselected*. **The task is not finished**:
what the magnifier and high contrast draw, and the shell's own half of
keyboard-only operation, are still open and are named at the end.

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
it, for the magnifier and for the focus ring.

**No screen reader has read this tree.** Orca has never been run against it: the
reader in the measurement is the agent's, asking the same questions on the same
bus. A person hearing it is not the same as a test reading it, and the plan is
explicit that those differ.
