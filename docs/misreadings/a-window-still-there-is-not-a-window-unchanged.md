# A window still there is not a window unchanged

**Found 2026-11-16 by two walks on `main`, not by any test.** One lane, one
check: the VSCodium walk's *alo's teal edge is gone once the turn has ended*.

## What was concluded

*"VSCodium is not there, so the edge being gone proves nothing."* The check
failed twice, on runs 43864271712 and 43872632801, with *"no edge, and the
window it was round … is not there either"*.

## What was true

Picture 59 of run 43872632801 shows VSCodium exactly where the edge had been,
with the border gone and the text saved. The product was right and the check
was wrong. The check, `the_same_window_inside`, asked that at least half the
pixels inside where the edge was be **identical** to those in the picture where
the edge was first seen. That picture, 57, was taken as soon as the edge
appeared, which was before VSCodium had drawn anything: its window was a flat
`#202020`. By picture 59 the editor was drawn, and every pixel inside had
changed. This is **checked**:
`vscodium_after_its_edge_has_gone_is_still_there` reads pictures 57 and 59
from `crates/alo-walking/tests/fixtures`. It failed before the change and
passes after.

## The mechanism

**The guard measured a stand-in for the thing.** It was written to stop *gone*
passing on a screen gone black or locked. It did that by asking for the
window's pixels to be unchanged. That stands in for "the window is still
there" only while the window's content holds still, and a window alo works in
is exactly the one whose content does not: it was typed into and saved between
the two pictures. The guard was tested with a fixture window whose inside
changed in 300 pixels out of tens of thousands, so the stand-in and the thing
agreed every time it ran.

## The cure

- **Ask the screen the question directly.** *Is a window here?* is answered by
  the window's own edge: does the pixel just inside where the edge ran differ
  from the one just outside? Not by comparing two moments' contents.
- **When a check guards against a failure, build its fixture from the case it
  must pass, too.** Here that is a window whose content changed completely. The
  pictures from the walk that found the fault are now its fixtures.
