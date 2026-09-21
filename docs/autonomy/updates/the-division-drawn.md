# The division, drawn — and the state under it, named

**Date:** 2026-09-21
**Workstream:** v0.5 — the shell, task 10
**Task:** 10, *Dividing the screen, virtual desktops and gestures, drawn.*
**Done for the part it ends at since its split.** The rest is task 16.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** Apple M3 with 8 GB; built, linted and tested in the Lima VM —
Ubuntu 24.04.4 aarch64. `alo-shell` does not build on macOS. Nothing is ticked
*on the machine*.
**Egress:** `git fetch` and `git push` against `github.com/aloworld-org/alo-os`.

## What is drawn

`division_raster.rs`. It takes the answers `alo-dividing` already gives and
turns them into pixels, and decides none of them: there is no branch here on how
near an edge a pointer is, no half, no quarter, and nowhere one could be added.

- **Each share**, from `Division::shares`.
- **The rule where two meet**, found from the shares themselves rather than from
  the tree, so a boundary cannot come to be drawn where no two shares actually
  meet. A boundary moved by the division moves the rule with it.
- **The outline a drop would take**, from `alo_dividing::Proposal::area` — the
  same rectangle `commit` will use. A person dragging a window is deciding
  whether to let go; the outline is the answer to *what happens if I do*, and
  one that disagreed with the commit would be the machine lying at the one
  moment somebody could still change their mind.

It is an **outline and not a fill**, asserted by a test that the middle of the
proposed area is not painted: what a person is deciding about is where the
window is going, and a filled rectangle hides it.

Nothing offered and a refusal both draw nothing. A refusal is said in words by
`alo-dividing`'s own vocabulary, and an outline that meant *no* would be a shape
a person has to learn.

## The constraint, half kept — and the half that is not, written down

The plan says: *v0.01's `window_tiling` yields to the division; there are never
two tiling decisions in one compositor.* Reading the crates rather than the
sentence, that turned out to be two different claims.

**The one I fixed.** *Which side a chord means* had **two** answers:
`alo_dividing::keyboard::side_for` maps `SnapLeft → Side::Left`, and
`window_command.rs` mapped the same two actions to `TileSide` itself. It now
asks the dividing crate, and a test walks **every** `Action` and fails if the
shell ever names a different side from the crate that decides. The shell may
*refuse* a side it has no half to lay out in — a top or a bottom — because that
is a narrowing of the decision and never a different one.

**The one I did not.** `window_tiling` computes a half of an output; a
`Division` computes a share of a tree. Those are two layout decisions — **but
only once the `Server` holds a division, and it does not.** The `Server` holds
an overlay, a press, a presentation, its surfaces, its socket and a switch
order. There is no division and no desktop in it.

So today there is exactly one layout decider, which is sound, and the constraint
is about a state that has not arrived. Saying that plainly is better than
removing a mechanism nothing has replaced yet.

## Why the task is split rather than ticked

The rest of task 10's acceptance — divisions drawn per display *as decided and
restored as remembered*, desktops, swipes, the indicator on every desktop — is
not drawing. It is `Server` state with a lifecycle: windows opening and closing,
displays plugged in and unplugged.

**That is the same lifecycle a running session needs.** Building it twice — once
for a nested compositor and once for the real one — would be two answers to
*what is on this display now*, which is the same fault the constraint is about,
one level up. So it is **task 16**, and it depends on task 13 of
`v0-01-delivery-plan.md`, the session that stands the desktop up, rather than on
this task.

Task 16's acceptance carries the removal explicitly: `window_tiling`'s half goes
out as the division goes in, **with a test that there is exactly one layout
decider**, read from the crates rather than from a list kept beside them. The
two-deciders problem cannot arrive quietly.

## What I did not do

No desktop state, no swipes, no restore. Nothing in `alo-dividing` or
`alo-desktops` — they decide and this shows. `window_tiling` still computes its
half, and is still the only thing that does.

Task 14's blocker now names **16** rather than 10, because a walk cannot step
through a second desktop that nothing holds, and a blocker that outlives its
cause makes takeable work look untakeable.

## Gates

Nine in the Lima VM.

## Crates touched

`crates/alo-shell` only — the new `division_raster.rs`, `window_command.rs`, and
`alo-dividing` and `alo-desktops` added as dependencies.
