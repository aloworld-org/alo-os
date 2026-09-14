# The same ten exercises through the engine's own server, held to the whole call

**Date:** 2026-09-14
**Workstream:** v0.5 — the models, measured
**Task:** *The same ten exercises through the engine's own server, held to the
whole call* (`docs/autonomy/v0-5-the-models-measured-plan.md`, task 17)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** **Apple M3**, **8 GB unified memory**, macOS 26.5.2. `llama.cpp`
**0.4.0** (build 10809, commit 5266f24da) on `127.0.0.1:8081`, the Linux VM and
the pinned runtime both stopped during every measurement. Gates in Ubuntu 24.04
aarch64 under Lima, kernel 7.0.0-31-generic, as root.
**Egress:** one — `brew install llama.cpp`, which fetched the bottle for
llama.cpp 0.4.0 from Homebrew's own servers. The binary it installed is
`/opt/homebrew/Cellar/llama.cpp/0.4.0/bin/llama-server`, SHA-256
`8939a1cf8a4e9a5cc18d6cd4d2d55440b48f8a44db00abe459a28d837ea051f7`. **No weights
were fetched**: the engine was pointed at the bytes the pinned runtime already
holds.
**Status:** done. **[ADR 0035](../../decisions/0035-the-wrapper-or-the-engine.md)
is rejected**, in the same commit as this report.

## The three questions ADR 0035 accepts on, answered first

**Did a model clear the bar here that cannot through the wrapper? No.** **Is the
margin on the one that already does materially wider? No — it is narrower.**
**Was the whole call held where only the envelope could be? Yes, and it bought
nothing.** The engine's server held every token of the answer to a grammar for
the entire call in the protocol's own key order, which the pinned runtime cannot
do; the grade did not improve, and under the instructions every earlier grade was
earned under it got worse. So the ADR is **rejected**: the wrapper stays, the
image does not change, and the four frictions in it stay quirks rather than
reasons.

## The measurement

The same file throughout — `qwen2.5:7b-instruct-q4_K_M`, digest
`sha256-2bada8a7450677000f678be90653b85d364de7db25eb5ea54136ada5f3933730`,
4,683,073,952 bytes, the blob the pinned runtime holds — served by
`llama-server -ngl 99 -c 4096`. The same ten exercises, the same bar, the same
second-round rule, the same scoring through `alo-protocol` and `alo-capability`,
eight rounds each. The only things that differ from the grades already in the
catalogue are the runtime and what held the answer.

| Instructions | The wrapper (Ollama 0.34.0) | The engine, whole call held |
|---|---|---|
| One example per door (ADR 0034) | 80 of 80, `reliably` | **80 of 80, `reliably`** |
| As first written | 71 of 80, `sometimes` | **65 of 80, `sometimes`** |

The grammar: 1,986 bytes, `sha256 efa8b951cbb44e6fdd41dfb7b1bcd5dc3196e14a5ab725be67dc26b0975bbae8`,
written from the verb registry the run is scored against rather than typed.

**Both grades are in the catalogue beside the wrapper's**, in
`[[model.also_under]]`, each naming its runtime, its instructions, the grammar
that held it, its counts and what was loaded. Nothing was written over.

## What the grammar holds, and why a high number was expected

Held by construction: the envelope, the door each verb's effect requires, the
spelling of every verb and argument name, the protocol's `named`-then-`is` key
order, and the shape of a value. Left to the model: **which** verb the request
calls, and the values. Whole classes of failure the catalogue's other grades
count — a stray key, an argument written as an object, a change through the read
door — cannot happen here at all. A grade under this grammar is therefore not
comparable with one under the envelope, which is why it is written beside them
and never over them.

## What the fifteen failures were, and why they matter more than the number

Under the first instructions the engine scored 65 of 80. **Every one of the
fifteen was the wrong verb**, and every one went through the door the grammar
required:

| Asked | Answered | Times |
|---|---|---|
| `rename` | `read_file` | 5 |
| `close` | `read_file` | 5 |
| `close` | `find_in_folder` | 3 |
| `focus` | `read_file` | 1 |
| `focus` | `find_in_folder` | 1 |

The first instructions show one example and it goes through the read door. Under
the pinned runtime that produced changes sent through the read door — a call
`alo_capability::Authorised::read` refuses. Under the grammar that door is
unreachable, so the same pull produced a **read verb** instead: a well-formed
call, through the right door, that a machine would act on. The measurement
catches it only because every exercise names the verb a correct answer calls.

That is the finding of this task. A constraint on the shape of an answer does not
fix a model reading the request wrongly; it moves the failure from where the
machine refuses it to where nothing would. ADR 0034's instructions fixed it —
under those, both runtimes answer 80 of 80.

## A fifth friction, and this one is the engine's

ADR 0035's case rested partly on four frictions that live in the wrapper. The
trial found one that would arrive with the engine. `qwen2.5:7b-instruct-q5_K_M`
(5,444,831,648 bytes) does not fit this machine's graphics processor.

- The **wrapper** serves it anyway, splitting the model itself — 5,959,592,178
  loaded, 4,563,287,407 on the graphics processor — and it carries two grades
  earned that way.
- The **engine** with every layer offloaded loads, then fails every request with
  *Insufficient Memory* and answers 500. Told by hand to keep 21 of its 28
  layers on the processor, it served one sixty-token answer in **256 seconds**,
  past the five minutes `alo_models` waits.

So **the five-bit weights could not be measured through the engine on this
machine at all**, and that is recorded as a result rather than left out. Model
placement is a feature of the wrapper; whoever proposes the engine again owns it,
on the smallest certified machine.

## What was built, and what it leaves behind

| Where | What |
|---|---|
| `alo-driving/src/the_whole_call.rs` | `grammar_for(&Verbs)` writes the GBNF for the whole call from the registry — one alternative per verb under the door its effect requires, its arguments in the order it declares them — and `digest_of` names a grammar as a grade records it. |
| `alo-driving/tests/the_grammar_holds_every_call_this_machine_accepts.rs` | every one of this machine's ten verbs is written out as the call a correct answer makes and put to the grammar; a change through the read door, the protocol's keys reversed, an unknown verb, an unknown argument, another envelope version and a sentence after the call are all refused. |
| `alo-driving/tests/gbnf/mod.rs` | a reader for the subset of GBNF the generator emits. `llama.cpp` is the authority on GBNF and nothing here stands in for it; this says the grammar means what it is meant to mean, and the run itself is what says the engine reads it the same way. |
| `alo-asking/src/held_to_the_whole_call.rs` | `to_a_service_on_this_machine_held_to`: the door beside `to_a_service_on_this_machine`, same address, same permission, with the answer held to a grammar the caller wrote. It knows nothing about verbs. |
| `alo-asking/src/openai.rs` | a second outgoing shape carrying `grammar`, and one road both shapes take. `Sent` is unchanged, because it is also what a body arriving here is read by, and widening it would widen what this machine accepts. |
| `alo-models` `MeasuredOn` | `held_to`: the digest of what held the answer, beside the instructions' digest. |
| `alo-models` `AlsoUnder` | now *the same weights measured another way*: other instructions, another runtime, or another shape holding the answer. Two grades measured all three ways the same are refused. |

Nothing in the product asks through the new door, and `ollama.rs` is untouched.
Both are kept so that the next candidate is measured rather than built again.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root, run by the Mac lane's publish script on the tree combined with
`main`. It publishes only when formatting, clippy with warnings denied, rustdoc,
the supervisor's three and both BPF gates pass and the workspace's tests fail on
exactly one test — `alo-bounding`'s
`ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`, the
aarch64 failure recorded in the first task's report.

## The eighty answers under ADR 0034's instructions, verbatim

```text

----- round 1, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 1, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 1, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 1, rename: Drove
{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 1, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 1, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 1, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 1, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 1, close: Drove
{"format":1,"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
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

----- round 2, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 2, rename: Drove
{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 2, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 2, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 2, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 2, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 2, close: Drove
{"format":1,"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 2, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end

----- round 3, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 3, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 3, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 3, rename: Drove
{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 3, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 3, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 3, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 3, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 3, close: Drove
{"format":1,"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 3, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end

----- round 4, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 4, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 4, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 4, rename: Drove
{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 4, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 4, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 4, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 4, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 4, close: Drove
{"format":1,"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 4, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end

----- round 5, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 5, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 5, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 5, rename: Drove
{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 5, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 5, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 5, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 5, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 5, close: Drove
{"format":1,"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 5, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end

----- round 6, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 6, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 6, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 6, rename: Drove
{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 6, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 6, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 6, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 6, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 6, close: Drove
{"format":1,"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 6, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end

----- round 7, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 7, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 7, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 7, rename: Drove
{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 7, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 7, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 7, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 7, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 7, close: Drove
{"format":1,"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 7, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end

----- round 8, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 8, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 8, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 8, rename: Drove
{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 8, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 8, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 8, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 8, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 8, close: Drove
{"format":1,"asks":{"propose":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 8, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end
```

## The eighty answers under the first instructions, verbatim

```text

----- round 1, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 1, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 1, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 1, rename: AnotherVerb { named: "read_file" }
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"}]}}}
----- end

----- round 1, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 1, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 1, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 1, focus: AnotherVerb { named: "find_in_folder" }
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/Users/Anna/Documents"},{"named":"named","is":"notes"},{"named":"most","is":5}]}}}
----- end

----- round 1, close: AnotherVerb { named: "read_file" }
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/Applications/org.alo.Writer.app"}]}}}
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

----- round 2, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 2, rename: AnotherVerb { named: "read_file" }
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"}]}}}
----- end

----- round 2, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 2, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 2, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 2, focus: AnotherVerb { named: "read_file" }
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/Users/Anna/Library/Application Support/org.alo.Writer/User Document 1"}]}}}
----- end

----- round 2, close: AnotherVerb { named: "read_file" }
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/Applications/org.alo.Writer.app"}]}}}
----- end

----- round 2, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end

----- round 3, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 3, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 3, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 3, rename: AnotherVerb { named: "read_file" }
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"}]}}}
----- end

----- round 3, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 3, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 3, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 3, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 3, close: AnotherVerb { named: "find_in_folder" }
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/Users/Anna/Documents"},{"named":"named","is":"org.alo.Writer"},{"named":"most","is":1}]}}}
----- end

----- round 3, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end

----- round 4, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 4, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 4, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 4, rename: Drove
{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 4, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 4, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 4, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 4, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 4, close: AnotherVerb { named: "read_file" }
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/Users/Anna/Library/Application Support/org.alo.Writer/User Document 1.txt"}]}}}
----- end

----- round 4, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end

----- round 5, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 5, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 5, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 5, rename: Drove
{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 5, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 5, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 5, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 5, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 5, close: AnotherVerb { named: "find_in_folder" }
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/Users/Anna/Applications"},{"named":"named","is":"org.alo.Writer"},{"named":"most","is":1}]}}}
----- end

----- round 5, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end

----- round 6, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 6, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 6, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 6, rename: AnotherVerb { named: "read_file" }
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"}]}}}
----- end

----- round 6, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 6, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 6, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 6, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 6, close: AnotherVerb { named: "read_file" }
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/Applications/org.alo.Writer.app"}]}}}
----- end

----- round 6, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end

----- round 7, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 7, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 7, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 7, rename: Drove
{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}}}
----- end

----- round 7, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 7, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 7, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 7, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 7, close: AnotherVerb { named: "find_in_folder" }
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/Users/Anna/Applications"},{"named":"named","is":"org.alo.Writer"},{"named":"most","is":1}]}}}
----- end

----- round 7, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end

----- round 8, list: Drove
{"format":1,"asks":{"read":{"verb":"list_folder","given":[{"named":"folder","is":"/home/anna/Invoices"}]}}}
----- end

----- round 8, read: Drove
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"}]}}}
----- end

----- round 8, find: Drove
{"format":1,"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}]}}}
----- end

----- round 8, rename: AnotherVerb { named: "read_file" }
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"}]}}}
----- end

----- round 8, move: Drove
{"format":1,"asks":{"propose":{"verb":"move_file","given":[{"named":"file","is":"/home/anna/Invoices/march.pdf"},{"named":"into","is":"/home/anna/Archive"}]}}}
----- end

----- round 8, archive: Drove
{"format":1,"asks":{"propose":{"verb":"archive_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"into","is":"/home/anna/Archive"},{"named":"name","is":"invoices.zip"}]}}}
----- end

----- round 8, open: Drove
{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 8, focus: Drove
{"format":1,"asks":{"propose":{"verb":"focus_application","given":[{"named":"application","is":"org.alo.Writer"}]}}}
----- end

----- round 8, close: AnotherVerb { named: "read_file" }
{"format":1,"asks":{"read":{"verb":"read_file","given":[{"named":"file","is":"/Applications/org.alo.Writer.app"}]}}}
----- end

----- round 8, arrange: Drove
{"format":1,"asks":{"propose":{"verb":"arrange_application","given":[{"named":"application","is":"org.alo.Writer"},{"named":"where","is":"left_half"}]}}}
----- end
```
