# ADR 0133 — A question is answered inside the envelope

**Status:** **accepted, 2026-10-27, by the owner**, who chose option 2 below
knowing its cost.

**Amends:** [ADR 0032](0032-a-local-model-is-held-to-the-envelope-not-the-call.md)
(its cost *"a model held to the envelope cannot answer in prose"*) and
[ADR 0037](0037-the-words-a-turn-shows-a-model-are-the-products-own.md)
decision 2 (which set of words a turn shows).

**Context:** [ADR 0107](0107-a-provider-is-held-to-the-shape-its-api-accepts.md),
`crates/alo-agent/src/driving.rs`, `crates/alo-instructing/src/instructions.rs`,
`crates/alo-models/src/in_the_envelope.rs`, `docs/walks/the-first-walk.md`.

## What a person saw

On booted machines on 2026-10-25 and 2026-10-27, a person granted a folder and
asked alo to *read* something: *list my documents folder*, *what does the
record say was done to this file*. Each time they were asked to approve an
install of Krita, GIMP or Inkscape. They said no, nothing was installed, and
nobody got an answer.

## Why

Two decisions were each right when they were taken, and together they left a
question with no way to end.

- **ADR 0032** holds a turn's reply to a shape: the pinned runtime's grammar
  for a local model and, since ADR 0107, a provider's strict schema for a
  hosted one. The shape had `read` and `propose` in it and nothing else. The
  ADR wrote down the cost in advance: *"A model held to the envelope cannot
  answer in prose"*. It accepted it because an agent's reply was always a
  request.
- **The agent's loop**, written later, ends a turn with an answer only when the
  reply is **not** JSON.

So a model that had read what it needed could only read again until the turn
ran out, or propose something.

## What was measured, 2026-10-27

`crates/alo-agentd/tests/a_question_is_answered_in_words.rs` follows a whole
turn through the agent's own loop, with the agent's own twelve verbs and the
granted folder named as `alo-agent` names it, against `gpt-4o` at OpenAI.

| what a turn showed | requests to read answered |
|---|---|
| `one-example-per-door`, as it was | 0 of 6: three ran out of rounds; three proposed a rename or an archive |
| the same, plus a paragraph saying *answer in plain words, as a line that is not JSON* | 0 of 6: all ran out of rounds |
| the same, with the last line of a continued turn saying so too | 0 of 9: two proposed a rename, seven ran out of rounds |

**No wording could have worked.** The provider's strict schema will not let a
token outside the shape be written. A prompt printed on the second round showed
that the model had been given the folder's listing; it had no way to answer
with it.

### And under the words this decision chose

The same test and the fixed ten (`crates/alo-driving/tests/against_a_model_at_a_provider.rs`),
`gpt-4o` at OpenAI, three rounds each, on 2026-10-27:

| what a turn showed | requests to read answered | fixed ten drove |
|---|---|---|
| `one-example-per-door`, as it was | 0 of 9 | 30 of 30 |
| `every-door-and-an-answer`, final wording (digest `cf522fc5…`) | **9 of 9** | **30 of 30** |

**One weakness the count hides.** *"what does the note in Documents say"* was
answered in three rounds out of three, with *"there is nothing at
…/Documents/note.txt"*. The file is `notes.txt`. The model guessed a name
instead of listing the folder first. That is truthful and unhelpful, and it is
the looking-first question `SeveralSteps` was written for, not this one.

Both runs were made once, by hand. They ask a paid provider, so both tests are
`#[ignore]`d and CI does not run them.

## The options put to the owner

1. **An answer door, from the second round on.** No local model's grade is
   touched, because every grade is earned on a first round. A question that
   needs no look cannot be answered in one step.
2. **An answer door from the first round.** Every question can be answered. The
   words a turn shows change, so **every local model's grade for a turn is set
   aside until it is earned again** under them, and a machine that answers with
   a local model is not given the agent in the meantime. A hosted provider is
   unaffected.
3. **No shape at all.** Answers come back. So do the shape errors that ADR 0107
   measured as the reason the shape exists.

## Decision

**Option 2.**

1. **The envelope gains an answer door**, beside `read` and `propose` inside
   `asks`:
   ```
   {"format":1,"asks":{"answer":{"text":"…"}}}
   ```
   Both renderings admit it: the local one as *any object*, and the provider's
   as an object with one string, `text`.
2. **It is not a line of the protocol.** An answer is said to the person by
   the agent; the machine is never asked anything by it. `alo-instructing`
   reads it (`the_answer_in`), and the loop turns it into the turn's answer.
   `alo-protocol` refuses it, and a test holds that refusal, so it cannot reach
   the daemon. Every client written against the daemon's protocol is
   unchanged.
3. **The words a turn shows are `every-door-and-an-answer`**: today's
   `one-example-per-door` and one paragraph, held to exactly that by a test.
   `alo_models::THE_WORDS_A_TURN_SHOWS` is its digest.
4. **Every local model's turn grade is set aside, not deleted.** The grades
   stay in the catalogue, naming the text they were earned under, and
   `one-example-per-door`'s digest is pinned by a test so they can be re-run.
   Each is earned again under the new words before a machine is given that
   model as its agent.
5. **The last line of a continued turn names the three ways on**: a read, a
   proposal, or an answer.
6. **An answer is only for a request that asks to be told something, and it
   never says something was done.** The first wording, *"when what you have
   been shown already answers the request"*, failed the fixed ten's `focus`
   exercise three rounds in three. That request reads *Anna **already** has
   org.alo.Writer running … and wants it in front*, and `gpt-4o` answered
   *"org.alo.Writer is already running; focusing it now"*. It did nothing and
   said it was doing it, which is worse for a person than no answer. The
   paragraph now says a request to have anything done is proposed, and that an
   answer does nothing, so it never says anything was done.

## What it costs

- **A local model gives no agent until it is graded again.** A machine with a
  hosted provider is unaffected. A machine that answers on itself has no agent
  until then, which is the cost the owner chose over a turn that cannot
  answer. `crates/alo-driving/tests/every_grade_names_instructions_this_crate_has.rs`
  used to require that at least one shipped entry be decided by the words a
  turn shows. It now requires that **none** is, and says to turn the check back
  the day one is graded again. `crates/alo-models/src/graded_for_turns.rs`
  holds that all thirteen grades set aside are still in the catalogue, under
  words this repository can still show a model.
- **A model can now end a turn by answering instead of acting.** The answer
  changes nothing on the machine, so the person loses nothing by it; at worst
  they ask again. The fixed ten under the new words is recorded in the change
  that lands this.
