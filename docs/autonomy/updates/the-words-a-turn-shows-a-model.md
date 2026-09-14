# The words a turn shows a model are the product's own

**Date:** 2026-09-14.
**Workstream:** the models, measured — `docs/autonomy/v0-5-the-models-measured-plan.md`, task 18.
**Contributor:** the Mac lane (measuring), on an Apple M3 with 8 GB unified
memory, macOS 26.5.2; gates in the Linux VM `alo` (Lima, `vz`, aarch64, 6 CPUs,
4 GiB).
**Status:** ready for integration.

## What a person would notice

Nothing today, and that is the point of the finding. When the turn is wired
(the task written for it below), an agent asking the machine's own model for its
next request will be shown words this operating system wrote: how to answer,
every verb the machine really has in the sentence the verb itself declares, and
the person's request last — the same words, built by the same function, that
every grade in the catalogue will have been earned under.

## What was found

`alo_turn::Turning::asking_for_the_next_request` takes what a model is shown as
a `&str`. Its own rustdoc says what that string is — *what the agent composed
for the model — its instructions, the verbs and what the person said* — and
**nothing in this repository composes one**. It arrives through the daemon's
door as a question, from a client outside the OS.

The only such text that exists is `alo_driving::prompt_under`, in the
measurement harness. So every grade this lane has produced — including the first
one to clear the bar, 80 of 80 on 2026-09-14 — was a measurement of words a
shipped machine does not show a model. ADR 0034 wrote the risk down as a cost
(*if a turn is shown different instructions from the ones its model was graded
under, the grade says nothing about that turn*) and left the remedy with
whoever writes the turn's prompt; reading the code, there is no such prompt.

Task 16 had just measured how much this matters: same weights, same runtime,
same exercises, same envelope — **80 of 80** under one set of instructions,
**71 of 80** under the other.

## What changed

**A crate for the words: `crates/alo-instructing`.**

- `src/instructions.rs` — the two sets of instructions, moved byte-for-byte from
  `alo-driving` (`HOW_TO_ANSWER` came from `exercise.rs`, `ONE_EXAMPLE_PER_DOOR`
  and `Instructions` from `instructions.rs`), with `Instructions::SHOWN_TO_A_TURN`
  added: the set an agent turn shows.
- `src/verb_as_told.rs` — one verb as a model is told about it: its name, its
  door read off `alo_capability::Effect`, its own `purpose_as_written`, and each
  argument with the bound that argument declared.
- `src/shown.rs` — `shown_to_a_model(instructions, verbs, request)` and
  `shown_to_a_turn(verbs, request)`.
- `src/instructions_first_written.sha256` — the pinned digest of the first
  instructions, moved with the text (`git mv`), which is what proves the move
  changed nothing.
- `tests/nothing_behind_these_words.rs` — reads the crate's own manifest and
  asserts the dependency list is exactly `alo-capability` and `ring`, and reads
  every shipped source file for a way off the machine (`std::net`, `std::fs`,
  `ureq`, `Asking`, `Catalogue`, …). The reason the crate exists is that a
  daemon can take the words without taking a catalogue, an HTTP client and a
  settings store; that is a claim about a dependency list, so it is read rather
  than promised.

**`alo-driving` composes from it.** `prompt_under` is now
`shown_to_a_model(instructions, verbs, exercise.asked())`; `src/instructions.rs`
is gone and `Instructions`, `HOW_TO_ANSWER` and `ONE_EXAMPLE_PER_DOOR` are
re-exported from `lib.rs`, so no caller's path changed. A new test,
`an_exercise_is_asked_in_the_words_a_turn_would_be_shown`, asserts a measurement
adds nothing to those words and drops nothing from them, under both sets.

**[ADR 0037](../../decisions/0037-the-words-a-turn-shows-a-model-are-the-products-own.md),
accepted.** The words are the product's (decision 1); the set a turn shows is
`OneExamplePerDoor` on the measurements rather than on taste (decision 2); **no
grade moves until a turn composes from the crate** (decision 3); and when it
does, the grade that decides is the one earned under that digest, with no grade
at all for an entry never measured under it (decision 4).

**`docs/quirks.md`** gains *a turn asks a model in English, whatever language the
machine runs in* — the limit that was a fact about a measurement and becomes a
fact about the product the day a turn uses these words. Recorded before it is
true rather than after.

**`crates/alo-models/src/graded_for_turns.rs`** — prose only. It says why nothing
moves here yet and what it becomes when the wiring lands. No logic, no grade and
no catalogue byte changed.

## Decisions taken in this task

1. **A crate of its own, not a dependency on `alo-driving`.** Letting the turn
   depend on the harness would put `alo-models`' catalogue, `ureq`, a settings
   store and a runtime client behind a privileged daemon, and would make the
   second half of `alo-driving`'s own header (*it is not run on a machine*)
   false. The options and what each costs are in the ADR.
2. **`OneExamplePerDoor` is what a turn shows.** ADR 0034 decision 4 left this
   to the lane that owns the turn. The turn had no words at all, and the lane
   that made the measurement is the one that can name which text they are:
   80 of 80 against 71 of 80 through the pinned runtime, 80 of 80 against 65 of
   80 through `llama.cpp`'s server. A lane that disagrees overturns it with a
   measurement, which the ADR says how to make.
3. **Nothing about grades moved.** Reading the enveloped-and-one-example-per-door
   grade for a turn that is still shown a client's words would be the same
   mistake one variable on. `Model::grade_for_the_turn` is untouched.

## Verification

Every command ran in the Linux VM `alo` (Lima, aarch64), from
`/Users/disanssebowabasalidde/dev/alo-os`, on the tree this report is committed
with. The Mac itself gated nothing.

Every gate was run as root — `limactl shell alo sudo bash -lc …`, which is what
`a-loop-on-a-mac.md` says the gates are — building in `/root/alo-builds/alo-os-main`,
the directory the loop uses.

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo test --workspace --no-fail-fast` | **4,846 passed, 1 failed, 26 ignored** — the one failure is another lane's and is described below |
| `cargo doc --workspace --no-deps`, `RUSTDOCFLAGS=-D warnings` | clean, 52 crates |
| `cargo fmt --all --check` in `tools/kernel-loop` | clean |
| `cargo clippy --all-targets -- -D warnings` in `tools/kernel-loop` | clean |
| `cargo test` in `tools/kernel-loop` | 115 passed, 0 failed |
| `cargo fmt --all --check` in `crates/alo-bounding-kernel` | clean |
| `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings` | clean — **both BPF gates pass on aarch64**, which this lane had not established before |

`cargo test -p alo-instructing -p alo-driving --locked` on its own: 13 in the new
crate (eleven unit — four on the instructions and their digests, seven on the
words — and the two that read the manifest and the source), 23 unit and 14
integration in `alo-driving`, 0 failed.

### The one failing test, and why it is not this change's

`-p alo-bounding --test the_boundary_decides_and_forgets`,
`ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`, fails at
line 364: *an ordinary program can set a flag on its own files: Os { code: 95,
kind: Unsupported }*. It fails **deterministically**, alone and in the suite, and
it is the only failure among 4,846 tests.

Three things say it is the machine's and not this change's, and they were
checked rather than assumed:

1. `alo-bounding` depends on **none** of `alo-instructing`, `alo-driving` or
   `alo-models` (`cargo tree -p alo-bounding -e normal,dev`), and no file in it
   was touched.
2. The **same failure, with the same message**, is in this VM's own gate log
   from the loop's last run (`/root/alo-builds/gate-the_workspace's_tests.log`,
   2026-09-14 21:43).
3. With no boundary attached, `chattr +d` on a file in the same `/tmp` succeeds
   on this kernel (`7.0.0-31-generic`, aarch64), so the flag itself is
   supported.

Recorded in `docs/quirks.md` for `alo-bounding`'s owner — the crate is not this
lane's — with the measurement that would settle it. No gate was weakened, no
test was ignored, and nothing about it was worked around.

### Gated twice, because `main` moved

`main` advanced while this was being gated — lane A's *a self-hosted workspace on
the network is found, not configured* (`alo-nearby`, `alo-agentd`,
`alo-protocol`, `alo-changing`) and a `tools/kernel-loop` fix. This was rebased
onto it and **every gate above was run again on the combined tree**: fmt clean,
clippy clean, rustdoc clean over 52 crates, the supervisor's 115 tests, both BPF
gates, and `cargo test --workspace --no-fail-fast` at **4,872 passed, 2 failed,
26 ignored** — the deterministic `alo-bounding` failure described above, and one
`EBUSY` transient in the same crate's fixture
(`what_a_turn_inherits.rs:328`) which passes alone, 10 of 10, and which
`docs/quirks.md` already records. The rebase also renumbered the task written
for lane A from 18 to 19: lane A had taken 18 meanwhile.

**Two environmental failures were tidied, not worked around.** The first run of
the suite failed five `alo-agentd` boundary tests on debris this VM already had:
a stale `home` control group in `session-4.scope` and two pins
(`alo-agentd-gone-38200`, `alo-agentd-test-38198`) whose processes no longer
existed — exactly the cascade `docs/quirks.md` describes. Removing only debris
whose pid was gone, the five pass. Two later runs hit the recorded `EBUSY`
transient in `alo-bounding`'s own fixture (`a_turn_is_this_thread.rs:127`), which
passes alone; the `--no-fail-fast` run above is the complete picture.

**No measurement was made in this task.** No model was loaded, no runtime was
started, nothing was fetched and no grade was earned or changed. The numbers
quoted above (80 of 80, 71 of 80, 65 of 80) are tasks 16 and 17's, already in
the catalogue and their own reports.

## Limitations

- **The turn is not wired.** Until the task below lands, a model on a shipped
  machine is still shown whatever a client composed, and the catalogue's grades
  are still about the harness's words. This task ends at the words.
- **No hardware is claimed.** Nothing here runs on a certified machine, and no
  *On the machine* box moves.
- **The words are English.** See the quirk added in this change. A turn asking
  in a person's own language needs the verbs' sentences translated and a grade
  per language; neither exists.
- **One model, two instruction sets, one machine** is the whole evidence behind
  decision 2. It is the best measurement anybody has made here and it is not a
  fleet: task 20 is what makes it one.
- **One workspace test fails on this VM** and is another lane's, above and in
  `docs/quirks.md`. The whole-suite gate is therefore green except for it, which
  is stated rather than rounded up.

## Proposed shared-document updates

- **`CHANGELOG.md`:** *The words a model is shown before it is asked for a
  request are now the operating system's own, in one place — how to answer,
  every verb in the verb's own sentence, the request last — so that the model a
  machine grades and the model it asks are shown the same text.*
- **`ROADMAP.md`:** nothing. No capability is finished by this and no machine
  half is touched.
- **Plans:** `v0-5-the-models-measured-plan.md` marks task 18 done and adds
  tasks 19 (blocked) and 20 (ready); `v0-5-the-local-network-plan.md` gains task
  19, *a turn shows a model the words the product wrote*, for the lane that owns
  `alo-turn` — written there because a finding in a report is not a queue, and
  task 14 set the precedent when it handed the envelope door over. (It was
  written as task 18 and renumbered on the rebase: lane A published a task 18 of
  its own while this was being gated.)
