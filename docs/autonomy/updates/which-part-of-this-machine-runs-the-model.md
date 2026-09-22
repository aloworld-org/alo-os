# Which part of this machine runs the model

**Date:** 2026-09-22
**Workstream:** `docs/autonomy/v0-5-the-models-measured-plan.md`, task 22 —
*The graphics card, when there is one, and the machine when there is not*
**Machine:** the third PC, `AGAI01`, Windows Server 2022 with WSL 2 Ubuntu
(kernel `6.18.33.2-microsoft-standard-WSL2`), x86_64, **no discrete graphics**
**Contributor:** the models-measured lane on the third PC
**Status:** ready for integration. The processor road is measured here; the
card road is `- [x] The code.` and waits on task 23's hardware.

## The sentence this closes, and the half it does not

[ADR 0007](../../decisions/0007-the-cpu-is-the-default.md) settled three weeks
before this was written that *the CPU is the default; a GPU is acceleration*
and that *a GPU changes speed, not capability*. **Nothing implemented it.**
`on_the_gpu_bytes` was `None` at every site in the repository, because nothing
ever looked at a card — so a machine with a graphics card ran exactly as slowly
as a machine without, and a person who asked *is my card being used* had no way
to find out except by timing something.

After this change a machine answers, in the person's own language:

```
this machine draws with: NoCard
this machine's road: OnTheProcessor(NoCard)
  said: this machine has no graphics card, so it is running the model on its processor
```

That is this machine, measured by `this_machines_own_road_is_measured_and_said`
on 2026-09-22 — `/sys/class/drm` holds a `version` file and nothing else, there
is no `/dev/dri`, and `nvidia-smi` is not on the machine. **The card road is
not shown from here and is not claimed from here.** Task 23 is that half, and
this report says exactly which machine it needs.

## What changed

### `crates/alo-models/src/card.rs` — what this machine has to draw with

New. Reads `/sys/class/drm` the way `alo_power`'s battery reads
`/sys/class/power_supply`: the kernel's own files, no daemon, no program of
ours run as a subprocess. `WhatDrawsHere::on_this_machine()` is the door;
`WhatDrawsHere::among(&Path)` is the same reader pointed at a folder, which is
how one machine is held to the answer it would give on five.

Three answers rather than a list that might be empty, and the difference is the
point:

| | |
|---|---|
| `TheMachineWouldNotSay` | the kernel keeps no such list. Not a card, and **not** *no card* |
| `NoCard` | the list is there and nothing on the bus draws |
| `These(Vec<ACard>)` | these, in the order the kernel lists them |

An `ACard` carries the vendor from the bus (`device/vendor`), **the driver
bound to it or `None`**, and the memory it publishes or `None`.

**Two readings that were easy to get wrong, and are tested:**

- **A card with no driver is a card.** The task named this and it is not a
  detail: the base is rented and unmodified ([ADR 0011](../../decisions/0011-the-base-is-rented-and-the-image-is-a-container.md))
  and ships no proprietary driver, so an NVIDIA machine with no NVIDIA driver
  is the likely state of every NVIDIA machine alo OS meets today. A person told
  *no card* goes looking for hardware they already own.
- **A connector is not a card.** `/sys/class/drm` carries `card0`,
  `card0-eDP-1`, `renderD128` and a `version` file; counting the second would
  report a laptop with one card and two screens as having three cards.

**A card that will not say how much memory it has says `None`, never nought.**
`amdgpu` publishes `mem_info_vram_total`; the proprietary NVIDIA driver
publishes nothing of the kind. Nothing here guesses one from a model name or
from the catalogue — that is the assumption ADR 0007 rejected — and what a card
really held is read from the runtime instead.

### `crates/alo-models/src/road.rs` — which road, and why

New. `Road::of(residency, draws_here, cards_it_can_use, the_weights_need)`.

**The card is the runtime's answer; the reason is the machine's.** Whether a
card was used is a question only the runtime can answer, because only it knows
what it put where: `Loaded::on_the_gpu_bytes` above zero **is** the card road,
and nothing decides it from a vendor table. Why *not* comes from the bus, and
there are six reasons, none of them a failure:

| `WhyTheProcessor` | |
|---|---|
| `NoCard` | nothing on the bus draws — the fleet this product exists for |
| `NoDriverForTheCard { made_by }` | a card is here and nothing can reach it |
| `TheRuntimeCannotUseThatCard { made_by }` | the pinned runtime does not use this vendor's |
| `NotEnoughOnTheCard { the_card_has, the_weights_need }` | it said, and it is smaller than the weights |
| `TheRuntimeLeftItThere` | a usable card, and the runtime loaded onto the processor anyway |
| `TheMachineWouldNotSay` | no list to read, so nothing is claimed either way |

Where a machine has several devices, **the one nearest to usable decides the
sentence**, because that is the one a person could act on: a laptop with an
integrated Intel processor and a driverless NVIDIA card is told about the
driver, not about a vendor whose device was never going to run a model.

`Road::recorded_in(&mut MeasuredOn)` is what finally puts a real device's
figure into `on_the_gpu_bytes`. The processor road writes **neither** figure
rather than writing nought, because `MeasuredOn` refuses half a residency and a
residency nobody read is not a residency of nought.

### `crates/alo-models/src/runtime.rs` — the residency, whole

`Loaded` carried only `vram_bytes`. It now carries **`loaded_bytes` and
`on_the_gpu_bytes`** — the pair `MeasuredOn` records and refuses half of. That
is why every residency beside a grade in `data/catalogue.toml` had been copied
out by hand: the code could not produce the pair.

`ModelRuntime` gains `cards_it_can_use()`, **defaulted to none**, so no
existing implementor changes and a runtime that has not said is read as having
said nothing. Which cards a runtime accelerates is as much a fact about that
runtime as an endpoint path is ([ADR 0006](../../decisions/0006-the-pinned-model-runtime.md)),
so the list lives in the one file allowed to know the runtime exists.

### `crates/alo-models/src/ollama.rs` — what the runtime says, and one new way to ask

`/api/ps`'s `size` is now read beside `size_vram`.
`CARDS_THE_PINNED_RUNTIME_USES` is `[Nvidia, Amd]` — its releases ship CUDA and
ROCm and nothing for anybody else's card.

`answers_in_the_envelope_on` and `answers_on` take a `WhichRoad`.
`WhichRoad::TheProcessor` sends `"options":{"num_gpu":0}`;
`WhichRoad::AsTheMachineIs` sends **no `options` field at all**, so every
question a person's turn asks is byte for byte the request it was before this
change — asserted against a real socket in
`a_question_that_names_no_road_is_the_request_that_was_always_sent`, which also
holds that the two requests differ by that field and nothing else.

### `crates/alo-driving/src/both_roads.rs` — the two roads held to each other

New. `BothRoads::of(OnARoad, OnARoad)` takes one run from each road, in either
order, and **refuses a pair that is not one of each** — `NeitherWasOnACard` or
`BothWereOnACard`. On a machine with no card the only pair available is two
processor runs, and that refusal is what stops this file passing green while
showing nothing.

### `crates/alo-models/src/words.rs` — seven sentences

`models.road.*`, with two rules held as tests over `ABOUT_THE_ROAD`:

- **none of them says how long anything took** — the road is asked for and
  answered, and a sentence that offered speed as the answer would teach a
  person to go back to inferring it from a stopwatch;
- **none of them apologises** — ADR 0007 rejected CPU support as a degraded
  mode with warnings, on the grounds that a default which apologises for itself
  teaches people the product is not for them.

## Decisions I made, and why

**1. *The same answers from both* is the same grade, not the same text.** The
acceptance asks that one question be put to each road and the results held to
each other. Held to *identical text* it would be a test of the sampler: a model
differs from itself between two runs on **one** road, which `measured.rs`
already says where it explains why repeats are a larger sample rather than a
different method. So the roads are held at the resolution the product actually
uses — `Driving`, the grade the catalogue records and the thing that decides
whether a machine is given an agent. *A GPU changes speed, not capability* is,
exactly, *the grade is the same on both roads*. Per-exercise differences are
**reported** by `where_they_differed()` and deliberately not asserted.

**2. Which vendors the runtime can use is asked of the runtime.** A table
outside the adapter would be a second statement of what Ollama does, stale the
day the pinned release moves. It went on `ModelRuntime` with a default of none
rather than as a new required method, so no implementor outside this crate
changes — `alo-agentd`, `alo-asking` and `alo-turn` all keep their stubs
untouched.

**3. The road a card's memory is judged against is the weights' own size.**
Never `min_vram_gb`. ADR 0007 took that figure out of the offering decision and
this task was exactly the change that could have put it back;
`what_a_machine_offers_does_not_change_with_what_is_on_its_bus` holds both
halves — the offered list is identical across four different buses, and a card
of three gigabytes is enough for a road whose entry states `min_vram_gb = 4.0`.

**4. The driver is read from `uevent`, not only from the `driver` symlink.**
The kernel writes `DRIVER=` there exactly when a driver is bound, which is the
question this file asks; it is also a plain file, so a test can write a machine
that does not exist without needing the privilege a symlink costs on some
hosts. The symlink is still read where `uevent` is silent, because a card with
a driver must never read as one without.

**5. The two-road comparison asks through `Ollama`, not through `alo-asking`.**
`measuring/mod.rs` puts a graded run through `alo-asking`'s local door so that
a grade is earned the way a turn asks. There is no road in that door, and
putting one there is a change to a crate this plan does not own. What the
comparison needs is that the **two runs are asked identically**, and they are:
the same method, the same machine, the road as the only difference — which the
wire test above holds byte for byte. Adding a road to `alo-asking`'s local door
is a reasonable follow-up for whoever owns it; it is not needed for either
half of this acceptance.

## Verification

Run from the Windows checkout through WSL 2 Ubuntu, in the serialised Linux
source copy (`/root/alo-trees/this-machine`) against the shared build directory
(`/root/alo-builds/this-machine`), with mold and `RUSTDOCFLAGS=-D warnings`,
which is how `tools/kernel-loop/src/gates.rs` runs a gate on this host.

| Command | Result |
|---|---|
| `cargo fmt --all` | clean (run against the checkout) |
| `cargo clippy -p alo-models -p alo-driving --all-targets -- -D warnings` | clean |
| `cargo clippy` over **every crate that depends on `alo-models`** (22 crates) `--all-targets -- -D warnings` | clean, 45 s |
| `cargo test -p alo-models` | 245 unit + 71 integration + 2 doc tests, 0 failed |
| `cargo test -p alo-driving` | 29 unit + 16 integration, 0 failed, 3 ignored |
| `cargo doc -p alo-models -p alo-driving --no-deps` | clean under `-D warnings` |

**Not run by me, deliberately:** `cargo test --workspace`. The supervisor runs
it, and two finished tasks have died at a deadline waiting on it. Every crate
that depends on `alo-models` was clippy-checked with all targets instead, which
is what catches the shape of break a changed public type causes.

**The measurement on this machine**, printed by the test rather than typed
here:

```
$ cargo test -p alo-models --test which_road_this_machine_takes -- --nocapture this_machines_own_road
this machine draws with: NoCard
this machine's road: OnTheProcessor(NoCard)
  said: this machine has no graphics card, so it is running the model on its processor
test this_machines_own_road_is_measured_and_said ... ok
```

```
$ cargo test -p alo-driving --test the_same_question_on_each_road -- --nocapture
this machine draws with: NoCard
its road: OnTheProcessor(NoCard)
the card road is not shown from here, and this is why: neither of these runs was
on a graphics card, so comparing them says nothing about one — this needs a
machine with a card the runtime can use
it waits on a named machine with a card the pinned runtime can use (task 23)
test the_card_road_cannot_be_walked_on_a_machine_with_no_card ... ok
test one_question_on_each_road_of_a_machine_that_has_both ... ignored
```

### Evidence, one line per acceptance criterion

| Criterion | Test |
|---|---|
| the machine says what it has, measured, and `on_the_gpu_bytes` carries it | `alo-models` / `which_road_this_machine_takes` / `what_a_card_held_is_read_from_the_runtime_and_carried_into_a_grade` |
| … measured on **this** machine rather than assumed | `alo-models` / `which_road_this_machine_takes` / `this_machines_own_road_is_measured_and_said` |
| … and its memory is read where published, never guessed | `alo-models` / `lib` / `card::tests::memory_is_read_where_it_is_published_and_never_guessed` |
| one question of each road, held to each other | `alo-driving` / `lib` / `both_roads::tests::a_card_that_changed_nothing_about_what_the_model_could_do_agrees` |
| … and a machine with no card is refused rather than agreeing with itself | `alo-driving` / `the_same_question_on_each_road` / `the_card_road_cannot_be_walked_on_a_machine_with_no_card` |
| the road is asked for and said in the vocabulary | `alo-models` / `which_road_this_machine_takes` / `the_reason_is_read_in_the_language_the_person_reads` |
| … never inferred from how long it took | `alo-models` / `lib` / `road::tests::no_road_is_described_by_how_long_it_took` |
| a card the runtime cannot use is a machine that runs and says why | `alo-models` / `which_road_this_machine_takes` / `every_card_the_runtime_cannot_use_is_a_machine_that_runs_and_says_why` |
| … and *no driver* is never *no card* | `alo-models` / `which_road_this_machine_takes` / `a_laptop_with_an_integrated_processor_and_a_driverless_card_is_told_about_the_driver` |
| `min_vram_gb` does not come back as a judge | `alo-models` / `which_road_this_machine_takes` / `what_a_machine_offers_does_not_change_with_what_is_on_its_bus` |
| a question that names no road is the request that was always sent | `alo-models` / `which_road_this_machine_takes` / `a_question_that_names_no_road_is_the_request_that_was_always_sent` |

Each was run on its own before this report was written.

### The refusal paths, tested beside the legitimate ones

- a card of a vendor the runtime cannot use → the processor, named by vendor;
- a card with no driver → the processor, named as **no driver**, never as no
  card, including when an unusable integrated processor sits beside it;
- a card smaller than the weights → the processor, with both figures beside the
  sentence and neither inside it;
- a machine that keeps no list → the processor, claiming nothing about a card;
- a runtime holding weights with nothing on the card → the processor road,
  however much it loaded in all;
- a folder with no readable vendor → not counted as a card;
- two runs on the same road → `NotAComparison`, both ways round;
- a run that skipped an exercise → already refused by `Measured::of`.

## Limitations, stated plainly

**The card road has not been run.** No machine in this fleet has a discrete
graphics card, and no emulated device stands in for one — ADR 0056's reason for
rejecting a simulated chip applies unchanged: a run against something that
agrees with itself is evidence about the simulator. Task 23 is where a card
being used gets shown, on a named machine.

**`"options":{"num_gpu":0}` has not been put to a real Ollama.** There is no
runtime on this machine (`ollama` is not installed) and no card to hide from
one. It is Ollama's own documented option, it is sent only when a caller names
a road, and the ordinary path sends no `options` field at all — held by a test
against a real socket. Confirming it against the pinned release is part of task
23's run, and `docs/quirks.md` should gain a line if the release disagrees.

**The residency in a two-road run is matched by position, not by name.** One
model is loaded during a run, so the first `/api/ps` entry is the weights being
measured. Matching by name would mean spelling the runtime's naming convention
outside the one file allowed to know it (ADR 0006).

## Proposed shared-document updates

These are proposals for the integration owner; I have not edited
`CHANGELOG.md`, `ROADMAP.md`, `docs/autonomy/QUEUE.md` or
`docs/autonomy/STATE.md`.

**CHANGELOG.md**, under v0.5:

> **A machine says which part of it is running the model.** Graphics hardware
> is read from the kernel's own list rather than assumed, and a machine that
> runs a model on its processor says why: no card, a card nothing can reach, a
> card the model runtime does not use, or one with less memory than the model
> needs. None of those is a failure and none of them mentions speed — which
> part of the machine is working is something you ask for, not something you
> time. Where a graphics card is used, how much of the model is on it is now
> recorded beside the measurement instead of being copied out by hand.

**ROADMAP.md**, v0.5, against ADR 0007's line: `- [x] The code.` for the
processor road. **Not** ticked on the machine, and **not** ticked at all for
the card road: that needs a machine with a discrete graphics card the pinned
runtime can use, which `docs/hardware.md` still lists none of.

**docs/features.md:** nothing changes. *It runs on the machine you already own*
is unchanged and is now measured rather than assumed.

**QUEUE.md / STATE.md:** task 22 done, 2026-09-22; task 23 remains blocked on
hardware, and its dependency on 22 is cleared.
