# Every catalogue entry graded, or refused with the reason

**Date:** 2026-09-13
**Workstream:** v0.5 — the models, measured
**Task:** *Every entry graded, or refused with the reason*
(`docs/autonomy/v0-5-the-models-measured-plan.md`, task 2)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3** (8 cores), **8 GB unified memory**, macOS 26.5.2;
runtime Ollama **0.34.0** (the pinned version) on `127.0.0.1`, the Linux VM
stopped during every run. Gates in Ubuntu 24.04 aarch64 under Lima, kernel
7.0.0-31-generic, as root.
**Status:** open — blocked on one line in `alo-image`, which is not this lane's crate. Everything else is done and published.

## What changed, for somebody outside this repository

Every model in the catalogue now either carries a verb-driving grade with the
machine it was measured on, or says in one translatable sentence **why** it has
none — with one exception, named below, that waits on another crate. Two more 7B-class models were measured — Mistral 7B (0 of 10) and
Llama 3.1 8B (1 of 10), both `rarely` — and four could not be, because the
measuring machine has 8 GB. **No local model clears the bar**, so the catalogue
still gives no local model the agent, and now it says so with ten measurements
behind it rather than seven.

## Every entry

| Entry | Result | Where |
|---|---|---|
| `eurollm-9b-instruct` | not measured — *too large for the measuring machine* (no answer within five minutes; 6.49 GB loaded) | Apple M3, 8 GB, 2026-09-13 |
| `teuken-7b-instruct` | not measured — *too large for the measuring machine* (GPU out of memory on the first question, twice) | Apple M3, 8 GB, 2026-09-13 |
| `mistral-7b-instruct` | **measured: 0 of 10, `rarely` — not yet written into the catalogue** (see *Why the task is open*) | Apple M3, 8 GB, 2026-09-13 |
| `mixtral-8x7b-instruct` | not measured — *too large for the measuring machine* (26.4 GB against 8 GB; not fetched) | Apple M3, 8 GB, 2026-09-13 |
| `qwen2.5-7b-instruct` | `rarely`, 4 of 10 (task 1) | Apple M3, 8 GB, 2026-09-13 |
| `phi-3-mini-instruct` | `rarely` | development PC's WSL guest, 2026-09-04 |
| `llama-3.1-8b-instruct` | **`rarely`, 1 of 10** | Apple M3, 8 GB, 2026-09-13 |
| `gemma-2-9b-instruct` | not measured — *too large for the measuring machine* (no answer to the first exercise within five minutes; 7.45 GB loaded, 4.13 GB of it on the GPU) | Apple M3, 8 GB, 2026-09-13 |
| `llama-3.2-3b-instruct`, `qwen2.5-3b-instruct`, `gemma-2-2b-instruct`, `smollm2-1.7b-instruct` | `rarely` | development PC's WSL guest, 2026-09-04 |
| `qwen3-1.7b`, `granite-3.2-2b-instruct` | `rarely` | development PC's WSL guest, 2026-09-11 |

**The recommendation, re-derived from the grades.** `agent_for_cpu` still
refuses, with `NoneClearsTheBar { to_choose_from: 7, measured: 5 }` on a
16 GB machine (six once Mistral's grade is written). There was no recommended entry before this task, so none moved;
the refusal a person reads is the one for a measurement that was made.

**No entry was removed**, and the bar, the prompt, the scoring and the wait did
not move.

## How the two new grades failed

**Mistral 7B** got the envelope and the reads' door right and lost the argument
list on every answer: `given` was a plain object of name to value rather than
the list of `{named, is}` pairs, which is how `qwen2.5-3b-instruct` failed on
2026-09-04. Every change also named its verb as the door. **Llama 3.1 8B**
mixed the two argument shapes inside one answer and dropped a closing brace on
its best-formed read; its one success was `arrange`.

### Mistral 7B, verbatim

Run 2026-09-13 19:21:20–19:21:56 UTC.

```text

----- round 1, list: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"list_folder","given":{"folder":"/home/anna/Invoices"}}}}
----- end

----- round 1, read: NotAMessage(NotReadable)
 {"format":1,"asks":{"read":{"verb":"read_file","given":{"file":"/home/anna/Invoices/march.pdf"}}}}
----- end

----- round 1, find: NotAMessage(NotReadable)
 {"format":1,"asks":{"find_in_folder":{"verb":"FIND_IN_FOLDER","given":{"folder":"/home/anna/Invoices","named":"october","most":20}}}}
----- end

----- round 1, rename: NotAMessage(NotReadable)
 {"format":1,"asks":{"rename_file":{"file":"/home/anna/Invoices/scan001.pdf","name":"march.pdf"}}}
----- end

----- round 1, move: NotAMessage(NotReadable)
 {"format":1,"asks":{"move_file":{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}}}
----- end

----- round 1, archive: NotAMessage(NotReadable)
 {"format":1,"asks":{"archive_folder":{"folder":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices"}}}
----- end

----- round 1, open: NotAMessage(NotReadable)
 {"format":1,"asks":{"open_application":{"application":"org.alo.Writer"}}}
----- end

----- round 1, focus: NotAMessage(NotReadable)
 {"format":1,"asks":{"focus_application":{"application":"org.alo.Writer"}}}
----- end

----- round 1, close: NotAMessage(NotReadable)
 {"format":1,"asks":{"close_application":{"application":"org.alo.Writer"}}}
----- end

----- round 1, arrange: NotAMessage(NotReadable)
 {"format":1,"asks":{"arrange_application":{"application":"org.alo.Writer","where":"left_half"}}}
----- end
```

### Llama 3.1 8B, verbatim

Run 2026-09-13 19:46:06–19:47:12 UTC. The first run, at 19:22, stopped on its
tenth exercise when the GPU ran out of memory (the runtime answered HTTP 500,
`llama-server terminated`); the harness stopped rather than score it, and this
second run, with nothing else loaded, is the measurement.

```text

----- round 1, list: NotAMessage(NotReadable)
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}
----- end

----- round 1, read: NotAMessage(NotReadable)
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"file":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 1, find: NotAMessage(NotReadable)
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"october","is":""},{"folder":"/home/anna/Invoices","is":20}]}}}
----- end

----- round 1, rename: NotAMessage(NotReadable)
{"format":1,"asks":{"rename_file":{"file":"/home/anna/Invoices/scan001.pdf","name":"march.pdf"}}}
----- end

----- round 1, move: NotAMessage(NotReadable)
{"format":1,"asks":{"move_file":{"file":"/home/anna/Invoices/march.pdf","into":"/home/anna/Archive"}}}
----- end

----- round 1, archive: NotAMessage(NotReadable)
{"format":1,"asks":{"propose":{"archive_folder":{"folder":"/home/anna/Invoices","into":"/home/anna/Archive","name":"invoices.zip"}}}}
----- end

----- round 1, open: NotAMessage(NotReadable)
{"format":1,"asks":{"open_application":{"application":"org.alo.Writer"}}}
----- end

----- round 1, focus: NotAMessage(NotReadable)
{"format":1,"asks":{"focus_application":{"application":"org.alo.Writer"}}}
----- end

----- round 1, close: NotAMessage(NotReadable)
{"format":1,"asks":{"close_application":{"application":"org.alo.Writer"}}}
----- end

----- round 1, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end
```

## Why the task is open

`alo-image`'s `weights_naming_a_model_nobody_measured_are_caught` uses
`mistral-7b-instruct` as its example of *a model nobody has put to
`alo-driving`*. With Mistral's grade in the catalogue, the gate failed:
`checking::tests::weights_naming_a_model_nobody_measured_are_caught ... FAILED`.
The check is right and its example stopped being true. `alo-image` is not in
this lane's plan, so per the lanes' rule the change is a finding rather than an
edit: the grade is held out of `data/catalogue.toml`, the two tests that require
every entry to be graded or say why name Mistral as their one exception (and
fail the day its grade arrives, so the exception cannot outlive its reason), and
the task stays open. The one-line change, for `alo-image`'s owner, is in
`docs/quirks.md`: an example still unmeasured, or better, one read off the
catalogue.

## What was built

- **`crates/alo-models/src/unmeasured.rs`**: a `[model.unmeasured]` block —
  `because` (`too-large-for-the-measuring-machine`,
  `the-runtime-refused-the-file` or `weights-not-published`) with the machine,
  date and runtime, held to the same checks a grade's machine is. The loader
  refuses a reason beside a grade. `Unmeasured::said` is the sentence a person
  reads.
- **Three words** in `alo-models`' vocabulary, each with a translator's note,
  each about *the machine that measured this catalogue* and never the reader's
  — a test holds them to that.
- **Tests:** `every_entry_we_ship_is_graded_or_says_why_not`,
  `a_reason_beside_a_grade_is_refused`, `unmeasured`'s own four, and in
  `alo-choosing`, `a_model_measured_rarely_is_offered_for_what_it_is.rs`:
  choosing a measured `rarely` model still stands, the agent is refused with
  `NONE_CLEARS_THE_BAR` and not `NONE_MEASURED`, and every reason reaches a
  person in the vocabulary `alo-saying` collects.

**One reading of the plan, stated.** The acceptance says `alo-choosing`'s offer
of such a model is *the sentence the crate already has*. `alo-choosing` has no
sentence about verb driving: it holds a person's choice and deliberately never
gates it on a grade. The sentence is `alo-models`' `NONE_CLEARS_THE_BAR`, shown
when a machine withholds the agent. The test is on `alo-choosing` as asked, and
checks both halves: the choice stands, and the refusal is the measured one.

## Egress

`ollama pull` of five artefacts, 18:53:42–19:01:23 UTC: `mistral:7b-instruct-v0.3-q4_K_M`,
`llama3.1:8b-instruct-q4_K_M` and `gemma2:9b-instruct-q4_K_M` from
`registry.ollama.ai`; Teuken and EuroLLM from `hf.co` — about 25.7 GB in all.
The two requantised files are the ones the catalogue pins: the runtime stores
blobs by content hash, and `sha256-03fd13da…630b` (5,018,868,512 bytes) and
`sha256-785a3b28…806b` (5,582,838,496 bytes) are on disk. Mixtral was not
fetched: 26.4 GB for a model the machine cannot hold would be egress with no
purpose. The measurements caused none.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root on kernel 7.0.0-31-generic:

| Gate | Result |
|---|---|
| formatting | pass |
| clippy, warnings denied | pass — zero warnings |
| the workspace's tests | **fail — one test, not this task's** |
| the supervisor's formatting, clippy and own tests | pass |
| rustdoc, warnings denied | pass |
| the BPF target's formatting and clippy | pass |

`cargo test --workspace --no-fail-fast`: **4,221 passed, 1 failed, 23 ignored**.
The one is `alo-bounding`'s
`ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`, the
aarch64 failure recorded on the untouched tree in task 1's report and
`docs/quirks.md`. With Mistral's grade written in, a second test failed —
`alo-image`'s, above — and that is why the grade is held out. The publish script
re-runs all nine on the tree combined with whatever `main` has by then, and
pushes only if the result is this one.

## Findings, in `docs/quirks.md`

- *What 8 GB of unified memory holds*, with each model's loaded size, the share
  on the GPU and the outcome.
- *Teuken's GGUF carries no chat template*: the runtime warns and answers with
  `<|im_end|>` in the text, so a grade through it would measure the missing
  template. That belongs to task 3.

## What would unblock the four

A machine with more memory, or this one with the GPU's share raised —
`sudo sysctl iogpu.wired_limit_mb=6800` needs the owner's password, and was not
run. Teuken also needs its template settled first (task 3). Nothing here moves
an *On the machine* box; a Mac is not the machine.
