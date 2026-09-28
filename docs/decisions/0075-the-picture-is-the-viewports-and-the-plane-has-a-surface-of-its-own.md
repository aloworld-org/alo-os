# ADR 0075 — The picture is the viewport's, and the plane has a surface of its own

**Status:** accepted, 2026-09-28, by the owner.

## What forced it

The canvas landed and nobody said where the background goes.

Measured before writing this: `docs/autonomy/the-smallest-canvas-worth-showing.md`
has ten tasks and **mentions the background nowhere** — not once, in any of them.
Nothing in `crates/alo-shell/src/` ties a wallpaper to either layer.
`alo-appearance` decides what a background *is*; `alo-canvas` decides how the
plane moves; no record decides which of the two a background belongs to.

The promise it was written against predates the canvas. `ROADMAP.md` v0.5 says
*"Making it yours: background from a file, folder or colour and per display"*,
with its code box unticked. That sentence describes a desktop where a background
is simply what is behind the windows, and there is no such desktop here any more:
there is a **plane** that pans and zooms, and a **viewport** that does not.

## The decision

**A person's picture belongs to the viewport. The plane has a surface of its own,
and it is a material rather than a picture.**

Two things, not one.

## Why the picture is not on the plane

- **Panning would drag it.** A wallpaper that slides off the side when a person
  moves their work is not a background, it is another object to manage.
- ***Show all* would ruin it.** The canvas's own sentence is *zooming out shows
  you everything at once*. If the picture is on the plane, the one moment a
  person most wants the machine to look composed is the moment their photograph
  becomes a postage stamp in the middle of an empty field.
- **A plane does not end and a picture does.** The plane is a surface a person
  keeps moving along. A photograph on it has to repeat, stretch for ever, or run
  out — and all three are worse than not being there.

## Why the plane still needs a surface

Move a plane with nothing drawn on it and nothing appears to move. The frames
slide and there is no sense of anything underneath them, which makes a pan feel
like the windows jumping rather than like the desk sliding. **The surface is what
makes the plane legible as a place** — it is the difference between *my windows
moved* and *I moved.*

So the plane carries a **material**: continuous, quiet, and cheap at any zoom. It
travels with the plane because it *is* the plane, which keeps the canvas crate's
own rule intact — nothing in the viewport layer reads the camera to correct
itself, and the material does not need to, because it was never in that layer.

## What this keeps that already works

`alo_appearance::Of` already offers `Shipped(name)` and `File(path)` with five
`Fitting`s, and the lock screen already decodes an image
(`crates/alo-shell/src/lock_image_decode.rs`), with a contract forbidding it to
quietly substitute a colour for a picture a person chose. None of that changes.
This record says **which layer draws it**, and the answer is the one those
already assume.

## What is now owed, and was not visible before

- **A material for the plane**, with its own name in `alo-appearance` so a person
  can choose it, and a default that is quiet enough to sit under work all day.
- **The folder, and per display** — the two halves of the v0.5 promise that are
  genuinely missing. `Of` has no `Folder`, and there is one background rather
  than one per screen. Both are about the viewport's picture and are untouched by
  this decision.

## Rejected

- **The picture on the plane.** Above: panning drags it, *Show all* ruins it, and
  a finite thing cannot cover an unbounded one.
- **Nothing on the plane at all.** Cheapest, and it makes pan illegible. A person
  who cannot see that the surface moved cannot tell a pan from a glitch.
- **The material reading the camera to stay still.** That is the exact thing the
  canvas crate says means a thing is in the wrong layer. A surface that does not
  move is a viewport surface; if it should not move, it belongs up there with the
  picture, and then the plane has nothing again.
- **Deciding it in the shell without a record.** It would have been decided
  implicitly by whoever drew it first, and the next person would have had to read
  the renderer to find out what the product thinks a desktop is.
