# The agent overlay's summoning seam: one key, from anywhere

**Date:** 2026-09-10. **Workstream:** v0.01 delivery plan, task 2 (*The agent
overlay: one key, from anywhere*). **Contributor:** Claude, in the
`alo-os-claude` checkout, while the desktop worker is away.

## What changed

A new crate, `crates/alo-overlay`, holds the seam between the key that already
exists and the overlay that does not yet: the summoning state one chord drives,
the surface request a compositor can honour or refuse, and the refusal a person
reads when there is nowhere to show the agent. Its words are declared onto the
machine's one vocabulary through `alo-saying`, whose collector now reaches
sixteen crates.

**User-readable change description** (for the integration owner's CHANGELOG
consolidation):

> Pressing the agent's key now has a defined answer everywhere on the machine.
> One press asks the compositor to show the agent exactly once; pressing the
> key again while the agent is up does not ask again; and when there is
> nowhere to show the agent — no desktop running, or no screen connected —
> the press is answered with a sentence in the person's own language instead
> of nothing happening. What the overlay looks like is still to come; what a
> press of the key *means* is now settled and tested.

Source paths:

- `crates/alo-overlay/src/summoning.rs` — `Summoning`, the two-state machine
  the chord drives; `Pressed`, the three things a press can come back as; and
  `NotOpen`, the wiring bug's diagnostic.
- `crates/alo-overlay/src/surface.rs` — `SurfaceRequest` (no rendering in it,
  constructible only by a press), `SurfaceRefused`, and the `Compositor` port
  `alo-shell` will implement.
- `crates/alo-overlay/src/refusing.rs` — `NotSummoned`, each way of failing
  mapped to a sentence through `alo-strings`.
- `crates/alo-overlay/src/words.rs` — the two sentences, with translator
  notes, under the new `overlay` area.
- `crates/alo-overlay/tests/one_key_summons_the_agent.rs` — the acceptance,
  one test per criterion.
- `crates/alo-saying/src/collecting.rs`, `crates/alo-saying/src/lib.rs`,
  `crates/alo-saying/Cargo.toml` — the sixteenth list, and the counts kept
  truthful.
- `Cargo.toml`, `Cargo.lock` — the new workspace member.
- `docs/autonomy/v0-01-delivery-plan.md` — task 2's `**Done,**` line, per the
  plan's own rule that whoever finishes writes it in the same change. The next
  increment (task 3) already existed, so no new task was needed.

## Decisions

The task said *decide rather than stop*; these are the decisions and why.

1. **A new crate, not files in `alo-shell`.** The task's constraint keeps this
   work out of the desktop worker's window-control chain, and `SHARED_MAIN.md`
   gives that worker `alo-shell`'s compositor. A separate model crate is also
   this repository's established shape (`alo-shortcuts`, `alo-dock`,
   `alo-appearance` all model what the shell later wires), and task 3 — what
   the overlay shows — has an obvious home in the same crate. Nothing in
   `alo-shell` was touched, not even `shortcut_dispatch.rs`.
2. **The seam is a synchronous port** (`Compositor::honour`). The summoning
   and the compositor will live in the same process — the shell routes the
   chord and owns the surfaces — so *may the agent appear* needs no pending
   state. That keeps the machine two states (closed, open) instead of three,
   and makes *asks exactly once* a counted fact in tests.
3. **Only a press can create a `SurfaceRequest`.** The type has no public
   constructor (`#[non_exhaustive]`, in-crate `asked()`), so *one press, one
   request* is held by the compiler rather than by convention. The request is
   empty because there is exactly one agent overlay; anything it one day
   carries is an additive field.
4. **A second press while open is a no-op, not a toggle.** The acceptance only
   forbids asking twice. Making the key dismiss on second press is a product
   decision about the overlay's behaviour that belongs with task 3's content
   work, and a no-op is forward-compatible with either answer.
5. **A press with no compositor closes whatever the model thought was open.**
   If the compositor went away while the overlay was up, a stale `Open` would
   leave the key dead forever. The press that discovers the absence refuses in
   words and resets, so the key works the moment a compositor is back. Tested.
6. **Two refusal sentences, not one.** *The desktop is not running* and *no
   screen is connected* tell the person to fix different things; one sentence
   for both would have them fixing the wrong one. `SurfaceRefused` carries the
   compositor's reason; `NotSummoned` maps each to its word.
7. **The clash rule needed no new code.** The plan's first bullet — a shortcut
   action refusing to clash with a bound key — has been true since
   `alo-shortcuts` shipped `Action::TheAgent` on `Super+A` with
   `Shortcuts::bind`'s refusals. The integration test pins the resolution
   (chord → action, both directions) rather than re-implementing it.

## Acceptance and verification

All commands run on Windows 11 (the host this checkout gates on; nothing in
this change is platform-gated). Executed 2026-09-10, after pulling
`origin/main` at `3018fa6`:

| Check | Command | Result |
|---|---|---|
| Format | `cargo fmt --check` | clean |
| Lints | `cargo clippy --workspace --all-targets -- -D warnings` | clean, zero warnings |
| Whole suite | `cargo test --workspace` | all green (139 result lines, 0 failed) |
| New crates | `cargo test -p alo-overlay -p alo-saying` | 17 + 5 + 51 + 4 tests, 2 doctests, 0 failed |

Acceptance criteria, each with the test run **on its own** with `--exact`
(all: `ok. 1 passed; 0 failed`), all in
`crates/alo-overlay/tests/one_key_summons_the_agent.rs`:

- *the chord resolves to the action* —
  `the_chord_resolves_to_summoning_the_agent`
- *pressing it asks for the surface exactly once* —
  `pressing_it_asks_for_the_surface_exactly_once`
- *a second press while it is open does not ask twice* —
  `a_second_press_while_it_is_open_does_not_ask_twice`
- *with no compositor there is a refusal a person could read* —
  `with_no_compositor_there_is_a_refusal_a_person_could_read`
- *a refusal in words when there is nowhere to show it* (the compositor's own
  nowhere) — `a_compositor_with_nothing_to_show_on_refuses_in_words`

*No pixels are claimed and none are tested*: nothing in the crate names a
size, position, colour or surface geometry, and no test asserts one.

Refusal paths tested beside the happy paths, in the crate's unit tests: a
dismissal with nothing open is refused (`NotOpen`); a refused surface leaves
the summoning closed so the next press asks again; a press with no compositor
refuses and resets; a vocabulary key already taken is not replaced; the
translated refusal arrives in the reader's language and the untranslated one
says it is English.

## Limitations that remain

- **Nothing presses the chord yet.** The shell does not implement
  `alo_overlay::Compositor` and its input path does not route
  `Action::TheAgent` into a `Summoning`; that wiring crosses the desktop
  worker's chain and waits for their return (they are back 2026-09-15). The
  seam is designed so that the wiring is an `impl` and one dispatch arm.
- **The overlay has no content.** What it shows when it opens is task 3.
- **No translations of the two new sentences exist** — like every other
  string on the machine so far; they are declared, noted for the translator,
  and counted by `Strings::unanswered`.

## Proposed shared-document updates (integration owner's to make)

- `CHANGELOG.md`: the user-readable description quoted above.
- `docs/autonomy/QUEUE.md` / `STATE.md`: task 2 of the v0.01 delivery plan
  done, report at this path; tasks 3 and 6 unblocked.
- No `ROADMAP.md` movement: this is one task inside phase 3's spine, not an
  exit gate.

## Status

Ready for integration.
