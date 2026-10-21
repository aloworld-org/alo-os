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

## Holes eight and nine, and the control I had already written down and not applied

*Added 2026-10-20 at 23:30. **Every figure above this line is superseded**, and
they are left in place because the sequence is the point.*

Triaging the list meant checking entries one at a time, and two of them turned
out to be called. Each exposed a different way the script **drops text**, and
both are the same family as hole six: text that goes missing is attributed to
whatever unit swallowed it, and a call inside it then reads as a self-reference.

**Hole eight: only `pub fn` bounded a unit.** `pub(crate) fn` and private `fn`
bodies were absorbed into the preceding `pub fn`'s unit, so a call they made to
that neighbour was discounted. `alo_models::tried_on` is called on line 105 by
`pub(crate) fn what_is_wrong_with_it` and was reported as called by nothing.
Units are now bounded by **every** `fn` and reported on only for `pub fn`.

**Hole nine: units were keyed by `(file, name)`.** Two `impl` blocks in one file
can define the same method name — `alo-installer/src/ended.rs` has `pub fn
said_as` at line 161 **and** line 219 — so the second silently overwrote the
first, and the first's text vanished from both `units` and `leftover`. It
contained the only two callers of `alo_installer::sizes::in_megabytes`, which
was therefore reported as an orphan. Keyed by position now.

**That one also moved the denominator**: collapsing same-named methods had been
under-counting the workspace's `pub fn` by **293**.

### The control that closes the class, which I wrote down this morning

> **assert the partition is total** — the halves sum to the whole, or the count
> of inputs that should have matched equals the count that did

Units plus leftover must account for **every byte** of each file's product half.
That check is now in the script and refuses the run otherwise. It catches holes
six, eight and nine at once, and it would have caught all three on the day each
was introduced. **It was already written in this lane's own notes, from the
earlier holes, and was not applied to the very script that produced them** —
which is a better argument for a control than any of the reasoning above it.

### The figures, and this time something other than me agrees

| | |
|---|---|
| `pub fn` under `crates/**/src` | **7,125** |
| no live caller and no test, to a fixed point | **40** |

`alo-shell` 18, `alo-dock` 3, and two each in `alo-adapters`, `alo-corridor`,
`alo-tasks`, `alo-agentd`, `alo-models` and `alo-notifying`, then one each in
seven more. `alo-installer` and `alo-acting` are **off the list entirely**.

**All forty were then verified by a separate script sharing no code with the
sweep**: for each, count every occurrence of its name across the workspace's
product halves with comments and `use` statements removed, subtract *every*
definition of that name anywhere — so a name two crates share is not read as a
mention — and require zero, plus zero mentions in any test text. **Forty of
forty passed.** That is the first figure on this page with corroboration rather
than a single tool's word.

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

## What a reference count cannot see, and it shipped a grey screen the same night

*Added 2026-10-21. The sweep above is a count of callers, so the only fault it
can find is **nothing calls it**. The fault below is **not everything that
should, does**, and no reference count will ever find it: the orphan test passes
on the first caller.*

**What happened.** `which_choices_are_shown` and `where_a_list_screen_goes` are
the installing environment's scrolling for a list longer than the screen, built
in #892. One screen uses them. The screens added five hours later in #899 reach
for `where_a_screen_goes` instead, which lays out every card with no bound — so
past the number that fit, the flow runs off the screen and **nothing is drawn at
all**, rather than a truncated list. Measured by the desktop lane on 1920×1080:
**four** choices draw, and at five there is no screen. Eight keyboard layouts
are offered, so the layout screen never drew on any display.

Measured here 2026-10-21, over all 55 files of `crates/alo-installing/src`, with
every read proved rather than assumed — **55 of 55 readable**, because the first
version of this measurement doubled the path and reported a confident **0** for
every file:

| file | mentions of the list layout |
|---|---|
| `the_screen_that_lists_every_layout.rs` | **9** — the one screen that scrolls |
| `where_a_list_screen_goes.rs` + its tests | 4 + 18 — the mechanism itself |
| `which_choices_are_shown.rs` + its tests | 4 + 10 — the mechanism itself |
| `lib.rs` | 5 — the exports |
| **`the_asking_walks.rs`** — the flow, 16,190 bytes | **0** |
| **`where_a_screen_goes.rs`** — the general layout, 13,720 bytes | **0** |
| `a_screen_is_drawn.rs` | 0 |

**Why the sweep is blind to it.** Every function in both modules has callers.
`WhereInTheList` is exported at `lib.rs:168` and constructed at
`the_screen_that_lists_every_layout.rs:90`. A caller count of one and a caller
count of nine are the same answer to *is this reached*, and the question that
matters is *is this reached from everywhere it should be* — which is not a
property of the function at all. It is a property of the **set of surfaces**,
and nothing in this page's method knows that set exists.

**One of the six the desktop lane reported is this fault wearing the other
one's clothes, and is worth keeping as a worked example.** They flagged
`first_one_shown` as a `pub fn` nothing calls. It has **four** callers, all
inside its own file — lines 102, 116, 125 and 131 of
`which_choices_are_shown.rs`, against the definition at 89. By this page's
method it is reached and was never a candidate. Their instrument excludes
same-file callers somewhere, which is a different and defensible question to
ask, and it found a real fault **for the wrong stated reason**. A list built
that way will be mostly normal shapes: a function used only by its own module's
other methods is how an `impl` block is written.

**And the timeline is the argument for the gate the section above proposes,
better than that section makes it.** The sweep was not wrong and no crate was
skipped:

| | |
|---|---|
| this document landed, `alo-installing` at **0** orphans | 2026-10-20 **22:52:52Z** |
| #892 built the scrolling | 2026-10-21 **04:04:57Z** |
| #899 built screens that do not use it | 2026-10-21 **10:44:35Z** |

**The fault is newer than the instrument by five hours and twelve minutes.** So
the case for a standing check is not *a hand sweep might miss something* — it is
*a hand sweep missed nothing, and what it could not have seen shipped the same
night*. The gate the section above proposes would still not have caught **this**
one, which is the honest half: it would have caught a function with no callers,
and this is a function with one.

**What would catch it** is a different check, and it is not written either: for
each mechanism with more than one plausible surface, the **set** of surfaces
that use it against the set that exists. That needs somebody to say what the set
is, which is a judgement and not a count — so it is named here rather than
proposed as a gate, because a gate nobody can define is worse than a paragraph
somebody reads.

## Hole eight, and it is in a reimplementation of this page's own method

*Added 2026-10-21. The script that produced the figures above was never
committed — it lived in a scratchpad and is gone, so re-running the measurement
means rewriting the instrument. **That is the first finding here**: a page whose
numbers nobody can reproduce is a page whose numbers age without anybody
noticing. The rewrite, carrying holes three, four and five from above by
reading them off this page, promptly invented a ninth of its own.*

**What it reported.** Three `pub fn` in `alo-access` with no caller and no test:
`a_delay_of` in `key_filter.rs`, `after_the_last` in `reaching.rs`,
`has_a_voice` in `voices.rs`. This page says `alo-access` had **one**, and that
one — `ignore_the_same_key_for` — was fixed in the change that wrote this page.

**All three are false, and every one of the seven call sites is inside an inline
`#[cfg(test)]` block in the function's own file:**

| | caller lines | the file's `#[cfg(test)]` span | inside |
|---|---|---|---|
| `a_delay_of` | 111, 115, 119, 122 | 84–124 | all four |
| `after_the_last` | 276 | 231–282 | yes |
| `has_a_voice` | 89, 91 | 66–108 | both |

**The mechanism, and it is a swap of two correct steps.** The rewrite strips
`#[cfg(test)] mod` blocks out of the product text, which is right — a test is
not a product caller, and hole two above is that tests were counted as callers.
But it **discards** the stripped text rather than adding it to what is counted
as *test* usage. So a function whose only callers are in its own file's test
module is counted in neither place, and comes out as *no product caller **and**
no test* — which is the exact conjunction that defines an orphan here.

**Two right ideas, applied in an order that cancels them.** Hole two fixed
over-counting by excluding tests from callers. This hole is the same fix applied
once too often: excluded from callers *and* from tests. The sweep that produced
this page's 44 did not have it, which is why its `alo-access` figure was 1 and
correct.

**What made it visible, and it was not care.** Two independent signals
disagreed with the run:

1. The first run of the rewrite **refused** — *no `.rs` files under
   `crates/alo-terminal/src`* — because it was pointed at the repository's main
   checkout, which was **222 commits behind** `origin/main` and predates that
   crate. A guard written for a different reason caught the wrong subject.
2. On that stale tree the rewrite reported `ignore_the_same_key_for` as an
   orphan — a function **this page records as fixed**. A result that contradicts
   a written record is worth more than a result that agrees with nothing, and
   it is what sent the run to a worktree at `origin/main`.

Neither signal was a test of the sweep. Both were accidents of it being wrong
about something else first, which is the uncomfortable part: on a current tree,
with the crate present, the three false orphans would have read as a finding.

**Why this matters more than three names.** The section above proposes a gate
built on this method. **An instrument that falsely accuses three functions in
one eleven-file crate would have failed three honest builds**, and the failure
would have read as the author's regression rather than as the gate's hole — the
same shape as a peer lane's `grep -c … || echo "no such file"` the same evening,
where *zero matches* and *no such file* were reported identically and the
collapsed answer was the more damning one.

**So the gate's subject is not settled, and that is the finding to act on.** Two
implementations of *what nothing calls* disagree by three in one small crate,
and the disagreement is in the definition rather than the arithmetic: whether a
caller inside the callee's own `#[cfg(test)]` block counts as a test, as a
caller, or as neither. The answer is *as a test* — it is what hole two is about
— but a gate encodes whichever answer its author happened to implement, silently.
**Settle the definition, in prose, before writing the gate**, and keep the
instrument in the repository so the next reader argues with code rather than
rewriting it.

## The gate exists, and triage by sampling found three false findings in five

*2026-10-21. The section above asked for the definition to be settled in prose
and the instrument kept in the repository. Both are done:
`crates/alo-reached` is the sweep as a crate, and
`crates/alo-reached/what-nothing-calls.txt` is its committed snapshot.*

**This measurement runs forever.** `every_pub_fn_is_reached_or_accepted` is a
test in CI, and it fails two ways: a `pub fn` nothing reaches that is not in the
snapshot, and a name in the snapshot that is now reached. The snapshot may
shrink silently and may not grow silently. Regenerate with:

    UPDATE_REACHED_SNAPSHOT=1 cargo test -p alo-reached

The hand sweep above, and the triage described below, each ran **once**.

### The definition, settled

A `pub fn` is reached if its name appears anywhere other than its own
declarations, counting:

| | |
|---|---|
| product code outside any `pub fn` body | always |
| a `pub fn`'s body | only while that function is itself reached |
| `*_tests.rs`, `tests/`, and `#[cfg(test)]` blocks | as a **test**, which counts |
| `examples/` | as a **test**, which counts — see the hole below |
| comments, `use` lines, `pub(crate)` and private declarations | never |

### What triage found, and why the first number was wrong

The first run answered **69**, against this page's **44**. Rather than report
the difference, five entries were sampled and checked by hand. **Three of the
five were false**, which would have failed three honest builds:

- `Bounds::reaching_nothing_but` — called in product code at
  `crates/alo-agentd/src/bounding.rs:214`.
- `Nested::submit_with_approval` and `Nested::pump_sign_in` — called from
  `crates/alo-shell/examples`.

Two faults, and the first is the one worth remembering. **The fixed point was
dropping whole files**: a file was dead once every `pub fn` in it was unreached,
and since `pub(crate)` and private functions are deliberately not counted as
public surface, a file holding one dead `pub fn` and twenty live private ones
was dropped entire — and every call those twenty made stopped counting, over
twelve rounds, cascading.

**The unit that dies is a function body, not a file.** What makes that sound is
a guarantee `rustc` already gives and this page had not used: `dead_code` is
warn-by-default and the gate runs `clippy -D warnings`, so a private or
`pub(crate)` function nothing reaches **fails CI today**. Verified rather than
assumed — an unused private `fn` added to `alo-reached` produced
`error: function ... is never used`. Only a `pub fn` can be dead and unnoticed,
and those are exactly the ones the snapshot names.

The second fault was that `examples/` was not read at all, though
`clippy --all-targets` compiles all 49 of them.

### The figures, and the baseline that was itself retracted

| | |
|---|---|
| first run, file-sized death, no examples | **69** |
| after both fixes | **39** |
| this page's corrected hand figure | **40** |

**39 matches the hand sweep in every crate with two or more entries, exactly**:
`alo-shell` 18, `alo-dock` 3, and two each in `alo-adapters`, `alo-corridor`,
`alo-tasks`, `alo-agentd`, `alo-models` and `alo-notifying`. It differs by one
in the singletons, and the tree has moved by a day.

**The 44 was the wrong baseline, and this page says so above.** It is this
page's *first* figure, superseded by **40** four paragraphs later in the same
document. The gap being explained was 69 against 44, and it decomposes as
**30 of real bug** (69 down to 39), **4 of retracted baseline** (44 down to 40)
and **1 of a day's drift** — so the arithmetic only closes once the baseline is
the one the page still holds. *Before explaining a disagreement with a
document, check whether the document still holds the figure being disagreed
with.*

A second sample of five, after the fixes, found **five true findings out of
five**.

### The hole, named rather than discovered

**A function reached only by an example passes this gate.** An example is
compiled, so calling it prose would be false; but it is a developer's check and
not a road a person walks, so a function only an example reaches has no road at
all. `Nested::submit_with_approval` and `Nested::pump_sign_in` are in exactly
that position today. This gate answers *does anything call it*. The owner's
test — mechanism, caller, a person can reach it, somebody has seen it — asks
two more questions than that, and no reference count will answer either.
