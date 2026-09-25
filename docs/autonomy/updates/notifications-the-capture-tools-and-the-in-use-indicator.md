# Notifications, the capture tools and the in-use indicator, drawn

**Date:** 2026-09-25
**Workstream:** v0.5 — the shell, task 11
**Task:** 11, *Notifications, the capture tools and the in-use indicator, drawn.*
**Done for what is drawn.** Two things are named rather than ticked, below.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** Apple M3 with 8 GB; built, linted and tested in the Lima VM —
Ubuntu 24.04.4 aarch64. `alo-shell` does not build on macOS. Nothing is ticked
*on the machine*.
**Egress:** `git fetch` and `git push` against `github.com/aloworld-org/alo-os`.

## The inventory first

Taken before any code was written.

- **Nothing in the shell drew any of the three**, and none of `alo-notifying`,
  `alo-capturing` or `alo-in-use` was even a dependency. The gap the task named
  was the whole of it.
- **Every decision was already made in the crate that owns it**, which turned
  out to be truer than the acceptance's wording suggested — see the
  notifications below.
- **`alo_in_use::Line` is shaped like the egress indicator's**, so that surface
  is `egress_status_raster.rs`'s twin rather than an invention.
- **They can reach a real display at all** only because delivery task 39 widened
  the direct seam from one scene to every layer the day before.

One inventory entry was **half wrong and is corrected here**: it said blur was
already destructive inside `alo-capturing`. The crate decides *which* areas are
hidden and demands they be burnt in; the burning is the shell's, and was
nobody's until this task.

## What is drawn

**Notifications.** One card per notification the crate handed over, at the end
of the dock **opposite** the status area. The two indicators own that corner and
are permanent; a notification arrives and goes, and one that covered *what is
leaving this machine* or *your camera is on* would be trading a promise for a
convenience. Every line is the sender's own words — who it is from, the title,
the body when there is one, each action's label — compared in the tests against
what the crate says, so a title shortened or an action renamed here fails.

**The in-use indicator.** One row per line `alo-in-use` wrote, in the status
area, ordered by that crate's `Position` — the screen, then the camera, then
the microphone — and **never by who is using something**. It takes the status
corner and the egress indicator stacks beyond it, because ADR 0010 makes this
indicator's position one of the three things a person is given besides a colour,
and that only holds while its origin does not move.

**The capture tools.** The region as an outline with its middle untouched and
nothing outside it dimmed, and the marks a person is making: an arrow, a box, a
line drawn by hand, their own words, and a blur.

## The strongest claims, and how they are held

- **A held notification cannot be drawn, and not because anything checks.** The
  drawing takes `alo_notifying::Shown`, and the only thing that makes one is
  that crate's `arrives` — which has already asked whether the machine is
  locked, whether the person set quiet hours, whether the screen is being shared
  or recorded, and whether the machine **could not tell** whether its screen is
  being read. *Never while locked, shared or recorded* is carried by the type.
- **Who is using something never moves it.** A test draws the same three uses
  with an application holding the camera and with the agent holding it, and
  compares the rows' positions. If position depended on who, the glance a person
  had learned would stop working at exactly the moment it mattered most.
- **The three marks are compared as pixels**, not as code: the shapes a person
  sees with every hue taken off must differ, and two that happened to rasterise
  the same would fail.
- **A blur is drawn as the flat block it will be saved as.** `capture_flatten`
  replaces a hidden area with one flat colour for good, because a blur is a
  filter and a filter can be undone. Drawing it soft would show a person one
  thing and save another at the one moment they are deciding whether a colleague
  may see what is underneath. A test holds the two marks apart: a box round
  something must not cover what is inside it, and a blur must.
- **An area that would hide nothing is refused rather than saved.** Somebody who
  marked something to hide and got a file with it still showing has been failed
  in the one way the feature exists to prevent.

## Two things named rather than ticked

**In high contrast, colour cannot say *the agent* at all.** `Contrast::High`
collapses every accent to that palette's one accent, deliberately, so the
agent's terracotta and an application's navy are the same colour there. ADR 0010
is why that is safe — terracotta never arrives alone, and the mark and the word
carry it — and the test asserts the equality, with a message saying that if it
ever fails the test is out of date rather than the palette wrong. My first
version of that test demanded colour tell them apart in every palette, which was
asking high contrast to stop being high contrast.

**Both indicators are on the desktop frame only.** Settings, a waiting question
and the record window are submitted by their own paths, which carry the egress
indicator and not yet the in-use one. A person who opens Settings while their
camera is on should not lose the line that says so. That is the remaining
wiring, and it is named here rather than left to be found.

## One thing shared rather than duplicated

Two indicators sit in the status area and a person reads them as one surface, so
the row itself moved into `status_row.rs`: the height of a row, the padding
round its words, the side its mark sits on and the way its words are cut at the
screen's edge are decided once. Each indicator keeps what is its own — which
lines there are, what each says, and the shape of its mark.

## Gates

Nine in the Lima VM. `alo-shell`: 497 unit tests and its integration suites
green, `cargo clippy --all-targets -D warnings` clean, `cargo fmt` clean.

## Crates touched

`crates/alo-shell` only, plus the three crates it gained as dependencies —
`alo-in-use`, `alo-notifying` and `alo-capturing` — each read and never decided
for, and `alo-applications` as a dev-dependency so a test can say who is using a
camera.
