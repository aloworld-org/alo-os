# ADR 0032 — A local model is held to the envelope, not the call

**Status:** accepted
**Date:** 2026-09-14
**Context:** [ADR 0007](0007-the-cpu-is-the-default.md) (the catalogue says whether
a model can drive the verbs, measured by us), [ADR 0006](0006-the-pinned-model-runtime.md)
(the model runtime is a pinned engine, configured and never patched),
[ADR 0001](0001-the-capability-model.md) (every capability is an enumerated verb
with validated arguments), task 7 of
`docs/autonomy/v0-5-the-models-measured-plan.md`, `crates/alo-models`,
`crates/alo-driving`, `crates/alo-protocol`, `crates/alo-asking`, `crates/alo-turn`

## The question in one line

**When an agent turn asks a model on this machine for its next request, does it
ask the pinned runtime to hold the answer to the protocol's shape — and if so,
to how much of it?**

## What was true before this decision

Ten catalogue entries were measured driving the verbs, and every one of them
graded `rarely`. The 7B-class models, measured on 2026-09-13 on a machine with
room for them, did not fail at reasoning. They failed at the **grammar** of the
call: every read reached the right door, and nearly every change named its verb
where the door belongs, or wrote its arguments as a plain object. No local model
on the catalogue could be given the agent, so every machine alo OS ships would
arrive with no local agent at all.

The pinned runtime, Ollama 0.34.0, can hold a model's answer to a JSON schema
through `/api/chat`'s `format` field, and the protocol already has a shape every
agent request must match. Nothing here asked for it.

## What was measured

The same ten exercises, the same prompt and the same scoring
(`alo_driving::Exercises::attempt`), on an Apple M3 with 8 GB under Ollama
0.34.0; only `format` differed. The first two schema columns were a scratch
probe admitting the two doors the exercises use; the last column is the
measurement harness itself, with the envelope this decision adopts — all three
doors an agent has.

| Model | Asked freely (the catalogue's grades) | Whole call's schema (probe) | Envelope, two doors (probe) | **Envelope, three doors (harness)** |
|---|---|---|---|---|
| Qwen 2.5 7B, Q4_K_M | 8 of 20, `rarely` | 3 of 20, `rarely` | 19 of 20 | **35 of 40 — 87.5%, `sometimes`** (and 17 of 20 in a first run) |
| Llama 3.1 8B, Q4_K_M | 1 of 10, `rarely` | 2 of 20, `rarely` | 5 of 20 | **5 of 20, `rarely`** |
| Mistral 7B v0.3, Q4_K_M | 0 of 10, `rarely` | 0 of 20, `rarely` | 0 of 20 | **0 of 20, `rarely`** |

**No model clears the bar in the envelope either, and the probe's 19 of 20 is
not the grade.** It admitted two doors, it was twenty samples, and the runtime
samples every answer; forty attempts with the envelope as adopted land at 87.5%,
two and a half points under the line that gives the agent. The decision rests on
what is true of all three runs: the envelope takes Qwen 2.5 7B from 40% to
between 85% and 95%, and it harms no model.

**The whole call's schema made things worse, and the reason is the runtime's.**
Every answer held to it came back with its object keys in alphabetical order —
`asks` before `format`, `given` before `verb`, and inside each argument **`is`
before `named`**. The runtime turns the schema into a grammar with the
properties sorted, so the model is made to write an argument's value before it
has written which argument it is. Qwen then put the value where the name belongs
in fifteen of twenty answers (`{"is":"/home/anna/Invoices/march.pdf","named":"read_file"}`).
The envelope was always right and the call was nearly always wrong.

**Holding only the envelope and the door fixed exactly the failure that was
measured.** With `format` naming `{"format":1,"asks":{<one door>:{…}}}` and
leaving the object inside the door to the model, Qwen 2.5 7B wrote the call in
the order the prompt teaches. Of its five failures in forty, three asked through
the wrong door and two wrote an argument out of shape inside a right one — the
errors that are left are the model's, no longer the envelope's. Llama 3.1 8B improved and
stayed `rarely`: inside a correct envelope it still mixed argument shapes.
Mistral 7B did not move: inside a correct envelope it still writes `given` as a
plain object of name to value, which no envelope constraint reaches.

## The decision

1. **An agent turn asks a model on this machine for the envelope and the door,
   and never for the call.** The schema names the protocol's version and the
   three doors an agent has — `read`, `propose` and `ask` — as exactly one key,
   and says nothing about what is inside the door. Everything inside is still
   read by `alo-protocol` and validated by `alo-capability`, exactly as it is
   now: the schema removes a way for a model to fail to be understood, and it
   never lets anything through that was not let through before.
2. **The whole call's schema is not used**, and the reason is written down so it
   is not tried again by default: on the pinned runtime its keys are ordered
   alphabetically, and the protocol's argument is `named` then `is`. If a later
   runtime keeps the schema's order, that is a new measurement and may be a new
   decision; it is not this one.
3. **Only questions that must answer in the protocol are held to it.** A
   question a person puts to a model — *may the tenant sublet?* — is answered in
   prose and is never given a schema. The schema belongs to the road an agent
   turn takes and to the measurement of that road, and to nothing else.
4. **Only the pinned runtime.** A hosted provider or a paired machine answers
   through other doors with other capabilities; whether they are held to a
   shape is a separate question, measured separately.
5. **A grade says how the model was asked.** A grade earned with the envelope
   held is a different measurement from one earned asking freely, and one is
   never written over the other. The catalogue carries both, each with its
   machine, date, runtime and counts; the recommendation reads the grade for the
   way turns actually ask, and until the agent turn asks this way it keeps
   reading the free one.

## What it costs

- **A model held to the envelope cannot answer in prose.** An agent that wanted
  to say *I cannot do that* in words cannot, on this road. That is already the
  protocol's rule — an agent's line is a request or it is not understood — so
  the cost is a failure mode removed rather than a freedom taken away; but it is
  written here so nobody discovers it as a surprise.
- **The runtime's grammar is now part of the measurement.** A runtime upgrade can
  move a grade by changing how it builds the grammar, as 0.34.0's key ordering
  shows. `THE_PINNED_RUNTIME` already makes an upgrade deliberate; a grade
  earned this way names the runtime beside it, as every grade does.
- **The door wiring is not the measuring lane's.** `alo-models` builds the
  request and `alo-driving` measures it; the agent turn reaches the runtime
  through `alo-asking`'s local door and `alo-turn`, which belong to lane A.
  Until they ask this way, no machine gives a local model the agent on the
  strength of an envelope grade — decision 5 is what keeps that true.

## What would change this decision

A measurement showing the envelope-held road lets through a request the free
road refused, or refuses one a person needed — which the design rules out, and
which a test must keep ruling out. Or a pinned runtime that honours a schema's
property order, which reopens decision 2 and nothing else.
