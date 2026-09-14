# A grade near a line gets a second round

**Date:** 2026-09-14
**Workstream:** v0.5 — the models, measured
**Task:** *A grade that lands one short of a line gets a second round*
(`docs/autonomy/v0-5-the-models-measured-plan.md`, task 6)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3**, **8 GB unified memory**, macOS 26.5.2, Ollama **0.34.0**
(pinned) on `127.0.0.1`, the Linux VM stopped during the run. Gates in Ubuntu
24.04 aarch64 under Lima, kernel 7.0.0-31-generic, as root.
**Status:** done.

## What changed, for somebody outside this repository

Qwen 2.5 7B, the model closest to the next grade up, was measured again with
twice the attempts: **8 of 20, still `rarely`**. Every grade in the catalogue now
says how many attempts drove the verbs and out of how many, so anybody can check
it, and a grade from a single round of ten that lands right next to a line can
no longer be written without a second round.

## The measurement

`qwen2.5:7b-instruct-q4_K_M`, two rounds of the fixed ten, 2026-09-13
23:51:52–23:52:47 UTC (2026-09-14 on the machine's clock). **8 of 20 — 40%,
`rarely`** (the `sometimes` line is 50%). The first round drove 4, the second
4. The first round of task 1 also drove 4, and task 4's by-file round 3: four
rounds of ten, 3 to 4 each, all short of five.

### All twenty, verbatim

```text

----- round 1, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 1, read: NotAMessage(NotReadable)
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}}
----- end

----- round 1, find: NotAMessage(NotReadable)
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}}
----- end

----- round 1, rename: NotAMessage(NotReadable)
{"format":1,"asks":{"rename_file":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 1, move: NotAMessage(NotReadable)
{"format":1,"asks":{"move_file":{"verb":"move_file","given":[{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}]}}}
----- end

----- round 1, archive: NotAMessage(NotReadable)
{"format":1,"asks":{"archive_folder":{"folder":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices.zip"}}}
----- end

----- round 1, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 1, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 1, close: NotAMessage(NotReadable)
{"format":1,"asks":{"close_application":{"verb":"close_application","given":[{"application":"org.alo.Writer","is":"org.alo.Writer"}]}}}
----- end

----- round 1, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end

----- round 2, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 2, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 2, find: NotAMessage(NotReadable)
{"format":1,"asks":{"find_in_folder":{"verb":"read","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 2, rename: NotAMessage(NotReadable)
{"format":1,"asks":{"rename_file":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 2, move: NotAMessage(NotReadable)
{"format":1,"asks":{"move_file":{"verb":"move_file","given":[{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}]}}}
----- end

----- round 2, archive: NotAMessage(NotReadable)
{"format":1,"asks":{"archive_folder":{"folder":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices.zip"}}}
----- end

----- round 2, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 2, focus: NotAMessage(NotReadable)
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"application":"org.alo.Writer"}]}}}
----- end

----- round 2, close: NotAMessage(NotReadable)
{"format":1,"asks":{"close_application":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 2, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end
```

## What was built

- **`alo-driving`**: `grade_of(drove, of)`, which is now what
  `Measured::grade` computes, so a written count is graded by the same bar as a
  run; and `owes_a_second_round(drove, of, one_round)` — a grade from no more
  than one round landing within one attempt of either line.
- **`alo-models`**: `MeasuredOn` carries `drove` and `of` (both or neither,
  and a count that could have been a run); the catalogue loader refuses a grade
  with no counts. In a person's settings they are optional, keeping that
  contract additive.
- **`data/catalogue.toml`**: counts beside all ten grades, from the reports that
  made them; the rule written where a curator reads it; Qwen 2.5 7B's grade
  replaced by the twenty-attempt one, dated 2026-09-14.
- **`crates/alo-driving/tests/the_catalogue_grades_are_what_their_counts_earn.rs`**:
  every shipped grade is what its counts earn and none owes a second round; and
  the refusals shown — a grade its counts do not earn, and every one-round
  count within one of five or nine.
- The brought-file harness writes the counts beside a person's grade too.

## Egress

None.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root: formatting, clippy with warnings denied, rustdoc, the supervisor's
three and both BPF gates pass. The workspace's tests failed on the known aarch64
test and, in that run, on one more: `alo-bounding`'s
`an_inherited_descriptor_cannot_be_reopened_through_the_name_the_kernel_gives_it`,
which run alone passed once and failed once with `EBUSY` taking a control group
away — a race in that fixture, in a crate this task does not touch, now in
`docs/quirks.md`. Every test in `alo-models` and `alo-driving`, including the
new ones, passes. The publish script re-runs all nine on the tree combined with
`main` and pushes only when the one failure is the known aarch64 test.
