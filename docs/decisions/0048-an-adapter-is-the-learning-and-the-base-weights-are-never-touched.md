# ADR 0048 — An adapter is the learning, the base weights are never touched, and deleting an adapter takes back what one grant taught

**Status:** accepted
**Date:** 2026-09-17
**Supersedes:** [ADR 0047](0047-a-revoked-grant-stops-the-next-fine-tune-and-does-not-unlearn-the-last.md),
which said the true thing about merged weights and accepted the wrong shape.
**Context:** [ADR 0001](0001-the-capability-model.md) §3 (grants are visible,
revocable and expiring), [ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md)
(engines are rented, configured, never patched),
[ADR 0014](0014-our-own-hosted-model-is-a-provider-like-any-other.md),
`crates/alo-adapting`, and the *models a person adapts* plan

## The question in one line

**Where does what a fine-tune learned actually live — and can a person take
back what one folder taught, without losing everything else their model
learned?**

## What ADR 0047 got wrong

ADR 0047 answered the question as posed: weights that have been trained on
somebody's documents contain them, revocation cannot reach inside them, so tell
the person plainly and offer to delete the model. Every sentence of that is true
**of a model whose adapter has been merged into its base weights.**

It accepted that shape without noticing it was a choice. Once the learning is
merged, the only thing a person can delete is the whole model, and every folder
they ever trained on is in there together. *This cannot be undone* becomes true
at the moment of merging, and it is true afterwards for ever.

So the decision is not how to word a limitation. It is not to create it.

## The decision

1. **The base weights are never written to.** What ships as the model stays
   byte-for-byte what it was: `alo_adapting::TheBaseIsUntouched` records its
   digest before a fine-tune and checks it after, and a test fails if a single
   byte moved. A fine-tuning stack that can only merge is a stack we do not use
   (ADR 0011: rented, configured, never patched — *and* never used in the one
   mode that forecloses this).

2. **Everything learned lives in an adapter**, a separate file beside the base
   model, applied when the model answers and not before.

3. **An adapter is tied to the grant that produced it.** It carries the folder
   it was trained on, the grantee the grant was made to, and when — so the
   question *what taught my model this?* has a file-level answer.

4. **Deleting an adapter removes what that grant taught, and nothing else.**
   That is the sentence a person reads, and it is true because of decisions 1 to
   3 rather than in spite of them:

   > **Delete this adapter and your model no longer has what these documents
   > taught it. Everything else it learned stays.**

5. **One adapter per granted folder, composed when the model answers.** v0.5
   ships a single adapter and needs no more; the shape is fixed now because the
   cost of getting it wrong is not paid now — it is paid the first time somebody
   wants one folder back out of three and the answer is *all or nothing*.

6. **Revoking a grant stops new training and offers to delete that folder's
   adapter**, in the same place. What the person is told is what they can do,
   not what they cannot.

## Why this is the part worth building

Fine-tuning a model on somebody's own documents, locally, is a feature anybody
can copy in a fortnight. **Learning a person can take back, one source at a
time, is not** — not because the technique is hard, but because it has to be
decided before the first fine-tune runs. Every system that merges has already
chosen, and it cannot be unchosen afterwards: the bytes are mixed.

This is also the only honest version of what ADR 0001 §3 already promises. A
grant is *revocable*; a grant that permanently taught something nothing can
remove was revocable in name.

## What it costs

- **Inference composes.** The runtime loads the base model and applies the
  adapters, which costs memory and a little speed, and the pinned runtime must
  support it — a finding to be settled in the task that measures an adapted
  model, not assumed here.
- **Adapters multiply.** One per folder means several files to keep, show and
  delete, and a person must be able to see which is which — hence decision 3.
- **Quality may be lower than merging.** Composed adapters can interfere where
  merged training would not. If a measurement ever shows that costing a grade,
  it is written down as the price of revocability rather than quietly paid by
  merging.

## Rejected

- **Merging, with a clear warning** (ADR 0047's shape). The warning is honest
  and the person still cannot act on it. A limitation that can be designed away
  is not something to word well.
- **One adapter for everything.** Simpler to ship and it makes revocation
  all-or-nothing: the first person who wants one folder back loses the rest.
- **Saying a model "forgets" or is "untrained"** in any form. Nothing here
  forgets. A file that held what one grant taught is deleted, and the model goes
  back to what it was without it — which is a stronger claim, and a true one.
