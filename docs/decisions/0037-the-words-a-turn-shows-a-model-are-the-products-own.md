# ADR 0037 — The words a turn shows a model are the product's own

**Status:** accepted
**Date:** 2026-09-14
**Context:** [ADR 0032](0032-a-local-model-is-held-to-the-envelope-not-the-call.md)
(a local model is held to the envelope, not the call),
[ADR 0034](0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md)
(the instructions show every door they ask a model to choose), task 18 of
`docs/autonomy/v0-5-the-models-measured-plan.md`, `crates/alo-driving`,
`crates/alo-turn`

## The question in one line

**A grade says how a model answers the words it was shown; who writes the words a
shipped machine shows it?**

## What was true before this decision

Nobody. `alo_turn::Turning::asking_for_the_next_request` takes what a model is
shown as a `&str` — its own rustdoc says *what the agent composed for the model —
its instructions, the verbs and what the person said* — and no crate in this
repository composes one. The only such text that exists is
`alo_driving::prompt_under`, inside the measurement harness, behind the
catalogue, an HTTP client and a settings store.

So the catalogue's grades were measurements of words the product does not ship.
ADR 0034 named the risk in its costs — *if a turn is shown different
instructions from the ones its model was graded under, the grade says nothing
about that turn* — and left which instructions a turn shows to whoever writes the
turn's prompt. Reading the code, there is no such person and no such prompt: the
words arrive from outside the operating system, through the daemon's door, as a
question.

That is a gap of the kind law 2 exists for, read one level up. The capability
model makes it safe for a model to be wrong; it does not make a grade mean
anything about a machine whose words nobody controls. And task 16 had just
measured how much the words matter: the same weights, the same runtime, the same
exercises, 80 of 80 under one set of instructions and 71 of 80 under another.

## The options

**1. Leave the words in `alo-driving` and let the turn depend on it.**
- *Costs:* a privileged daemon gains the measurement harness, and behind it
  `alo-models`' catalogue, `ureq`, a settings store and a runtime client. The
  crate header of `alo-driving` promises it asks nothing and is not run on a
  machine; this would make the second half false.
- *Gains:* one text, no new crate.

**2. Leave composing to whoever calls the daemon** — today's state, written down
as a decision.
- *Costs:* every client invents its own prompt; two clients ask two ways; no
  grade in the catalogue is about any of them. `can_be_the_agent` becomes a
  claim about a harness rather than about the machine.
- *Gains:* nothing this repository can point at.

**3. The words are a crate of their own**, depended on by the measurement and,
when it is wired, by the turn.
- *Costs:* a crate, and a regrade — the catalogue's entries were measured under
  `AsFirstWritten` and the set a turn shows is the other one.
- *Gains:* the measurement and the machine are shown one text built by one
  function, which is the only arrangement in which a grade is a statement about
  a turn.

## The decision

1. **The words a model is shown are `alo-instructing`'s**, a crate that carries
   the verb registry and SHA-256 and nothing else —
   `tests/nothing_behind_these_words.rs` holds that to the manifest. It offers
   `shown_to_a_model(instructions, verbs, request)`: how to answer, every verb in
   the sentence the verb itself declared, and the request last.
   `alo_driving::prompt_under` is that function with an exercise's request, so a
   model being measured and a model being asked for a turn's next request cannot
   be shown two texts.
2. **The set a turn shows is `Instructions::OneExamplePerDoor`**, named once as
   `Instructions::SHOWN_TO_A_TURN` and taken through `shown_to_a_turn`. It is the
   measured choice rather than a taste: on an Apple M3 with 8 GB, in the
   envelope, `qwen2.5-7b-instruct` drove the verbs **80 of 80** under these and
   **71 of 80** under `AsFirstWritten` through the pinned runtime, and **80 of
   80** against **65 of 80** through `llama.cpp`'s own server (tasks 16 and 17).
   ADR 0034 left this to the turn; the turn had no words at all, and the lane
   that made the measurement is the one that owns naming which text they are.
3. **No grade moves until a turn is composed from here.** `alo-agentd` and
   `alo-turn` are another lane's, and until they compose what they show a model
   from `alo-instructing`, a turn is still shown whatever its caller wrote —
   so `Model::grade_for_the_turn` keeps reading the grade earned under
   `AsFirstWritten`, exactly as ADR 0034 decision 4 says. The wiring is written
   as a task on `docs/autonomy/v0-5-the-local-network-plan.md`.
4. **When that lands, the grade that decides is the one earned under
   `SHOWN_TO_A_TURN`'s digest**, and an entry never measured under it has no
   grade for the turn — not a grade assumed for it. That is a regrade of every
   entry this measuring machine can hold, and it is written as a task on the
   measuring lane's own plan rather than left as an implication.

## What it costs

- **A third regrade.** The catalogue carries grades under two sets of
  instructions already; naming one as the turn's makes the others history for
  the purpose of deciding, though they stay in the entry and stay readable.
  Decision 4 is the bill.
- **The English limit becomes the product's.** A measurement asked in English
  was a fact about a measurement; words the machine shows a model are a fact
  about the machine, and a person whose machine runs in Latvian is served by an
  agent asked in English. `docs/quirks.md` records it under *a turn asks a model
  in English*; the person's own words and every sentence they read back are
  untouched by this crate.
- **The prompt is now a public surface in practice.** Changing the text changes
  the digest, and a changed digest is a catalogue whose grades are about text
  that no longer exists — which is why `Instructions` adds sets and never edits
  one, and why the first set's digest is pinned in a file a test reads.

## What would change this decision

A measurement on more than one model showing that `AsFirstWritten` grades a
model's grasp of the doors better than `OneExamplePerDoor` does — the second set
pulling a model toward `propose` for reads as hard as the first pulled it toward
`read` for changes — would move decision 2 back, without touching decisions 1, 3
or 4. And a turn that must ask in a person's own language would need the verbs'
sentences translated and a grade per language; that is a new set of instructions
and a new column of grades, not an edit to these.

## Since it was accepted

**2026-09-15, the same day.** Decision 3 said no grade moves until a turn is
composed from `alo-instructing`, and named the wiring as a task for the lane that
owns `alo-turn`. That lane landed it: `Turning::asking_for_the_next_request` now
builds what it shows a model with `alo_instructing::shown_to_a_turn`, and its
tests read the request off the socket and find these words.

So decision 4 is in force. `Model::grade_for_the_turn` reads the grade earned in
the envelope, under `Instructions::SHOWN_TO_A_TURN`, through the pinned runtime,
and an entry measured only another way has **no grade for the turn** — its
`can_be_the_agent` is false whatever else it earned. Every other grade stays in
the entry and is read by nobody.

Two consequences worth writing down, because both are what the decision was for:

- **Two entries now clear the bar and may be given the agent**:
  `qwen2.5-7b-instruct` at 80 of 80 (task 16) and `qwen3-8b` at 20 of 20 (task
  20). A machine with 16 GB is given the second, by the ordering
  `Catalogue::agent_for_cpu` already had. Before this, no machine was given any
  model, and the sentence a person read was that nothing measured here clears
  the bar.
- **Nothing lost a grade it appeared to have.** That is task 20's doing rather
  than this decision's: every entry an 8 GB machine can hold had already been
  measured under these words, so the rule could be applied without twelve entries
  falling silently to *not measured*. A decision of this shape taken before that
  measurement would have been a decision to blank the catalogue.
