# The recommendation reads the grade for the way turns ask

**Date:** 2026-09-14
**Workstream:** v0.5 — the models, measured
**Task:** *The recommendation reads the grade for the way turns ask*
(`docs/autonomy/v0-5-the-models-measured-plan.md`, task 15)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** nothing was measured for this task. Every grade it reads was earned
on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2, under Ollama
**0.34.0**. Gates in Ubuntu 24.04 aarch64 under Lima, kernel 7.0.0-31-generic,
as root.
**Egress:** none.
**Status:** done.

## What changed, for somebody outside this repository

- **Whether a model may be given the agent is now decided by the grade it
  earned the way an agent turn asks.** Since lane A's `e99be94`, a turn asks a
  local model for its next request held to the protocol's envelope, so a
  model measured that way is judged by that measurement. A model that was only
  ever measured by asking freely is judged by that, because it is the only
  measurement there is. Neither grade is rewritten, and both stay in the
  catalogue.
- **A person is told which way the grade that decides was earned**, in one of
  two sentences in the machine's vocabulary.
- **No local model is given the agent yet.** The best local model, Qwen 2.5 7B
  Instruct at Q4_K_M, is judged by its enveloped grade: **71 of 80,
  `sometimes`**, one attempt short of the 72 the bar needs, measured on the
  Apple M3 with 8 GB under Ollama 0.34.0. Its 80 of 80 from task 16 was earned
  under instructions no agent turn is shown, and so it does not decide — see
  *the finding* below.

## What was built

| Where | What |
|---|---|
| `alo-models/src/graded_for_turns.rs` | `Model::grade_for_the_turn` → `(Driving, AskedTheWay)`: the enveloped grade where one was measured, the free grade only where none was. `AskedTheWay::said` puts the way into words. |
| `Model::can_be_the_agent` | reads `grade_for_the_turn`, not `drives_verbs`. |
| `Catalogue::agent_for_cpu` | ranks and refuses by the same grade; its *none measured* versus *none clears the bar* count reads it too. |
| `Weights::can_be_the_agent` | the same rule, documented: weights somebody brought carry one grade, earned freely, so it is the grade that decides for them. |
| `alo-models` words | `models.graded.in-the-envelope` and `models.graded.freely`, with translator notes, in `EVERY_WORD`. |
| `alo-choosing`'s offer test | `an_offer_says_which_way_the_grade_that_decides_was_earned`: the shipped Qwen 2.5 7B is offered by its enveloped `sometimes`, with its free `rarely` kept, and both sentences reach a person as words rather than keys. |
| tests in `catalogue.rs` | the enveloped grade decides both ways — `rarely` freely and `reliably` enveloped **is** given the agent, `reliably` freely and `sometimes` enveloped **is not** — and a free `reliably` with no enveloped grade is. |
| `data/catalogue.toml` | the comments that said the recommendation does not read the enveloped grade now say it does. No grade changed. |

`alo-choosing` recommends nothing — its header says so, and that is ADR 0016 —
so *the offer* is the catalogue's grade and `alo-models`' sentence as the
chooser shows them. That is what its test holds.

## Is any local model now given the agent?

**No.** Every measured entry in the shipped catalogue was also measured in the
envelope, so every one is now judged by its enveloped grade:

| Entry | Free (history) | Enveloped (decides) |
|---|---|---|
| `qwen2.5-7b-instruct` | rarely, 8 of 20 | **sometimes, 71 of 80** |
| `qwen3-8b` | sometimes, 10 of 20 | sometimes, 10 of 20 |
| the other twelve measured | rarely | rarely |
| `eurollm-9b-instruct`, `gemma-2-9b-instruct`, `mixtral-8x7b-instruct` | not measured | not measured |

On a machine with no graphics card, `agent_for_cpu` still answers
`NoneClearsTheBar` — the sentence for a measurement that was made.

## The finding: the 80 of 80 does not reach a turn

ADR 0032 decision 5 asked whether a turn asks *in the envelope*, and it does. It
did not ask what a turn is *shown*, and ADR 0034 decision 4 did: a grade under
instructions a turn is not shown decides nothing on a shipped machine.

A turn is shown the agent's own words. `alo-agentd`'s `doing` passes the
question an agent sent with `"answered":"as-the-next-request"` to
`Asking::to_this_machine_in_the_envelope`, which holds the answer's shape and
adds nothing to the question. Neither of `alo-driving`'s sets of instructions
reaches the model. So the catalogue's grades — under either set — are a
measurement of instructions that stand in for whatever an agent writes.

That was already true of the 71 of 80, and this task reads it anyway, because it
is the grade the plan names, the one ADR 0032 decided and the method every
entry is compared by. What it means is that **the step between the 80 of 80 and
a local model given the agent is not in the recommendation.** It is a decision
about what an agent turn's model is shown, which lives where the turn's question
is written — the agent, in `alo-workplace`, or a door that puts instructions in
front of it — and neither is this lane's. Task 16 measured what that decision
would buy: every change sent through `propose`, eighty times in eighty.

What would close it, in the order it would have to happen: a decision that an
agent turn's model is shown an example request through each door it offers;
the turn showing it; and then a catalogue grade earned under the instructions
the turn shows, which this task's rule would read without a line changing.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima
VM as root, run by the Mac lane's publish script on the tree combined with
`main`. It publishes only when formatting, clippy with warnings denied, rustdoc,
the supervisor's three and both BPF gates pass and the workspace's tests fail on
exactly one test — `alo-bounding`'s
`ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`, the
aarch64 failure recorded in the first task's report. Before publishing, in the
VM: `alo-choosing`'s, `alo-saying`'s, `alo-telling`'s and `alo-driving`'s tests
pass; on the Mac, `alo-models`' tests pass and its clippy is clean.
