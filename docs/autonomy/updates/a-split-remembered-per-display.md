# A split remembered, per display — and a blocker that had been stale for two days

**Date:** 2026-09-20
**Workstream:** v0.5 — hands on the desktop
**Task:** 2, *A split remembered, per display*.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3 with 8 GB unified memory**, macOS 26.5.2; built, linted
and tested in the Lima VM on that Mac — **Ubuntu 24.04.4 aarch64, 6 CPUs, 4 GB**.
Nothing here is ticked *on the machine*: this crate decides and draws nothing.
**Egress:** none.

## The blocker was stale, and that is worth recording first

Task 2's status said **blocked** on the session plan's task 3, whose
`alo-displays` gives a screen the stable identity a remembered split is kept
under. That task was marked **Done, 2026-09-18**. Task 1, its only dependency,
was done on 2026-09-17.

So the task had been takeable for two days, and every machine surveying these
plans read it as untakeable — including the supervisor, which will not select a
task whose status word is `blocked`. **A blocker outlives its cause silently**,
and the cost is not that somebody does the wrong work: it is that nobody does the
right work, invisibly.

The first commit here moves the line and names what cleared it and when.

## What is remembered

**Applications and shares. Never a window's title, never a document's name**
(ADR 0038).

A `WindowId` could not be what is kept, and that is the whole reason this module
exists: it is what the compositor calls a window *this time*. Close the window
and it is gone; open it again and the number is different. Remembering it would
remember nothing — and would pass any test that never closed anything.

## The tree, not the rectangles

A division is a tree of cuts, and that is what is kept: each cut's direction and
where it falls, with each leaf naming an application.

Keeping the rectangles instead would restore four windows to four positions that
happened to tile, and the first time a screen came back at another size they
would overlap or leave a gap. **A cut re-cut still divides; a rectangle
remembered does not.** The test asserts this rather than arguing it: a division
restored on a 1280×720 screen covers it **exactly**, no gap and no overlap.

## This crate does not learn how applications are named

The plan lets `alo-dividing` read `alo-displays` and **not** `alo-applications`.
So `HeldBy` is a name handed in — checked as a key, never interpreted — the way
`alo-playing` is handed a machine and `alo-access` is handed a folder. The shell
hands the name down; this crate decides.

Screens are keyed the same way, by a string the caller supplies from
`alo_displays::Identity`. That type is another crate's and is not serialisable,
and making it so would be editing a crate this plan may only read.

## One field removed for being unread

`OnADisplay` first held the display's area at the time. `restored()` is handed
the display as it is **now** — the only size that can matter — so yesterday's was
a field nothing read and a second answer to a question already answered. It is
gone. It also would have needed `Deserialize` on `Area`, which is validated
through `Area::of`; deriving it would have let a settings file build an invalid
`Area` by going around the check.

## Evidence — tests this task publishes

`crates/alo-dividing/tests/a_split_remembered_per_display.rs`, **5 tests**, each
throwing the window numbers away and asking for new ones:

| | |
|---|---|
| `returning_to_a_pair_of_windows_gives_back_the_division` | divide, close both, reopen as **4001 and 4002** instead of 1 and 2 — mail returns to the share it had, under a number it never had |
| `two_screens_divide_independently` | what one screen holds does not reach the other |
| `an_unplugged_display_keeps_its_divisions` | nothing forgets a screen for going away; it comes back at 1280×720 and still divides |
| `a_division_restored_on_a_smaller_screen_still_shares_it` | the shares cover the screen **exactly** — no gap, no overlap |
| `nothing_a_person_named_reaches_the_file` | the written bytes are read back, and fail if a window, a title or a document ever reaches them |

`crates/alo-dividing/src/remembering.rs` adds 7 unit tests and
`src/keeping.rs` 4 more, including a settings file that is there and is not these
settings being refused rather than half-read (ADR 0016).

`alo-dividing` in full: **58 tests**, every one of task 1's unchanged.

## What this does not do

Nothing draws. The agent's *arrange* verb is untouched: when it proposes a
division it will go through `Division` like any other change and be approved like
any other change, and this task gives it no second road to place windows.
