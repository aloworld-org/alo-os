# ADR 0047 — A revoked grant stops the next fine-tune and does not unlearn the last

**Status:** **superseded** by
[ADR 0048](0048-an-adapter-is-the-learning-and-the-base-weights-are-never-touched.md),
2026-09-17, the same day. This decision answered *how do we tell a person that
merged weights cannot be untrained*, and accepted merging without noticing it was
a choice. ADR 0048 does not create the limitation: the base weights are never
written to, what is learned lives in an adapter tied to the grant that produced
it, and deleting that adapter takes back what that grant taught. Read it instead;
this is kept because the reasoning it contains about what weights hold is still
true, and because the loop does not rewrite what it decided.
**Date:** 2026-09-17
**Context:** [ADR 0001](0001-the-capability-model.md) §3 (grants are visible,
revocable and expiring), [ADR 0014](0014-alos-own-model-is-a-provider-like-any-other.md)
(alo's own service is a provider like any other),
[ADR 0038](0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md),
`crates/alo-adapting`, `crates/alo-picking`, and task 1 of
`docs/autonomy/v0-5-models-a-person-adapts-and-subscribes-to-plan.md`

## The question in one line

**A person revokes the folder grant their model was fine-tuned on. What has
actually changed, and what does the machine tell them?**

## What is true, technically

A fine-tune changes the weights. The sentences it trained on can be drawn back
out of them: not always, not word for word, but often enough that the weights
must be treated as containing the documents. That is what makes a locally
adapted model worth having — it knows the person's own material — and it is what
makes this question sharp.

So, after a fine-tune:

- **the dataset** is gone when the working folder is removed (task 1);
- **the grant** governs reading the folder again;
- **the adapted model** contains what it learned, and revoking a grant does not
  reach inside it.

Every other revocation in this system takes effect immediately and completely:
ADR 0001 §3 says a revoked grant stops the next call, and `alo-capability` makes
that true within the same turn. A person who has learned that revocation *works*
will reasonably expect it to work here too.

## The decision

1. **Revoking stops the next fine-tune. It does not unlearn the last one.** The
   grant is what a fine-tune reads a folder under; without it, no further
   training happens and nothing further is read. What a model already learned
   stays in the weights.

2. **The machine says so, in one plain sentence, where the revocation happens**
   — not in a manual, not afterwards. The sentence a person reads is:

   > **This stops new training. A model that has already learned from these
   > documents still has what it learned — delete the adapted model to be rid of
   > it.**

   That is the whole of it: what stops, what does not, and the one thing they
   can do.

3. **Deleting the adapted model is offered in the same place.** A person told
   that something cannot be undone must be given the action that *can* be taken,
   at the moment they are told. Deleting the adapted model removes the weights
   the fine-tune produced; the machine then answers with the model it had
   before.

4. **No sentence in this system may imply forgetting.** Not *the model will
   forget*, not *this removes your documents from the model*, not *unlearn*. If
   a translator's note is needed to keep that true in another language, it is
   written. A false sentence here is worse than a blunt one, because a person
   would rely on it: they would revoke, believe the documents were gone from the
   model, and then send that model to somebody.

5. **What it learned from is written down**, so the question *what has my model
   learned from?* is answerable a year later:
   `alo_adapting::LearnedFrom` — the folders, the grant, when, how many files,
   how many were left out — carried into the record when the fine-tune runs.

## What it costs

- **A person may be left with a model they cannot separate from documents they
  have withdrawn.** Deleting the adapted model is the only complete answer, and
  it costs them the adaptation they paid time for. We say that rather than
  soften it.
- **A model adapted on a tenant's records inherits this**, so an organisation
  that withdraws a folder has the same two options and the same sentence. That
  is a real limit on what *revocable* means once training has happened, and it
  belongs in whatever the organisation is told about fine-tuning, not only here.

## Rejected

- **Saying the model forgets.** It does not. Machine unlearning is a research
  field, not a feature, and the approximations that exist do not give the
  guarantee the word implies. Shipping the word would be the machine lying
  about the one thing the person is asking.
- **Refusing to fine-tune unless the grant is permanent.** That protects the
  sentence by taking away the feature, and it makes grants less revocable
  everywhere else in order to make one of them honest.
- **Deleting the adapted model automatically when a grant is revoked.** It reads
  as tidy and it destroys something the person may want: the adaptation is theirs,
  they may have revoked the folder for an unrelated reason, and a machine that
  silently deletes what somebody built is a machine nobody trusts with anything
  else. It is offered, at the moment of revocation, and they choose.
