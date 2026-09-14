# The ordinary desktop, drawn: the dock, its status area, and what is running and filling the disk

**Date:** 2026-09-14
**Workstream:** v0.5 — the shell (`docs/autonomy/v0-5-the-shell-plan.md`, task 5)
**Responsible contributor:** Claude worker in `C:\dev\alo-os-shell`, for the owner
**Status:** ready for integration — the code. Not seen on a certified machine.
The clock, battery, network and volume are **not** drawn; they are task 7 in the
plan, blocked on crates that do not exist (see *Findings*).

## What changed

The shell has a desktop to put everything else in. Every session frame drawn
with `Nested::submit_with_desktop` carries a dock on the edge `alo-dock` names —
laid out for that display's own size, text size and reading direction — with
the person's accent along its inside edge and a status area at its far end, from
which the egress indicator's lines grow. Over the room the dock leaves, a person
can have two windows open, both drawn from `alo-measuring`: **what is running**
(every process, what each is using, what ended) and **what is filling the disk**
(a folder as a tree of sizes that open up). Light and dark, the accent and the
text size are whatever the person's `alo_appearance::Appearance` answers at that
moment; terracotta is never offered. The record window and a waiting question
travel in the same frame, above the desktop, and the indicator above them all.

| File | What it is |
|---|---|
| `crates/alo-shell/src/desktop_look.rs` | `DesktopLook` (made only from an `Appearance` at a time of day), `DesktopPalette` (accent only through `Accent::of_colour`), `Measure` |
| `crates/alo-shell/src/dock_raster.rs` | The dock on one display: band, accent rule, status area, from `Dock::layout_on` |
| `crates/alo-shell/src/desktop_list.rs` | A panel of rows in one region: remarks, columns, whole rows or wait, rail, open/closed mark in shapes, accent selection bar, refusal |
| `crates/alo-shell/src/desktop_list_tests.rs` | The panel's layout and refusals |
| `crates/alo-shell/src/running_window.rs` | `RunningWindow`, `RunningShows`, `RunningPressed`: two readings and the interval, read again, the view |
| `crates/alo-shell/src/running_rows.rs` | `Running` as rows: every number as its digits or `alo-measuring`'s sentence |
| `crates/alo-shell/src/running_keys.rs` | `RunningKey`: Up, Down, Home, End, F5, Escape, nothing else |
| `crates/alo-shell/src/running_window_tests.rs` | The window on a written kernel and on this machine's `/proc` |
| `crates/alo-shell/src/filling_window.rs` | `FillingWindow`, `FillingShows`, `FillingPressed`: count, open, shut, count again |
| `crates/alo-shell/src/filling_rows.rs` | `Holding` as rows: the tree with open folders open, sizes as counted |
| `crates/alo-shell/src/filling_keys.rs` | `FillingKey`: opening follows the reading direction |
| `crates/alo-shell/src/filling_window_tests.rs` | The window on folders on a real disk |
| `crates/alo-shell/src/desktop_raster.rs` | The whole desktop for one display; the room the dock leaves, shared by two windows |
| `crates/alo-shell/src/desktop_raster_tests.rs` | Never terracotta; light and dark as `alo-appearance` says; no window covers the dock |
| `crates/alo-shell/src/desktop_paint.rs` | Validating and painting the desktop |
| `crates/alo-shell/src/desktop_seat.rs` | `Server::running_key`, `Server::filling_key`: keys through the seat, intercepted |
| `crates/alo-shell/src/nested_desktop.rs` | `DesktopFrame`, `Nested::submit_with_desktop`, `pump_running`, `pump_filling` |
| `crates/alo-shell/src/nested_desktop_tests.rs` | The status area holds the indicator; a frame refused whole |
| `crates/alo-shell/src/desktop_testing.rs` | A written kernel, an appearance, a folder of documents |
| `crates/alo-shell/tests/desktop_source.rs` | The shape promises, read from the shipped source |
| `crates/alo-shell/examples/desktop_check.rs` | The WSLg measurement |
| `docs/contracts/native-desktop.md` | The new public surface |

Touched to make room: `scene_native.rs` (`NativeLayers` gains a `desktop`
layer), `scene_drawing.rs` (validates it and paints it after controls, before the
record window), `nested.rs`, `offscreen.rs`, `nested_approval.rs` and
`nested_record.rs` (no desktop layer), `presentation.rs`
(`RenderError::DesktopScene`, `RenderError::AccentRefused`), `lib.rs`,
`Cargo.toml` (`alo-measuring` as a dependency), `Cargo.lock`.
`docs/autonomy/v0-5-the-shell-plan.md` marks task 5 done and adds task 7.

**User-readable change description:** *alo OS now has a desktop. The dock sits
on the edge of the screen you chose, sized for each screen it is on, in your
accent colour, with a status area at its far end — and that is where you see
anything leaving the machine. Two windows show what your machine is doing: what
is running and what each program is using, and what is filling a folder as sizes
you can open up. Every number is the one the kernel or the disk gave, never
rounded or estimated, and where there is no number the window says why. The
desktop turns light and dark with your appearance settings, and the agent's
terracotta is never used for anything else. Nothing on the dock or in either
window can grant, approve, delete or stop anything.*

## Decisions, and why

- **A look can only be made from an `Appearance`.** `DesktopLook::of(&appearance,
  now, reading)` is the one constructor, so *light and dark as `alo-appearance`
  decides it, never as this crate decides it* is structural. The mapping from a
  scheme to the design brief's tokens (cream/porcelain/navy, charcoal/cream) is
  drawing, and is a `match` on the scheme the appearance answered; a source test
  holds `Scheme::Light`/`Scheme::Dark` to match arms in that one file.
- **Terracotta is refused at the door `alo-appearance` already built.** The
  palette takes the accent through `Accent::of_colour`, which refuses terracotta
  as `AccentError::Reserved`; a look that somehow carried it refuses the frame
  (`RenderError::AccentRefused`). Tested three ways: the palette refusal, every
  pixel of a full desktop across all five accents, both schemes and both
  directions, and a source test that no desktop file names terracotta or makes a
  colour.
- **Per display means asked per display.** The dock is laid out by
  `Dock::layout_on` for each display's own size on every frame; a laptop and a
  large screen get different thicknesses and names that give way on one and not
  the other. There is one edge for every display because that is what `alo-dock`
  holds (a finding).
- **The status area is a segment of the band, and the indicator grows from its
  corner.** The band's far-end segment (twice the band's thickness) is set off by
  a rule. The indicator keeps task 2's placement — lines stacked out from the
  far corner, clear of the dock — and `submit_with_desktop` lays it out from the
  **same** `Dock` as the band, so the two cannot disagree. A test on every edge,
  both directions and two display sizes holds the first line's far end inside
  the status area's span and every line clear of the band.
- **Two readings and the host's interval.** The running window holds no clock
  and takes no reading: it is opened with two and read again with one, and the
  interval is the host's, as `alo-measuring` requires of its callers. F5 answers
  `RunningPressed::ReadAgain` for the host to take one.
- **The filling window counts when it opens and when asked, never otherwise.**
  Opening a folder only changes what is in view; what is open is kept by path,
  so counting again leaves the same folders open.
- **Nothing is reordered.** Processes are in `Running`'s order (process id) and
  a folder's contents in `Holding`'s (the order a person reads a folder). Sorting
  by use or size would be the more useful view, and it is also the drawing crate
  deciding what comes first; with no column names to sort by (a finding), it
  waits.
- **Numbers are digits, not grouped.** How `53248000` is grouped for a reader is
  regional and no crate decides it yet (task 4's date finding, again). A source
  test allows exactly four conversions to text — a `Number`'s value, two process
  ids and a node's size — only in the two row files, which may not scale, round,
  sum, sort or filter.
- **One panel shape for both windows (law 4).** `desktop_list.rs` sets out rows
  of cells in columns as wide as the widest cell in view, wraps a cell that does
  not fit rather than cutting it, draws a row whole or waits, and marks what is
  out of view with a rail shape. The two windows differ only in the rows they
  hand it.
- **The accent is spent twice**: along the dock's inside edge and on the
  selected row of the filling window. Both measured against the grounds by
  `alo-appearance`'s own tests.
- **Two windows share the room along its longer side**, the running window on
  the side a person starts reading from; a room too small for either refuses the
  frame (`RenderError::DesktopScene`) rather than drawing a cut row.

## Findings

- **The status area holds no clock, battery, network or volume.** No crate
  measures a battery or a network's state or owns a volume, and none decides how
  a time is written for a region. The plan forbids deciding these in a drawing
  crate, so they are **task 7** in the plan, `blocked` on those crates, with its
  own acceptance. `docs/features.md` is unchanged; the promise stands and is
  tracked. **Proposed:** a measuring crate for power and the network's state, an
  audio crate owning the volume, and a regional time format in `alo-strings`
  (or a crate of its own).
- **Nothing is in the dock.** No crate decides what a dock holds — pinned
  applications, a launcher, running windows. **Proposed:** a crate of its own,
  or `alo-dock`, declares the dock's items; `alo-applications` already answers
  what is installed.
- **Nothing opens either window**, as with the record window: `alo-shortcuts`
  declares no action for them and the dock holds nothing to click. The doors
  exist (`RunningWindow::opened`, `FillingWindow::opened`). **Proposed:**
  `alo-shortcuts` actions for the record window, what is running and what is
  filling the disk, and dock items for them.
- **No column names, units or window titles.** `alo-measuring` declares the
  sentences that stand in for a number but not the words for *memory*,
  *processor*, *read* or *bytes*, so the numbers are drawn unlabelled in a fixed
  column order (documented in the contract). **Proposed:** `alo-measuring`
  declares column names and titles.
- **One edge for every display.** `alo-dock` has no per-display exception to
  the edge (`docs/features.md` v0.5 *per display*). **Proposed:** `alo-dock`
  gains it the way `alo-appearance` made a display an exception to a
  background; the shell already asks per display.

## Acceptance criteria and evidence

| Criterion (plan, task 5) | Test |
|---|---|
| A dock drawn on the edge `alo-dock` names, per display | `alo-shell lib dock_raster::tests::a_dock_is_drawn_on_the_edge_alo_dock_names_with_the_status_area_at_its_far_end`; `alo-shell lib dock_raster::tests::each_display_gets_the_dock_laid_out_for_its_own_size` |
| A status area at its far end holding the egress indicator of task 2 | `alo-shell lib nested_desktop::tests::the_status_area_at_the_far_end_of_the_dock_holds_the_egress_indicator` |
| … holding the clock, battery, network and volume | **Not done** — plan task 7, blocked (see *Findings*) |
| The accent is `alo-appearance`'s and terracotta is never offered — a test refuses a palette that offers it | `alo-shell lib desktop_look::tests::a_palette_that_offers_terracotta_is_refused`; `alo-shell lib desktop_raster::tests::no_pixel_on_the_desktop_is_terracotta`; `alo-shell lib dock_raster::tests::the_dock_carries_the_persons_accent` |
| A window from `alo-measuring` shows what is running and what is using the machine, each number the kernel's | `alo-shell lib running_window::tests::every_number_in_the_running_window_is_the_one_the_kernel_gave`; `alo-shell lib running_window::tests::the_running_window_reads_this_machines_kernel` |
| A second shows what is filling the disk as sizes that open up, each number the count's | `alo-shell lib filling_window::tests::what_is_filling_the_disk_is_drawn_as_sizes_that_open_up` |
| Everything follows light and dark as `alo-appearance` decides it | `alo-shell lib desktop_look::tests::the_desktop_follows_light_and_dark_as_alo_appearance_decides_it`; `alo-shell lib desktop_raster::tests::the_whole_desktop_turns_dark_when_alo_appearance_says_so`; `alo-shell desktop_source light_and_dark_are_alo_appearances_and_terracotta_is_never_offered` |
| Constraint: the dock holds no authority | `alo-shell desktop_source the_dock_grants_approves_and_revokes_nothing`; `alo-shell desktop_source neither_window_acts_on_what_it_shows` |
| Constraint: this surface adds no number of its own | `alo-shell desktop_source the_desktop_adds_no_number_of_its_own` |

Refusal paths tested beside those:

- Same reading twice (`SameMoment`), no interval (`NoInterval`) and an
  unreadable kernel (`Unreadable`, naming `/proc`) are `alo-measuring`'s
  sentence in the running window, never an empty list; a failed read-again is
  refused and the next good one recovers.
- A folder that is not there and a file named as a folder are
  `alo-measuring`'s refusal in the filling window; a folder deleted before
  counting again becomes the refusal; a folder the machine would not read has
  `alo-measuring`'s sentence beside its size (as root the folder is read and the
  size is whole, and the test says so).
- A withheld number is the kernel's sentence and never `0`; a process that
  began since the last reading says so; one that ended is drawn as ended.
- Closed windows do nothing with any key or reading; the view stops at both
  ends; a selection past a smaller count comes back to the last row.
- No key stops a process or deletes a file; a chord does nothing; a held key
  means something once; strange key codes and a keyboard-less display are
  refused.
- A display `alo-dock` refuses, one larger than 16 384 pixels, a room too small
  for an open window, a region outside the display, and sentences that do not
  fit all refuse the frame rather than cutting it; the dock is still drawn when
  no window is open.
- A desktop frame whose indicator was never told is refused whole.
- A palette asked for terracotta, a ground colour or an invented colour is
  refused in `alo-appearance`'s own words.

The tests were checked against one deliberate mutation — the running rows
drawing `value / 1000` and the dock's accent rule painted in ink. Four tests
failed (`every_number_in_the_running_window_is_the_one_the_kernel_gave`,
`the_running_window_reads_this_machines_kernel`,
`the_dock_carries_the_persons_accent`,
`the_whole_desktop_turns_dark_when_alo_appearance_says_so`); the mutation was
removed.

## Verification

Executed on 2026-09-14, Windows 11 host, Ubuntu under WSL 2,
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-shell-cd217193b5311c25`:

- `cargo fmt --all -- --check` — clean.
- `cargo clippy -p alo-shell --all-targets -- -D warnings` — clean. No other
  crate depends on `alo-shell`, and no other crate was changed.
- `cargo test -p alo-shell` — lib 317 passed (37 new), `approval_source` 5,
  `client_lifecycle` 262, `desktop_source` 4 (new), `egress_status_source` 3,
  `record_source` 5, `sign_in_source` 4, `socket_ownership` 3, doc tests passed;
  0 failed.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-shell --no-deps`, with and
  without `--document-private-items` — clean.
- Each evidence test above run on its own with `--exact` — 1 passed each.
- `WAYLAND_DISPLAY=wayland-0 XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir timeout 600s
  …/examples/desktop_check` under WSLg — exit 0. On all four edges, light at nine
  and dark at eight in the evening, read both ways, it drew:
  - the dock alone;
  - the dock with a question leaving;
  - what is running — 66 processes read from this machine's `/proc`;
  - what is running, read again — 58 running and 9 ended;
  - what is filling a folder — 41 500 bytes in `Documents`, one folder open;
  - a folder opened — two open;
  - both windows at once;
  - a folder that is gone — `alo-measuring`'s refusal in the window.

  134 frames were submitted.

Not run: the full workspace suite (the supervisor runs it).

## Limitations

- **Not seen on a certified machine.** No direct-display (DRM/KMS) submission
  carries the desktop yet.
- **The status area is empty but for the egress indicator** — task 7.
- **Nothing in the dock, nothing opens the windows, no labels** — see
  *Findings*.
- **Keyboard only.** No pointer, so *click through* is keys through, for now.
- **No screen-reader tree.** Pixels; AT-SPI is v0.5 accessibility work.
- **Shaping cost.** Rows in view are shaped every frame, as the record window's
  entries are; measured acceptable under the debug build in WSLg, not profiled.

## Proposed changes to shared documents

**CHANGELOG.md** — *alo OS has a desktop: a dock on the edge you chose, sized for
each screen, in your accent, with the status area at its far end where anything
leaving the machine is shown; and two windows showing what is running and what
is filling a folder, every number exactly as the kernel or the disk gave it. It
follows your light and dark settings, never uses the agent's terracotta, and
nothing on it can grant, approve, delete or stop anything. The clock, battery,
network and volume are not in the status area yet. Measured under a nested
compositor; not yet on a certified machine.*

**ROADMAP.md** — under v0.5: *the ordinary desktop* `- [x] The code.` for the
dock, the status area with the egress indicator, *what is running* and *what is
filling the disk*; the status area's clock, battery, network and volume stay
open; nothing on the machine.

**docs/autonomy/QUEUE.md** — the shell plan's task 5 done; task 7 added and
blocked. Proposed follow-ups: a measuring crate for power and the network's
state; an audio crate owning the volume; a regional time and number format;
dock items; `alo-shortcuts` actions for the record window and the two desktop
windows; column names and titles in `alo-measuring`; a per-display edge in
`alo-dock`; a direct-display submission of the desktop.

**docs/autonomy/STATE.md** — reference this report.
