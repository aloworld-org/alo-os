# What the machine did, drawn in the record window

**Date:** 2026-09-14
**Workstream:** v0.5 — the shell (`docs/autonomy/v0-5-the-shell-plan.md`, task 4)
**Responsible contributor:** Claude worker in `C:\dev\alo-os-shell`, for the owner
**Status:** ready for integration — the code. Not seen on a certified machine.

## What changed

`docs/features.md`'s *afterwards, ask what it did* now has a window. The record
`alo-recounting` reads off the disk is drawn in one tall panel: first what the
record says about itself — whether anything answered, whether this is all of
it, whether it goes all the way back, whether it is what it says it is, what
could not be read — and then every entry, most recent first, with the
vocabulary's clause at its head and the machine's own words after it. Refusals
are drawn exactly as plainly as what ran. alo OS's own errands are drawn with no
agent named. The window opens by hand, with no agent anywhere on the road, and
asking the agent *what did you do?* opens the same account. It writes nothing
to the record, has no filter, no search and no summary.

| File | What it is |
|---|---|
| `crates/alo-shell/src/record_window.rs` | `RecordWindow`, `RecordShows`, `RecordOpened`: the two roads, the view, reading again, closing |
| `crates/alo-shell/src/record_window_tests.rs` | The window opened, moved, read again and refused on a real record on a real disk |
| `crates/alo-shell/src/record_shown.rs` | The `alo_recounting::Compositor` implementation — the only door an account comes in by |
| `crates/alo-shell/src/record_lines.rs` | One entry as the words drawn for it, most recent first |
| `crates/alo-shell/src/record_raster.rs` | The panel laid out and rasterised; refuses rather than cuts |
| `crates/alo-shell/src/record_room.rs` | `RecordLook`; the panel's room, measures and two colours |
| `crates/alo-shell/src/record_raster_tests.rs` | The acceptance, read as text, boxes and pixels |
| `crates/alo-shell/src/record_paint.rs` | Validating and painting a laid-out window |
| `crates/alo-shell/src/record_keys.rs` | `RecordKey`: Up, Down, Home, End, F5, Escape, and nothing else |
| `crates/alo-shell/src/record_seat.rs` | `Server::record_key`: keys through the seat, intercepted, never forwarded |
| `crates/alo-shell/src/record_testing.rs` | A record written by a real turn and the daemon's other entries, on a disk |
| `crates/alo-shell/src/nested_record.rs` | `RecordFrame`, `Nested::pump_record`, `Nested::submit_with_record` |
| `crates/alo-shell/tests/record_source.rs` | The shape promises, read from the shipped source |
| `crates/alo-shell/examples/record_check.rs` | The WSLg measurement |
| `docs/contracts/native-record-window.md` | The new public surface |

Touched to make room: `scene_native.rs` (`NativeLayers` gains a `record` layer),
`scene_drawing.rs` (validates it and paints it after controls and before the
approval surface), `nested.rs`, `offscreen.rs` and `nested_approval.rs` (no
record layer), `presentation.rs` (`RenderError::RecordScene`), `lib.rs`,
`Cargo.toml` (`alo-recounting` as a dependency; `alo-record` moved up from
dev-dependencies for `Asking`), `Cargo.lock`.
`docs/autonomy/v0-5-the-shell-plan.md` marks task 4 done; tasks 5 and 6 already
follow it.

**User-readable change description:** *You can now see what your machine did.
A window lists everything in the machine's record, most recent first: what an
agent did, what you said no to, what was refused before you were asked, what
left the machine and where it went, and what alo OS did on its own. Refusals
are shown just as plainly as everything else, and the window tells you when the
record does not go all the way back or when it is showing only the most recent
part. You do not need an agent to open it — and asking the agent "what did you
do?" shows you this same record rather than a summary. Nothing in the window
can change the record, hide part of it or search it.*

## Decisions, and why

- **Two named doors, one private read.** `opened_by_hand` and
  `asked_what_it_did` both call one `read`, which is the only call to
  `Recounting::show` in the crate. A single door with a "road" parameter would
  invite the road to change something; two doors whose bodies a source test
  holds to exactly `self.read(recounting)` make *the same account* structural.
- **How the agent's road is reached is not decided here.** Recognising *what
  did you do?* in what somebody typed would be this crate interpreting a
  question, and `alo-recounting` is deliberately unreachable from a turn. The
  door exists for the overlay to call; nothing calls it yet (a finding).
- **The whole record, always.** `Asking::anything()` at `AtMost::ONE_SITTING`
  and nothing else: no filter exists to default to something narrower, and the
  bound is `alo-recounting`'s, which says itself when it left something out.
- **Newest first is the one reordering.** The account holds entries oldest
  first; the window reverses it and does nothing else to the order.
- **The record's sentences about itself stay above the entries** and never move
  with the view, so no scrolled view can hide that the record is incomplete,
  shortened, disagrees with itself or has damage.
- **Entries are drawn whole or wait.** An entry that does not fit waits for the
  view to move; if the record's sentences and the top entry cannot fit, the
  frame is refused (`RenderError::RecordScene`), as the approval surface does.
- **A rail, not words, for "more out of view".** No crate declares a sentence
  for it, and inventing one here is the bug the plan forbids. The rail is a
  shape: a track and a thumb in the same two colours, on the trailing edge for
  the reading direction.
- **What a model typed is not drawn on its own line.** `Told::asked_for` is
  quoted inside `alo-capability`'s refusal (*there is no verb called …*), which
  is the machine's sentence and is drawn. A separate line would put model text
  where the machine's words go.
- **No date is drawn.** `Told::at` is a moment and `alo-recounting` says how it
  is written belongs to the reader's region; no crate decides that yet (a
  finding). The order carries the sequence.
- **No title.** No crate declares one (a finding).
- **Up/Down, not Tab.** The view moves through time, which has no reading
  direction. Escape closes: closing a reading is not an answer, unlike on the
  approval surface. F5 reads the disk again; the account never refreshes
  itself.
- **Layering.** Record window above clients and controls, a waiting question
  above the record, the egress indicator above both. `submit_with_record` takes
  an optional `ApprovalFrame` so a question that arrives while somebody reads
  the record is never under it.
- **Law 4 split.** Geometry and colours were first written inside
  `record_raster.rs`; they change for a different reason from what goes in the
  window, so they moved to `record_room.rs` in this change.

## Findings

- **Nothing opens the window yet.** `alo_shortcuts::Action` has no action for
  it and the dock is task 5. **Proposed:** `alo-shortcuts` (not this crate)
  declares an action for the record window, and task 5's dock or status area
  offers it.
- **Nothing calls the agent's road.** The overlay has no drawn question box.
  **Proposed:** when the overlay is drawn, the plain phrase *what did you do?* —
  whichever crate decides how it is recognised — calls
  `RecordWindow::asked_what_it_did`.
- **No date or time on an entry.** **Proposed:** a regional date-and-time format
  decided in `alo-strings` (or a crate of its own), then drawn here.
- **No window title.** **Proposed:** `alo-recounting` declares one.

## Acceptance criteria and evidence

| Criterion (plan, task 4) | Test |
|---|---|
| Each entry's clause at its head and the machine's sentence after it, in the record's order, newest first | `alo-shell lib record_raster::tests::entries_are_drawn_newest_first_in_the_order_the_record_has_them` |
| Every clause is the vocabulary's rather than this crate's, held by a test | `alo-shell lib record_raster::tests::every_clause_drawn_is_the_vocabularys_and_none_is_this_crates`; `alo-shell record_source the_record_window_writes_no_words_of_its_own` |
| What was refused is drawn as plainly as what ran | `alo-shell lib record_raster::tests::what_was_refused_is_drawn_as_plainly_as_what_ran` |
| An entry that names no agent is drawn without inventing one | `alo-shell lib record_raster::tests::an_entry_that_names_no_agent_is_drawn_without_inventing_one` |
| The window is reachable without asking an agent anything (ADR 0009), and a test names that road | `alo-shell lib record_window::tests::the_window_opens_by_hand_with_no_agent_anywhere`; `alo-shell record_source the_plain_road_passes_through_no_agent` |
| Asking the agent *what did you do?* reaches the same account | `alo-shell lib record_window::tests::asking_the_agent_what_it_did_reaches_the_same_account`; `alo-shell record_source both_roads_reach_the_one_account` |
| The surface reads the record and writes nothing to it | `alo-shell lib record_window::tests::the_window_reads_the_record_and_writes_nothing_to_it`; `alo-shell record_source the_record_window_writes_nothing_to_the_record` |
| No filter that could hide a refusal, no search, no summary | `alo-shell lib record_window::tests::the_window_holds_every_entry_the_record_holds`; `alo-shell record_source no_filter_that_could_hide_a_refusal_and_no_search` |

Refusal paths tested beside those:

- A record that is not there is a refusal in the window in `alo-keeping`'s
  words, never an empty list.
- A record others can write is refused, and shown once it is the machine's own
  again.
- A file that stopped being a record takes the shown account down.
- With no output the window stays closed and the refusal goes to the log.
- An empty record is an account that says so, beside what the record is.
- A closed window does nothing with any key, reading included.
- The view stops at both ends of the account.
- An entry kept while the window is open is not shown until the person reads
  again.
- A chord, a letter, Enter and Delete do nothing; a held key moves once;
  strange key codes and a keyboard-less display are refused.
- A frame whose indicator was never told is refused whole, record and all.
- A window too short, too narrow or oversized refuses rather than cuts; a
  picture laid out for one output is refused on another.
- Terracotta is in neither scheme.

The tests were checked against one deliberate mutation — `record_lines.rs`
drawing only entries that ran, oldest first. Seven raster tests and
`no_filter_that_could_hide_a_refusal_and_no_search` failed; the mutation was
removed.

## Verification

Executed on 2026-09-14, Windows 11 host, Ubuntu under WSL 2,
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-shell-cd217193b5311c25`:

- `cargo fmt --all` — clean.
- `cargo clippy -p alo-shell --all-targets -- -D warnings` — clean. No other
  crate depends on `alo-shell`.
- `cargo test -p alo-shell` — lib 280 passed (29 new), `approval_source` 5,
  `client_lifecycle` 262, `egress_status_source` 3, `record_source` 5 (new),
  `sign_in_source` 4, `socket_ownership` 3, doc tests passed; 0 failed.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-shell --no-deps
  --document-private-items` — clean.
- Each evidence test above run on its own with `--exact` — 1 passed each.
- `WAYLAND_DISPLAY=wayland-0 XDG_RUNTIME_DIR=/mnt/wslg/runtime-dir timeout 180s
  …/examples/record_check` under WSLg — exit 0. It drew, in light and dark and
  both reading directions, with the egress indicator on every frame:
  - a closed window;
  - 44 entries opened by hand, the newest *alo OS reached the network itself*;
  - the view at the oldest entry, 43 newer out of view;
  - 45 entries after reading again, the newest the entry kept meanwhile;
  - the same account reached by asking the agent (checked equal);
  - `alo-keeping`'s refusal once the record file was removed;
  - nothing again after Escape.

  36 frames were submitted.

Not run: the full workspace suite (the supervisor runs it).

## Limitations

- **Not seen on a certified machine.** No direct-display (DRM/KMS) submission
  carries the window yet.
- **Nothing opens it yet** — no shortcut action, no dock item, no overlay
  wiring (see Findings).
- **No date, no title** (see Findings).
- **Keyboard only.** No pointer scrolling.
- **No screen-reader tree.** The panel is pixels; AT-SPI is v0.5 accessibility
  work.

## Proposed changes to shared documents

**CHANGELOG.md** — *A window shows what the machine did, read from its record:
every entry most recent first, refusals as plainly as what ran, alo OS's own
errands with no agent named, and the record's own word on whether it is
complete. It opens without an agent, asking the agent what it did shows the
same record, and nothing in it can change, filter or summarise the record.
Measured under a nested compositor; not yet on a certified machine.*

**ROADMAP.md** — under v0.01's exit gate (*afterwards ask what it did and get an
answer from the record*) and v0.5, the record window: `- [x] The code.`;
nothing on the machine.

**docs/autonomy/QUEUE.md** — the shell plan's task 4 done. Proposed follow-ups:

- an `alo-shortcuts` action (and a dock entry) that opens the record window;
- the overlay calling `RecordWindow::asked_what_it_did`;
- a regional date-and-time format, then dates on entries;
- a declared title for the window;
- a direct-display submission of the window.

**docs/autonomy/STATE.md** — reference this report.
