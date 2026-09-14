# The instructions show every door

**Date:** 2026-09-14
**Workstream:** v0.5 — the models, measured
**Task:** *The instructions' one example, and what a second one costs*
(`docs/autonomy/v0-5-the-models-measured-plan.md`, task 16)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3**, **8 GB unified memory**, macOS 26.5.2, Ollama **0.34.0**
(pinned) on `127.0.0.1`, the Linux VM stopped during both measurements. Gates in
Ubuntu 24.04 aarch64 under Lima, kernel 7.0.0-31-generic, as root.
**Egress:** none. Both sets of weights were already on this machine from tasks 7
and 13; nothing was pulled.
**Status:** done.

## What changed, for somebody outside this repository

- **The instructions a model is measured under now show an example request
  through each door** — one for a question, one for a change — instead of one
  example through the question door and a sentence saying to use the other one
  for changes. The first instructions are kept, named, and are still what every
  existing grade means.
- **Every grade in the catalogue now says which instructions earned it**, by
  a fingerprint (SHA-256) of their text, and the catalogue refuses a grade that
  does not.
- **Qwen 2.5 7B, asked in the envelope through the product's own door, drove the
  verbs 80 times in 80 under the new instructions** — `reliably`, where the old
  instructions earned 71 in 80. **It is the first local model in the catalogue
  whose grade clears the bar**, and what it took, in one sentence: *an example
  request through the propose door beside the one through the read door, and
  nothing else.* The same weights at five bits went from 18 in 40 to 40 in 40.
- **Nobody is given the agent by this.** The new grade sits beside the old in the
  catalogue and the recommendation reads neither: a shipped machine's agent turn
  is not yet shown these instructions nor asked in the envelope, and until it is
  a grade earned that way describes a turn nobody runs (ADR 0032, decision 5;
  ADR 0034, decision 4). That wiring is lane A's.

## The decision

[ADR 0034](../../decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md)
sets out the three options the task names — the instructions as they are; one
example per door; one example whose door is a placeholder — with what each
measures, hides and costs, and takes **one example per door**. The placeholder
was refused on evidence already in `docs/quirks.md`: on 2026-09-04 SmolLM2
copied the prompt's placeholders (`"verb":"NAME"`) into its answers, so a door
left as a placeholder would measure placeholder-copying in the models least able
to choose a door.

It was written as 0033 and published as **0034**: the owner's decision on the
certified laptop took 0033 on `main` while this task was being gated, and
`alo-citing`'s test caught the two files sharing a number.

The ADR's own reopening condition — a model sending *reads* through `propose` as
often as it sent changes through `read` — did not occur: across 120 attempts
under the new instructions, every read went through `read`.

## What was built

| Where | What |
|---|---|
| `alo-driving/src/instructions.rs` | `Instructions::{AsFirstWritten, OneExamplePerDoor}`, each with its text, its name and its SHA-256. `AsFirstWritten` *is* `HOW_TO_ANSWER`, unchanged, and a test holds it to the digest it had today so an edit to it fails rather than silently relabelling every grade. |
| `alo-driving` | `prompt_under` and `Exercises::prompt_under`; `prompt` is `prompt_under(AsFirstWritten, …)`. Exercises, verbs, scoring and bar untouched. |
| the harness | `ALO_DRIVING_INSTRUCTIONS=one-example-per-door`; the run prints the digest it was under. A brought file is measured under the first instructions and its grade says so. |
| `alo-models` `MeasuredOn` | an optional `instructions` digest, refused when it is not sixty-four lowercase hexadecimal characters. |
| the catalogue loader | a grade — free, in the envelope, at another quantisation — that does not name its instructions is refused. |
| `alo-models/src/also_under.rs` | `[[model.also_under]]`, and `[[model.also_at.also_under]]` for another quantisation: a grade under other instructions, refused beside an entry with no envelope grade of its own, under the same instructions as the grade it sits beside, twice under one set, or without counts and residency. |
| `alo-driving/tests/every_grade_names_instructions_this_crate_has.rs` | every digest the catalogue ships is a set `alo-driving` has, and every entry's own grade names the first set. |
| `data/catalogue.toml` | the first digest beside all 27 existing grades — `HOW_TO_ANSWER` has not changed since it was written on 2026-09-03, before the first grade — and the two new grades beside the old. |
| `docs/contracts/person-settings.md` | `instructions` in `[brought.measured]`, additive, with the counts and residency the table had not yet listed. |
| `docs/quirks.md` | *A model copies the door of the example it is shown, not the sentence under it.* |

The digests:

| Instructions | SHA-256 |
|---|---|
| as first written (`HOW_TO_ANSWER`) | `d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce` |
| one example per door (`ONE_EXAMPLE_PER_DOOR`) | `93a7f458ce9d017d6d12759a281e5f0b347a0d6963c04eae03b4c30581aebd02` |

## The measurements

Both in the envelope, through `alo_asking::Asking::to_this_machine_in_the_envelope`
— the door task 14 built — against the same ten exercises and alo OS's own ten
verbs, residency read from `/api/ps` at the end of each run.

| Weights | Instructions | Drove | Grade | Loaded | On the GPU |
|---|---|---|---|---|---|
| `qwen2.5:7b-instruct-q4_K_M` | as first written (task 14) | 71 of 80 | sometimes | 5,197,833,172 | 4,583,210,351 |
| `qwen2.5:7b-instruct-q4_K_M` | **one example per door** | **80 of 80** | **reliably** | 5,197,833,172 | 4,583,210,351 |
| `qwen2.5:7b-instruct-q5_K_M` | as first written (task 13) | 18 of 40 | rarely | 5,959,592,178 | 4,563,287,407 |
| `qwen2.5:7b-instruct-q5_K_M` | **one example per door** | **40 of 40** | **reliably** | 5,959,592,178 | 4,563,287,407 |

The four-bit run was eight rounds so it is the same sample as the grade it sits
beside; the five-bit run four, for the same reason. Neither lands within one
attempt of a line, so neither owes a second round.

**Other models were not re-measured.** Qwen3 8B (10 of 20 in the envelope, task
11) came closest after Qwen 2.5 7B, but its weights were removed from this disk
after task 11, and fetching them again is egress this task did not need to
decide its question. The small models and the rest are graded under the first
instructions only, and their grades say so.

## The five from task 12, re-read one by one

Task 12's five were the failures of task 7's forty-attempt run. Under the new
instructions, the same exercise in the same round of the new eighty:

| Round | Exercise | Under the first instructions | Under one example per door | Changed |
|---|---|---|---|---|
| 1 | `find` | a correct read with a stray `"path"` key — not a message | a correct read, no stray key — **drove** | yes |
| 1 | `rename` | the right call through `read` — wrong door | the same call through `propose` — **drove** | yes |
| 2 | `rename` | the right call through `read` — wrong door | through `propose` — **drove** | yes |
| 2 | `close` | the argument written as `{"application": …}` — not a message | written as `{"named":"application","is":…}` — **drove** | yes |
| 3 | `close` | the right call through `read` — wrong door | through `propose` — **drove** | yes |

All five changed, and **none did not.** The three wrong doors are the change the
decision was made for. The two that were not about doors changed too, and this
report does not claim to know why: a second example shows the `given` shape a
second time, which may be why an argument stopped being written as an object,
but a stray key in one run of forty and its absence in one of eighty is not
enough to say so. Across the eighty, none of task 14's nine failures recurred —
not the five wrong doors, and not the four `close` arguments written as objects.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root, run by the Mac lane's publish script on the tree combined with
`main`. It publishes only when formatting, clippy with warnings denied, rustdoc,
the supervisor's three and both BPF gates pass and the workspace's tests fail on
exactly one test — `alo-bounding`'s
`ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`, the
aarch64 failure recorded in the first task's report. Before publishing, on the
Mac and so not gated: `alo-models`' and `alo-driving`'s tests pass and their
clippy is clean.

## The eighty answers under one example per door, Q4_K_M, verbatim

```text

----- round 1, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 1, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 1, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 1, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 1, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 1, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 1, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end

----- round 2, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 2, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 2, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 2, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 2, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 2, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 2, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end

----- round 3, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 3, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 3, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 3, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 3, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 3, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 3, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 3, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 3, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 3, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end

----- round 4, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 4, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 4, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 4, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 4, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 4, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 4, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 4, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 4, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 4, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end

----- round 5, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 5, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 5, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 5, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 5, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 5, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 5, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 5, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 5, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 5, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end

----- round 6, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 6, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 6, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 6, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 6, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 6, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 6, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 6, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 6, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 6, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end

----- round 7, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 7, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 7, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 7, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 7, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 7, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 7, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 7, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 7, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 7, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end

----- round 8, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 8, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 8, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 8, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 8, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 8, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 8, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 8, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 8, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 8, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end
```

## The forty answers under one example per door, Q5_K_M, verbatim

```text

----- round 1, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 1, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 1, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 1, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 1, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 1, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 1, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end

----- round 2, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 2, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 2, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 2, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 2, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 2, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 2, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end

----- round 3, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 3, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 3, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 3, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 3, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 3, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 3, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 3, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 3, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 3, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end

----- round 4, list: Drove
{"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}},"format":1}
----- end

----- round 4, read: Drove
{"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}},"format":1}
----- end

----- round 4, find: Drove
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}},"format":1}
----- end

----- round 4, rename: Drove
{"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 4, move: Drove
{"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 4, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 4, open: Drove
{"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 4, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 4, close: Drove
{"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 4, arrange: Drove
{"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end
```
