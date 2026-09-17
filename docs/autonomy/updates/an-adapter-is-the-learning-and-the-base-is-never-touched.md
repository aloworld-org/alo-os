# An adapter is the learning, and the base weights are never touched

**Date:** 2026-09-17
**Workstream:** v0.5 — models a person adapts, and the one they subscribe to
**Task:** a change of shape inside task 1, before task 2 builds on it
(`docs/autonomy/v0-5-models-a-person-adapts-and-subscribes-to-plan.md`)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written and tested on an **Apple M3 with 8 GB unified memory**,
macOS 26.5.2; `alo-saying`'s tests in the Lima VM. No model was run.
**Egress:** none.
**Status:** done. **[ADR 0047](../../decisions/0047-a-revoked-grant-stops-the-next-fine-tune-and-does-not-unlearn-the-last.md)
is superseded by [ADR 0048](../../decisions/0048-an-adapter-is-the-learning-and-the-base-weights-are-never-touched.md)**
the same day.

## What changed, and why it matters more than the feature

Yesterday's decision said a true thing about merged weights: once a fine-tune is
merged into a model, the documents are in there, revocation cannot reach them,
and the only remedy is deleting the whole model. It wrote that limitation down
carefully — **and accepted it without noticing it was a choice.**

The owner's correction: *do not create the limitation.*

- **The base model is never written to.** It stays byte-for-byte what it
  shipped as, and `TheBaseIsUntouched` records its digest before a fine-tune and
  refuses after if a byte moved.
- **Everything learned lives in an adapter**, a file beside the model, applied
  when the model answers.
- **An adapter is tied to the grant that produced it** — folder, grantee, when.
- **Deleting one adapter takes back what that folder taught, and leaves the
  rest.** Not a promise about behaviour: a property of never having mixed the
  bytes.

**One adapter per granted folder, composed at inference.** v0.5 ships one, and
`Composed` takes a list anyway, because the day a second folder is trained on
must not be the day this is argued again.

## The sentence a person reads

Before:

> This stops new training. A model that has already learned from these documents
> still has what it learned — delete the adapted model to be rid of it.

Now:

> **Delete this adapter and your model no longer has what these documents
> taught it. Everything else it learned stays.**

and, where a grant is revoked:

> **This stops new training on these documents. What they already taught your
> model is kept in an adapter you can delete.**

The first told a person what they could not have. The second tells them what
they can do, and both are literally true. The tests that forbade *forget* and
*unlearn* still stand; *cannot be undone* is now forbidden too, because in this
shape it is false.

## Why this is the part that is ours

Fine-tuning a model on somebody's own documents, on their own machine, is a
feature anyone can copy in a fortnight. **Learning a person can take back, one
source at a time, is not** — not because the technique is difficult, but because
it has to be decided before the first fine-tune runs. Every system that merges
has already chosen, and the bytes cannot be unmixed afterwards.

It is also the only honest reading of ADR 0001 §3, which says a grant is
revocable. A grant that permanently taught a model something nothing could
remove was revocable in name.

## What it costs, written down rather than discovered later

- **Inference has to compose.** The runtime must load the base model and apply
  adapters; whether the pinned runtime does that well is a finding for the task
  that measures an adapted model, not an assumption here.
- **Adapters multiply**, so a person must be able to see which is which — which
  is why each carries its folder and grant.
- **Quality may be lower than merging.** Composed adapters can interfere. If a
  measurement ever shows that costing a grade, it is written down as the price
  of revocability rather than quietly paid by merging.

## What is in the code now

| Where | What |
|---|---|
| `adapter.rs` — `Adapter` | the file, the base it applies to, the base's digest, the grant it came from, when |
| `adapter.rs` — `TheBaseIsUntouched` | the digest before, checked after; `TheBaseMoved` says where the learning belongs |
| `adapter.rs` — `Composed` | the adapters a model answers with, and `without_what_that_grant_taught` |
| `words.rs` | the three sentences above, with translator notes forbidding *forget* |
| ADR 0048 | the decision; ADR 0047 marked superseded, kept, not rewritten |

Four tests hold the shape: deleting one folder's adapter leaves another folder's
file on the disk and its learning in the model; an adapter says which grant made
it; a base that moved is refused with a message naming where learning belongs;
and a model with nothing composed is the model as it shipped.

## Next

Task 2 — a fine-tune run with a pinned, rented stack on the small model this
machine already holds — now has one more requirement to meet: **it must produce
an adapter without writing to the base.** A stack that can only merge is a
finding, not a patch (ADR 0011), and this is the first thing that will be
checked of it.
