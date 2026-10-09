# The window frame is designed, and the code draws none of it

**The owner said, of the picture of a real application on alo OS: *the frame in
the figma*.** They were right, as they were right about the dock an hour
earlier, and for the same reason: two lanes reported a thing missing after
looking only at the code.

**Measured 2026-10-08** from `docs/design/figma-snapshot/70-28.xml`, the
committed snapshot refreshed 2026-10-07. Node ids are the design file's own.

## What the design says a window is

`Window / <name>` is a component, one instance per application — *Development ·
Terminal*, *Budget · Sheets*, *Launch files · Files*, fifteen of them. Opened
(`219:13934`, 936.36 × 572.22):

```
Window / Development · Terminal        936.36 × 572.22
  Canvas / Window title    219:13935   x=0      y=0    936.36 × 83.23
  Glass / Window controls  219:14009   x=794.36 y=8    140 × 44
    Minimize               219:14010   x=4      y=4     44 × 36
      Minimize / alo line icon         x=12     y=8     20 × 20
    Full screen            219:14012   x=48     y=4     44 × 36
      Full screen / alo line icon      x=12     y=8     20 × 20
    Close                  219:14014   x=92     y=4     44 × 36
      Close / alo line icon            x=12     y=8     20 × 20
```

Four things in that, each of which the code disagrees with:

1. **There is a title band.** `Canvas / Window title`, full window width, at the
   top. It appears **1,814 times** across the file, and its height is **48** in
   1,574 of them — the rest are instances drawn at canvas zoom. 48 is the
   figure; 83.23 above is one window's scaled instance.
2. **The controls sit at the top right**, inset 8 from the top, in a group named
   `Glass / Window controls` — which names the material §18 asks for,
   *restrained glass effects on floating system controls*.
3. **Each control is 44 × 36 holding a 20 × 20 glyph.** The glyph and the target
   are different measurements, exactly as the dock's implementation contract
   says — *icons may reduce from 32 to 28; targets remain 44×44*.
4. **The three controls are Minimize, Full screen, Close.** Not maximize.
   `window_maximize.rs` exists in the shell and no maximize control is drawn
   here, which is a question for whoever owns the chrome rather than a fault.

## What the code has

| | |
|---|---|
| files in `crates/alo-shell/src/` naming `window_control` | **36** |
| files naming a title band — `title_band`, `TitleBand`, `title_bar`, `TitleBar` | **0** |

The zero has the 36 beside it as its positive control, in the same command, so
it is a measured zero rather than a search that found nothing.

**And photographed, not inferred:** `a_real_application --run foot` drew the
controls **at the top left, directly onto the application's own first line of
output**. The design puts them top right, inside a band that does not exist.

## What this does not establish

**The snapshot carries geometry and names, not appearance.** `get_metadata`
gives ids, names, x/y/w/h and `hidden` — so corner radius, border, fill,
elevation and the actual glass treatment are **not in this reading** and nothing
here says what they are. §18 asks for *consistent window corners, borders and
elevation*; this document measures none of those. Reading them needs the live
file, which the compositor lane has.

**And it is a reading of the 7th.** The live file may have moved.

## Why both lanes missed it

The same shape, twice in one day. The dock was reported as *a literal `0`* and
the window frame as *there is no band*, and both were true about the code and
silent about the design. **A thing absent from the code is not a thing absent
from the product** — it is a thing not yet built, and which of those two it is
cannot be read off the code at all.

The owner said both times, in four words, what neither lane found by searching.
