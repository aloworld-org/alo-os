# The door that asks in the envelope

**Date:** 2026-09-14
**Workstream:** v0.5 — the models, measured
**Task:** *The door that asks in the envelope*
(`docs/autonomy/v0-5-the-models-measured-plan.md`, task 14)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3**, **8 GB unified memory**, macOS 26.5.2, Ollama **0.34.0**
(pinned) on `127.0.0.1`, the Linux VM stopped during the measurement. Gates in
Ubuntu 24.04 aarch64 under Lima, kernel 7.0.0-31-generic, as root.
**Status:** done.

## What changed, for somebody outside this repository

- **The door.** The way of asking a local model that nearly doubled how often
  Qwen 2.5 7B produces a request alo OS can act on is now a door the product
  itself has, not something only the measuring tool did. An agent turn can ask
  through it once the turn is wired to it, which is lane A's work.
- **The measurement uses it.** The measurement now asks through that same door,
  and through it Qwen 2.5 7B drove the verbs **71 times in 80 (88.75%)**.
- **The bar.** That is one attempt short of the 72 in 80 the bar requires. No
  local model is given the agent yet.

## The door

`Asking::to_this_machine_in_the_envelope(question, runtime: &Ollama)`, in
`crates/alo-asking/src/in_the_envelope.rs`.

- **Shared with `to_this_machine`.** It uses the same permission check and the
  same mapping from a runtime failure to the sentence a person reads. Both now
  live in `locally.rs` as `the_runtime_is_the_place` and `answered_here`, so the
  two doors cannot disagree about either.
- **What it sends.** It asks `Ollama::answers_in_the_envelope` — the pinned
  runtime, typed as itself (ADR 0032, decision 4).

Tests, reading requests off a socket:

- **`an_agent_turns_question_goes_to_the_runtime_held_to_the_envelope`** — the
  request's `format` is exactly `the_envelope()`, with no `verb`, `given`,
  `named`, `is` or `question` inside it, and the answer says *on this
  machine*;
- **`a_persons_question_through_the_runtime_door_carries_no_envelope`** — the
  door a person's question takes sends no `format` (decision 3);
- **`a_permission_for_somewhere_else_asks_the_runtime_nothing`** — a provider's
  permission is refused before any request;
- **`the_hosted_and_served_doors_never_hold_an_answer_to_the_envelope`** —
  `hosted.rs`, `served.rs`, `openai.rs` and `corridor.rs` name nothing of the
  envelope.

**The harness uses the door.** `alo-driving`'s `Asked::InTheEnvelope` now puts
every question through `to_this_machine_in_the_envelope`, with the answer's
source asserted, instead of calling the runtime itself.

## What the door measured

The harness through the door on the real runtime, 2026-09-14 06:53:56–06:57:23
UTC, eight rounds, Qwen 2.5 7B Q4_K_M (5,197,833,172 bytes loaded,
4,583,210,351 on the GPU):

| | Drove | Grade |
|---|---|---|
| Task 7, through the harness's own request, 40 attempts | 35 of 40 (87.5%) | `sometimes` |
| **This task, through the door, 80 attempts** | **71 of 80 (88.75%)** | **`sometimes`** |

- **A first single round through the door** gave 10 of 10. It was ten samples
  in the second-round band, so by the catalogue's own rule it is not a grade.
  It is why the eighty were run.
- **Replacing task 7's grade.** The catalogue's `drives_verbs_in_the_envelope`
  for the entry is now 71 of 80, the larger sample of the same method, with
  its residency beside it.

**The nine failures:**

| Exercise | Failed | Of |
|---|---|---|
| `rename` | 5 — all through the wrong door | 8 |
| `close` | 4 — one wrong door, three with the argument written as an object | 8 |
| the other eight | 0 | 64 |

At eighty attempts **one exercise is most of the gap**: `rename`, sent as a read
five times in eight. That is the answer task 12 could not see in five failures.
It is also the change the prompt's one `read` example most plausibly draws,
which task 12 named and did not change.

## For lane A

The wiring task is lane A's task 13. Until the turn asks through this door, the
catalogue's recommendation reads the free grade (ADR 0032 decision 5,
task 15). Nothing here changes which grade the recommendation reads.

## Egress

None.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root, run by the Mac lane's publish script on the tree combined with
`main`. It publishes only when formatting, clippy with warnings denied, rustdoc,
the supervisor's three and both BPF gates pass and the workspace's tests fail on
exactly one test — `alo-bounding`'s
`ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`, the
aarch64 failure recorded in the first task's report. Before publishing, in the
VM: `alo-asking`'s 108 library tests including the door's four pass, and
`cargo clippy -p alo-asking -p alo-driving --all-targets -- -D warnings` is
clean.

## The eighty answers, verbatim

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

----- round 2, close: TheWrongDoor
{"asks":{"read":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
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

----- round 3, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
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

----- round 4, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
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

----- round 4, close: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"close_application","given":[{"application":"org.alo.Writer"}]}},"format":1}
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

----- round 5, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
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

----- round 5, close: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"close_application","given":[{"application":"org.alo.Writer","is":"anna's_document"}]}},"format":1}
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

----- round 6, close: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"close_application","given":[{"application":"org.alo.Writer"}]}},"format":1}
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

----- round 8, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}
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
