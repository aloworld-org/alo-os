# Every entry this machine can hold, graded in the words a turn shows

**Date:** 2026-09-14 (the runs; written up in the small hours of the 15th, local).
**Workstream:** the models, measured — `docs/autonomy/v0-5-the-models-measured-plan.md`, task 20.
**Contributor:** the Mac lane.
**Machine:** **Apple M3, 8 GB unified memory**, macOS 26.5.2, weights on the GPU
through Metal, **Ollama 0.34.0** (the pinned runtime) on `127.0.0.1`, the Linux
VM stopped throughout. Gates in that VM afterwards.
**Status:** ready for integration.

## What a person would notice

**A second local model now drives alo OS's verbs reliably, and it is one an 8 GB
machine can hold.** `qwen3-8b` answered all twenty attempts correctly — 20 of 20
— when it was shown the words ADR 0037 says a turn shows. Under the old
instructions the same weights, the same runtime and the same ten exercises got
10 of 20.

And it is not one model. **Every entry this machine can hold improved or held
steady; none got worse.** Across the twelve entries re-measured, 23 attempts in
240 drove the verbs under the first instructions and **98 in 240** under the
turn's — four times as many, from two extra lines of text. Four entries moved a
grade band.

## The grades

Each entry fetched through `Ollama::fetch` (the product's own road, by the
entry's catalogue id), then two rounds of the fixed ten, in the envelope, through
`alo-asking`'s door, under `Instructions::SHOWN_TO_A_TURN` — digest
`93a7f458ce9d017d6d12759a281e5f0b347a0d6963c04eae03b4c30581aebd02`. 2026-09-14,
20:54–22:35 UTC.

| Entry | Under the turn's words | Under the first instructions | Moved |
|---|---|---|---|
| `qwen3-8b` | **20 of 20, `reliably`** | 10 of 20, `sometimes` | **+10, a band** |
| `mistral-7b-instruct` | **15 of 20, `sometimes`** | 0 of 20, `rarely` | **+15, a band** |
| `qwen3-4b` | **14 of 20, `sometimes`** | 3 of 20, `rarely` | **+11, a band** |
| `qwen3-1.7b` | **12 of 20, `sometimes`** | 1 of 20, `rarely` | **+11, a band** |
| `llama-3.1-8b-instruct` | **10 of 20, `sometimes`** | 5 of 20, `rarely` | **+5, a band** |
| `gemma-2-2b-instruct` | 8 of 20, `rarely` | 0 of 20, `rarely` | +8 |
| `qwen2.5-3b-instruct` | 7 of 20, `rarely` | 0 of 20, `rarely` | +7 |
| `granite-3.2-2b-instruct` | 6 of 20, `rarely` | 0 of 20, `rarely` | +6 |
| `llama-3.2-3b-instruct` | 3 of 20, `rarely` | 3 of 20, `rarely` | — |
| `teuken-7b-instruct` | 3 of 20, `rarely` | 1 of 20, `rarely` | +2 |
| `phi-3-mini-instruct` | 0 of 20, `rarely` | 0 of 20, `rarely` | — |
| `smollm2-1.7b-instruct` | 0 of 20, `rarely` | 0 of 20, `rarely` | — |
| `qwen2.5-7b-instruct` | 80 of 80, `reliably` (task 16) | 71 of 80, `sometimes` | a band |

The three entries this machine cannot hold — `gemma-2-9b-instruct`,
`eurollm-9b-instruct` and `mixtral-8x7b-instruct` — keep the reason task 2 gave
them, with this machine named in it. They are task 9's, still blocked on memory.

**Two entries now clear the bar in the words a turn shows:**
`qwen2.5-7b-instruct` (task 16) and `qwen3-8b`. Neither decides anything yet —
ADR 0037 decision 3 keeps the catalogue reading the grade for the way a turn
*actually* asks until `alo-turn` composes from `alo-instructing`, which is task
19 of the local-network plan. When it lands, task 19 of this plan makes these the
grades that decide, and this task is what stops that taking a grade away from
twelve entries.

## Residency, because a grade earned against swap is a different grade

Four entries did not fit the graphics processor whole, and the catalogue records
both numbers for each:

| Entry | Loaded | On the graphics processor |
|---|---|---|
| `llama-3.1-8b-instruct` | 6,248,645,102 | 5,196,218,367 |
| `qwen3-8b` | 6,033,736,988 | 4,585,359,932 |
| `teuken-7b-instruct` | 6,022,953,429 | 5,229,426,769 |
| `mistral-7b-instruct` | 5,397,094,069 | 4,631,067,360 |

`qwen3-8b` earned `reliably` with about a fifth of itself off the processor —
which is a harder position than a machine with room would give it, not an easier
one. The eight smaller entries were wholly on the graphics processor.

## What the words changed, read off the answers

- **The wrong door nearly disappears.** Under the first instructions the
  commonest failure across the small entries was a change sent through `read`
  (task 10's report, and ADR 0034's context). In these 240 attempts **not one failure
  was `TheWrongDoor`** — zero of 142. What is left is models that
  cannot produce the message at all (`NotAMessage`, 113 of the 142 failures),
  an argument the protocol refuses (17), a verb this machine does not have (7)
  and another verb than the one asked for (5).
- **It does not rescue a model that cannot write the shape.** `smollm2-1.7b` and
  `phi-3-mini` are still 0 of 20; SmolLM2 writes `given` as an object every
  time, and Phi-3 names each argument by its value. Two extra lines about doors
  do not teach a list.
- **It helps most where the training is there.** The three Qwen3 entries — the
  ones their publisher trains for tool calls — moved +10, +11 and +11, and the
  smallest of them, at 1.7 billion parameters, is now the smallest entry in this
  catalogue to reach `sometimes`.

## Two runs that produced no grade, and why that is the harness working

- **`teuken-7b-instruct` answered nothing usable twice**, both times on the first
  enveloped request within seconds of being fetched. The same weights answered
  the same warm-up correctly once loaded, and graded 3 of 20. Recorded in
  `docs/quirks.md` (*asked in the envelope within seconds of being fetched…*).
- **`llama-3.1-8b-instruct` exceeded the 300 seconds `alo-models` waits**, twice,
  on a machine with 5.1 GB of swap in use. Warm, with nothing else started, it
  answered all twenty and graded 10 of 20. Recorded in `docs/quirks.md` (*an 8B
  model on this 8 GB machine…*).

In both cases the harness refused to write a grade, because a runtime that could
not answer is this machine failing and scoring it would blame the model for it.
That refusal is the reason these two entries have a real grade rather than a
`rarely` earned by a swap file. **No wait, bar, exercise or scoring rule was
changed to get either of them measured**; what changed was that the weights were
warm before the set was put to them.

## Egress

Fourteen fetches from `registry.ollama.ai`, 2026-09-14 20:54–22:08 UTC,
**44.1 GB** by the catalogue's own `download_bytes`: the twelve entries'
artefacts, plus `teuken-7b-instruct` and
`llama-3.1-8b-instruct` a second time after their first runs produced no grade.
Each went through `Ollama::fetch` — the product's own road — rather than by hand.
The measurements themselves caused none: every run is under
`SourcePolicy::ThisMachineOnly`, which forbids anything that would leave.

Each artefact was removed after its entry was graded, so the machine holds what
it held before: `qwen2.5:7b-instruct-q4_K_M` and `qwen2.5:7b-instruct-q5_K_M`.

## Gates

All nine, as root in the Lima VM (Ubuntu, kernel 7.0.0-31-generic, aarch64),
building in `/root/alo-builds/alo-os-main`. The measurements themselves ran on
macOS, because that is where the runtime and the weights are; nothing was gated
there.

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo test --workspace --no-fail-fast` | **4,873 passed, 1 failed, 26 ignored**; **4,917 passed, 1 failed, 26 ignored** on the tree combined with `main` |
| `cargo doc --workspace --no-deps`, `RUSTDOCFLAGS=-D warnings` | clean, 52 crates |
| supervisor `fmt`, `clippy`, `test` in `tools/kernel-loop` | clean; 115 passed |
| BPF target `fmt` and `clippy` | clean |

The one failure is `alo-bounding`'s
`ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`, the
deterministic aarch64 refusal recorded in `docs/quirks.md` and in the first
task's report: another lane's crate, untouched here, failing the same way on this
VM before this change. Nothing was weakened, ignored or narrowed to get a green.

`main` moved twice while this was being gated — three `alo-bounding` commits
(which are what the `EBUSY` transient this lane reported was for) and then lane
A's workspace-discovery work. This was rebased onto each and the gates were run
again on the combined tree both times, with the same one failure; the count in
the table is the last of those runs.

Every test in `alo-models` and `alo-driving` passes, including the two that hold
this change: `the_catalogue_grades_are_what_their_counts_earn.rs` (a grade is
what its counts earn, and the second-round rule) and
`every_grade_names_instructions_this_crate_has.rs` (every digest in the catalogue
names instructions that exist).

## What this does not claim

- **Nothing about a certified machine.** A Mac is not the machine; no *On the
  machine* box moved and `ROADMAP.md` is untouched.
- **No grade decides anything yet.** ADR 0037 decision 3; the catalogue still
  reads the grade earned under the first instructions, because that is still
  what a turn is shown.
- **Not a grade per quantisation.** Every entry was measured at the artefact its
  own entry names, and nothing was re-quantised to make it fit.
- **Twenty attempts is twenty attempts.** `qwen3-8b` at 20 of 20 has cleared the
  bar on this sample on this machine; `qwen2.5-7b-instruct`'s eighty attempts
  remain the larger evidence, and the entry that ships on a certified machine
  will need its own run there.

## Proposed shared-document updates

- **`CHANGELOG.md`:** *Every model in the catalogue that an 8 GB machine can hold
  has been measured again, in the words an agent turn will show a model. Two now
  drive the verbs reliably — a 7B and an 8B — where one did before, and every
  entry did at least as well as it had.*
- **`ROADMAP.md`:** nothing.
- **Plan:** `v0-5-the-models-measured-plan.md` marks task 20 done. Task 19 stays
  blocked on the local-network plan's task 19.

## Every answer, verbatim

Two rounds of ten for each entry, in the order they were asked, with the
outcome the daemon's own door gave each. A grade a reader cannot re-derive is
a grade they have to take on trust.

**Nothing here is tidied.** Three of Gemma 2 2B's lines carry a space followed by
a tab, which `git diff --check` flags as *space before tab in indent* — that
whitespace is a newline in the middle of the model's own answer, which is exactly
why the door read it as `MoreThanOneMessage`, so it stays as it was written.

### qwen3-8b — 20 of 20, `reliably`

```text
round 1, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}

round 1, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}

round 1, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}

round 1, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}

round 1, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}

round 1, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices"}]}},"format":1}

round 1, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 1, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 1, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 1, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}

round 2, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}

round 2, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}

round 2, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}

round 2, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}

round 2, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}

round 2, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}

round 2, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}

```

### mistral-7b-instruct — 15 of 20, `sometimes`

```text
round 1, list: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},
"format":1}

round 1, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}

round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":{"folder":"/home/anna/Invoices","named":"october","most":20}} },"format":1}

round 1, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}

round 1, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}

round 1, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}

round 1, open: NotAMessage(MoreThanOneMessage)
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},
"format":1}

round 1, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 1, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 1, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}

round 2, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}

round 2, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}

round 2, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":{"folder":"/home/anna/Invoices","named":"october","most":20}}}, "format":1}

round 2, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}

round 2, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}, "format":1}

round 2, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}

round 2, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, focus: NotAMessage(MoreThanOneMessage)
{"asks": {"propose": {"verb":"focus_application", "given": [{"named":"application", "is":"org.alo.Writer"}] } },
"format":1}

round 2, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}

```

### qwen3-4b — 14 of 20, `sometimes`

```text
round 1, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}

round 1, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}

round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"october","is":"\"october\""},{"folder":"/home/anna/Invoices","is":"/home/anna/Invoices"},{"most":20,"is":20}]}},"format":1}

round 1, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}

round 1, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}

round 1, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices"}]}},"format":1}

round 1, open: AnotherVerb { named: "list_folder" }
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/path/to/folder"}]}},"format":1}

round 1, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 1, close: AnotherVerb { named: "read_file" }
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/path/to/file.txt"}]}},"format":1}

round 1, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}

round 2, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}

round 2, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}

round 2, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}

round 2, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}

round 2, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}

round 2, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}

round 2, open: NotAMessage(NotReadable)
{"asks":{"ask":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, focus: NotAMessage(NotReadable)
{"asks":{"ask":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, close: NotAMessage(NotReadable)
{"asks":{"ask":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}

```

### qwen3-1.7b — 12 of 20, `sometimes`

```text
round 1, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}

round 1, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}

round 1, find: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"find_in_folder","given":[{"named":"october","is":"/home/anna/Invoices"},{"most":20}]}},"format":1}

round 1, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}

round 1, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}

round 1, archive: ArgumentRefused(NoSuchArgument { verb: "archive_folder", argument: "/home/anna/Invoices" })
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"/home/anna/Invoices","is":"folder"},{"named":"/home/anna/Archive","is":"into"},{"named":"invoices","is":"name"}]}},"format":1}

round 1, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 1, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 1, close: ArgumentRefused(NoSuchArgument { verb: "close_application", argument: "org.alo.Writer" })
{"asks":{"propose":{"verb":"close_application","given":[{"named":"org.alo.Writer","is":"application"}]}},"format":1}

round 1, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}

round 2, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}

round 2, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}

round 2, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"october","is":"october"}],"most":20}},"format":1}

round 2, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}

round 2, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}, "format":1}

round 2, archive: ArgumentRefused(NoSuchArgument { verb: "archive_folder", argument: "/home/anna/Invoices" })
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"/home/anna/Invoices","is":"folder"},{"named":"/home/anna/Archive","is":"into"},{"named":"invoices","is":"name"}]}},"format":1}

round 2, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, focus: ArgumentRefused(NoSuchArgument { verb: "focus_application", argument: "org.alo.Writer" })
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"org.alo.Writer","is":"org.alo.Writer"}]}},"format":1}

round 2, close: ArgumentRefused(NoSuchArgument { verb: "close_application", argument: "org.alo.Writer" })
{"asks":{"propose":{"verb":"close_application","given":[{"named":"org.alo.Writer","is":""}]}}, "format":1}

round 2, arrange: ArgumentRefused(NoSuchArgument { verb: "arrange_application", argument: "org.alo.Writer" })
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"org.alo.Writer","is":"application"},{"named":"left_half","is":"where"}]}},"format":1}

```

### llama-3.1-8b-instruct — 10 of 20, `sometimes`

```text
round 1, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}

round 1, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}, "format":1}

round 1, find: ArgumentRefused(Argument(NotAFullPath { argument: "folder" }))
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"\\/home\\/anna\\/Invoices"},{"named":"named","is":"\"october\""},{"named":"most","is":20}]}},"format":1}

round 1, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}

round 1, move: ArgumentRefused(Argument(NotAFullPath { argument: "file" }))
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"(/home/anna/Invoices/march.pdf)"},{"named":"into","is":"(/home/anna/Archive)"}]}},"format":1}

round 1, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"archive_folder":{"folder":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices"}}},"format":1}

round 1, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 1, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 1, close: NotAMessage(NotReadable)
{"asks":{"read":{"close_application":{"application":"org.alo.Writer"}}},"format":1}

round 1, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}, "format":1}

round 2, list: ArgumentRefused(Argument(NotAFullPath { argument: "folder" }))
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"\"/home/anna/Invoices\""}]}},"format":1}

round 2, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}, "format":1}

round 2, find: NotAMessage(NotReadable)
{"asks":{"read":{"find_in_folder":{"folder":"/home/anna/Invoices","named":"october","most":20}}}, "format":1}

round 2, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}

round 2, move: NotAMessage(NotReadable)
{"asks":{"propose":{"move_file":{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}}},"format":1}

round 2, archive: ArgumentRefused(Argument(NotAFullPath { argument: "folder" }))
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"path/to/folder"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}

round 2, open: NotAMessage(NotReadable)
{"asks":{"propose":{"open_application":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}},"format":1}

round 2, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}, "format":1}

round 2, close: NotAMessage(NotReadable)
{"asks":{"propose":{"close_application":{"application":"org.alo.Writer"}}},"format":1}

round 2, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}

```

### gemma-2-2b-instruct — 8 of 20, `rarely`

```text
round 1, list: NotAMessage(NotReadable)
{"asks":{"read":{"folder":"/home/anna/Invoices"}},"format":1}

round 1, read: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is": "/home/anna/Invoices/march.pdf"}]}},
 	"format":1}

round 1, find: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"most":20}]}},
 	  "format":1}

round 1, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}, "format":1}

round 1, move: NotAMessage(NotReadable)
{"asks":{"propose":{"move_file":{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}}},"format":1}

round 1, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"name":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices"} ]}},"format":1}

round 1, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is": "org.alo.Writer"}]}},"format":1}

round 1, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 1, close: NoSuchVerb { named: "close" }
{"asks":{"propose":{"verb":"close","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 1, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}

round 2, list: NotAMessage(NotReadable)
{"asks":{"read":{"folder":"/home/anna/Invoices"}},"format":1}

round 2, read: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},
 	 	  "format":1}

round 2, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}

round 2, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"rename_file":{"file":"/home/anna/Invoices/scan001.pdf","name":"march.pdf"}}},"format":1}

round 2, move: NotAMessage(MoreThanOneMessage)
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is": "/home/anna/Invoices/march.pdf"},{"named":"into","is": "/home/anna/Archive"}]}},
"format":1}

round 2, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices"}]}},"format":1}

round 2, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"name":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, close: NoSuchVerb { named: "close" }
{"asks":{"propose":{"verb":"close","given":[{"named":"application","is": "org.alo.Writer"}]}},"format":1}

round 2, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}

```

### qwen2.5-3b-instruct — 7 of 20, `rarely`

```text
round 1, list: NotAMessage(NotReadable)
{"asks":{"read":{"given":[{"folder":"/home/anna/Invoices"}]}},"format":1}

round 1, read: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"read_file","given":[{"file":"/home/anna/Invoices/march.pdf"}]}},"format":1}

round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":[{"folder":"/home/anna/Invoices","named":"october","most":20}]}},"format":1}

round 1, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}

round 1, move: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"move_file","given":[{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}]}},"format":1}

round 1, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"archive_folder","given":[{"folder":"/home/anna/Invoices"},{"into":"/home/anna/Archive"},{"name":"invoices.zip"}]}},"format":1}

round 1, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 1, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 1, close: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"close_application","given":[{"application":"org.alo.Writer"}]}},"format":1}

round 1, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}

round 2, list: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"list_folder","given":[{"folder":"/home/anna/Invoices"}]}},"format":1}

round 2, read: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"read_file","given":[{"file":"/home/anna/Invoices/march.pdf"}]}},"format":1}

round 2, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":[{"folder":"/home/anna/Invoices"},{"named":"october"},{"most":20}]}},"format":1}

round 2, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"rename_file","given":[{"file":"/home/anna/Invoices/scan001.pdf","name":"march.pdf"}]}},"format":1}

round 2, move: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"move_file","given":[{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}]}},"format":1}

round 2, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"archive_folder","given":[{"folder":"/home/anna/Invoices"},{"into":"/home/anna/Archive"},{"name":"invoices"}]}},"format":1}

round 2, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, close: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"close_application","given":[{"application":"org.alo.Writer"}]}},"format":1}

round 2, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}

```

### granite-3.2-2b-instruct — 6 of 20, `rarely`

```text
round 1, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}

round 1, read: AnotherVerb { named: "list_folder" }
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}, {"named":"named","is":"march.pdf"}]}},"format":1}

round 1, find: Drove
{"asks": {"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}

round 1, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"rename_file":{"file":"/home/anna/Invoices/scan001.pdf","name":"march.pdf"}}},"format":1}

round 1, move: NotAMessage(NotReadable)
{"asks":{"propose":{"move_file":{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}}},"format":1}

round 1, archive: NotAMessage(NotReadable)
{"asks": {"propose":{"archive_folder":{"file":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices"}}},"format":1}

round 1, open: NotAMessage(NotReadable)
{"asks":{"propose":{"open_application":{"file": "org.alo.Writer"}}},"format":1}

round 1, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"open_application":{"file": "org.alo.Writer"}}},"format":1}

round 1, close: NotAMessage(NotReadable)
{"asks":{"propose":{"close_application":{"file":"org.alo.Writer"}}},"format":1}

round 1, arrange: NotAMessage(NotReadable)
{"asks":{"propose":{"open_application":{"application":"org.alo.Writer","where":"left_half"}}},"format":1}

round 2, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}

round 2, read: AnotherVerb { named: "list_folder" }
{"asks":{"read":{"verb":"list_folder","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}

round 2, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}

round 2, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}

round 2, move: NotAMessage(NotReadable)
{"asks":{"propose":{"rename_file":{"file":"/home/anna/Invoices/march.pdf","name":"/home/anna/Archive/march.pdf"}}},"format":1}

round 2, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"archive_folder":{"file":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices"}}},"format":1}

round 2, open: NotAMessage(NotReadable)
{"asks":{"propose":{"open_application":{"file": "org.alo.Writer"}}},"format":1}

round 2, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"open_application":{"file":"org.alo.Writer", "most":1}}},"format":1}

round 2, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, arrange: NotAMessage(NotReadable)
{"asks":{"propose":{"open_application":{"application":"org.alo.Writer","given":[{"named":"where","is":"left_half"}]}}},"format":1}

```

### llama-3.2-3b-instruct — 3 of 20, `rarely`

```text
round 1, list: NotAMessage(NotReadable)
{"asks":{"read":{"folder":["/home/anna/Invoices"]}},"format":1}

round 1, read: NotAMessage(NotReadable)
{"asks":{"read":{"file":{"named":"march.pdf","is":"/home/anna/Invoices/march.pdf"}}},"format":1}

round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"find_in_folder":{"verb":"find_in_folder","given":[{"named":"folder"},{"is":"/home/anna/Invoices"},{"named":"named"},{"is":"october"},{"named":"most"},{"is":"20"}]}}}, "format":1}

round 1, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"rename_file":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}},"format":1}

round 1, move: NotAMessage(NotReadable)
{"asks":{"propose":{"move_file":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}, "format":1}

round 1, archive: NotAMessage(NotReadable)
{"asks":{"read":{"archive":{"verb":"NAME","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices"},{"named":"format","is":"zip"}]}}}, "format": 1}

round 1, open: NotAMessage(NotReadable)
{"asks":{"read":{"open_application":{"given":[{"named":"application","is":"org.alo.Writer"}]}}}, "format": 1}

round 1, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"focus_application":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}, "format":1}

round 1, close: NotAMessage(NotReadable)
{"asks":{"propose":{"close":{"verb":"close","given":[{"named":"application","is":"org.alo.Writer"}]}}}, "format":1}

round 1, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}

round 2, list: NotAMessage(NotReadable)
{"asks":{"read":{"folder":["/home/anna/Invoices"]}},"format":1}

round 2, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}

round 2, find: NotAMessage(NotReadable)
{"asks":{"read":{"find_in_folder":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":"20"}]}}}, "format": 1}

round 2, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"file":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}},"format":1}

round 2, move: NotAMessage(NotReadable)
{"asks":{"propose":{"file":{"verb":"move","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}},"format":1}

round 2, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"archive":{"verb":"NAME","given":[{"named":"name","is":"\"invoices\""}]},"folder":{"verb":"NAME","given":[{"named":"folder","is":"\"/home/anna/Archive\""}]},"file":{"verb":"NAME","given":[{"named":"file","is":"\"/home/anna/Invoices.zip\""}]},"into":{"verb":"NAME","given":[{"named":"into","is":"\"/home/anna/Archive\""}]}}},"format":1}

round 2, open: NotAMessage(NotReadable)
{"asks":{"read":{"application":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}, "format":1}

round 2, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"focus_application":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"application","is":"org.alo.Writer"}]}}},"format":1}

round 2, close: NotAMessage(NotReadable)
{"asks":{"ask":{"verb":"close","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}

round 2, arrange: Drove
{"asks": {"propose": {"verb": "arrange_application", "given": [{"named": "application", "is": "org.alo.Writer"}, {"named": "where", "is": "left_half"}]}},"format":1}

```

### teuken-7b-instruct — 3 of 20, `rarely`

```text
round 1, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoicers"}]}},"format":1}

round 1, read: AnotherVerb { named: "list_folder" }
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoicers/"}]}},"format":1}

round 1, find: ArgumentRefused(Missing { verb: "find_in_folder", argument: "named", purpose: Key { named: "files.verb.find-in-folder.argument.named" } })
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoic.. 0"}]}},"format":1}

round 1, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"rename_file","given":["/home/anna/Invoicers/scan001.pdf", "march.pdf"]}},"format":1}

round 1, move: NotAMessage(MoreThanOneMessage)
{"asks":{"propose":{"verb":"move_file","given":["/home/anna/Invoicers/march.pdf",{"folder":"/home/anna/Archive"}]}},
"format":1}

round 1, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"archive_folder","given":[{"folder":"/home/anna/Invoic","into":"/home/anna/Archive"}]}},"format":1}

round 1, open: NoSuchVerb { named: "start" }
{"asks":{"propose":{"verb":"start","given":[{"named":"org.alo.Writer","is":"org.alo.Writer"}]}},"format":1}

round 1, focus: NoSuchVerb { named: "arrange_applicaton" }
{"asks":{"propose":{"verb":"arrange_applicaton","given":[{"named":"applicaton","is":"org.alo.Writ"}]}},"format":1}

round 1, close: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"close_applicaion","given":[{"applicaion":"org.alo.Writer"}]}},"format":1}

round 1, arrange: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"arrange_applicaton","given":[{"named":"applicaton","is":"org.alo.Writer"}],"args":{"where":"left_half"}}},"format":1}

round 2, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoicers"}]}},"format":1}

round 2, read: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoicers/"}]}},
"format":1}

round 2, find: NotAMessage(MoreThanOneMessage)
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoicers"}]}},
 "format":1}

round 2, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoicers/scan001.pdf"},{"named":"name","is":"march"}]}},"format":1}

round 2, move: ArgumentRefused(NoSuchArgument { verb: "move_file", argument: "folder" })
{"asks":{"propose":{"verb":"move_file","given":[{"named":"folder","is":"/home/anna/Archive"},{"named":"file","is":"/home/anna/Invoic..pdf"}]}},"format":1}

round 2, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"archive_folder","given":[{"folder":"/home/anna/Invoic...","into":"/home/anna/Archive","name":"invoices"}]}},"format":1}

round 2, open: NoSuchVerb { named: "start" }
{"asks":{"propose":{"verb":"start","given":[{"named":"applicaton","is":"org.alo.Writ"}]}},"format":1}

round 2, focus: NoSuchVerb { named: "focus_applicatio" }
{"asks":{"propose":{"verb":"focus_applicatio","given":[{"named":"applicatio","is":"org.alo.Writ"}]}},"format":1}

round 2, close: NoSuchVerb { named: "close_applicaton" }
{"asks":{"propose":{"verb":"close_applicaton","given":[{"named":"applicaton","is":"org.alo.Writer"}]}},"format":1}

round 2, arrange: NotAMessage(MoreThanOneMessage)
{"asks":{"propose":{"verb":"arrange_applicatio, given: [","org.alo.Writ, whole_screen":
"org.alo.Writ, left_half"}},"format":1}

```

### phi-3-mini-instruct — 0 of 20, `rarely`

```text
round 1, list: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"list_folder","given":[{"named":"/home/anna/Invoices"}]}},"format":1}

round 1, read: ArgumentRefused(NoSuchArgument { verb: "read_file", argument: "/home/anna/Invoices/march.pdf" })
{"asks":{"read":{"verb":"read_file","given":[{"named":"/home/anna/Invoices/march.pdf","is":"a whole number"}]}},"format":1}

round 1, find: NotAMessage(MoreThanOneMessage)
{"asks":{"propose":{"verb":"find_in_folder","given":[{"named":"Invoices","is":"/home/anna/Invoices"},{"named":"october","is":""},{"named":"most","is":20}]}},

"format":1}

round 1, rename: NotAMessage(MoreThanOneMessage)
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"/home/anna/Invoices/scan001.pdf","is":true},{"named":"march.pdf","is":true}]}},

 "format":1}

round 1, move: NotAMessage(MoreThanOneMessage)
{"asks":{"propose":{"verb":"move_file","given":[{"named":"/home/anna/Invoices/march.pdf","is":"/home/anna/Archive"},{"named":"march.pdf","is":"Anna"}]}},

"format":1}

round 1, archive: NotAMessage(MoreThanOneMessage)
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"/home/anna/Invoices","is":1},{"named":"/home/anna/Archive","is":1},{"named":"invoices","is":1}]}},

    "format":1}

round 1, open: NotAMessage(NotReadable)
{"asks":{"propose":{"application":"org.alo.Writer"}},"format":1}

round 1, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"org.alo.Writer","is":true}]}},"format":1}

round 1, close: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"close_application","given":[{"named":"org.alo.Writer","is":true}]}},"format":1}

round 1, arrange: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"org.alo.Writer","is":true},{"named":"left_half","is":true}]}},"format":1}

round 2, list: ArgumentRefused(NoSuchArgument { verb: "list_folder", argument: "/home/anna/Invoices" })
{"asks":{"propose":{"verb":"list_folder","given":[{"named":"/home/anna/Invoices","is":1}]}},"format":1}

round 2, read: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"read_file","given":[{"named":"/home/anna/Invoices/march.pdf","is":true}]}},"format":1}

round 2, find: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"find_in_folder","given":[{"named":"/home/anna/Invoices","is":true},{"named":"october","is":true},{"named":"20","is":20}]}},"format":1}

round 2, rename: ArgumentRefused(NoSuchArgument { verb: "rename_file", argument: "march.pdf" })
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"march.pdf","is":"/home/anna/Invoices/scan001.pdf"}]}},"format":1}

round 2, move: ArgumentRefused(NoSuchArgument { verb: "move_file", argument: "/home/anna/Invoices/march.pdf" })
{"asks":{"propose":{"verb":"move_file","given":[{"named":"/home/anna/Invoices/march.pdf","is":"file"},{"named":"/home/anna/Archive","is":"folder"}]}},"format":1}

round 2, archive: NotAMessage(MoreThanOneMessage)
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"/home/anna/Invoices","is":1},{"named":"/home/anna/Archive","is":1},{"named":"invoices","is":"Invoices.zip"}]}},

"format":1}

round 2, open: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"open_application","given":["application","org.alo.Writer"]}},"format":1}

round 2, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"org.alo.Writer"}]}},"format":1}

round 2, close: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"close_application","given":[{"named":"org.alo.Writer"}]}},"format":1}

round 2, arrange: ArgumentRefused(NoSuchArgument { verb: "arrange_application", argument: "org.alo.Writer" })
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"org.alo.Writer","is":"left_half"}]}},"format":1}

```

### smollm2-1.7b-instruct — 0 of 20, `rarely`

```text
round 1, list: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"list_folder","given":{"name":"/home/anna/Invoices"}}},"format":1}

round 1, read: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"file","given":{"named":"/home/anna/Invoices/march.pdf"}}},"format":1}

round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"list_folder","given":{"named":"folder","is":{"path":{"fileSystem":"path","is_absolute":true},"path":"/home/anna/Invoices","extension":{"type":"string","is_case_sensitive":false,"regex":["[0-9]+"]}}}}},"format":1}

round 1, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"rename_file","given":{"named":"/home/anna/Invoices/scan001.pdf","name":"march.pdf"}}},"format":1}

round 1, move: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"rename_file","given":{"named":"file","is":"/home/anna/Invoices/march.pdf"}}},"format":1}

round 1, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"name":"rename_file","given":{"folder":"/home/anna/Invoices","to_be_renamed":"invoices","most":1}}},"format":1}

round 1, open: NotAMessage(NotReadable)
{"asks":{"propose":{"read":{"named":"ORG_ALO_WRITER","is":true}}},"format":1}

round 1, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"read":{"named":["application","focused","application","focused","application","where","window","whole_screen"],"given":{"named":["application","focused","application","focused","application","focused","application","whole_screen"]}}}},"format":1}

round 1, close: NotAMessage(NotReadable)
{"asks":{"propose":{"read":{"verb":"org","given":[{"named":"application","is":"org.alo.Writer"}]}}},"format":1}

round 1, arrange: NotAMessage(NotReadable)
{"asks":{"propose":{"read":{"verb":"ALOWriter","named":{"named":"org.alo.Writer","is":"left_half"}}}},"format":1}

round 2, list: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"list_folder","given":{"named":{"folder":{"path":"/home/anna/Invoices"}}}}},"format":1}

round 2, read: NotAMessage(NotReadable)
{"asks":{"propose":{"read":{"named":{"named":"march.pdf","is":{"name":{"name":"a whole number with no quotes","min":0,"max":255},"length":{"length":{"min":0,"max":255}}}}}}},"format":1}

round 2, find: NotAMessage(NotReadable)
{"asks":{"propose":{"name":"search_file","given":{"folder":{"full_path":{"named":"/home/anna/Invoices"}},"named":"october","most":20}}},"format":1}

round 2, rename: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"rename_file","given":{"folder":"/home/anna/Invoices","name":"scan001.pdf","named_files":{"name":"march.pdf"}}}},"format":1}

round 2, move: NotAMessage(NotReadable)
{"asks":{"propose":{"read":{"verb":"move_file","given":{"folder":"/home/anna/Archive","file":"/home/anna/Invoices/march.pdf"}}}},"format":1}

round 2, archive: NotAMessage(NotReadable)
{"asks":{"propose":{"read":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{"named":{}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}}

round 2, open: NotAMessage(NotReadable)
{"asks":{"propose":{"read":{"verb":"org","named":"application","is":"alowriter"}}},"format":1}

round 2, focus: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"open_application","given":{"named":"application","is":{"type":"installed","identifier":"org.alo.Writer"}}}},"format":1}

round 2, close: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"close_application","given":{"named":"org.alo.Writer"}}},"format":1}

round 2, arrange: NotAMessage(NotReadable)
{"asks":{"propose":{"read":{"named":"window","is":"org.alo.Writer","where":{"is":{"value":{"type":"constant","constant":{"value":"left_half"}},"with":{"type":"literal","literal":{"value":"left_half"}}}}}}},"format":1}

```
