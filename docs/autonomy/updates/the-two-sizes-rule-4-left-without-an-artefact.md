# The two sizes rule 4 left without an artefact

**Date:** 2026-09-11
**Workstream:** v0.01 lane B — accounts and session entry
(`docs/autonomy/v0-01-lane-b-plan.md`, task 13)
**Contributor:** Claude Code worker, `C:\dev\alo-os-b`
**Status:** ready for integration

## What was wrong

Task 12 made `quantisation` a claim a catalogue entry has to be able to point
at, and two entries could point at nothing: `eurollm-9b-instruct` and
`teuken-7b-instruct` name publishers who ship no GGUF, so both now state no
quantisation at all. **Their sizes were not touched**, and that left the file in
a worse state than the one rule 4 fixed:

| | `eurollm-9b-instruct` | `teuken-7b-instruct` |
|---|---|---|
| `download_bytes` | 5_600_000_000 | 4_600_000_000 |
| bytes per parameter | 0.61 | 0.66 |
| `min_vram_gb` | 8.0 | 6.0 |
| `min_ram_gb` | 12.0 | 10.0 |

0.61 and 0.66 bytes per parameter is what a four-bit GGUF costs and nothing else
does. So both entries stated the size of an artefact they no longer claimed, and
`min_vram_gb` and `min_ram_gb` came from the same assumption — Teuken asked for
ten gigabytes of system memory beside weights that are fifteen. Rule 2's *sizes
are what the disk and the card actually lose* was being honoured for a file
neither entry named, and a person sizing a laptop off `min_ram_gb` would have
bought the wrong one.

It is worth saying what it was not. Neither figure was invented: both were true
when each entry claimed `Q4_K_M`. One half of a pair was corrected and the other
half was left, which is the ordinary way a data file goes wrong — and why the
fix here is a rule with arithmetic under it rather than two better numbers.

## The decision, and why

**The road taken is the publisher's own release.** Where no first-party
quantised artefact exists, an entry states the weights its publisher actually
publishes, read off that repository's own file list.

The alternative was to name a third party's requantisation — `mradermacher`,
`bartowski`, `QuantFactory` and `lmstudio-community` all publish Q4_K_M of both
models. It was refused for the reason `docs/quirks.md` already gave about
Teuken: this catalogue would be putting its authority behind a file it never
chose, and choosing whose is a decision with nobody's name on it. The plan's
task 13 said in as many words that if the honest answer required a third party's
artefact, that belonged in an ADR — and the honest answer does not require one,
because the publisher's own release exists, is checkable, and is the thing the
`upstream` link already points at.

That road has a cost and the cost is stated rather than softened: both entries
are now large, `on_cpu = "slow"`, and out of reach of an ordinary laptop. That
is the true sentence about a model nobody has quantised for us. It is also why
**task 14 is the ADR** — refusing to name a stranger's artefact means the two
European entries, which the catalogue leads with *because nobody else lists
them*, are the two it can say least about, and somebody has to decide whether
that is acceptable rather than have it fall out of two refusals.

### The figures, off the publishers' own manifests

Read from the Hugging Face file lists of each entry's `upstream` on 2026-09-11:

| | `eurollm-9b-instruct` | `teuken-7b-instruct` |
|---|---|---|
| safetensors shards | 4_991_396_632 + 4_983_059_672 + 4_999_836_776 + 3_330_390_280 | 4_936_228_560 + 4_929_565_048 + 4_929_565_072 + 110_125_512 |
| `download_bytes` | 18_304_683_360 | 14_905_484_192 |
| parameters in the index | 9_152_319_488 | 7_452_725_248 |
| bytes per parameter | 2.00 | 2.00 |
| `min_vram_gb` | 20.5 | 17.0 |
| `min_ram_gb` | 24.0 | 21.0 |

Two bytes per parameter, because both releases are `bfloat16`. The memory
figures are the weights plus roughly two gigabytes of working room on a card and
six in system memory, which is the headroom the four-bit entries around them
already carry.

### Two things that moved with them

Recorded here rather than slipped in, because neither is a size:

- **Teuken's `parameters_b` was 7.0** — the publisher's product name rather than
  the count in its own manifest, which is what every other entry in this file
  states (`mistral-7b-instruct` says 7.2, `qwen2.5-7b-instruct` says 7.6). It is
  now 7.5. Without this the bytes-per-parameter rule below would be measured
  against a number that is not the model's.
- **Teuken's `on_cpu` moves from `workable` to `slow`.** That word was true of a
  four-bit download and is not true of fifteen gigabytes of `bfloat16` on a
  processor, by the definition `OnCpu` itself gives. `eurollm-9b-instruct` was
  already `slow`.

**No grade moved.** A size is not a measurement of driving; both entries stay
`not-measured`, and nothing in this change ran `alo-driving`.

## What changed

### The rule, as arithmetic — `crates/alo-models/src/catalogue.rs`

Rule 5 is enforced by `Catalogue::parse` rather than asked of a curator, so it
cannot rot the way rule 4's other half did:

- `Model::bytes_per_parameter()` divides `download_bytes` by `parameters_b`. An
  entry that names a quantised artefact must land **below** 1.5 bytes per
  parameter and one that names none must land **above** it. Every four-bit entry
  in the catalogue we ship is between 0.56 and 0.80 and `bfloat16` is 2.0, so
  the line separates the two claims without being near either. This is the exact
  refusal the bug needed: a four-bit figure cannot be left behind on an entry
  that has given up its quantisation.
- A size below 0.35 bytes per parameter (smaller than any served quantisation)
  or above 4.5 (larger than `float32`) is refused as not being about the same
  model as the parameter count beside it.
- `Model::weights_gb()` is the floor under both memory figures: `min_vram_gb`
  and `min_ram_gb` are refused below the size, because a machine that cannot
  hold the weights cannot run them at any speed. This is the check that would
  have caught Teuken's ten-beside-fifteen.
- `parameters_b` and `min_ram_gb` are now required to be stated and positive.
  `parameters_b` had no check at all, and the ratio is meaningless without it.

### The catalogue — `crates/alo-models/data/catalogue.toml`

Four rules became five. Rule 5 states the road, what was refused, and both
halves of the arithmetic, because the next curator reads the file's own rules
and not this lane's plan. Rule 2 now points at it. Both entries carry the new
figures with a comment naming the shards they were read from and the figures
they replaced.

### The measurement — `docs/quirks.md`

- The carry-or-fetch table carries both new sizes. Correcting a size moves the
  measurement ADR 0025 owes, and `the_carry_or_fetch_measurement.rs` insists on
  the agreement rather than suggesting it.
- A paragraph beside the table says the two rows grew and why, and the channel
  half now distinguishes the four-bit 7B class (4.4–4.9 GB, the order the image
  already moves) from these two at 14.91 and 18.30 GB — which says something the
  four-bit rows hid: a model nobody has quantised for us is not a layer this
  channel carries comfortably. **The verdict is untouched**: neither entry was
  ever a candidate, because neither is measured.
- A new entry, *Two entries kept a four-bit size after they stopped claiming a
  four-bit file*, records the fault, the road and the arithmetic.

### One count outside this crate — `crates/alo-driving/tests/`

`Catalogue::to_choose_from_on_cpu(16.0)` no longer offers Teuken: 21 GB of
system memory and `slow` both take it out. So
`the_catalogue_we_ship_now_refuses_for_the_reason_a_measurement_gave_it` reads
six rather than seven, with the reason in its doc comment. This is the
correction working — the entry was on that list because of a size belonging to a
file it does not claim. The measured count and the sentence a person is shown
are unchanged.

### Six fixtures that said `download_bytes = 1`

`catalogue.rs`'s own test entries and `choosing.rs`'s fixture builder stated a
token size that rule 5 now refuses. `choosing.rs` derives its sizes from the
parameter count (0.62 bytes each, the four-bit figure the shipped entries
actually cost) with memory headroom that keeps every fixture model inside the
16 GB those tests ask about; `catalogue.rs`'s entries state real four-bit sizes,
and the quantisation-pair test's two accepted shapes now carry the size each
claim implies.

## Verification

Windows 11, `C:\dev\alo-os-b`, run from the checkout before this report was
written. These are the first attempt's runs; the second attempt's are below,
and they are the ones this handover stands on.

| Command | Result |
|---|---|
| `cargo fmt --all` | clean |
| `cargo clippy --all-targets -p alo-models -p alo-driving -p alo-telling -p alo-choosing -- -D warnings` | zero warnings |
| `cargo test -p alo-models` | 149 + 6 + 5 + 6 + 6 + 10 + 2 doc = all pass |
| `cargo test -p alo-driving -p alo-telling` | all pass |

`alo-driving` and `alo-telling` were run although neither was the task's subject:
both read the catalogue this change edits, and `alo-driving` held the count that
moved.

**Not run here:** the full workspace suite, which the supervisor runs. The
`alo-agentd` tests that call `Catalogue::built_in()` are Linux-only and were not
reachable from this host; none of them asserts a size, a memory figure or a
choice count (checked by search), and the catalogue loads or the service refuses
to start, which is `starting.rs`'s existing behaviour either way.

**Not measured, deliberately:** nothing in this change ran a model. Both entries
remain `not-measured`, and under the road taken neither can be measured at all
until task 14's decision is made — measuring the `bfloat16` release would need a
machine far past the one that could not hold a four-bit 7B, and measuring a
requantisation is the thing this change refused.

### Evidence

One line per acceptance criterion in the plan. Each was run on its own.

| Criterion | Workspace | Crate | Target | Test |
|---|---|---|---|---|
| Each of the two entries states a size, a video-memory figure and a system-memory figure a reader can check against something that exists | `.` | `alo-models` | `tests/sizes_an_entry_can_point_at.rs` | `the_two_entries_that_name_no_artefact_state_their_publishers_own_release` |
| `docs/quirks.md`'s carry-or-fetch table is brought back into agreement in the same change | `.` | `alo-models` | `tests/sizes_an_entry_can_point_at.rs` | `the_carry_or_fetch_table_carries_the_two_corrected_sizes` |
| Whichever road is taken is written into the catalogue's own rules beside rule 4 | `.` | `alo-models` | `tests/sizes_an_entry_can_point_at.rs` | `the_road_taken_is_written_into_the_catalogues_own_rules` |
| **Constraint:** no grade moves | `.` | `alo-models` | `tests/sizes_an_entry_can_point_at.rs` | `correcting_a_size_moved_no_grade` |
| **Refusal:** every way a size can stop belonging to the artefact its entry names | `.` | `alo-models` | `tests/sizes_an_entry_can_point_at.rs` | `a_size_that_belongs_to_no_artefact_the_entry_names_is_refused` |
| **Refusal:** the shipped catalogue states no size from a precision it does not claim | `.` | `alo-models` | `src/catalogue.rs` (unit) | `catalogue::tests::the_catalogue_we_ship_states_no_size_from_a_precision_it_does_not_claim` |

The refusal test puts eight faults in front of the loader, starting from two
sound entries so each refusal is about the fault and not about a fixture that was
never loadable: the four-bit size left behind (the actual bug), its mirror on a
quantised entry, a token size, a size larger than full precision, a card too
small for the weights, ten gigabytes beside fifteen (Teuken exactly as it
stood), a missing parameter count and a missing memory figure.

### Why the supervisor's gate refused this once, and what the second attempt changed

The first handover of this work was refused by `clippy, warnings denied` with
three `E0599`s — `no method named bytes_per_parameter`, `no method named
weights_gb` — reported against
`crates/alo-models/tests/sizes_an_entry_can_point_at.rs`. **No source change was
needed to make them go away, and none was made:** both methods were in
`crates/alo-models/src/catalogue.rs` when the gate ran, and they still are, byte
for byte.

The refusal was a stale build unit, and the gate's own output says so if it is
read closely. `Catalogue::parse` calls `bytes_per_parameter()` at
`catalogue.rs:385` and `weights_gb()` at `catalogue.rs:417`, and the shipped
catalogue's unit test calls both again — yet **not one of those lines errored**.
Only the new test file did. A build in which the library genuinely lacked those
methods could not have got past the library. So the library did not get rebuilt:
Cargo held that unit fresh and handed the new test target an `.rmeta` from
before the methods existed. The one `Checking alo-models v0.0.1` line in the
output is Cargo announcing the *package*, which it prints when any target of it
needs building — here the new test file, which had never been compiled and so
could not be fresh.

This is the fault `gates.rs`'s own `forget_what_was_built_of` was written for
after it parked finished work three times, and its rustdoc names the cause: the
checkout is on `/mnt/c`, every source mtime crosses drvfs from Windows, and
Cargo's freshness test is delicate there. That guard is best-effort by design —
`drop(asking.output())` — and the `wsl` bridge it runs over had already failed
transiently once in this session's `loop.log`
(`Wsl/Service/0x8007274c`). A `cargo clean -p` that silently did not happen
leaves exactly this.

What the second attempt did, in the shared Linux target directory the gates use:
`touch` on the five changed source files from inside WSL, so their mtimes come
from the Linux clock rather than across drvfs; `cargo clean -p alo-models`; then
every gate below from cold. The whole-workspace clippy that refused now passes
from a cleaned `alo-models` in 21 seconds.

**This is worth a durable fix and it is not this task's.** The guard should
refuse the gate run rather than proceed when its clean could not be executed —
a clean that fails silently converts an environment fault into a refusal of
somebody's finished work, which is the expensive failure this loop keeps paying
for. That belongs in `tools/kernel-loop/src/gates.rs`, as its own change with
its own tests, and it is proposed to the integration owner below rather than
mixed into a catalogue correction.

### Gates run on the second attempt

Windows 11, `C:\dev\alo-os-b`, every command through `wsl -d Ubuntu` with
`CARGO_TARGET_DIR=$HOME/target-claude` — the same bridge, toolchain and target
directory the supervisor's gates use, because a check run somewhere else is a
check of a different machine.

| Command | Result |
|---|---|
| `cargo fmt --all`, then `cargo fmt --all --check` | clean, no file changed |
| `cargo clippy --workspace --all-targets -- -D warnings`, after `cargo clean -p alo-models` | zero warnings |
| `cargo test -p alo-models` | 149 + 6 + 5 + 6 + 6 + 10 + 2 doc, all pass |
| `cargo test -p alo-driving` | 25 + 0 (1 ignored, needs a model) + 5, all pass |
| `cargo doc -p alo-models -p alo-driving --no-deps`, `RUSTDOCFLAGS=-D warnings` | clean |
| each of the six evidence tests, `--exact`, on its own | 1 passed each |

The full workspace suite was not run here: the supervisor runs it, and two
finished tasks have already died at the ninety-minute deadline waiting on it.

## Files touched

- `crates/alo-models/src/catalogue.rs`
- `crates/alo-models/src/choosing.rs`
- `crates/alo-models/data/catalogue.toml`
- `crates/alo-models/tests/sizes_an_entry_can_point_at.rs` (new)
- `crates/alo-driving/tests/from_a_prompt_to_what_a_machine_offers.rs`
- `docs/quirks.md`
- `docs/autonomy/v0-01-lane-b-plan.md`
- `docs/autonomy/updates/the-two-sizes-rule-4-left-without-an-artefact.md` (new)

## Limitations and what is left

- **The two European entries cannot be graded.** That is the state this change
  makes visible rather than one it creates, and task 14 is the decision it waits
  on.
- **The bytes-per-parameter line is a rule about published artefacts, not a law
  of arithmetic.** A publisher who shipped `float8` weights would land near 1.0
  and be read as quantised. Nobody in this catalogue does, the rule refuses
  nothing that exists today, and the alternative — a field saying which precision
  the size is — would be a third half of the same claim for a curator to get
  wrong. If such an entry arrives, the rule is where to change it and the
  refusal will say so in words.
- **The memory figures are reasoned, not measured.** They are the weights plus
  the headroom the entries around them carry. Nothing in this repository
  measures what a model costs to run, and `crates/alo-models/src/costing.rs`
  says why a multiplier somebody guessed would be worse than a stated figure.

## Proposed shared-document updates

For the integration owner (`SHARED_MAIN.md` — this contributor does not edit
`CHANGELOG.md`, `ROADMAP.md`, `QUEUE.md` or `STATE.md`).

**CHANGELOG.md**, under the current release:

> **The catalogue's sizes belong to artefacts that exist.** Two models —
> EuroLLM 9B Instruct and Teuken 7B Instruct — are published by people who ship
> no four-bit file, and their stated download sizes and memory requirements were
> four-bit figures for artefacts nobody publishes. They now state what their
> publishers actually publish, which is roughly four times larger, and the
> catalogue says plainly that neither will run on an ordinary laptop. A new
> rule refuses any entry whose size does not match the file it names, so the
> next one cannot be written down.

**A change to propose, owned by whoever owns `tools/kernel-loop`:**
`forget_what_was_built_of` drops its clean's result. When that clean cannot run
— a transient `wsl` failure is enough, and one is in this session's `loop.log` —
the gates proceed against whatever stale units are in the shared target
directory, and a worker's finished work is refused for a method that is in the
tree. It should be a gate that can fail rather than an attempt that cannot, with
its own test saying so. This task's first attempt was refused by exactly that,
and the ninety-minute worker deadline makes each such refusal expensive.

**STATE.md:** references this report path. Task 13 of
`docs/autonomy/v0-01-lane-b-plan.md` is done; task 14 (an ADR on whether this
catalogue may name a third party's requantisation) is written and ready. No task
in `v0-01-delivery-plan.md` matched this one, so nothing was marked there.
