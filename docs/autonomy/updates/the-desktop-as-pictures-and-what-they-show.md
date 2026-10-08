# The desktop as pictures, and what they show

**`desktop_check --save-to <folder>` writes one picture for each standing it
draws.** Run on this machine on 2026-10-08 it wrote **32 PNGs**, and the first
person to look at them was the owner, whose verdict was one sentence:

> This is not the screens that we designed or built

This document is why that sentence was available to be said, and what in the
pictures it is about. **It is a finding, not a fix**, and nothing here changes
what the compositor draws.

## Why pictures at all

`docs/autonomy/updates/what-the-probes-draw-today.md` ended on a debt:

> A look. Thirteen probes draw and nobody has described what a person would make
> of them — whether the dock reads as a dock, whether the egress line is
> findable, whether the sign-in screen looks like one. That is not a test and
> cannot be automated, and it is the cheapest thing on this list.

It stayed owed because looking meant sitting at this machine while a fixture ran
for two minutes. `alo_shell::Nested::keep_each_frame` and
`Nested::the_frame_just_drawn` already existed; what was missing was somewhere to
put the pixels. That is the whole change.

## What was measured

Run as `cargo run -p alo-shell --example desktop_check -- --save-to <folder>`,
under WSLg's Wayland parent, `WAYLAND_DISPLAY=wayland-0`:

| | |
|---|---|
| pictures written | 32 |
| frames submitted | 32 |
| exit | 0 |
| an unknown argument | refused, exit 1 |
| `--save-to` with no folder after it | refused, exit 1 |

Each file begins `89504e470d0a1a0a`, and their sizes run from 17,415 bytes for
the dock alone to 346,165 for what is running read again — content, not one
flat colour.

### The frame count is not a constant, and one of these numbers was noise

An earlier draft of this document said *32 frames, where a run without the flag
reported 33*, and explained the difference by the cost of keeping a frame. **That
was reading noise as signal**, and running it three times says so:

| `--save-to` | frames submitted |
|---|---|
| off | 32 |
| off | 34 |
| off | 33 |
| on | 32 |
| on | 32 |

The loop that draws one standing ends **on a clock, not on a count** — it submits
for 60 milliseconds and stops — so the number measures how fast the machine was.
Keeping a frame paints the scene twice and should cost frames, but 32 is inside
the spread of the runs with the flag off, so **this measured that the count
varies and did not measure what the flag costs.**

What it does establish is that `what-the-probes-draw-today.md`'s *33 frames* is
one sample of a varying quantity, not a property of the renderer, and that a test
asserting it would go red on a slower machine.

## What the pictures show, and what is the fixture's doing

Two things in them are **the fixture handing the compositor an empty machine**
and would differ on a real one: no window carries chrome, because the fixture
passes no controls, and no application is running.

The rest is not the fixture's doing.

### The dock is drawn from a hardcoded zero

This was got wrong twice before it was got right, which is worth recording
because both wrong answers were plausible.

- *the fixture pins nothing* — *wrong.* The fixture cannot affect it.
- *`alo-dock` cannot model a filled dock* — **also wrong.**
  `crates/alo-dock/src/holding.rs:48` declares `OnTheDock` holding `app: AppId`,
  `pinned: Pinned`, `windows: usize` and `put_aside: usize`, and
  `announcing.rs` reads an icon out to a screen reader.

What is actually there is in `crates/alo-shell/src/desktop_raster.rs:197`:

```rust
let dock_picture = crate::dock_raster::picture(
    dock, look, size,
    // Nothing in this crate decides what the Dock holds yet:
    // `alo_dock::Holding` answers that and is not plumbed into a
    // compositor. A bar holding nothing is narrow, which is true
    // rather than a placeholder.
    0,
)?;
```

`dock_raster::picture` takes `holding: usize` and sizes a bar by it —
`Room::a_bar_holding(holding)`. **It draws no icon.** `alo_dock::Holding` reaches
`crates/alo-shell/src/` in exactly two places, and both are comments; it is
named in no line of code.

So the 14-pixel sliver is **a bar holding zero by a constant in the shipped
drawing path.** It is not the fixture, and it would be a sliver on a certified
machine for every person, whatever they had pinned. `docs/features.md` promises
*The alo Dock — it shows what you can open and brings what is already open into
focus*; the second half exists and **the first half is a literal `0`.**

### The desktop draws no surface, which is not the same as no wallpaper

`crates/alo-shell/src/desktop_raster.rs` is 348 lines and 5 functions, and a
search of it for `background` or `wallpaper` returns nothing — positive control
on the same file, so the zero is real. The black behind all 32 pictures is the
desktop's own.

**It would be wrong to call that a missing wallpaper.** ADR 0075 is accepted and
decides:

> alo OS ships no wallpaper. The plane's own surface is the desktop, and what a
> person chooses is that surface's material — not a photograph behind it.

So black is not a missing photograph. It is **a surface with no material**, and
*Set the surface's material* is a `[v0.5]` line. A background is drawn for the
lock screen (`lock_background.rs`) and for the screens panel
(`screens_raster.rs`); the desktop has none.

And two are what the built windows say, which a picture makes plain and a frame
count cannot:

- **the two readouts are unlabelled.** `what is running` opens on two numbers
  with no words beside them — `33657806848` and `32728788992` — then a table
  whose eight columns have no headings. `what is filling a folder` gives every
  size in bytes: `Documents 41500`, `photo.jpg 40000`;
- **a per-row sentence repeats under every row.** *Network traffic counted
  together with 104 other processes.* appears ten times in one window.

**The egress indicator is the one surface that reads as intended.** Bottom
right, found without being looked for: *@mail is asking a question of alo, in
the EU*, beside a mark. The first law, drawn, in words a person can read.

## What this does not establish

- **It is not a comparison against the design.** Nobody has put these beside the
  design files in this session, and this document makes no claim about what the
  design says. The owner's sentence is theirs, from looking.
- **It is software rendering on a virtual output.** Mesa falls back here
  (`ZINK: failed to choose pdev`), so geometry, text layout and colour are the
  design's while filtering and blending need not be. And `desktop_check`'s own
  closing line still holds: *a virtual output proves the drawing path, not a
  panel.*
- **It is what the renderer produced, not what a screen showed.** The kept frame
  is a second painting into a readable buffer, after the swap.

## What it is not

It is **not** the `[v0.5]` screenshot promise in `docs/features.md`. No agent
verb, no D-Bus method, no server request and no key reaches the saving.
`support/saving_frames.rs` carries that distinction in its header, because the
next reader of a fixture that writes frames to files will otherwise find a
screenshot facility no features line authorised.

## What is owed next

The gap between these pictures and the design is now a question somebody can
hold two images up to. Naming it precisely — surface by surface, against the
design files — is the next piece of work, and it is the compositor lane's.
