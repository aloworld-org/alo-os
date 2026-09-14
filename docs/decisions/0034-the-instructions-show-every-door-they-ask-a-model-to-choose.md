# ADR 0034 — The instructions show every door they ask a model to choose

**Status:** accepted
**Date:** 2026-09-14
**Context:** [ADR 0007](0007-the-cpu-is-the-default.md) (the catalogue says whether
a model can drive the verbs, measured by us),
[ADR 0032](0032-a-local-model-is-held-to-the-envelope-not-the-call.md) (a local
model is held to the envelope, not the call), tasks 12 and 16 of
`docs/autonomy/v0-5-the-models-measured-plan.md`, `crates/alo-driving`

## The question in one line

**The instructions every model is measured under show one example request, and it
goes through the read door; should they show a model every door it is asked to
choose between?**

## What was true before this decision

`alo_driving::HOW_TO_ANSWER` tells a model how to answer, in one example line and
a paragraph:

```text
{"format":1,"asks":{"read":{"verb":"NAME","given":[{"named":"ARGUMENT","is":VALUE}]}}}

Use "read" for a verb that only answers a question, and "propose" for a verb
that changes something; each verb below says which it is.
```

Every grade in the catalogue was earned under that text. Then the failures were
read:

- **Task 12:** of Qwen 2.5 7B's five failures in forty, three were a correct
  request through the wrong door, every one of them a change sent as a *read*.
- **Task 13:** the same weights at five bits sent 22 changes of 28 through
  `read`.
- **Task 14:** at eighty attempts, five of nine failures were `rename` sent as a
  read.
- **Task 10:** the small models' wrong doors were `read` too.

The sentence says which door; the only line a model can copy says `read`.

A bar that moves to meet a candidate is not a bar. Neither is an instruction that
shows one door and then counts a model wrong for taking it.

## The options

**1. The instructions as they are.**
- *Measures:* whether a model follows a sentence over an example — a real skill.
- *Hides:* how much of every grade is the example's pull toward `read` rather
  than the model's grasp of what changes something.
- *Costs:* nothing, and every existing grade stays comparable with every future
  one.

**2. One example per door.** The same line twice, once through `read` and once
through `propose`, each labelled with the kind of verb it is for.
- *Measures:* whether a model maps a verb's effect to its door when both doors
  are in front of it equally. That is the skill the protocol needs, since
  `alo_capability::Authorised::read` refuses a change through the read door.
- *Hides:* nothing the first option measured that a turn relies on; a model that
  still takes the wrong door now takes it with both shown.
- *Costs:* two more lines of text, the same for every model, and a new grade
  that is not comparable with the old ones — which is why it is recorded beside
  them rather than over them.

**3. One example whose door is a placeholder** — `"DOOR"` — naming neither.
- *Measures:* whether a model fills a placeholder with the right key.
- *Hides:* the thing it means to measure. On 2026-09-04 SmolLM2 copied the
  prompt's own placeholders into its answers (`"verb":"NAME"`), so a door left
  as a placeholder measures placeholder-copying in exactly the models least able
  to choose a door.
- *Costs:* the same as option 2, for a noisier measurement.

## The decision

1. **The instructions show one example per door.** `alo-driving` keeps the text
   every existing grade was earned under, unchanged and still named, and adds
   the second as another set of instructions a measurement can be run under. The
   exercises, the verbs they name, the scoring and the bar do not change.
2. **Every grade names the instructions it was earned under**, by the SHA-256 of
   their text, so two grades under different instructions can never be read as
   one measurement. A grade earned under the old instructions is not rewritten.
3. **The new grade sits beside the old, never over it.** For an entry measured
   both ways, the catalogue carries both — the new one in `[[model.also_under]]`
   — and a reader tells them apart by the instructions' digest.
4. **The recommendation does not change here.** Which instructions an agent turn
   is shown is the turn's — lane A's — and until a turn is shown these, no grade
   earned under them decides anything on a shipped machine. ADR 0032's decision
   5 already keeps the recommendation reading the grade for the way turns ask.

## What it costs

- **Grades split in two.** From here a model can carry a grade under each set of
  instructions, and the only honest comparison between models is between grades
  under the same set. The digest beside every new grade is what makes that
  visible rather than a convention.
- **The prompt a turn is shown is now a measured choice.** If a turn is shown
  different instructions from the ones its model was graded under, the grade
  says nothing about that turn — the same kind of mismatch ADR 0032 closed for
  the envelope. Whoever writes the turn's prompt writes it from this decision.

## What would change this decision

A measurement under option 2 in which a model takes `propose` for reads as often
as it took `read` for changes under option 1 — the example pulling the other way
— would show that an example's door leans a model whichever door it names, and
would reopen option 3 with a remedy for placeholder-copying beside it.
