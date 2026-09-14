# The five that failed, read one by one

**Date:** 2026-09-14
**Workstream:** v0.5 — the models, measured
**Task:** *The five that failed, read one by one*
(`docs/autonomy/v0-5-the-models-measured-plan.md`, task 12)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** the attempts were made on an **Apple M3 with 8 GB unified memory**
under Ollama **0.34.0** on 2026-09-14 (task 7's forty-attempt run, 00:43:48–00:45:38
UTC). Nothing was measured again for this task. Gates in Ubuntu 24.04 aarch64
under Lima, kernel 7.0.0-31-generic, as root.
**Status:** done.

## What changed, for somebody outside this repository

Nothing was re-graded. The five times Qwen 2.5 7B failed to produce a request
alo OS would act on — out of forty — were read one at a time. **Three of the five
were a correct request sent through the wrong door**: a change asked for as a
read, which alo OS refuses because a change must wait for a person. All three
chose the *read* door, and the only example in the instructions a model is given
uses the read door. That is the most likely explanation for most of the gap, and
it is a question about the instructions rather than the model — which is exactly
why nothing about the instructions was changed here.

## The five, verbatim and classified

| Round | Exercise | What went wrong | Class |
|---|---|---|---|
| 1 | `find` | a correct call, with an extra `"path"` key beside `verb` and `given` | **not a call the protocol reads** |
| 1 | `rename` | the right verb and arguments, through `read` | **wrong door** (see below) |
| 2 | `rename` | the right verb and arguments, through `read` | **wrong door** |
| 2 | `close` | the right verb, the argument written as `{"application": …}` | **not a call the protocol reads** |
| 3 | `close` | the right verb and argument, through `read` | **wrong door** |

```text
round 1, find: NotAMessage(NotReadable)
{"asks":{"read":{"verb":"find_in_folder","given":[{"named":"folder","is":"/home/anna/Invoices"},{"named":"named","is":"october"},{"named":"most","is":20}],"path":"/home/anna/Invoices"}},"format":1}

round 1, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}

round 2, rename: TheWrongDoor
{"asks":{"read":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/Invoices/scan001.pdf"},{"named":"name","is":"march.pdf"}]}},"format":1}

round 2, close: NotAMessage(NotReadable)
{"asks":{"propose":{"verb":"close_application","given":[{"application":"org.alo.Writer"}]}},"format":1}

round 3, close: TheWrongDoor
{"asks":{"read":{"verb":"close_application","given":[{"named":"application","is":"org.alo.Writer"}]}},"format":1}
```

**A fourth class, because three were not enough.** The task names three: a valid
call to the wrong verb, the right verb with an argument the protocol refuses, and
output that is not a call. The most common failure is none of them. In all three
wrong-door attempts the verb was right and `alo-capability` accepted every
argument; what was refused was the door — `alo_capability::Authorised::read`
answers `NotAuthorised::ChangeWaits` for a change asked as a read, which is the
same refusal `alo-driving` scores as `TheWrongDoor`. So it is a refusal the real
turn makes, not one only the measurement enforces, and it is classed as itself
rather than forced into a class it is not. **No attempt called the wrong verb.**

## By exercise

| Exercise | Failed | Of |
|---|---|---|
| `rename` | 2 — both wrong door | 4 |
| `close` | 2 — one wrong door, one argument out of shape | 4 |
| `find` | 1 — a stray key | 4 |
| the other seven | 0 | 28 |

No single exercise accounts for most of the gap; two change exercises account for
four of the five.

## Does any exercise ask for something the protocol never validates

No. Every failure is refused by something the product itself runs:
`alo-protocol`'s reader refuses the unknown `path` key and the argument written as
an object, and `alo-capability` refuses a change through the read door. No
exercise asks for a thing nothing checks, and none was changed.

## What the harness might contribute

The prompt every exercise uses (`alo_driving::HOW_TO_ANSWER`) shows the model one
example line, and that line names the read door:

```text
{"format":1,"asks":{"read":{"verb":"NAME","given":[{"named":"ARGUMENT","is":VALUE}]}}}
```

The sentence under it says to use `propose` for a change, but the only envelope a
model can copy says `read` — and every wrong-door failure, here and in the small
models in task 10, chose `read`. ADR 0032's envelope also lists `read` first
among the doors. Neither was changed: the prompt is the method every grade in the
catalogue was earned under, and editing it to suit a candidate moves the bar.

**Whose gap it looks like, in one sentence:** most of it looks like the model
following the one example the instructions show, so it is as much the harness's
as the model's — and what would settle it is the same forty attempts under a
prompt whose example names neither door or both, graded as a separately named
method under a decision that says so, never written over the grade task 7 earned.

## Gates

This task changed documents only. The nine commands of `tools/kernel-loop`'s
`EVERY_GATE` were run on `e4a13f2` (task 11, as published) alone in the Lima VM
as root, because two publish runs had overlapped on one build directory while
publishing it: formatting, clippy with warnings denied, rustdoc, the supervisor's
three and both BPF gates pass; the workspace's tests passed 4,483 and failed 4 —
the known aarch64 test in `alo-bounding` and three `alo-agentd` boundary tests
failing inside the same run on one shared control group (*"Device or resource
busy"* removing it, then *"File exists"* making it), the race recorded in
`docs/quirks.md`. The Mac lane's publish script now holds a lock so two runs
cannot overlap, and it publishes this task only when the combined tree's one
failure is the known aarch64 test.
