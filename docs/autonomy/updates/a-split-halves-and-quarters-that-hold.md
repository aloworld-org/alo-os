# A split: halves and quarters that hold

**Date:** 2026-09-17
**Workstream:** v0.5 — hands on the desktop (`docs/autonomy/v0-5-hands-on-the-desktop-plan.md`, task 1)
**Contributor:** Claude Code worker in `C:\dev\alo-os-b`, for the repository owner
**Status:** ready for integration

## What changed

A new crate, `crates/alo-dividing`, decides how one display is divided between
windows. It draws nothing; the shell's later tasks draw what it decides.

**In words a person outside this repository can read:** alo OS can now divide a
screen into halves and quarters that stay a pair. Drag a window to an edge and an
outline shows the half it would take; drag it to a corner and the outline shows a
quarter; let go somewhere else and nothing has moved. Resize the boundary between
two windows and both change size together, with no overlap and no gap. Close a
window and its neighbour takes its space. A window that cannot be made as small as
its share is never squeezed: the screen is left as it was, and the person is told
why.

| File | Responsibility |
|---|---|
| `src/area.rs` | Points, sizes and rectangles in logical units |
| `src/scale.rs` | Logical units into pixels, edge by edge, so neighbours still meet |
| `src/side.rs` | Which way a share is cut, and which side a window goes on |
| `src/window.rs` | A window as a division knows it: an id and a minimum size, never a title |
| `src/share.rs` | One window's share, laid out |
| `src/node.rs` | The tree of shares, and the arithmetic that moves a boundary |
| `src/division.rs` | One display's division: reading, halving, moving a boundary, closing |
| `src/dropping.rs` | Drag to an edge or a corner: a proposal before a commit |
| `src/keyboard.rs` | The focused share divided with the next window, through `alo-shortcuts` |
| `src/place.rs` | What a half, a quarter or a part is called |
| `src/refusing.rs` | Why a division was left as it was |
| `src/words.rs` | The crate's 15 strings, each with a translator's note |

Registration: the crate is a workspace member, and its words are collected by
`crates/alo-saying/src/collecting.rs` (52 → 53 lists).

## Decisions

1. **A division is a tree, and a cut is stored as one length.** Every cut hands its
   area to two pieces that meet at one whole logical unit, so shares tile the
   display by construction. The boundary between a pair is that one number;
   moving it moves both. Pieces inside either side that are cut the same way keep
   their proportion, and are moved off proportion only as far as a window's
   minimum demands.
2. **Exact halves, refused rather than squeezed.** A split is an exact half (first
   piece rounded down). If either window's minimum does not fit, the split is
   refused naming the window — the acceptance asks for a refusal, and a clamped
   "almost half" would hide the reason. A boundary move is all or nothing: it does
   not stop part of the way.
3. **Decided, then applied.** Every change is worked out on a copy of the tree, so a
   refusal always leaves the division byte-for-byte as it was. The random walk
   test holds this after every refused step.
4. **Edges propose a half of the display; corners halve the share in that corner**
   along its longer axis. On a display already in halves that gives the corner's
   quarter. On a single undivided window a corner can only honestly give a half
   (a quarter would leave three quarters that are not a rectangle), and the
   proposal then says *Left half* because its place is read off the real area.
5. **The first division needs a second window.** An empty display has no shares,
   and a half with nothing in the other half would be the gap this task rules out.
   So a drop on an undivided display names the window in front, and a keyboard
   split names the next window; with none, both say *there is no other window open
   to share the screen with*.
6. **A proposal is a value.** `propose_drop` returns the whole would-be division;
   the division is untouched until `commit`, which takes the proposal by value
   (one commit per proposal) and refuses it with `Refused::Changed` if the
   division changed since, or if it came from another display's division.
   Abandoning is dropping the value.
7. **The keyboard split answers to `alo-shortcuts`' existing `SnapLeft` and
   `SnapRight`** (*Put the window on the left/right half*). This plan reads that
   crate and never edits it, and those actions already mean what the split does.
   `side_bound_to(shortcuts, chord)` asks the person's own shortcuts, so a rebound
   or cleared *left half* moves or removes the split. The v0.5 addition is what
   fills the other half: the next window. A floating focused window over a divided
   display is refused (*drag it to an edge*) rather than rearranging windows the
   person was not looking at.
8. **Pixels are rounded per edge, never per size**, in 120ths of a pixel (the
   Wayland fractional-scale unit), so two shares meeting at one logical boundary
   meet at one pixel at 125 % or 150 % too.
9. **No sentence names a window.** A division knows windows only by compositor id.
   `Refused::window()` tells the shell which window to mark beside the sentence.
10. **Drop zones:** 16 logical units from an edge proposes a half; within 96 of
    both edges of a corner proposes a quarter. Both are public constants
    (`NEAR_AN_EDGE`, `NEAR_A_CORNER`), so the shell and a later setting can use them.

No ADR was needed: nothing here contradicts one, weakens a gate, or narrows
`docs/features.md`.

## Acceptance criteria and evidence

All in `crates/alo-dividing/tests/`:

| Criterion | Test |
|---|---|
| A tree of shares: halves, quarters, a split of an existing half, each naming its window | `halves_and_quarters_that_hold::a_division_is_halves_quarters_and_a_split_half_each_naming_its_window` |
| Resizing the boundary between two shares resizes both | `halves_and_quarters_that_hold::resizing_the_boundary_between_two_shares_resizes_both` |
| No share ever overlaps another or leaves a gap | `a_division_never_overlaps_or_leaves_a_gap::no_share_overlaps_another_or_leaves_a_gap_whatever_is_done_to_it` — 5 000 seeded steps across every operation, including refusals, checked in logical units and in pixels at 100/125/150/200 % |
| Edge proposes a half, corner a quarter, shown before commit, abandonable | `halves_and_quarters_that_hold::an_edge_proposes_a_half_a_corner_a_quarter_and_either_can_be_abandoned` |
| Keyboard split of the focused share with the next window, bound through `alo-shortcuts` | `halves_and_quarters_that_hold::a_keyboard_split_divides_the_focused_share_with_the_next_window_through_the_shortcuts` |
| A closed window gives its share to its neighbour | `halves_and_quarters_that_hold::a_window_closed_inside_a_division_gives_its_share_to_its_neighbour` |
| A window larger than its share is not squeezed; the division refuses and says why | `halves_and_quarters_that_hold::a_window_larger_than_its_share_is_not_squeezed_and_the_refusal_says_why` |
| Constraint: logical units, the same on a scaled screen | `halves_and_quarters_that_hold::a_division_means_the_same_thing_on_a_scaled_screen` |

Refusal paths also have unit tests beside each operation (`src/division.rs`,
`src/dropping.rs`, `src/keyboard.rs`, `src/refusing.rs`): no neighbour on a display
edge, a window with no share, a move stopped by a window deep inside the far side,
a stale proposal, a proposal from another display, nothing to share with, a floating
focused window.

## Verification

Executed on 2026-09-17:

- Windows host: `cargo test -p alo-dividing` — 33 unit, 1 + 7 integration, 1 doc test,
  all pass. `cargo clippy -p alo-dividing --all-targets -- -D warnings` clean.
- WSL Ubuntu (`CARGO_TARGET_DIR=/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`):
  - `cargo fmt --all -- --check` — clean;
  - `cargo clippy -p alo-dividing -p alo-saying --all-targets -- -D warnings` — clean;
  - `cargo clippy --workspace --all-targets -- -D warnings` — clean;
  - `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-dividing --no-deps` — clean;
  - `cargo test -p alo-dividing -p alo-saying -p alo-collected` — all pass (the last
    confirms the new crate's words are collected).

Not run by this worker, by instruction: the full workspace test suite (the
supervisor runs it). Not done, and not claimable here: **nothing has been dragged
on a real machine.** There is no compositor drawing a division yet; the physical
check belongs to the shell plan's task that honours this crate.

Noted in passing: on the Windows host, `cargo clippy -p alo-saying` reports
dead-code errors in `crates/alo-converting/src/inventory/` that are not in this
change and do not appear on Linux, where the gates run. Left alone as outside this
plan.

### Second attempt: the list count after rebasing

The supervisor refused the first handoff at `clippy, warnings denied`:
`EVERY_LIST` in `crates/alo-saying/src/collecting.rs` was declared `[&str; 48]`
but held 49 names. The work was written against a `main` with 47 lists; the
locking task (`alo-locking`) landed on `main` meanwhile and took the count to 48,
so after the rebase both changes had added one entry and the declared length
counted only one of them. The fix is the two lengths alone — `EVERY_LIST` and the
test table `ONE_STRING_EACH` both go to 49. No logic changed.

Re-run on 2026-09-17 in WSL Ubuntu after that fix: `cargo fmt --all -- --check`
clean; `cargo clippy --workspace --all-targets -- -D warnings` clean;
`cargo test -p alo-dividing` (33 unit, 1 + 7 integration, 1 doc) and
`cargo test -p alo-saying` (63 unit, 4 integration, 1 doc) all pass.

### Third attempt: the by-hand check after the screen verb landed

The supervisor then refused the combined tree at `the workspace's tests`, in
`crates/alo-by-hand/tests/every_verb_can_be_done_by_hand.rs`. Nothing in
`alo-dividing` was named. While this task waited, `main` gained
`alo-capturing`'s `picture_of_the_screen` verb and its entry in
`docs/by-hand.md`, but the by-hand check was never handed `alo-capturing`, so it
found a crate declaring verbs nobody handed in and an answer about a verb it
could not see.

The fix registers the crate rather than weakening the check. `alo-capturing` is
now a dev-dependency of `alo-by-hand`, is on `WHO_DECLARES_THEM` (10 crates) and
declares into `what_this_machine_ships`. Once the verb was visible, the check
found one more real thing: the entry quoted a promise
(`Capture: screenshots, annotation, …`) that `docs/features.md` no longer says.
It now quotes the two promises as they stand: *Screenshots: whole screen, one
window, a selected region — to a file or the clipboard* and *Annotate a
screenshot without opening anything else*. No promise was changed.

Re-run on 2026-09-17 in WSL Ubuntu: `cargo fmt --all` clean; `cargo clippy
--all-targets -p alo-by-hand -p alo-dividing -p alo-saying -p alo-capturing --
-D warnings` clean; `cargo test -p alo-by-hand` (27 unit, 13 integration),
`-p alo-dividing` and `-p alo-saying` all pass.

### Fourth attempt: conflict markers left by a rebase

The supervisor refused the next handoff at `formatting`, twice: `cargo fmt`
printed its usage and failed. The cause was in the tree, not the machine. The
held work (`wip(dividing) … held while the disk was full`) had been reapplied over
a `main` that had since gained `38ccba8`, which made the third attempt's by-hand
fix itself, and the reapplication left unresolved `<<<<<<<`/`>>>>>>>` markers in
`crates/alo-by-hand/Cargo.toml`,
`crates/alo-by-hand/tests/every_verb_can_be_done_by_hand.rs` and
`crates/alo-saying/src/collecting.rs`. None of the three would parse.

The resolution is the smallest one available. The by-hand crate and
`docs/by-hand.md` are returned to `main` exactly: `main` already registers
`alo-capturing` and quotes the Screenshots promise, so this change no longer
touches them, and the earlier *Annotate a screenshot* addition, written against a
`main` that did not have that fix yet, is dropped. In `collecting.rs` the markers
are resolved to `main`'s side, and both `EVERY_LIST` and `ONE_STRING_EACH` go from
52 (on `main` now) to 53 for `alo-dividing`. No logic changed.

Re-run on 2026-09-17 in WSL Ubuntu: `cargo fmt --all --check` clean; `cargo clippy
--all-targets -p alo-dividing -p alo-saying -- -D warnings` clean; `RUSTDOCFLAGS="-D
warnings" cargo doc -p alo-dividing --no-deps` clean; `cargo test -p alo-dividing`
(33 unit, 1 + 7 integration, 1 doc), `-p alo-saying` (63 unit, 4 integration,
1 doc), `-p alo-collected` and `-p alo-by-hand` (27 unit, 13 integration) all pass.

## Limitations

- A division has a fixed display area. What happens when a display's resolution or
  scale changes, and remembering a division per display, is task 2 (blocked on
  `alo-displays`).
- A keyboard split divides side by side only, because `alo-shortcuts` has no
  *top half*/*bottom half* actions and this plan does not edit it. A drag can
  divide one above the other.
- `alo-shell`'s v0.01 `window_tiling` is unchanged; replacing it is the shell
  plan's.

## Proposed updates for the integration owner

- **CHANGELOG.md:** "Dividing the screen: halves and quarters by drag or keyboard
  that stay a pair — resizing the boundary resizes both windows, a closed window's
  neighbour takes its space, and a window is never squeezed below its minimum size
  (`alo-dividing`; nothing drawn yet)."
- **ROADMAP.md, v0.5 ★ Divide the screen:** the code for *halves and quarters by
  drag or keyboard, splits that hold while you work* exists; *remembered, per
  display* waits on task 2. Nothing is ticked *on the machine*.
- **QUEUE.md / STATE.md:** hands-on-the-desktop task 1 done; task 2 still blocked
  on the session-and-displays plan's task 3; tasks 3, 4 and 6 ready.
