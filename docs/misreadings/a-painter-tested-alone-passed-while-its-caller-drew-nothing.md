# A painter tested alone passed while its caller drew nothing

**Found 2026-11-16 by the VM walk, not by any test.** One lane, one feature:
the teal border round a window alo operates (#1297).

## What was concluded

*"The window alo is acting in wears alo's border."* #1297 landed with tests
that the border's painter draws a 2-wide teal ring round the card, that the
shell marks the window alo types into on real Wayland clients, and that the
walk's pixel finder recognises such a ring. All of them were green.

## What was true

On the walk of 2026-11-16 (run 43805568910, `main` at 84ea663e), the shell
logged `alo is at work in a window of com.vscodium.codium`, which proves the
window was marked. The pixel check then failed after 30 seconds: *"no edge: the
longest teal line across is 22 pixels and down 22"*. The kept screenshot,
`56-vscodium-wearing-alo.png`, shows VSCodium with the text typed and no border.

VSCodium sat 30 below the top of a 1280 × 800 display. A window's band is 54
tall and hangs **above** it (`window_edge::where_the_edge_goes`), so its region
began at −24. `EdgePicture::of` refuses a band that is not wholly on the
display: `if whole_output.intersection(region) != Some(region) { return None; }`.
It returned before it reached the line that draws alo's presence. The whole
picture was dropped, and the border with it. This is **checked**:
`a_window_alo_operates_near_the_top_still_wears_the_border` gives that same
window to `EdgePicture::of` through `edge_of`, which is the road
`every_windows_edge` takes. It failed with *"the window alo operates was given
no picture at all"* before the fix, and passes after it.

## The mechanism

**The tests called the painter directly.** `alos_band_in_a_window::drawn` was
given a card and drew a correct ring every time. Nothing asked whether the
compositor ever calls it for an application's window, or under what
conditions. The refusal that dropped it lived one function up, in the caller.
It had been written for a different thing (*a band half off the screen is not
drawn half*) and it carried the border out with it. The two ends were each
correct, and nothing checked that they meet. This is the gap `CLAUDE.md`'s
*nothing counts as done until a person can reach it* names.

The tests also used windows placed in the middle of a large screen (4000 ×
3000, a window at y 100 or 124). That is where the band fits, so the refusal
never fired. The real placement code puts windows wholly on the screen, which
near the top means *band off it*, the common case and not the edge case.

## The cure

- **Test a drawing through its caller.** For anything drawn on an
  application's window, give the window to `EdgePicture::of` via `edge_of`, the
  way `every_windows_edge` does, rather than to the piece that paints it.
- **Put the test window where the real placement puts one.** Use a display the
  walk's size and a window the walk saw. A window comfortably in the middle of
  a large screen is the one case where every early return stays quiet.
- **Read every `return None` above the line you are adding to.** A refusal
  written for one layer silently takes every later layer with it.
