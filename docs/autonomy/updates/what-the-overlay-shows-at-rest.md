# What the overlay shows when the agent has nothing to say yet

**Date:** 2026-09-10. **Workstream:** v0.01 delivery plan, task 3 (*What the
overlay shows when the agent has nothing to say yet*). **Contributor:** Claude,
in the `alo-os-claude` checkout, while the desktop worker is away.

## What changed

Task 2 gave the agent's key an answer. This gives the thing it opens something
to say. `crates/alo-overlay` now holds the overlay's **content before a
question is asked**: one value, derived from what the machine actually holds,
with the three states the plan names and the three readings underneath them.

The readings are the three things `alo-agentd` already answers and no screen
has ever shown — what would answer a question, what the agent may reach, and
whether anything is leaving the machine — each read from the crate that already
decides it rather than from a number handed to this one.

**User-readable change description** (for the integration owner's CHANGELOG
consolidation):

> Pressing the agent's key on a machine that is not set up yet now says what
> to do about it. The overlay reads the machine before it opens and shows one
> of three things: *nothing has been chosen to answer questions — choose a
> model or add a provider*, *the agent can reach nothing — grant it a folder*,
> or an invitation to ask. Under it, three lines a person can check at a
> glance: what would answer a question and **where** it would be answered,
> how much the agent has been granted right now, and whether anything is
> leaving this machine at this moment. Every one of those sentences is in the
> machine's own vocabulary, so a translation can arrive for all of them. What
> the overlay looks like is still the compositor's and is still to come.

Source paths:

- `crates/alo-overlay/src/resting.rs` — `AtRest`, the whole of what the
  overlay shows at rest: two constructors (compose, or read this machine),
  four lines in reading order, and a state it derives rather than accepts.
- `crates/alo-overlay/src/standing.rs` — `Standing`, the three-case value the
  acceptance names, and why *nothing chosen* leads when both are true.
- `crates/alo-overlay/src/answering.rs` — `WouldAnswer`, read from
  `alo_choosing::Settings` through `Picked::source`, so a provider choice can
  never read as answering here.
- `crates/alo-overlay/src/granted.rs` — `Granted`, counted from
  `alo_capability::Grants::held_by` at the moment it is asked, for one agent.
- `crates/alo-overlay/src/quiet.rs` — `Quiet`, read from
  `alo_egress::Indicator` and from nothing else.
- `crates/alo-overlay/src/words.rs` — nine strings and two counted ones, with
  translator notes; the `Counted` shape copied from `alo-keeping`.
- `crates/alo-overlay/src/lib.rs`, `src/testing.rs` — the new modules, and a
  fixture that holds the words this crate *quotes* as well as its own.
- `crates/alo-overlay/tests/what_the_overlay_shows_at_rest.rs` — the
  acceptance, one test per criterion, plus the refusal paths.
- `crates/alo-overlay/Cargo.toml` — the four crates the readings come from,
  and `alo-saying` plus `serde_json` for the tests.
- `crates/alo-saying/src/collecting.rs`, `crates/alo-saying/Cargo.toml` — the
  seventeenth list. See *A defect found on the way* below.
- `Cargo.lock` — the new edges.
- `docs/autonomy/v0-01-delivery-plan.md` — task 3's `**Done,**` line, per the
  plan's own rule that whoever finishes writes it in the same change. Task 4
  already existed, so no new task was needed.

## A defect found on the way, and fixed here

**`alo-saying` did not collect `alo-overlay`.** Task 2's report says its words
were declared into the machine's one vocabulary and that the list grew to
sixteen crates; `git log -S alo-overlay -- crates/alo-saying/src/collecting.rs`
returns nothing, and the list on `main` was sixteen crates *without* it. So on
a real machine both of task 2's refusal sentences would have rendered as
`«overlay.summon.no-compositor»`, marked `Said::is_a_bug` — which is the honest
failure `alo-strings` is built to produce, and still the wrong thing in front
of somebody whose desktop is not running.

It is fixed in this change rather than reported, because *every string
externalised* is this task's own acceptance and a string nothing collects is
not externalised — it is a key nobody can translate. `alo-saying` now declares
`alo-overlay` (seventeenth), and the integration test asserts every word and
every counted string this crate declares is in
`everything_this_machine_can_say()`, so the next crate that forgets fails in
CI rather than on a screen.

Per `docs/autonomy/updates/README.md` this is written here rather than by
editing task 2's report.

## Decisions

The task said *decide rather than stop*; these are the decisions and why.

1. **The readings are read from the real crates, not from numbers a caller
   supplies.** `alo-overlay` now depends on `alo-capability`, `alo-choosing`,
   `alo-egress` and `alo-models` — the same crates `alo-agentd` walks when a
   turn really asks something. The alternative was a small port type the shell
   fills in, which is lighter and would have made *derived from the daemon's
   own answers* a claim in a comment rather than a fact in the type. It costs
   this crate the model stack's dependencies; `alo-saying` already pays that
   price for the same reason, and `lib.rs` there argues it.

   It is **not** a dependency on `alo-agentd`: that crate is Linux-only and
   compiles to nothing everywhere else, so an overlay behind it would be an
   overlay that exists on one host and not another. The daemon's answers are
   its inputs, and its inputs are portable.

2. **The state is derived and cannot be set.** There is no constructor taking
   a `Standing`, and `AtRest::of` — the door for a shell that was *told* what
   the machine holds over a socket — still derives it. A shell holding three
   readings therefore cannot show *ask me anything* over a machine that has
   chosen nothing, which is the one line on this surface worth lying with.
   Tested (`a_composed_value_still_derives_its_own_state`).

3. ***Nothing chosen* leads when both are true.** A fresh machine is in both
   empty states at once and one sentence has to come first. Choosing leads,
   because granting a folder on a machine with no model changes nothing a
   person can do, while choosing a model on a machine with no grants makes the
   next press of the key useful immediately. Both cases stay reachable and the
   readings underneath show the other half either way, so nothing is hidden by
   the order. `standing.rs` documents it and a test pins it.

4. **Four lines: a state, then three readings.** The state is what a person
   needs and the readings are what they check, so that is the order
   `AtRest::lines` answers in. It is reading order and not layout — nothing
   here names a size, a place, a colour or a surface, exactly as task 2 left
   it. A fixed `[Said; 4]` rather than a list, because a machine with nothing
   to say about one of the three would be a machine hiding something: each
   reading has a sentence for its empty case.

5. **Two sentences say what to do, not one.** The acceptance asks it of
   *nothing chosen*; *nothing granted* is written the same way for the same
   reason. `words.rs` has a test that neither can shrink back into a bare
   report — each must name its verb and be two clauses.

6. **The state is what the machine is *set to*, not what it would manage
   today.** `alo-agentd` distinguishes *nothing chosen* from *nothing running*
   because it is about to put a question somewhere; finding that out means
   reaching for a runtime, and the overlay opens on a keystroke. So a chosen
   model whose runtime is down reads as chosen here — which is true — and the
   refusal when the question is actually asked is `alo-models`' own, in its own
   words. **This is a known gap and the limitation is listed below.**

7. **The place a question would go is composed, not concatenated.** The line
   reads `{model} answers, {where}`, and `{where}` arrives through
   `Filling::and_said` from `alo_models::InferenceSource::said`. So a German
   line with an English *on this machine* inside it reports itself as
   untranslated rather than as German, which is the failure `alo-strings`'
   provenance rules exist for. Tested.

8. **Two counted strings rather than numbers in English sentences.** *One
   thing is granted* and *3 things are granted* are one string with the forms
   the reader's own language uses, declared as `Plural`s. A sentence with a
   number written into it is a sentence Polish cannot have.

9. **This is not task 9.** The egress reading is a line of text; the indicator
   on a screen is its own task, and both draw from `alo_egress::Indicator` so
   the light and the line cannot disagree.

10. **Nothing in `crates/alo-shell` was touched**, as in task 2: the desktop
    worker's compositor and window-control chain are untouched, and the
    wiring of this value into a surface waits for their return.

## Acceptance and verification

All commands run on Windows 11 (the host this checkout gates on; nothing in
this change is platform-gated). Executed 2026-09-10 against a clean tree at
`0bae838`.

| Check | Command | Result |
|---|---|---|
| Format | `cargo fmt --check` | clean |
| Lints | `cargo clippy --workspace --all-targets -- -D warnings` | clean, zero warnings |
| Whole suite | `cargo test --workspace` | all green: 144 result lines, 1 905 passed, **0 failed**, 1 ignored (pre-existing) |
| The two crates changed | `cargo test -p alo-overlay -p alo-saying` | 54 + 5 + 6 + 51 + 4 tests, 2 doctests, 0 failed |

Acceptance criteria, each with its test run **on its own** with `--exact`, all
in `crates/alo-overlay/tests/what_the_overlay_shows_at_rest.rs`:

- *the overlay's state is a value derived from the daemon's own answers, with
  a case for each of nothing granted, nothing chosen and ready* —
  `the_overlays_state_is_derived_with_a_case_for_each_of_the_three`
- *every string externalised* —
  `every_string_the_overlay_shows_is_in_the_machines_own_vocabulary`
- *the nothing chosen case says what to do rather than being empty* —
  `the_nothing_chosen_case_says_what_to_do_rather_than_being_empty`

Each of the three reaches the machine the way the daemon does: the person's
settings are written to a file and read back through `alo_choosing::Settings::at`,
the grants are an `alo_capability::Grants` asked what is active at a stated
moment, and what is leaving comes off an `alo_egress::Indicator` that really
permitted something. The strings come from
`alo_saying::everything_this_machine_can_say()` — the vocabulary a shell really
holds — rather than from a fixture of the test's own.

**Refusal paths, tested beside the legitimate ones.** In the integration test:
a provider choice is never shown as answering on this machine; an expired grant
is not shown as something the agent can reach, and the machine falls back to
*nothing granted* rather than claiming to be ready; an egress the policy
refused is never shown as leaving. In the crate's unit tests: a revoked grant
stops being counted immediately; another agent's grants are not counted as this
one's; reading the number a hundred times leaves the grants byte-for-byte as
they were; a provider that has not said where it runs is not shown as safe; a
vocabulary that never received this crate's list answers with the key marked as
this repository's bug rather than with a blank line or with English from the
source; and a translated line with an untranslated place inside it does not
claim to be translated.

*No pixels are claimed and none are tested*: nothing in the crate names a size,
position, colour or surface geometry, and no test asserts one.

## Limitations that remain

- **Nothing draws this yet.** `alo-shell` neither implements
  `alo_overlay::Compositor` nor builds an `AtRest`; that wiring crosses the
  desktop worker's chain and waits for their return (2026-09-15). The value is
  shaped so the wiring is one call and four lines of text.
- **A chosen model whose runtime is not running reads as chosen.** Decision 6
  says why, and the cost is real: a person whose runtime is down reads *ask the
  agent anything* and finds out when they ask. Closing it means probing at the
  moment the key is pressed, which is a decision about latency and about what
  an overlay may reach for, and it belongs with the task that wires the shell
  to the daemon rather than with this one.
- **A settings file that is there and does not hold is not a case here.**
  `alo_choosing::NotSet` is the refusal and it names the file; `AtRest` is
  built from settings that were read, so whoever reads them shows that refusal
  instead. A fourth state carrying it is a small, honest addition and is not in
  this task's acceptance.
- **No translations of the eleven new strings exist** — like every other string
  on the machine so far. They are declared, noted for the translator, collected
  into the machine's one vocabulary, and counted by `Strings::unanswered`.

## Proposed shared-document updates (integration owner's to make)

- `CHANGELOG.md`: the user-readable description quoted above.
- `docs/autonomy/QUEUE.md` / `STATE.md`: task 3 of the v0.01 delivery plan
  done, report at this path; task 7 now waits only on task 6. Worth noting the
  `alo-saying` collection defect above, because it means task 2's *strings
  reach a person* claim was true only after this change.
- No `ROADMAP.md` movement: this is one task inside phase 3's spine, not an
  exit gate. No box moves in *On the machine*.

## Status

Ready for integration.
