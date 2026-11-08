# A review picture blended the way the renderer does not

**Found 2026-11-08, on a booted machine: Dolphin stood in an opaque white
rectangle. Nothing in the repository caught it; the walk did.**

## What was concluded

That the shadow round an application's window, and the shadows of Settings, the
record and the launcher, looked as the approved board draws them. The owner was
shown pictures of each, written by tests under `ALO_ROUND_2_PICTURES`, and they
did.

## What was true

On the machine every one of those shadows was a white halo as far out as the
shadow reaches: `w17-17-files-opened.png` is `255,255,255` from 57 pixels
beside Dolphin inwards, and `sixteenth-25-record.png` shows the same round the
record, a week earlier.

The rasters are made in straight alpha. Both renderers blend premultiplied —
smithay's GLES with `BlendFunc(ONE, ONE_MINUS_SRC_ALPHA)`, pixman because its
`a8b8g8r8` is premultiplied — so a pixel of navy at almost no alpha is drawn as
*the canvas plus the navy*, which on cream passes 255 in every channel. The
review pictures composited the same bytes by hand as `colour × alpha + under ×
(1 − alpha)`: the straight blend, the one the renderer does not do.

## The mechanism

**The check re-implemented the thing it checked, and re-implemented it the way
the author believed it worked.** The picture writer and the raster were written
from one idea of what the bytes mean, so they agreed with each other, and the
renderer, which was the only party with a different idea, was not asked. Every
unit test read the raster's own bytes, which were exactly as intended. The
fault was in the meeting between the bytes and the blend, and no test stood at
that meeting.

It was invisible for a second reason: the Dock's and the terminal's rasteriser
happens to write premultiplied colour, so the surfaces looked at most often were
right, and *shadows work here* was true of the ones anybody had checked.

## The cure

- **A claim about pixels a person sees is read back through the real
  renderer.** `software_scanout_tests::a_windows_shadow_never_lightens_the_ground_under_it`
  paints through pixman and reads the scanned-out bytes; a hand blend in a
  test is a second opinion, not a measurement.
- **A test that composites by hand says which blend it is doing and why that
  is the renderer's.** The three review writers now add the colour as it is
  and the ground times one less the alpha, which is what `painted::Washed`
  documents a renderer doing.
- **Ask a new raster which convention it writes before handing it to a
  renderer.** `Washed::premultiplied` is the door for a straight one.
