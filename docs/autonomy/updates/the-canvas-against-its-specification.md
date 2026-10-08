# The canvas against its specification

**`docs/design/the-canvas-as-a-workspace.md` arrived on 2026-10-08** and is the
owner's specification of what the canvas is. This is the other half: what the
repository already holds, measured against it.

## A correction, first, because the first draft of this document was misleading

Its first draft listed **three disagreements and two agreements** and then said
that §6, §13, §14 and §17 were *not measured*. Every sentence in it was true and
the document as a whole was wrong, because it was written from one afternoon's
photographs and read as though the photographs were the repository.

The owner's reply was *but most of it was built*, and they were right.

**What the first draft did** is the failure this repository keeps naming: a
check whose scope misses the question. Four fixtures and a handful of greps
measured what four fixtures and a handful of greps could reach, and the shape of
the report — three faults, two passes, the rest unknown — invited the reading
that the rest was absent. It was not absent. It was unvisited.

## What is actually there

| | |
|---|---|
| crates in the workspace | 113 |
| lines of Rust | 688,888 |
| `crates/alo-shell` | 104,267 lines, 496 files |
| test files in `alo-shell` alone | 22, several of them directories |

**`docs/autonomy/the-smallest-canvas-worth-showing.md` is the canvas's own plan,
and eight of its eleven tasks are marked done**, each with a date and the
acceptance it was closed against:

| task | status |
|---|---|
| 1. A plane that moves under a viewport that does not | **Done** 2026-09-27 |
| 2. A frame is where it looks, at any zoom | **Done** 2026-09-27 |
| 3. Dragging a frame | **Done** 2026-09-27 |
| 4. Resizing, and the application told as it happens | **Done** 2026-09-29 |
| 5. Pan | **Done** 2026-09-29 |
| 6. Zoom, and *Show all* | **Done** 2026-09-28 |
| 7. Every canvas answers as a list | **Done** 2026-09-29 |
| 8. A frame is never lost | **Open** |
| 9. The canvas is where they left it | **Open**, reopened by the owner 2026-10-03 |
| 10. The walk, which is also the film's sequence | **Done** 2026-09-30 |
| 11. The Dock at any edge a person chooses | **Open** — the blocker was stale and was cleared 2026-10-08 |

## The specification, section by section, against the files that answer it

Nearly every section has a file named after it. That is not a coincidence — the
specification and the code were written by people describing the same product.

| § | what it asks for | what answers it |
|---|---|---|
| 1 | Places, each remembering its windows, camera and panel | `alo-canvas/src/place.rs`, `canvas_place.rs`, `settings_places.rs`, `canvas_dragged_into_a_place.rs`, `canvas_a_place_remembers_time.rs`, `alo-put-aside/src/a_place_groups_its_windows.rs`, `the_collapse_choice_per_place.rs`; test suites `every_window_is_on_a_place`, `move_to_place`, `dragged_into_another_place` |
| 2 | two layers; fixed controls that panning never carries away | `canvas_fixed_controls.rs` (899 lines), `alo-put-aside/src/the_region_the_panel_claims.rs`, `egress_status_place.rs` |
| 3 | pan, by pointer and by keyboard | `canvas_pan.rs`, `canvas_arrow_pan.rs`, `canvas_space_drag.rs`, `canvas_pinch.rs` — plan task 5, **done** |
| 4 | zoom, 100%, Show all, fit selection, full screen | `canvas_wheel_zoom.rs`, `canvas_show_all.rs`, `canvas_camera.rs`, `canvas_command.rs`, `window_full_screen.rs` — plan task 6, **done** |
| 5 | a window with a stable identity and a title band | `window_placement.rs`, `window_number.rs`, `window_mode.rs`, `window_control_name_fallback.rs`, and ~45 `window_control_*` files. **The title band is the gap** — see below |
| 6 | focus and selection distinguished | `window_activation.rs`, `window_control_focus.rs`, `window_control_reader_selection.rs` |
| 7 | moving a window, by pointer and by keyboard | `window_move.rs`, `window_command.rs`, `canvas_space_drag.rs` — plan task 3, **done** |
| 8 | resizing from edges and corners | `window_resize.rs` (219 lines + 103 of tests), `canvas_resize.rs` — plan task 4, **done** |
| 9 | normal, compact, put aside, full screen | `window_mode.rs`, `window_mode_plan.rs`, `window_minimize.rs`, `window_maximize.rs`, `window_full_screen.rs`, `putting_a_window_aside.rs`, the whole of `alo-put-aside` (7,407 lines, 38 files) |
| 10 | full screen with no permanent margin, controls revealed | `window_full_screen.rs`, test suite `a_window_that_fills_the_screen` |
| 11 | the Dock, returning a person to their windows | `alo-dock` (8,702 lines, 30 files), `a_click_brings_a_window_back.rs` (226 lines + 120 of tests). **Plan task 11 is open; its *blocked on design* was stale and was cleared on 2026-10-08** |
| 12 | the panel: previews, rail, handle, peek, restore | `alo-put-aside/src/panel.rs`, `preview.rs`, `peeking_at_a_preview.rs`, `restoring.rs`, `where_it_goes_back.rs`, `restoring_into_a_taken_place.rs`, `peeking_at_a_put_aside_window.rs` |
| 13 | arranging several windows | `window_dividing.rs`, `alo-dividing` (4,760 lines, 21 files) |
| 14 | alo acting on an explicit scope, and Stop | `alo-put-aside/src/alo_at_work.rs`, `what_alo_is_doing.rs`, `how_far_alo_has_got.rs`, `the_scope_alo_may_change.rs`, `what_an_agent_may_do.rs`, `proposing.rs`, `requires_you.rs`, `whether_it_is_private.rs` |
| 15 | menus, modal dialogs, notifications | `alo-menus`, `popup_placement.rs`, `alo-notifying`, `alo-approving` |
| 16 | a window is never lost | `canvas_never_lost.rs` (305 lines) — **plan task 8, open** |
| 17 | persistence across a real restart | `canvas_remembered.rs` (288 lines), `alo-arranging` (2,154 lines), test suite `the_canvas_is_where_they_left_it` — **plan task 9, open, reopened by the owner** |
| 18 | light canvas surfaces, navy text, deep teal for alo | `alo-appearance` (7,920 lines), `docs/design/palette.toml`, ADR 0093 |

## What is wrong is wiring, not design - and one of the three is a chain

This is the finding, and it is the opposite of the first draft's.

**Every one of the three faults photographed on 2026-10-08 is a built thing with
something unfinished in front of it.** Two are a single value. The third is a
chain of four, and calling it a wire is the mistake this section made first.

### The surface is built and ships Cream. One black line covers it

- `crates/alo-appearance/src/shipped.rs:48` — `pub const THE_SURFACE: Token = Token::Cream;`
- `crates/alo-appearance/src/background.rs:31` — `pub enum Background`
- `crates/alo-appearance/src/appearance.rs:149` — `pub fn background_on(&self, display: &DisplayId) -> Background`

and in front of all of it, `crates/alo-shell/src/scene_drawing.rs:125`:

```rust
// Neutral clear, not the shell's pending token-based visual design.
.clear(Color32F::new(0.0, 0.0, 0.0, 1.0), &[damage])
```

`shipped.rs` even records dodging the near-miss: `Porcelain` now carries
`bg/surface`, which is white, and shipping that *would have shipped a pure white
desktop instead of the design's canvas*. Somebody thought carefully about the
exact colour §18 asks for, a month before §18 was written.

**An earlier draft of this document said ADR 0075's *material* did not exist.**
That was wrong and it was wrong in the usual way: the ADR says *material*, the
code says `Background`, and a search for the ADR's noun returns nothing and
means nothing.

### The dock is designed and built. It has no caller, and that is not one wire

**This section said *a literal `0` is passed instead*, as though connecting one
value would fill the dock. That is wrong**, and the owner said so first — *I
remember the dock was designed well, see in the repo, maybe it is just not
wired.* The design is there. The wiring is more than one wire.

`crates/alo-dock` is 8,702 lines across 30 files, and holds `OnTheDock { app,
pinned, windows, put_aside }` (`holding.rs:48`), what a click does per state
(`clicking.rs`), the window picker (`previews.rs`), how the bar grows and
overflows (`Room::a_bar_holding`), hover and focus (`Peeking`), concealing and
revealing (`revealing.rs`), dragging (`offering.rs`), and the accessible names
and states a screen reader is given (`announcing.rs`, `labels.rs`).

**What is missing is everything that would call it**, measured today, each with
a positive control in the same command:

| | |
|---|---|
| `alo_dock::Holding` outside its own crate | **2 mentions, both in comments** — `desktop_raster.rs:200` and `dock_raster.rs:26`. No line of code builds one |
| what `Holding::showing` needs | a `&Windows` — every window open on this machine in the order last used. Nothing tracks that |
| an icon in `alo-applications` | **0 of 17 files** name one. Control: *opener* appears in 3 |

So the bar is **the honest width of a dock holding nothing, on a machine where
nothing is pinned, nothing is tracked as open and no application has a picture.**
`desktop_raster.rs:197`'s own comment is right that this is *true rather than a
placeholder*.

**An earlier draft of this document, and a message to the compositor lane, said
it would be a sliver *whatever a person had pinned*.** That was wrong in the way
worth recording: it names a state nothing can reach, because nothing can pin
anything yet. A fault predicted for a situation that cannot occur is not a
measurement.

### The window controls are built. There is no band to put them in

Measured by photographing `foot` — a Wayland terminal nobody here wrote — on the
real compositor: minimise, maximise and close are drawn, and they land directly
on the application's own pixels, over its first line of output. There are ~45
`window_control_*` files including a full screen-reader path; §5's *shared window
component … consistent title bands* is the piece that is not there.

## What was run, and what it proved

| instrument | result |
|---|---|
| the canvas walk, `ALO_NESTED_SUBMODE=canvas-walk` | **all ten moments, exit 0** — three applications opened, one dragged, one resized, the canvas panned, Show all, zoomed back in, then pan, Show all, zoom and focus **by keyboard alone** |
| `a_real_application --run foot` | an outside application connected to the compositor's socket, drew, and was composited — 83 surfaces over 15 seconds |
| `desktop_check --save-to` | 32 pictures of the desktop in both schemes and both readings |

**The 32 pictures are evidence about the code and no evidence at all about
whether the code matches the design.** They are photographs of the running
compositor; no design node was consulted to make or to judge them. Where this
document reads a design it reads `docs/design/figma-snapshot/70-28.xml`, the
committed snapshot refreshed 2026-10-07 — **not the live file**, which may have
moved since.

So *this is not the screens we designed* is a judgement about a comparison
**nobody has yet made**. Making it means holding the live file against the code,
which is the compositor lane's and is under way.

The walk is §3 and §4 demonstrated end to end, including their keyboard routes.
**What it does not show is appearance**: 832 of 1,049,088 pixels are painted,
because its three applications are test rectangles.

## What is genuinely open

1. **Task 8, a frame is never lost** — §16. The file exists and the plan says
   dock occlusion and drag-extent protections pass *their current* checks while
   the task is not done.
2. **Task 9, the canvas is where they left it** — §17, reopened by the owner on
   2026-10-03. The specification's own warning applies: *a test that
   reconstructs objects in memory is not sufficient evidence that restart
   persistence works.* Nothing on this machine has restarted anything.
3. **Task 11, the Dock at any edge** — §11, and it was marked **blocked on
   design**. §11 and §18 of the specification are what it was blocked on.
4. **The three wires above.**

## What this document still does not claim

**Nothing here is a panel.** Every picture is a nested surface under another
compositor, software rendered under WSLg. A file existing is not a feature
working, and a count of lines is not a measurement of behaviour — the table of
sections above says what answers each section, not that each section is kept.
`desktop_check`'s own closing line stands: *a virtual output proves the drawing
path, not a panel.*
