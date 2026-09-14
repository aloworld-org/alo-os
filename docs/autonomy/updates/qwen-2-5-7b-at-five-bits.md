# Qwen 2.5 7B at five bits

**Date:** 2026-09-14
**Workstream:** v0.5 — the models, measured
**Task:** *The same weights at a higher quantisation*
(`docs/autonomy/v0-5-the-models-measured-plan.md`, task 13)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3**, **8 GB unified memory**, macOS 26.5.2, Ollama **0.34.0**
(pinned) on `127.0.0.1`, the Linux VM stopped during the run. Gates in Ubuntu
24.04 aarch64 under Lima, kernel 7.0.0-31-generic, as root.
**Status:** done.

## What changed, for somebody outside this repository

The same Qwen 2.5 7B, stored at five bits instead of four, did **worse** at
driving alo OS's verbs on this machine: **18 of 40**, against 35 of 40 at four
bits. Almost every change it was asked for, it sent through the wrong door. Six
bits was not measured: this machine cannot hold it without swapping. So a higher
quantisation is not the way over the bar here, and the four-bit file the catalogue
names stays the best measured.

## The measurement

`qwen2.5:7b-instruct-q5_K_M` (registry digest of the weights `55b396c9…`,
5,444,831,648 bytes), in the envelope, four rounds of the fixed ten through the
harness, 2026-09-14 05:41:37–05:51:03 UTC.

| Quantisation | Loaded | On the GPU | Drove | Grade |
|---|---|---|---|---|
| Q4_K_M (task 7) | 5,345,399,272 | 4,583,210,351 (86%) | 35 of 40 | `sometimes` |
| **Q5_K_M** | **5,959,592,178** | **4,563,287,407 (77%)** | **18 of 40** | **`rarely`** |
| Q6_K | — | — | not measured | refused: does not fit |

- **Residency.** `/api/ps` was read every twenty seconds during the Q5_K_M run.
  Q4_K_M's residency was read on loading it the same day, not during task 7's
  run: it is fixed by the model's size and the GPU's working-set limit, not by
  load, so it is the residency that run had.
- **Swap.** Before the load this Mac already had 5.93 GB in swap, from its other
  programs, and it rose to 6.26 GB during the run. The Q5_K_M weights that were
  not on the GPU were in memory, and the machine was paging other things to hold
  them. The grade is written with that stated, and with its residency beside it.
- **Q6_K was not fetched.** Its 6,254,199,200 bytes, loaded, would put about
  2.4 GB of weights off a GPU that holds about 4.6, on a machine already in swap.
  That is the grade against swap the task forbids, and fetching 6.3 GB to prove
  it would have been egress for nothing.

## How it failed

**22 of the 22 failures were the wrong door** — the right verb, the right
arguments, and a change sent as a read:

| Exercise | Wrong door |
|---|---|
| `rename`, `move`, `open`, `close` | 4 of 4 each |
| `arrange` | 3 of 4 |
| `focus` | 2 of 4 |
| `archive` | 1 of 4 |

Every read drove. At five bits the model followed the prompt's one example
envelope — which names `read` (task 12) — for nearly every change. That is task
12's finding at full strength, in the same weights: the gap between Qwen 2.5 7B
and the bar is, on this evidence, far more about which door the instructions show
than about how precisely the weights are stored.

## What was built

- **`crates/alo-models/src/also_at.rs`**: `AlsoAt`, an entry's other
  quantisations each with its own artefact, size and envelope grade.
  - `quantised_at()` tells one quantisation from another;
  - refused: a quantisation that is the entry's own, one with no size, one
    `not-measured`, or one without counts and residency.
- **`MeasuredOn`** gains `loaded_bytes` and `on_the_gpu_bytes` (both or neither,
  never more on the GPU than loaded), optional so settings files stay additive.
- **The catalogue:**
  - `qwen2.5-7b-instruct` carries Q4_K_M's residency beside its envelope grade;
  - it gains `[[model.also_at]]` for Q5_K_M;
  - its comment says why Q6_K is absent.
- **The counts test** holds every other quantisation's grade to its counts too.

## What the image would pin

Nothing new. No quantisation cleared the bar; the entry keeps naming Q4_K_M, the
best measured.

## Egress

`ollama pull qwen2.5:7b-instruct-q5_K_M` from `registry.ollama.ai`, 05:28:39–05:30:09
UTC, 5.4 GB; and reading four manifests to compare sizes. The measurement caused
none.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root, run by the Mac lane's publish script on the tree combined with
`main`. It publishes only when formatting, clippy with warnings denied, rustdoc,
the supervisor's three and both BPF gates pass and the workspace's tests fail on
exactly one test — `alo-bounding`'s
`ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`, the
aarch64 failure recorded in the first task's report — and it is retried, after
removing debris whose process is gone, when the boundary fixtures' control-group
race (`docs/quirks.md`) adds another. Every test in `alo-models` and
`alo-driving`, including the new ones, passes.

The first publish attempts refused this task for its own defect: the residency
fields made `Weights` larger, so `alo_telling::Warn::Say` became many times the
size of its sibling variants and clippy denied `large_enum_variant`. The warning
is now boxed in that variant; nothing outside `alo-telling` names it, and its
tests pass.

## The forty answers, verbatim

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

----- round 1, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 1, move: TheWrongDoor
{"asks":{"read":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 1, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 1, open: TheWrongDoor
{"asks":{"read":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, close: TheWrongDoor
{"asks":{"read":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 1, arrange: TheWrongDoor
{"asks":{"read":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
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

----- round 2, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 2, move: TheWrongDoor
{"asks":{"read":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 2, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 2, open: TheWrongDoor
{"asks":{"read":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, focus: TheWrongDoor
{"asks":{"read":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, close: TheWrongDoor
{"asks":{"read":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 2, arrange: TheWrongDoor
{"asks":{"read":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
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

----- round 3, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 3, move: TheWrongDoor
{"asks":{"read":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 3, archive: Drove
{"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 3, open: TheWrongDoor
{"asks":{"read":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 3, focus: TheWrongDoor
{"asks":{"read":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 3, close: TheWrongDoor
{"asks":{"read":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
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

----- round 4, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
----- end

----- round 4, move: TheWrongDoor
{"asks":{"read":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}},"format":1}
----- end

----- round 4, archive: TheWrongDoor
{"asks":{"read":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}},"format":1}
----- end

----- round 4, open: TheWrongDoor
{"asks":{"read":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 4, focus: Drove
{"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 4, close: TheWrongDoor
{"asks":{"read":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
----- end

----- round 4, arrange: TheWrongDoor
{"asks":{"read":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}},"format":1}
----- end
```
