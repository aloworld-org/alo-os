# The sentence a person approves, drawn on the nested compositor

**Date:** 2026-09-14
**Workstream:** v0.5 — the shell (`docs/autonomy/v0-5-the-shell-plan.md`, task 3)
**Responsible contributor:** Claude worker in `C:\dev\alo-os-shell`, for the owner
**Status:** ready for integration — the code. Not seen on a certified machine.

## What changed

ADR 0001 §5 — *an agent proposes and a person approves the sentence* — now has
a screen. When a turn proposes a change, `alo-shell` puts up one panel: which
agent is asking, the sentence `alo-approving` hands the compositor exactly as
the turn worded it, and two answers, *No* and *Approve*, with neither selected.
Tab selects, Enter answers, and the answer goes back through
`alo_approving::Approving` — once. A change that arrives while another is open
waits behind it. Nothing on the surface proceeds on silence.

| File | What it is |
|---|---|
| `crates/alo-shell/src/approval_screen.rs` | `ApprovalScreen`, `ApprovalShows`, `ApprovalAnswer`, `ApprovalOutcome`: the surface for one turn |
| `crates/alo-shell/src/approval_screen_tests.rs` | The surface answered and left alone on a real turn over a real disk |
| `crates/alo-shell/src/approval_shown.rs` | The `alo_approving::Compositor` implementation — the only door a question comes in by |
| `crates/alo-shell/src/approval_queue.rs` | Proposals that arrived behind the open one, in arrival order |
| `crates/alo-shell/src/approval_keys.rs` | `ApprovalKey`: Tab, Shift+Tab, Enter/Space, and nothing else |
| `crates/alo-shell/src/approval_seat.rs` | `Server::approval_key`: keys through the seat, intercepted, never forwarded |
| `crates/alo-shell/src/approval_raster.rs` | `ApprovalLook`; the panel laid out and rasterised; refuses rather than cuts |
| `crates/alo-shell/src/approval_answers.rs` | The two answer boxes: equal size, reading order, selection as shapes |
| `crates/alo-shell/src/approval_raster_tests.rs` | The acceptance, read as text, boxes and pixels |
| `crates/alo-shell/src/approval_paint.rs` | Validating and painting a laid-out panel |
| `crates/alo-shell/src/approval_testing.rs` | A whole turn — file verbs, record, grants — for the tests; a record that cannot write |
| `crates/alo-shell/src/nested_approval.rs` | `ApprovalFrame`, `Nested::pump_approval`, `Nested::submit_with_approval` |
| `crates/alo-shell/tests/approval_source.rs` | The shape promises, read from the shipped source |
| `crates/alo-shell/examples/approval_check.rs` | The WSLg measurement |
| `docs/contracts/native-approval-surface.md` | The new public surface |

Touched to make room: `scene_native.rs` (`NativeLayers` gains an `approval`
layer), `scene_drawing.rs` (validates it and paints it after controls and
before the egress indicator), `nested.rs` (`submit_native_scene` now delegates
to a new `submit_native_layers` that takes every layer; its four callers are
unchanged), `offscreen.rs` (no approval layer), `nested_egress_status.rs`
(`status_picture` is `pub(crate)` so an approval frame refuses exactly as an
indicator frame does), `presentation.rs` (`RenderError::ApprovalScene`),
`lib.rs`, `Cargo.toml` (`alo-approving`, `alo-capability` and `alo-turn` as
dependencies — `alo-capability` moved up from dev-dependencies; `alo-context`,
`alo-files`, `alo-keeping` and `alo-record` as dev-dependencies), `Cargo.lock`.
`docs/autonomy/v0-5-the-shell-plan.md` marks task 3 done.

**User-readable change description:** *When an agent wants to change something
on your machine, alo OS now asks you on the screen, in one sentence that says
exactly what will happen — the same sentence the machine checked, never a
summary of it. There are two answers, No and Approve, and neither is chosen for
you: press Tab to pick one and Enter to give it. Nothing happens if you walk
away. Saying no asks you for no reason. If an agent asks for something else
while you are reading, the new question waits until you have answered the
first. There is no "approve all", no "remember this", and no timer.*

## Decisions, and why

- **Queueing is the shell's, because `Approving::ask` replaces.**
  `alo-approving` replaces whatever was on the screen when asked again, and
  this plan may not edit it. So the shell keeps arrivals in `ApprovalQueue` and
  only calls `Approving::ask` when nothing is open. The replaced-question case
  never arises from this surface.
- **The screen does not guard a second answer.** `ApprovalScreen::answer`
  always forwards to `Approving::approve`/`decline`, so the second answer to one
  question is refused as `alo-approving`'s `NotAnswered::NothingToAnswer`. The
  test asserts that refusal and one moved file; `tests/approval_source.rs`
  holds the surface to exactly one call of each and no call to the turn's own
  doors.
- **Nothing is preselected, and a new question resets the selection.** Enter
  with nothing selected does nothing, so the Enter that answered one question
  cannot answer the next one, and a key held down (the seat drops repeats)
  cannot either.
- **Tab, not arrow keys.** Tab has no direction on the page; an arrow's meaning
  depends on which way the person reads. Escape is nothing: dismissing would be
  a third answer.
- ***No* first in reading order.** The answer that changes the machine is the
  last one a keyboard reaches (Tab from nothing selects *No*). Boxes are the
  same size so neither looks like the default; they mirror for right-to-left.
- **Selection is never colour alone.** A selected box gets a thicker edge and a
  bar under its words, in the same two colours as the rest of the panel. No
  terracotta: it means the agent acting, and this is the moment it has not.
- **Whole or refused.** The sentence wraps at word boundaries and is never
  shortened, ellipsised or cut. A window that cannot hold it and both answers
  refuses the frame (`RenderError::ApprovalScene`) — half a sentence is a
  different claim.
- **The indicator stays on top.** `submit_with_approval` takes an
  `EgressStatusFrame` and checks it first; the approval layer is painted below
  it. A frame showing a question cannot drop what is leaving.
- **A refused answer is read before the next question goes up.** The refusal
  is the refuser's own sentence (`NotAnswered::said`); Enter acknowledges it.
  A refusal that ends the turn (a record that cannot be written) forgets every
  kept question, because none of them could be carried out.
- **`looked_again` rather than a timer.** Before a frame, the host re-reads the
  open question through `Approving::ask` with the same number. A lapsed or
  otherwise answered question comes down with `alo-capability`'s sentence;
  nothing is ever answered by it. No countdown is drawn: `Asked::lapses_in` is
  deliberately unworded and the plan forbids a timer.
- **One `ApprovalScreen` per turn.** `ProposalId` counts from zero in each
  turn's `Approvals`, so a screen that outlived its turn could confuse two
  questions. The type's documentation and the contract say so.
- **Keyboard focus is left alone.** Unlike the sign-in screen, the approval seat
  intercepts keys without unfocusing the person's window, which is theirs again
  once the question is answered.
- **Law 4 split.** The answer boxes were first written inside
  `approval_raster.rs`; they have a second reason to change (reading order,
  stacking, selection shapes), so they moved to `approval_answers.rs` in this
  change.

## Findings

- **No word to acknowledge a refusal.** `alo-approving` declares *Approve* and
  *No* and nothing for "understood". A refusal panel therefore has no visible
  control; Enter acknowledges it. A pointer-only person cannot dismiss one.
  **Proposed:** `alo-approving` (not this crate) declares an acknowledging word,
  and a follow-up draws it.
- **No pointer input.** Answers are given by keyboard, or by a host calling
  `ApprovalScreen::answer`. The raster already records each answer's box
  (`AnswerDrawn::area`) for a hit test. **Proposed follow-up:** pointer
  answers on the approval surface.
- **Nothing wires a turn to the screen yet.** `alo-agentd` proposes over its
  socket; no session process yet hands those proposals to an `ApprovalScreen`.
  That wiring is session integration, outside a drawing crate.

## Acceptance criteria and evidence

| Criterion (plan, task 3) | Test |
|---|---|
| One proposal at a time, the sentence unedited — the text drawn is byte-for-byte the text the turn wrote | `alo-shell lib approval_raster::tests::the_text_drawn_is_byte_for_byte_the_text_the_turn_wrote`; `alo-shell approval_source the_approval_surface_draws_the_sentence_it_was_handed_unedited` |
| Two answers and no third | `alo-shell lib approval_raster::tests::two_answers_and_no_third_in_reading_order`; `alo-shell approval_source two_answers_no_third_and_nowhere_to_give_a_reason` |
| Nothing preselected; nothing proceeds on silence — a surface left alone answers nothing | `alo-shell lib approval_screen::tests::a_surface_left_alone_answers_nothing`; `alo-shell lib approval_screen::tests::nothing_is_preselected_and_enter_on_a_fresh_question_answers_nothing` |
| The answer goes back through `alo-approving` once; a second is refused there, not here | `alo-shell lib approval_screen::tests::one_answer_carries_one_change_and_a_second_is_refused_there_rather_than_here`; `alo-shell approval_source every_answer_goes_through_alo_approving_once` |
| A decline says nothing about why | `alo-shell lib approval_screen::tests::declining_carries_nothing_out_and_says_nothing_about_why` |
| A proposal arriving while another is open is queued, not replacing it | `alo-shell lib approval_screen::tests::a_proposal_that_arrives_while_another_is_open_waits_behind_it` |
| No approve all, no remember this, no timer; not a road to a grant | `alo-shell approval_source no_approve_all_no_remember_this_no_timer_and_no_road_to_a_grant` |

Refusal paths tested beside those:

- An approval with no grant behind it is refused in the turn's words, and the
  next question waits.
- A question that lapsed while it waited is never put up.
- A lapsed open question comes down on `looked_again`.
- A record that cannot be written ends the turn and forgets every kept question.
- With no output, every question is refused into the log and none is answered.
- The same proposal announced twice is one question.
- A chord, Escape and letters do nothing; a held Enter chooses once.
- Strange key codes and a keyboard-less display are refused.
- A frame whose indicator was never told is refused whole, question and all.
- A panel too short, too narrow or oversized refuses rather than cuts.
- Terracotta is in neither scheme.

The source tests were checked against one deliberate mutation — a `pub fn
remember_this` in `approval_screen.rs` that called `approving.approve` a second
time and returned a string literal. Three tests failed, each naming the file
and line; the mutation was removed.

## Verification

Executed on 2026-09-14, Windows 11 host, Ubuntu under WSL 2,
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-shell-cd217193b5311c25`:

- `cargo fmt --all -- --check` — clean.
- `cargo clippy -p alo-shell --all-targets -- -D warnings` — clean. No other
  crate depends on `alo-shell`.
- `cargo test -p alo-shell` — lib 251 passed (26 new), `approval_source` 5
  passed, `client_lifecycle` 262, `egress_status_source` 3, `sign_in_source` 4,
  `socket_ownership` 3, doc tests passed; 0 failed.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-shell --no-deps
  --document-private-items` — clean.
- Each evidence test above run on its own with `--exact` — 1 passed each.
- `WAYLAND_DISPLAY=wayland-0 XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir timeout 180s
  …/examples/approval_check` under WSLg — exit 0. It drew, in light and dark
  and both reading directions:
  - nothing open;
  - the first question with nothing selected;
  - the same with *Approve* selected;
  - the second question after the first was approved (`march.pdf` really
    moved);
  - a refusal in `alo-capability`'s words after an approval with no grant;
  - nothing again after the last *no*.

  46 frames were submitted, each carrying the egress indicator.

Not run: the full workspace suite (the supervisor runs it).

## Limitations

- **Not seen on a certified machine.** No direct-display (DRM/KMS) submission
  carries the surface yet.
- **Keyboard only**, and no visible acknowledging control on a refusal (see
  Findings).
- **No screen-reader tree.** The panel is pixels; AT-SPI is v0.5 accessibility
  work.
- **Not wired to `alo-agentd`.** The surface takes a `Turning` in-process.

## Proposed changes to shared documents

**CHANGELOG.md** — *When an agent proposes a change, alo OS asks on the screen
in the one sentence the machine checked, with No and Approve and neither chosen
for you; nothing happens on silence, a no asks no reason, and a second question
waits behind the first. Measured under a nested compositor; not yet on a
certified machine.*

**ROADMAP.md** — under v0.5 (and v0.01's *approve the sentence, see it
happen*), the approval surface: `- [x] The code.`; nothing on the machine.

**docs/autonomy/QUEUE.md** — the shell plan's task 3 done. Proposed follow-ups:

- pointer answers on the approval surface;
- an acknowledging word in `alo-approving` and its control;
- wiring a session's turns to `ApprovalScreen`;
- a direct-display submission of the surface.

**docs/autonomy/STATE.md** — reference this report.
