# The egress indicator, drawn in the status area

**Date:** 2026-09-14
**Workstream:** v0.5 — the shell (`docs/autonomy/v0-5-the-shell-plan.md`, task 2)
**Responsible contributor:** Claude worker in `C:\dev\alo-os-shell`, for the owner
**Status:** ready for integration — the code. Not seen on a certified machine.

## What changed

Law 1 — *nothing leaves silently* — is now something a person can see. While
anything is leaving the machine, every frame of a session drawn with
`Nested::submit_with_egress_status` carries one row per departure at the far
end of the dock: a terracotta mark with an arrow in it, and the sentence
`alo-egress` wrote for that departure. While nothing is leaving, not one pixel
of it is drawn.

| File | What it is |
|---|---|
| `crates/alo-shell/src/egress_status.rs` | `EgressStatus`: the `alo_indicator::Compositor` a session hands pictures to; refuses in `alo-indicator`'s words with no output |
| `crates/alo-shell/src/egress_status_place.rs` | The status area's corner on one output, from `alo_dock::Layout` |
| `crates/alo-shell/src/egress_status_mark.rs` | The mark: ink edge, terracotta fill, arrow told apart by lightness |
| `crates/alo-shell/src/egress_status_raster.rs` | `EgressStatusLook`; rows laid out and rasterised; the count row when lines do not fit |
| `crates/alo-shell/src/egress_status_raster_tests.rs` | The acceptance, read as rows and pixels |
| `crates/alo-shell/src/egress_status_paint.rs` | Validating and painting a laid-out indicator |
| `crates/alo-shell/src/egress_status_testing.rs` | Real departures and the machine's vocabulary for the tests |
| `crates/alo-shell/src/nested_egress_status.rs` | `EgressStatusFrame`, `Nested::submit_with_egress_status` |
| `crates/alo-shell/src/painted.rs` | `Solid`, `Inked` and the run painter, moved out of the sign-in files to be shared |
| `crates/alo-shell/src/painted_text.rs` | Sentence shaping and inking, moved out of `sign_in_raster.rs` to be shared |
| `crates/alo-shell/tests/egress_status_source.rs` | The shape promises, read from the shipped source |
| `crates/alo-shell/examples/egress_status_check.rs` | The WSLg measurement |
| `docs/contracts/native-egress-indicator.md` | The new public surface |

Touched to make room: `scene_native.rs` (`NativeLayers`: the selected scene and
the indicator above it), `scene_drawing.rs` (paints the indicator after
controls, before the cursor), `nested.rs`, `offscreen.rs`, `nested_sign_in.rs`
(pass no indicator), `presentation.rs` (`RenderError::EgressStatusScene`,
`RenderError::EgressStatusUnknown`), `sign_in_raster.rs` and `sign_in_paint.rs`
(now use `painted` and `painted_text`; behaviour unchanged, their tests
unchanged and passing), `lib.rs`, `Cargo.toml` (`alo-dock` and `alo-indicator`
as dependencies; `alo-capability`, `alo-egress` and `alo-models` as
dev-dependencies only), `Cargo.lock`, and
`tests/direct_keyboard/mod.rs` — two `vec![]` comparisons annotated as
`Vec<u8>`, because the new dev-dependencies link `serde_json` into the
integration tests and its `PartialEq<Value> for u8` made them ambiguous. No
assertion changed.

**User-readable change description:** *alo OS now shows what is leaving the
machine, where you already look. At the far end of the dock, each thing that is
leaving gets a line — which agent, and where to, in the words the machine uses
everywhere else — with a terracotta mark that is never colour alone. What alo
OS does on its own, like checking for an update, is shown the same way. A
question answered on your own machine shows nothing at all, and nothing can
hide the indicator or turn it off.*

## Decisions, and why

- **Quiet draws nothing, not a dark lamp.** The plan says *nothing when it is
  quiet* and *a question answered on this machine draws nothing*.
  `alo-indicator` argues for a dark indicator that is still up, so that absence
  cannot be mistaken for silence. Both are kept honest this way: the model
  still hands the compositor a dark `Drawn` (so *the indicator exists* is a
  fact the shell holds), and a frame is **refused** until that has happened
  (`EgressStatusUnknown`) — so no screen can show clients without having been
  told whether anything is leaving. The dark lamp's sentence remains available
  to a future screen-reader tree; drawing it would put a permanent sentence on
  the screen that the plan asked to be empty.
- **A frame that cannot carry the indicator is refused whole.** Never told, or
  something leaving on a window too small for a dock: the submission fails
  before anything is painted. The alternative — a frame with clients and no
  indicator — is exactly *leaves silently*.
- **Above every client, below the cursor.** Painted after clients, popups and
  controls, so no window can cover a line.
- **The mark.** A navy (light) or charcoal (dark) arrow on terracotta inside an
  ink edge. Tests hold the arrow to WCAG 2.1 §1.4.11's 3:1 against terracotta
  and the edge to 4.5:1 against the row's ground, so taking every hue away
  leaves a shape; the word beside it is the line itself. Terracotta appears in
  no word and in exactly one solid per row.
- **Placement is `alo-dock`'s, turned into pixels.** Rows stack from the status
  area's corner away from the dock, the first nearest it, so a new departure
  never moves one already on screen. The dock is `alo_dock::Dock::layout_on`
  for this output; nothing here chooses an edge or an end.
- **When lines do not fit, the last row is the indicator's own count**
  (`Drawn::lamp_said`, *40 things are leaving this machine right now*) with its
  mark. No line is dropped silently, and no summary is worded here.
- **A vocabulary missing the lines still lights it.** The row then shows the key
  `alo-strings` answers, which is ugly and far better than going quiet.
- **No output toggle on `EgressStatus`.** The first version had
  `output_gone`/`output_back`; its own test failed, showing that
  `Indicating` only calls a compositor when what is leaving changes, so a
  status area switched back on would wait untold. The host instead drops the
  `EgressStatus` and calls `Indicating::show(None, …)` — `alo-indicator`'s
  designed refusal, which forgets — and a new output gets a new one.
- **The painter and text shaper are shared, not copied.** Two surfaces now draw
  owned pixels and sentences; `painted.rs` and `painted_text.rs` hold that once
  (law 4), and the sign-in files use them.

## Finding: the sign-in screen has no status area

Before anybody signs in, alo OS can still leave the machine on its own errand
(`Errand::SigningIn`, `Errand::CheckingForAnUpdate`). The sign-in screen is the
whole output and has no dock, so nothing draws those lines there. The plan's
task 1 does not name a status area and task 2 names *the dock*, so this change
does not invent one. **Proposed follow-up:** the sign-in screen carries the
same indicator rows in a corner, through `submit_sign_in` taking an
`EgressStatusFrame`. That is a small change to this crate and should be a task
in this plan rather than a side effect of this one.

## Acceptance criteria and evidence

| Criterion (plan, task 2) | Test |
|---|---|
| Draws `Indicator`'s lines while live, nothing when quiet; local draws nothing, provider or paired machine draws its sentence — one test, both halves | `alo-shell lib egress_status_raster::tests::a_question_answered_here_draws_nothing_and_one_answered_elsewhere_draws_its_sentence` |
| In a status area at the far end of the dock, wherever the dock is | `alo-shell lib egress_status_raster::tests::it_is_drawn_at_the_far_end_of_the_dock_wherever_the_dock_is` |
| The line is `alo-egress`'s words, never re-worded here | `alo-shell lib egress_status_raster::tests::every_line_is_drawn_exactly_as_alo_egress_words_it_and_is_collected`; `alo-shell egress_status_source the_egress_indicator_writes_no_words_of_its_own` |
| Terracotta never arrives alone: a mark and a word | `alo-shell lib egress_status_raster::tests::terracotta_never_arrives_alone`; `alo-shell lib egress_status_mark::tests::the_arrow_stands_apart_from_terracotta_without_its_hue` |
| alo OS's own errand is on the same indicator | `alo-shell lib egress_status_raster::tests::what_alo_os_does_on_its_own_is_drawn_beside_what_an_agent_caused` |
| Shows, never decides | `alo-shell egress_status_source the_egress_indicator_shows_and_never_decides` |
| No dismissing, no hiding, no setting that turns it off | `alo-shell egress_status_source nothing_can_dismiss_hide_or_turn_off_the_egress_indicator`; `alo-shell lib egress_status_raster::tests::no_look_and_no_dock_draws_a_lit_indicator_as_nothing`; `alo-shell lib nested_egress_status::tests::a_frame_whose_indicator_was_never_told_is_refused_whole` |

Refusal paths beside those: a policy-refused egress draws nothing; no output
refuses in `alo-indicator`'s collected words; an output that came back is told
again; too small, negative or oversized windows refuse a lit frame and never a
quiet one; lines that do not fit end with the count; a vocabulary without the
lines still lights it.

All three source tests were checked against one deliberate mutation — a
`pub fn dismiss` in `egress_status.rs` holding a string literal bound to a
name `ended` — and each failed naming the file and line; the mutation was
removed.

## Verification

Executed on 2026-09-14, Windows 11 host, Ubuntu under WSL 2,
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-shell-cd217193b5311c25`:

- `cargo fmt --all -- --check` — clean.
- `cargo clippy -p alo-shell --all-targets -- -D warnings` — clean. No other
  crate depends on `alo-shell`.
- `cargo test -p alo-shell` — lib 225 passed, `client_lifecycle` 262 passed,
  `egress_status_source` 3 passed, `sign_in_source` 4 passed,
  `socket_ownership` 3 passed, doc tests passed; 0 failed.
- `WAYLAND_DISPLAY=wayland-0 XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir timeout 180s
  …/examples/egress_status_check` under WSLg — exit 0: refused a frame before
  the indicator was told; drew a quiet machine, a provider question, a paired
  machine beside an update check, 32 simultaneous departures, and a quiet
  machine again, on all four dock edges in light and dark; 85 frames submitted.

Not run: the full workspace suite (the supervisor runs it).

## Limitations

- **Not seen on a certified machine**, and no direct-display (DRM/KMS)
  submission carries it yet. `ROADMAP.md`'s v0.01 *indicator stayed dark
  through a working day* cannot be ticked on the strength of this.
- **The dock itself is not drawn** (task 5); the indicator is placed where
  `alo-dock` says the status area is, on one output.
- **No screen-reader tree.** The rows are pixels; the AT-SPI surface is v0.5
  accessibility work.
- **Not interactive.** Nothing opens from a row; *what did it do* is task 4.
- **The sign-in screen has no indicator** — see the finding.

## Proposed changes to shared documents

**CHANGELOG.md** — *alo OS shows what is leaving the machine at the far end of
the dock: a line per departure, in the machine's own words, with a terracotta
mark that is never colour alone, including what alo OS does on its own. A
question answered on this machine shows nothing. Measured under a nested
compositor; not yet on a certified machine.*

**ROADMAP.md** — under v0.5, the egress indicator in the status area:
`- [x] The code.`; nothing on the machine.

**docs/autonomy/QUEUE.md** — the shell plan's task 2 done; proposed follow-up:
the egress indicator on the sign-in screen; a direct-display submission of the
indicator.

**docs/autonomy/STATE.md** — reference this report.
