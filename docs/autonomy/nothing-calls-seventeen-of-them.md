# Seventeen public functions that nothing calls

**What this is.** A measurement, taken once by hand on 2026-10-20, of a fault
this repository has met before under the name *declared is not dispatched*: a
mechanism that is written, documented, often translated, and reached by nothing.
It is the second of the owner's four conditions — *the mechanism works,
**something calls it**, a person can reach it, somebody has seen it on a
machine* — and it is the only one of the four that can be measured from the
source alone.

**It runs once and nothing re-runs it.** There is no test, no gate and no build
step behind the figures below. The last section says what it would take to make
this run forever and why that cannot land today. **A reader a week from now
should assume these numbers describe 2026-10-20 and nothing since.**

## What was measured, and the two holes found while measuring

For every `pub fn` in `alo-terminal`, `alo-shell` and `alo-desktop`, count the
references to its name anywhere else in `crates/`, with doc comments and line
comments stripped — because a name in prose is not a call — and separate product
code from tests. **682 functions. 122 with no product caller outside their own
file. 19 with no caller and no test anywhere at all** — and 17 after this
change, because two of the nineteen were in `alo-terminal` and are fixed here.
The three figures are stated together because they have to agree: 19 is a subset
of 122 is a subset of 682, and a reader who finds them inconsistent has found a
mistake.

The method is grep, so it finds *candidates*, and only the last column is a
verdict a grep can reach on its own: a function whose name appears nowhere but
its own definition is not called, whatever anybody intended.

**Two holes in the method, both found by disbelieving a result rather than by
review**, and both are the reason the first run under-reported:

| the result that looked wrong | the hole it exposed |
|---|---|
| `alo-terminal` had only **3** candidates | `src/measures_tests.rs` is a test file living in `src/`, so tests counted as callers. Also `#[cfg(test)] mod` blocks inside product files. Corrected: **18** |
| `terminal_words` had **no caller at all**, which would mean the terminal's strings never reach the catalogue | the production road is `declare_into`, called 76 times from `alo-saying`'s collector; `*_words()` is a per-crate convenience that tests use. **Not a fault**, and the category *tests only* is mostly like this |

The first run also had **no working control**: it named three functions by hand
as controls and none of them existed in the crate being measured, so it printed
an empty control block and 3 findings, and nothing said the search might be
blind. The control is now drawn from the data — the three most-referenced items
— and the script refuses to report if the most-referenced item has zero callers.

## The 17, in four families

### The keyboard reaches a window edge and nothing happens — 2

`crates/alo-shell/src/the_edge_takes_the_keyboard.rs`

| | |
|---|---|
| `where_the_keyboard_is_on_this_windows_edge` | **is called** |
| `the_keyboard_reached_this_windows_edge` | nothing calls it |
| `this_windows_edge_was_told_a_key` | nothing calls it |

**The read is wired and both writes are not.** The module can be asked where the
keyboard is on an edge; telling the edge that a key was pressed reaches nothing.
This is the family's clearest case and the one that fails *a person can reach
it* rather than merely being untidy, because the file's own header describes
laying a revealed edge out under keyboard control.

### A second compositor loop — 2

`run_compositor` (`direct_loop.rs:79`) and `run_compositor_with_input`
(`direct_input_loop.rs`). `direct_loop.rs` also holds `pub(crate) fn run` at
line 107, and **that one is called** — so there is a live road and these two are
beside it. Five other files mention `run_compositor` in prose as the thing they
are *like*, which is how a reader concludes it is load-bearing.

### Driving a nested compositor — 11

`pump_keyboard`, `pump_approval`, `pump_filling`, `pump_running`, `pump_lock`,
`pump_record`, `pump_recovery`, `pump_settings`, `submit_recovery`,
`submit_desktop_or_recovery`, `submit_with_settings` — across seven `nested*.rs`
files. Not one is called by any product file or any test. This is the largest
family and the likeliest to be deliberate scaffolding that outlived its use;
either way nothing in the repository exercises it.

### Geometry computed and not consulted — 2

`how_big_a_target_looks` (`window_edge.rs:263`) and `is_a_corner`
(`frame_edges.rs:103`).

`how_big_a_target_looks(zoom_in_thousandths)` is **the apparent size of a
pressable target**, and its own module header says the figure is *not invented
here*. Nothing calls it, so something else decides how big a target is — which
is the fault recorded before as *a glyph is not a target: drawn size and
pressable size are different numbers, and one constant for both is a repeat
fault*. Worth a look by whoever owns the edges, because this is how that fault
comes back.

## What was fixed, and what was not, and why

**Fixed:** `alo-terminal`'s two — `a_programs_room_inside` and
`the_note_begins_at` — had no caller *and no test*, which is the worst of the
three states: the arithmetic could have been wrong the day it was written and
nothing would have said so. Both now have tests against figures measured off the
design file rather than against their own formulas restated, and all five
mutations of the constants and the arithmetic are caught. `alo-terminal` is in
no lane's column, so this was taken rather than reported.

**Not fixed:** all 17 above are in `alo-shell`, which is the **Windows lane's**
crate by the table in `two-lanes-that-do-not-wait.md`. Taking a crate that has
an owner is the fault that table exists to prevent, and it happened twice in one
day once already. So this is the measurement, handed over without a remedy
attached — a fix that does not work is worse than no fix, because it turns
*untested hypothesis* into *we tried that*.

**One duplication found on the way and also not fixed.**
`the_terminals_cells_are_drawn.rs:409` carries the comment *`measures::the_note_
begins_at` is the same arithmetic on a body* and then does that arithmetic
itself. It is **not** simply a missed call: the shell needs the figure in device
pixels with each edge scaled and rounded separately, and calling the logical
measure and scaling its result rounds once instead, which can disagree by a
pixel with where the gutter is drawn. So the duplication may be correct. What
was wrong is that nothing held the two together; the logical side now has tests,
so they can no longer drift silently in the same direction.

## What would make this run forever

A test that counts the references to every `pub fn` in these crates and fails
when a function has none. It cannot land today: 17 would have to go in an
accepted list, and an accepted list is only honest if the test refuses *growth*
of the list rather than merely tolerating its contents. That is a gate, it is
cheap, and it belongs to whoever owns the crates it would fail in — so it is
proposed here rather than written.

**Until then the honest statement is the one at the top: this was measured once,
by hand, on 2026-10-20.**
