# The cost test drew where the product never draws

**Concluded**, when live miniatures landed (#1198): *eight miniatures of a
full-HD window cost the software renderer under two milliseconds a frame* —
measured on pixman, every run, by
`put_aside_windows_draw_again_tests::eight_miniatures_cost_the_software_renderer_a_small_share_of_a_frame`.
Read as: pixman can draw a miniature.

**True.** The test drew into a column 112 pixels wide, with the miniatures at
x = 35. On a real screen the panel is at the right-hand edge, near x = 1 224 on
a 1 280-wide one. Pixman refuses a picture when *where it goes × how many times
smaller it is* reaches 32 768
([`../quirks/pixman-refuses-a-picture-drawn-much-smaller-far-from-the-corner.md`](../quirks/pixman-refuses-a-picture-drawn-much-smaller-far-from-the-corner.md)):
35 × 46 is 1 600, and 1 224 × 30 is 37 000. The first booted machine to put
Firefox aside ended its desktop, twice.

**The mechanism.** The test measured the thing it named — cost — correctly. Its
**placement was a fixture choice**, made small so the buffer was small, and
nothing in its sentence said the result depended on where the picture went.
Nobody thought a position could decide whether a draw happens at all, so a
convenient position read as a neutral one. *A check that stands in for the
thing*, in the clothes of a coordinate.

**The cure.** A test of a drawing path lays its target out with the product's
own layout code — here `crate::panel_raster::picture` on a real screen size —
rather than at a hand-picked place, and **draws the refusal case beside the
working one**. `a_picture_the_renderer_refuses_tests` now does both, on
pixman, every run.
