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

**32 frames, where a run without the flag reported 33.** The loop that draws one
standing ends on a clock rather than a count, and keeping a frame paints it a
second time, so fewer fit in the same 60 milliseconds. The count is a property
of the clock, not a claim about the renderer, and it is not the same number with
the flag on.

## What the pictures show, and what is the fixture's doing

Three things in them are **the fixture handing the compositor an empty machine**,
and would look different on a real one: nothing is pinned to the dock, no window
carries chrome because the fixture passes no controls, and no application is
running.

One is not the fixture's doing and is the finding:

- **`crates/alo-shell/src/desktop_raster.rs` draws no background.** The file is
  348 lines and holds 5 functions; a search of it for `background` or
  `wallpaper` returns nothing, with a positive control on the same file to prove
  the search ran. The black behind everything in all 32 pictures is what the
  desktop draws, not what the fixture passed. A background exists for the lock
  screen (`lock_background.rs`) and for the screens panel
  (`screens_raster.rs`). The desktop has none.

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
