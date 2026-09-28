# ADR 0075 — alo OS has no wallpaper; the canvas's own surface is the desktop

**Status:** accepted, 2026-09-28, by the owner.

## What forced it

The canvas landed and nobody said where the background goes.

Measured before writing this: `docs/autonomy/the-smallest-canvas-worth-showing.md`
has ten tasks and **mentions the background nowhere**. Nothing in
`crates/alo-shell/src/` ties a wallpaper to either layer. `alo-appearance`
decides what a background *is*; `alo-canvas` decides how the plane moves; no
record decided which of the two a background belonged to — because the wallpaper
was designed before the canvas existed.

`ROADMAP.md`'s v0.5 promise — *"Making it yours: background from a file, folder
or colour and per display"* — describes a desktop where the background is simply
what sits behind the windows. **There is no such desktop here.** There is a plane
that pans and zooms, and a viewport that does not.

## The decision

**alo OS ships no wallpaper. The plane's own surface is the desktop, and what a
person chooses is that surface's material — not a photograph behind it.**

## Why not simply put the picture somewhere

Both available answers are bad, which is the signal that the question was
inherited rather than asked.

**On the plane**, a picture is dragged off-screen by an ordinary pan, and
*Show all* — the canvas's headline, *zooming out shows you everything at once* —
shrinks a person's photograph to a postage stamp at the exact moment the machine
should look composed. A plane does not end and a picture does, so it must repeat,
stretch for ever, or run out.

**On the viewport**, it sits still while the plane moves under it, which makes it
a backdrop the work slides across rather than a surface the work sits on. It also
gives a person two separate things to make theirs — the picture behind and the
surface beneath — which is one too many for a machine whose whole idea is that
your applications float on **a surface you can move**.

## What a person chooses instead

The plane's material: continuous, quiet, and cheap at any zoom. It travels with
the plane because it **is** the plane, which keeps the canvas crate's own rule
intact — *nothing in the viewport layer may read the camera to correct itself* —
and it never needed to, because it was never in that layer.

A plane with nothing drawn on it does not appear to move at all: the frames slide
and nothing is underneath them. The material is what makes a pan legible, and it
is the difference between *my windows moved* and *I moved*.

## This is a refusal, and it is deliberate

Every desktop ships wallpapers. A person coming from Windows or macOS will look
for them, not find them, and may read it as unfinished.

**It is not unfinished.** A wallpaper is furniture for a desktop that stays
still. This one does not. Anybody who later adds `Of::File` back, or ships a
picture in the image, is undoing a decision rather than filling a gap — and
should read this record and say what changed before doing it.

## What comes out

Nothing here is deprecated in place; it is removed, because a half-removed
feature is how a machine ends up with two answers to *what is behind my work*.

- `alo-appearance`: `Picture`, `Of::Shipped`, `Of::File`, and `Fitting`'s five
  modes.
- `docs/contracts/shipped-wallpapers.md`, and the artwork it governs —
  `docs/artwork/wallpapers/alo-quiet-horizon.png` with its approval and SHA-256.
- The image's install of `/usr/share/alo/wallpapers/alo.png`.
- The lock screen's picture path, including `crates/alo-shell/src/lock_image_decode.rs`.
- The v0.5 promise's *file, folder* half. **Deleted rather than completed**, with
  this record as the reason.

## What the lock screen shows now

The same material as the plane, and nothing else. Its old contract forbade
silently replacing a chosen picture with an unchosen colour — a rule that exists
only because there was a picture to replace. There is no longer a choice to
betray, and the rule that replaces it is simpler: **the lock screen shows the
surface and no client's pixels**, which was always the part that mattered.

## What stays

Colour and accent are untouched. `alo-appearance` keeps every refusal it already
makes about a value a person chose, and the eleven colour names with their
translator notes. Making a machine yours is still a thing a person does — it is
done to the surface they work on rather than to a wall behind it.

## Rejected

- **Keeping the picture on the viewport.** The previous draft of this record.
  Written before the owner's decision and superseded by it: it left two things to
  personalise and kept a backdrop the work slides across.
- **Keeping `Of::File` and simply not shipping artwork.** The code path is the
  promise. A machine that can be given a photograph will be given one, and then
  the question of which layer it lives on returns unanswered.
- **Deprecating rather than removing.** Two answers to *what is behind my work*,
  one of them stale, is worse than either answer alone.
