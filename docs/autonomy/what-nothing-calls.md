# What nothing calls

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

## Hole six, and it ran the other way: two FALSE orphans

*Added 2026-10-20 at 22:00, an hour after the section above landed. Its figures
are superseded by these, and are left in place rather than overwritten so the
sequence of mistakes stays readable.*

**A line comment between `#[cfg(test)]` and the attribute before `mod` made the
splitter miss the test module.** `alo-acting/src/answer.rs` has this shape, and
it was one of nine files:

```rust
#[cfg(test)]
// Counted: one `expect` and no `unwrap` or `panic!`, so only the one it uses
#[expect(clippy::expect_used, reason = "...")]
mod tests {
```

The gap pattern allowed whitespace and further attributes, and **not comments**.
A missed module stays in the product half — and because this one sits after the
last `pub fn` in its file, it was absorbed into **that function's own body**,
where every call it makes is then discarded as a *self-reference*.

**So `alo_acting::could_answer` was reported as called by nothing, and four
tests call it.** Its sibling `is_this_machines_fault` went the same way. Both sit
in the refusal path — `could_answer` is the guard that stops `open` answering
*no window*, which would tell a person something that is not a reason — so these
were the two findings in the list that looked most worth having, and both were
artefacts of the tool.

### The control that closes the class rather than the instance

Widening the pattern fixes nine files. What stops a tenth shape is a control:

> **every product file whose text contains `#[cfg(test)] … mod X {` must be a
> file the splitter stripped.** Measured after the fix: **1,226 recognised, 0
> missed.**

That is checkable without knowing which shapes exist. The previous five fixes
were not: each was a shape somebody happened to notice.

### The corrected figures

| | |
|---|---|
| `pub fn` under `crates/**/src` | **6,827** |
| no live caller and no test, to a fixed point | **44**, on `main` at 22:00 on 2026-10-20 |

`alo-shell` **19** — not the 18 above — `alo-dock` 3, `alo-models` 3, two each in
`alo-adapters`, `alo-agentd`, `alo-notifying`, `alo-corridor` and `alo-tasks`,
one each in nine more, and **`alo-acting` 0, not 2**. It falls to 41 when the two
changes in flight land.

### What six holes in one script actually say

Every one was found by **a result that looked implausible**, never by reading the
code: three findings where a crate obviously had more; a function with a caller
sitting beside an identical one without; a sweep that returned zero; and here, a
guard in the refusal path that surely somebody had tested. **The method is sound
and its implementation keeps being wrong**, which is the ordinary condition of a
grep-based audit. So the figure is worth having only with its controls printed
beside it — and a reader who finds one of these numbers implausible should
disbelieve the number rather than the code.

## A triage pass, 2026-10-20 at 23:00 — six examined, one finding

**The list is a candidate list and this is what triaging it costs and yields.**
Six of the forty-four were taken apart. **Five are not faults.** Written down
with their reasons so nobody repeats the hour.

| | verdict |
|---|---|
| `alo-corridor::where_it_listens` | **not a fault** |
| `alo-nearby::where_it_listens` | **not a fault** |
| `alo-elsewhere::as_number`, `alo-notifying::as_number` | **not a fault** |
| `alo-acting::could_answer`, `::is_this_machines_fault` | **false positives** (see hole six) |
| `alo-adapters::of_this_person` | **a finding** |

### The two `where_it_listens` are near-duplicates, and the live road is elsewhere

Both answer *where this machine listens, for the presence that advertises the
port*, with identical doc comments and bodies differing only in how they build
the error — `why.to_string()` against `because(&why)`. One crate copied the
other.

**The reason it is not a fault is the part worth keeping.** A port advertised
that nothing listens on would be a real bug, so the question is where the
advertised number comes from. `alo-agentd/src/wire.rs` builds
`Presence::of(here, listeners.port())` — from the listener's **own** accessor, not
from either of these. So the advertised port is the bound port, and these two are
redundant rather than load-bearing.

### `as_number` in two crates is ordinary surface

*The number, for whoever has to write one into a surface's own bookkeeping.* An
accessor on a newtype, which callers outside this workspace are the point of.
Nothing to fix and nothing to report.

### `alo-adapters::of_this_person` — the finding, bounded carefully

`alo-adapters` is reached from every other crate in the workspace by exactly
**two** names, measured with test modules split out and comments stripped:

| | |
|---|---|
| `alo_adapters::verbs` | from `alo-declared` — declares its verbs into the catalogue |
| `alo_adapters::words` | from `alo-saying` — collects its sentences |

**And nothing else.** `Tree::of_this_person`, which finds the signed-in person's
accessibility bus, is called only by `Tree::at` inside its own file;
`announced_on` and `asks_as` have no product caller either; and
`accessibility_tree.rs` says in its own header that *a test hands in a tree of
its own*. So the machinery for reading an application's accessibility tree is
reached by nothing, while that crate's verbs and sentences do reach a person.

**What this is not.** It is **not** *the application verbs reach nothing* — they
have an execution road, and this page nearly said otherwise. See below.

`alo-shell/src/access_bus.rs` is the **other side** of the same bus — it serves
the shell's own tree to a screen reader — and says so in a comment that a
reference search counts as a use if it does not strip prose.

## Hole seven, found while triaging: a qualified-path search under-reports

**The six earlier holes all made the count too high or too low by misclassifying
*code*. This one misclassifies a *reference*, and it is the mirror image.**

Scanning for `alo_acting::` found nothing in `alo-agentd`'s product code, and
`alo-agentd` depends on `alo-acting`. The conclusion forming was that
application verbs have no execution road in the daemon — a serious claim about
the capability model.

**It is false.** `at_the_compositors_door.rs` carries
`use alo_acting::{Answer, Request};` and then writes `Answer` and `Request`
bare. The verbs are dispatched: the daemon sends a `Request` to the compositor's
door and reads an `Answer`.

**So a search for `crate::name` sees only the callers who spell it that way**,
and idiomatic Rust callers do not. Any claim of the form *this crate is not used
there* must search the `use` statements too — which is what the `alo-adapters`
table above does, and why that finding survived while this one did not.

**And a limit that no amount of grepping fixes:** dispatch through a trait
object or a function pointer is invisible to all of this. Thirteen crates
declare verbs and the daemon's product code names none of their `*_verbs()`
functions, because the catalogue is assembled elsewhere and dispatched
dynamically. **This method cannot settle whether a verb executes.** It can only
say which names nothing mentions.

## One triaged earlier, and deliberately not fixed

`alo-dock`'s `full_screen_ended` looked like the best candidate in the list: a
`Revealing` state nothing constructs, while full screen exists in `alo-shell`
and the dock's own menu offers `What::FullScreen`. **It is not a fault.** Its
whole body is `Self::covered()` — a named alias for a transition, carrying no
behaviour of its own — and the caller that would use the name rather than
`covered()` directly belongs to `alo-shell`, which is another lane's crate.
Written down so the next reader does not spend the same hour on it.

## The method had two more holes, and the whole workspace is worse than three crates

*Added the same evening, before the first version had landed. The figures above
are for three crates and are left as they were measured; these are for all 122,
and they are not comparable — a scope that lives in the method and not in the
sentence is a scope the reader cannot check.*

**Hole three: a reference from inside dead code counted as a caller.**
`alo-dock`'s `where_it_sits` looked reached because `where_a_preview_sits` names
it — and `where_a_preview_sits` is a one-line wrapper that nothing calls. So
reachability is not one hop: it needs a **fixed point**, removing orphans and
recounting until nothing changes. *The clue was a result that looked wrong: a
function with a caller sitting next to a function without one, in the same file,
doing the same job.*

**Hole four: a `pub use` counted as a caller.** A re-export is a declaration, not
a call, and nearly every crate re-exports its own public surface in `lib.rs` — so
this one flattered every crate at once. `use` statements are now stripped before
counting.

**Hole five, and this one is not fixed:** the first version was
`O(functions × files)` and **did not finish on 122 crates in ten minutes.** It
was rewritten as one tokenising pass — a `Counter` of identifiers per file, built
once — and runs in **0.9 seconds**. Worth recording because the slow version's
failure mode was silence: a sweep that never returns reports nothing, and a
reader would have taken *we did not find more* from it.

### What the workspace actually looks like

| | |
|---|---|
| `pub fn` in `crates/**/src` | **6,709** |
| with no live caller and no test, to a fixed point | **44** |

By crate, where there is more than one: `alo-shell` **18** — one more than the
one-hop count above found — `alo-dock` 3, `alo-models` 3, and two each in
`alo-notifying`, `alo-agentd`, `alo-adapters`, `alo-tasks`, `alo-corridor` and
`alo-terminal`. Then one each in `alo-access`, `alo-nearby`, `alo-elsewhere`,
`alo-installer`, `alo-put-aside`, `alo-bluetooth`, `alo-playing` and
`alo-reconciling`.

**The two independent implementations agreeing on `alo-shell` is the only
corroboration here**, and it is worth more than either number alone: the slow
one-hop script and the fast fixed-point one share no code and both say 17–18.

### One of the 44 is fixed in this change

`alo-access`'s `ignore_the_same_key_for` — the only way to change how long the
same key is ignored for, with **no caller and no test, not once**. It is a gap
rather than a tidy-up because the three key settings are a set: `Setting::every()`
offers sticky, slow **and** bounce keys, each has a translated word, and
`filtering.rs` genuinely honours the value. Its twin `hold_keys_for` is asserted
to round-trip; this one was reachable by nothing — in a test file whose own name
is *the values a person chose survive being written down*, which covered two of
the three.

**No road for a person to choose the delay was added.** `docs/features.md` offers
*sticky keys, slow keys, and keyboard-only operation of everything* and names no
duration, so building one would **add** a promise rather than keep one.

### What these three crates' figures do and do not mean now

The 682/122/19 above stands as what was measured for `alo-terminal`,
`alo-shell` and `alo-desktop` with the one-hop method. The workspace table is the
better measurement and it is the one to repeat. **Neither runs anywhere**, which
is still the last section's point.
