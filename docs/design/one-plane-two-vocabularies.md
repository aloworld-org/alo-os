# One plane, two vocabularies

**Status:** a design note. It records a duplication, says which side should go,
and says why the decision has to be taken before the next surface is built
rather than after.
**Date:** 2026-09-30
**Asked by:** the lane building the minimised-windows panel, which is the first
surface that needs both halves and therefore the first that has to choose.

## The measurement

Two crates model the workspace plane, and neither knows about the other.

| | |
|---|---|
| `crates/alo-canvas/src/plane.rs` | `At`, `Size`, `Span`, `Frame` |
| `crates/alo-canvas/src/camera.rs` | `Camera`, `Zoom` |
| `crates/alo-dock/src/on_the_canvas.rs` | `Spot`, `Patch`, `TheView` |

`Spot` is a point on the plane. `At` is a point on the plane. `Patch` is a
rectangle on the plane. `Frame` is a rectangle on the plane. **These are the
same four ideas, written twice, in crates that do not depend on each other.**

Measured 2026-09-30: `alo-canvas` does not depend on `alo-dock`, and `alo-dock`
does not depend on `alo-canvas`. `Spot` and `TheView` are used by ten files in
`alo-dock` and two in `alo-handing`.

## The file says it is in the wrong crate, in writing

`on_the_canvas.rs` opens with *"Where a window is on the workspace plane, and
what the person is looking at"*, and carries this as one of its own headings:

> **Nothing here knows about a dock, a screen or a display**

A file in `alo-dock` whose documentation states that it knows nothing about a
dock is a file in the wrong crate, and it has been saying so the whole time.
Nobody had to infer the duplication; it was written down, under a heading, and
read as a virtue rather than as an address.

That is worth naming on its own: **a module can describe its own misplacement
clearly and go on being misplaced, because a docstring is read as a description
of what the code does and not as a claim about where it lives.**

## What should happen

**The plane belongs to `alo-canvas`, and `alo-dock` should depend on it.**

1. `alo-canvas` owns the plane-to-screen transform and `Camera`. A surface that
   restores a window has to travel the camera to it, so it needs that half
   regardless.
2. `on_the_canvas` only ever compares two rectangles. That is a service the
   plane crate should offer everyone, not something each surface reimplements.
3. The dependency runs the safe way. `alo-canvas` does not depend on `alo-dock`,
   so `alo-dock` can depend on `alo-canvas` with no cycle. The reverse move —
   teaching `alo-canvas` about `Spot` — would be the wrong direction and is not
   proposed.

So `Spot` becomes `At`, `Patch` becomes `Frame`, and `TheView` becomes whatever
`alo-canvas` calls the part of the plane a person is looking at. Twelve files
change across two crates. It is mechanical and it is not small.

## Why now, and not after the panel

The panel needs `alo-dock`'s window model — `WindowId`, `AppId`, `HowItSits` —
**and** the plane, to put a restored window back and travel to it. It is the
first surface that needs both halves.

**Every surface built on a duplicated vocabulary has to choose one, and each
choice makes the next one harder to reverse.** Today two crates disagree and
twelve files depend on the disagreement. After the panel it is three crates, and
after the next surface four. The panel is not the reason to fix this; it is the
last cheap moment to not have to.

Writing a third vocabulary is the one option that is certainly wrong.

## What this note does not decide

- **Whether the panel's own crate is separate.** It should be — a panel of
  minimised windows has a different reason to change from a dock, which is law 3
  at crate scale — but that is the panel lane's to record, not this note's.
- **The names on the far side.** `At` and `Frame` are `alo-canvas`'s today and
  this note does not argue they are the better words; only that there should be
  one set.
- **When.** The move touches twelve files across two crates and belongs in a
  change somebody is awake for, not in the tail of a night's work.

## Who owns it

`alo-dock` is the dev PC lane's, so the move is the dev PC lane's. This note
exists so the panel can be built against `alo-canvas` from the start whether or
not the move has happened yet — a new surface should not be asked to wait for
a cleanup, and it should not be asked to speak the vocabulary that is leaving.
