# ADR 0095 — The release carries no model, and a person brings their own

**Status:** **ACCEPTED, 2026-10-06**, by the owner, stated directly: *"the
release should not have models, the users will download their own models or use
our apis or work without models"*, and clarified in the same conversation:
*"users can download their own weights on the machine, use third party apis or
use our apis (future) or use without apis or model"*, and *"alo os when the
users install them with our apis that will work out of the box"*.
**Date:** 2026-10-06
**Proposed by:** the owner
**Supersedes:** the *arrives ready to run* half of
[ADR 0025](0025-the-default-is-what-a-machine-arrives-able-to-do.md), and with it
the `docs/features.md` promise **The local model is what the machine arrives
ready to run**. ADR 0025's other half — *nobody chooses for the person* — is
untouched and is in fact what makes this decision safe to take.
**Context:** [ADR 0007](0007-the-cpu-is-the-default.md) (the CPU),
[ADR 0008](0008-where-inference-happens.md) (where inference happens),
[ADR 0009](0009-a-good-computer-without-the-agent.md) (the fourth answer, and no
nagging), [ADR 0014](0014-alos-own-model-is-a-provider-like-any-other.md) (our
own model is a provider like any other), `docs/features.md`,
`image/Containerfile`, `crates/alo-image`, `crates/alo-models`

## The question in one line

**Does the image a person downloads carry a model's weights?** Until today it
did: 4.87 GiB of them, pinned by content, in a layer of their own.

## What the old decision said, and why it was right

ADR 0025 settled that *the local model is what the machine arrives ready to
run*, and `image/Containerfile` states the reasoning plainly:

> they ride on the image rather than being fetched when somebody sets the
> machine up: a machine that fetches at setup has not arrived ready when it is
> offline at setup, and *the local model is what the machine arrives ready to
> run* is a promise about the box rather than about the network it is unpacked
> next to.

That argument is sound and is not being called wrong. A machine unpacked without
a network, carrying its own model, is a real thing to be able to offer, and it is
what *sovereignty is the default configuration* meant.

## What changed

**The owner decided that this is not what a person should be handed.** Three
things follow from the decision rather than arguing for it, and they are recorded
because they are what makes it a reasonable decision rather than a cheaper one:

- **A model chosen by us is a model chosen for them.** ADR 0025's surviving half
  says nobody chooses for the person. Shipping one model in the image is the one
  place where we did — not as a setting they could change, but as 4.87 GiB they
  received whether it suited them or not.
- **It is the wrong model for most machines.** The weights in the image were the
  catalogue's answer for the one 16 GB laptop `docs/hardware.md` certifies first.
  Every other machine got a model sized for somebody else's.
- **It is paid for by everybody, including those who will not use it.** A person
  who intends to use a provider, or no model at all, downloaded five gigabytes to
  delete.

## The decision

**The release carries no weights.** `image/Containerfile` stops fetching and
copying them. **The runtime stays**, because without it the first road below is
impossible.

Four roads, and alo OS chooses none of them:

| road | when | what it is |
|---|---|---|
| **weights the person already has** | now | `alo_models::Weights::at` — a model this catalogue never listed, run here. It needs no other machine, no account and no network |
| **a third party's API** | now | ADR 0008's other places, unchanged |
| **our own API** | **future** | ADR 0014 already says our model is a provider like any other, so this adds no new kind of thing |
| **no model at all** | now | `alo_models::NoAgentHere`, which says so and names the alternatives |

**Nothing moves between them on its own**, in either direction, exactly as
before. The machinery for all of this already exists and is tested: this decision
moves what the image carries, not how a machine behaves once it has or has not
got a model.

## What *out of the box* means now, and the honest version of it

**Today it means less than it did, and the file will say so.** A v0.01 machine
answers nothing until the person brings weights or an account. That is a real
reduction and it is not dressed up: *sovereignty is what a machine can do out of
the box* was true of a machine carrying weights and is not true of this one.
What stays true is the stronger half — **the machine lets you run weights you
own, and the catalogue recommends without gating.**

**Later it comes back, through our API rather than through the image.** The owner:
*alo OS when the users install them with our apis that will work out of the
box.* That is the future road, and it has a consequence worth writing down
before anybody is surprised by it:

> **Answering out of the box will then mean inference leaving the machine.**

Law 1 is not weakened by that — it is exactly the case Law 1 was written for.
Nothing leaves silently: the egress is visible at the moment it happens and
afterwards in a record, and the indicator is a feature rather than a diagnostic.
But the headline claim has to move with it. *With a local model a working day
produces zero inference egress, measured at the network boundary* now belongs to
the **bring-your-own-weights** road specifically, and must not be printed beside
a machine that answers through our API. A machine on the API road produces
inference egress all day, by design, with the person's knowledge.

## What this does not change

- **Law 1 and Law 2**, in full.
- **ADR 0008's three places**, and the rule against silent movement between
  them.
- **ADR 0009's fourth answer and no nagging.** A machine with no model is a
  supported state, not a half-installed one, and it does not ask again.
- **The catalogue, and every grade in it.** Grades were earned against a named
  artefact and still are. The catalogue recommends; it never gates.
- **ADR 0036 and ADR 0046.** A machine builds and pushes; only the owner signs.

## What accepting it changed

- `image/Containerfile`: the weights stage and its `COPY` removed; the model
  runtime kept. The image stops carrying 4.87 GiB.
- `docs/features.md`: **The local model is what the machine arrives ready to
  run** is replaced rather than reworded, and **Run a model we never catalogued**
  moves from v0.5 into the current release — the tier moves, the scope gate is
  not crossed, as when the canvas experience moved on 2026-09-30. Our own API is
  listed as future and marked as such.
- `crates/alo-image`: the rule flips from *the image carries the model the
  catalogue recommends* to *the image carries no weights*.
  `ArrivesWith::NoModel` was already the type for it.
- `docs/booting.md`: the written disk's measured size, which was dominated by the
  weights.
- `image/release-notes.md` and `README.md`: the promise a person reads.

**A consequence that is a benefit and was not the reason.** Without the weights
the image is small enough for a hosted CI runner to build, so
`.github/workflows/image.yml` becomes usable and the release stops depending on
whichever machine has 70 GB of free disk. That is worth having and it is written
here so that nobody later mistakes it for the argument.
