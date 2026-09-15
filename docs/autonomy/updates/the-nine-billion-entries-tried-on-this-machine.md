# The nine-billion entries, tried on this machine

**Date:** 2026-09-15.
**Workstream:** the models, measured — `docs/autonomy/v0-5-the-models-measured-plan.md`, task 9 (partly).
**Contributor:** the Mac lane.
**Machine:** **Apple M3, 8 GiB unified memory**, macOS 26.5.2, **Ollama 0.34.0**
on `127.0.0.1`, `iogpu.wired_limit_mb` at its default (`0`), the Linux VM
**stopped**. Gates in that VM afterwards.
**Status:** ready for integration. Task 9 stays blocked, for one entry instead of
three.

## What changed, for somebody outside this repository

Three catalogue entries said *too large for the measuring machine*. That was
never measured — it was this machine's memory subtracted from the entry's own
`min_ram_gb`. Two of the three have now actually been tried, and the sentence
turns out to be true for a different reason than anybody assumed: **they load
fine. They cannot answer in the time a person is waiting.**

## What was tried, and what it cost

The question was *how do we solve the memory issue*, and the first answer cost
nothing: **the memory was the development VM.** Lima holds 4 GiB of an 8 GiB
machine whether or not anything is building in it. With it stopped, both
nine-billion entries fetched, loaded and answered — which they had never been
given the chance to do.

| | `eurollm-9b-instruct` | `gemma-2-9b-instruct` |
|---|---|---|
| File | 5.58 GB | 5.76 GB |
| Loaded | 6,494,638,568 | 7,451,579,511 |
| On the graphics processor | 4,608,848,035 | 4,125,160,897 |
| **Adrift** | **1.9 GB** | **3.3 GB** |
| Answered when fetched | *"I'm ready to help with your question or instruction."* | *"ready."* |
| 19-token question, held to the envelope | **168 s** | **not inside 300 s** |
| `alo-driving`'s own prompt (~1,500 tokens) | **not inside 300 s**, three times | not attempted — it cannot answer the short one |

`alo-models` waits 300 seconds for an answer (`WHILE_A_MODEL_THINKS`). EuroLLM
was put to three full runs — asked freely, asked in the envelope, and asked in
the envelope under the instructions a turn shows (ADR 0037) — and each stopped at
`TookTooLong`, twice on the first exercise and once on the harness's own warm-up.
No run produced a grade, and the harness is what refused to invent one.

**So the limit is not the file, it is the prompt.** A 19-token question is
answerable at 1.9 GB adrift; the prompt a real turn sends — every verb this
machine declares, in the verb's own words — is not. That distinction is new, and
it is the useful part of this: a machine that can *load* a 9B is not a machine
that can *use* one.

## What the entries say now

Both keep `too-large-for-the-measuring-machine`. Its own words are *the machine
that measures the catalogue does not have the memory to run the model inside the
time `alo-models` waits for an answer* — which is exactly what was measured, so
the reason did not need rewording; it needed evidence. Each entry's
`[model.unmeasured]` block now carries **2026-09-15** and the comment above it
carries the numbers in this report, so a reader meets an attempt rather than a
subtraction.

**`mixtral-8x7b-instruct` was not attempted, and that is a decision.** 26.4 GB of
weights cannot be placed in 8 GiB by any setting — it is three times the whole
machine, and the two entries above could not answer while 1.9 and 3.3 GB were
adrift. Fetching it to watch that fail would cost 26 GB of somebody's connection
to learn arithmetic. Its entry says so in those words. Task 9 therefore stays
blocked **for Mixtral alone**, on a machine with the 48 GB the entry itself
states.

## What would lift the two nine-billion entries

- **Memory.** Both entries ask for 12 GB and this machine has 8. A 16 GB machine
  would hold either wholly on the graphics processor, which is where the 168
  seconds would collapse.
- **`iogpu.wired_limit_mb`.** It is `0` (the default, about 5.3 GB on 8 GiB), and
  every 7B-and-up run this lane has made has been split across that line.
  `sudo sysctl iogpu.wired_limit_mb=6144` would buy roughly a gigabyte of
  residency and reverts at reboot. It needs the owner's password, so no agent can
  set it; it is worth one attempt at the two 9Bs afterwards, and it cannot touch
  Mixtral.
- **Nothing else.** Not a smaller quantisation, not a shorter context, not a
  longer wait — task 9's own constraint, and the wait in particular is the one
  number that describes a person rather than a machine.

## Egress

Two fetches from `registry.ollama.ai`, 2026-09-15 03:07–03:20 UTC, 11.3 GB by the
catalogue's own `download_bytes`: `gemma2:9b-instruct-q4_K_M` and
`hf.co/bartowski/EuroLLM-9B-Instruct-GGUF:Q4_K_M`, both through `Ollama::fetch`
by the entry's id. Both removed afterwards; the machine holds what it held
before. The attempts themselves caused none — every run is under
`SourcePolicy::ThisMachineOnly`.

## Gates

All nine as root in the Lima VM, restarted for them after the measurements:
`cargo fmt --all --check` clean, `cargo clippy --workspace --all-targets --
-D warnings` clean, `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D
warnings` clean over 52 crates, the supervisor's three (115 tests) clean, both
BPF gates clean, and `cargo test --workspace --no-fail-fast` **4,917 passed, 1
failed, 26 ignored** — the failure being `alo-bounding`'s
`ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`, in a
crate this change does not touch.

**`main` moved before this was pushed, and with it that failure went.** Rebased
onto it — twice, as it moved again — every gate above was run again each time,
and the suite is **5,149 passed, 0 failed, 31 ignored** on the tree that was
pushed: the first wholly green workspace run this lane has seen.
Another lane settled what this lane could only report: the fixture was setting a
file's flags to exactly `nodump`, which asks ext4 to give up its extents and is
`EOPNOTSUPP` before any hook is asked. The boundary was never involved, and
`docs/quirks.md` now says so above the entry that guessed otherwise.

## What this does not claim

- **No grade was earned and none was written.** Three runs produced nothing, and
  nothing is in the catalogue that a run did not produce.
- **Nothing about a certified machine.** A Mac is not the machine, and what an
  8 GiB laptop cannot answer with says nothing about what a certified workstation
  can.
- **Not a claim that these models are slow.** They are slow *here*, 1.9 and
  3.3 GB adrift of the processor. The grade a machine with room would earn is
  unmeasured, which is what the entries say.
