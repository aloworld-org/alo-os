# ADR 0025 — The default is what a machine arrives able to do, and nobody chooses for the person

**Status:** proposed — the owner decides, because the recommendation asks for one
line of `docs/features.md` to be reworded and only the owner may move the
definition
**Date:** 2026-09-11
**Proposed by:** the v0.01 delivery workstream, as task 16 of
`docs/autonomy/v0-01-delivery-plan.md`
**Context:** `docs/features.md` (*The agents point at the **local** model by
default — sovereignty is the default configuration, not an option to find*),
[ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md) (the
organisation bounds and the person chooses),
[ADR 0008](0008-where-inference-happens.md) (where inference happens),
[ADR 0009](0009-a-good-computer-without-the-agent.md) (the fourth answer, and no
nagging), [ADR 0006](0006-the-pinned-model-runtime.md) (the pinned runtime),
[ADR 0007](0007-the-cpu-is-the-default.md) (the CPU),
[ADR 0019](0019-a-runtime-is-found-not-configured.md) (a runtime is found, not
configured), [ADR 0014](0014-alos-own-model-is-a-provider-like-any-other.md);
`crates/alo-choosing`, `crates/alo-image`, `image/Containerfile`,
`docs/autonomy/v0-01-evidence.md`

## The question in one line

**What does *by default* mean on a machine where nobody but the person is allowed
to choose?** `docs/features.md` promises that the agents point at the local model
by default; ADR 0016 settled that *a default is a choice, made by whoever set it*
and refused one. Both sentences are in force, one of them is wrong as written,
and no worker may narrow the promise or contradict the ADR to make a test pass.
So the audit has carried this promise with no evidence at all since it was
written down, and this decision is what unblocks it.

## What is true today, verified rather than remembered

Read off the tree this was written in, because the argument below rests on it
and half of it is not what the question assumes.

- **`alo-choosing` cannot produce a choice nobody made, on purpose.** A missing
  settings file is `Settings::untouched` — a person who has not chosen, not an
  error — and the crate documents in as many words that a store which could
  invent one would be ADR 0016's rejected *the organisation sets a default*
  wearing different clothes. There is no `Default`, and `Chosen` has no
  constructor that does not name a list and an entry.
- **A machine nobody has configured says so.** `agentd.nothing-answers-questions`
  is the true sentence about it, and ADR 0019 already re-affirmed that discovery
  which finds nothing is an answer.
- **The organisation's half is built and enforced.** `alo-agentd` reads the bound
  out of the machine description and hands it to `Chosen::asking`; absent is
  absent rather than a file full of permissions, and a choice outside the bound
  is refused in the rule's own words.
- **The image carries no model runtime and no weights.** `image/Containerfile`
  adds two binaries, two units, two directories and one description to a pinned
  base, and nothing else. **So on the machine this repository actually builds,
  there is nothing local to point at** — not because a setting is missing, but
  because the local model is not on the disk.
- **There is no setup flow anywhere in this repository.** ADR 0009's fourth
  choice has nowhere to be offered, nothing is pre-selected because nothing
  selects, and the ledger records that too.

The fourth of those is the finding that reframes the question. *Local by default*
reads like a dispute about a settings key, and the settings key is the cheap half
of it. **The expensive half is that the local model has to already be there**,
and today it is not.

## What may not be done, whichever option is taken

- **The promise may not be narrowed** to something already true.
- **ADR 0016 may not be contradicted** by a change that does not say it is
  superseding it. If the answer is that ADR 0016 is wrong, that is an ADR, not a
  paragraph in a commit message.
- **Nothing may be silently substituted.** `docs/features.md` says four times, in
  four different ways, that nothing moves between the sources on its own. A
  mechanism that resolves *unset* to a source is one hop from a mechanism that
  resolves *unavailable* to a source, and the second is the promise this product
  is sold on.
- **The fourth answer keeps its weight.** ADR 0009 gave *not at all* the same
  weight as the other three with no persuasion attached, and a person who wants
  no agent must not have to undo one.

## The options

### Option A — unset means local: a default in the person's settings

`alo_choosing::Settings::untouched` resolves to the catalogue's model for this
machine, and the promise is literally true of the code.

- **What it buys:** the plainest possible reading of the sentence a customer
  reads, one small change in one crate, and *sovereignty by default* stated
  bluntly where a sceptic can check it.
- **What it costs, and it is not small:**
  - **ADR 0016 is superseded.** Its reason was never *who* set the default — it
    was that a default **is** a choice, and that the unanswered state must stay
    visible. Reversing that in the person's favour is still reversing it, and it
    must be argued as such.
  - `agentd.nothing-answers-questions` stops being true, and a machine nobody
    has configured begins answering questions.
  - **ADR 0009's decline is broken at the edges.** A machine that answers before
    anybody chose has an agent for whoever has not yet said no — and *no agent*
    becomes a thing you turn off rather than a thing you were asked about.
  - **Something has to pick which model**, and that is
    `Catalogue::agent_for_cpu` recommending, adopted silently. A recommendation
    that becomes a setting by nobody's action is exactly the shape ADR 0016
    refused.
  - **On today's image it resolves to a model that is not there**, so the
    default would be a broken promise on first boot rather than a working one.

### Option B — the image ships the person's choice

A settings file under `$XDG_CONFIG_HOME/alo/` is baked into the image, already
naming a local model, so *the machine ships pointing at it* is a fact about an
image rather than a value somebody set.

- **What it buys:** it looks like the reading that keeps both sentences, and it
  needs no change to any crate.
- **What it costs:** it is us writing in **the person's own file** — the one
  place ADR 0016 keeps for them, and the thing it forbids an administrator from
  doing. We have less claim on it than an administrator does, not more. It is
  also impossible as stated: ADR 0024 established that the image ships no
  accounts, so at build time there is no person and no home directory to put it
  in; and ADR 0007 makes the right model a function of the hardware, so a file
  baked once is wrong on a large part of the fleet.
- **Rejected**, and named because it is the seductive one: it is Option A with
  the mechanism moved somewhere nobody would look for it.

### Option C — setup pre-selects local

The four choices appear together and *on this machine* is already selected; the
person confirms.

- **What it buys:** ADR 0016 is kept literally — a person did press the button —
  and the machine is sovereign for everyone who clicks through.
- **What it costs:** ADR 0009 gave the fourth choice the same weight and *no
  persuasion attached*, and a pre-selected control is persuasion by geometry. A
  person who clicks through has a default nobody chose wearing their own
  fingerprint on it, which is the worst of both readings. It also makes *not at
  all* the option you have to undo something to take.
- **Rejected on its own.** Its ordering half is kept below, because ordering is
  not weight.

### Option D — the default is what the machine arrives able to do *(recommended)*

**Nothing is written into anybody's settings and nothing is pre-selected. The
default is a property of the machine, and it has three parts:**

1. **The image carries the pinned runtime and a model sized for the machine**
   (ADR 0006, ADR 0007), so *on this machine* is complete on first boot: no key,
   no account, no card, no network and nothing to find. This is the expensive
   part and it is the real content of the promise.
2. **Setup asks once, lists the four configurations with the local one first,
   and pre-selects nothing.** Ordering is not weight: the four are equally
   styled, carry no persuasive copy, and pressing continue without choosing does
   not choose. Local is first because it is the one that needs nothing added.
3. **Until the person chooses, nothing answers**, and the machine says so.
   `alo-choosing` is unchanged — and being unchanged is the point.

- **What it buys:** both sentences stand. ADR 0016 is about *who may put a value
  in the person's file*, and under D nobody does. The promise is about *what a
  machine can do out of the box rather than an option to find*, and under D the
  local model is the only source present, already installed, and offered first —
  while every other source is something a person adds: a key, a pairing, a
  subscription.
- **What it costs:** the definition's current wording asserts a setting nobody
  may set, so **one line of `docs/features.md` is reworded by the owner**. And
  it is strictly more work than Option A rather than less: weights on a disk,
  against a resolution rule in one crate.
- **What is not yet measured**, and is the first thing the implementation owes:
  **whether the image carries the weights or fetches them at setup.** The
  smallest catalogued model that clears the verb-driving bar against what the
  update channel can carry is a number nobody here has, and the two answers
  differ in a way a person feels — a machine that fetches at setup is not local
  by default when it is offline at setup. The recommendation is to **carry them
  on the certified image** and to fetch only where an image cannot, but that is
  a recommendation with a measurement owed, and `docs/quirks.md` is where the
  answer goes. This ADR does not assume it.

## The recommendation

**Option D**, with the definition's line reworded to say what is actually being
promised. Proposed wording, for the owner to take or change:

> - [v0.01] ★ **The local model is what the machine arrives ready to run** —
>   sovereignty is what a machine can do out of the box, not an option to find.
>   Every other source is something a person adds; nothing is chosen on their
>   behalf, and until somebody chooses, nothing answers (ADR 0016).

**This is not narrowing the promise.** The rewritten line demands a model on the
disk of every machine we ship, which the current line does not, and which is the
half nobody has built. What it gives up is only the claim that a value exists in
a settings file before a person has touched one — a claim this repository has
deliberately made unbuildable for two ADRs, and which would not have made a
single machine more sovereign.

## Consequences if it is accepted

- **`crates/alo-choosing`: nothing changes, and that becomes a rule with a
  reason.** No `Default`, no resolution of *untouched* to a model, no `Chosen`
  from nothing. A later change adding one is this decision being reversed and
  needs an ADR of its own, rather than a convenience nobody notices.
- **`crates/alo-image`: the work.** The pinned runtime installed, the weights
  carried (or the fetch decided by the measurement above), and a check per
  promise with the twin that breaks one line, in the shape the accounts check
  already has — including **the image ships no settings file for a person**,
  beside `AnAccountShippedWithTheImage`. That check is what keeps Option B from
  arriving later by accident.
- **The setup flow** gains its shape: four choices, local first, none
  pre-selected, *not at all* no harder to take than the others (ADR 0009), and
  alo's own service with no pre-selection of any kind (ADR 0014). Where nothing
  catalogued clears the bar on a machine, setup says so and offers the honest
  alternatives rather than choosing one — which `docs/features.md` already
  promises separately.
- **`docs/features.md`** gains the reworded line, from the owner.
- **`docs/autonomy/v0-01-evidence.md`** names this decision under the promise and
  says what is owed until the image carries a model. It stays a promise with no
  evidence until then, and this ADR does not tick it.

## Consequences if it is rejected in favour of Option A

- **ADR 0016 is superseded in the change that does it**, and this repository has
  one settled decision fewer.
- `alo_choosing::Settings` gains a resolution, `agentd.nothing-answers-questions`
  stops being true of a configured-by-nobody machine, and the *nothing answers
  yet* state has to be reachable some other way or stop existing.
- **ADR 0009's decline must move to the front**: *no agent* has to be recorded
  before anything answers, which means a machine that has not run setup either
  answers questions or holds a choice nobody made — and one of those two is the
  thing this ADR is named after.
- The image still has to carry a model, or the default is a name for a failure.

## What this does not decide

- **Which model the catalogue names for a given machine.** ADR 0007 and the
  catalogue's measured verb-driving bar own that, and seven of its twelve entries
  are unmeasured — which is its own owed line in the ledger.
- **Whether alo's own service may ever be offered first.** ADR 0014 already
  answers no, and nothing here reopens it.
- **The look of the setup flow.** That is drawing rather than deciding, and it is
  the desktop lane's.
- **Paired machines.** ADR 0003 and ADR 0008 own them; a machine in the next room
  is added deliberately on both machines and is nobody's default.

## What the code waits on

**Nothing is built in the change that adds this ADR.** Until it is accepted, a
worker writing code would be choosing between the options rather than building
one — which is the failure ADR 0024 was written to avoid, and the reason this
promise has been carried untouched rather than quietly satisfied.
