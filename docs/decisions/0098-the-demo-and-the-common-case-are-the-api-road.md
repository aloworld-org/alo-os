# ADR 0098 — The demo and the common case are the API road

**Status:** PROPOSED, 2026-10-08. **Proposed by:** the owner, stated directly:
*"that is not the blocker because most of the users will use the apis as few
users have such machines that can run the models so also us we shall do the demo
with the apis not models on the local machine"*.
**Date:** 2026-10-08
**Amends:** `ROADMAP.md`'s v0.01 exit sentence, and the reasoning — not the
number — in `docs/hardware.md`'s memory row.
**Does not change:** [ADR 0007](0007-the-cpu-is-the-default.md),
[ADR 0095](0095-the-release-carries-no-model-and-a-person-brings-their-own.md),
or the first law.

## The question in one line

Does a person have to run the model on their own machine for v0.01 to be proved?

## What is wrong today, measured rather than felt

`ROADMAP.md` makes v0.01 one sentence, and its last clause is the problem:

> *an action a person would take by hand can be proposed by an agent, approved
> in one click, and afterwards explained — **and the model that proposed it ran
> on the customer's own machine**.*

ADR 0095 already decided, six weeks of work earlier, that a release carries no
model and that a person brings their own, uses a third party's API, uses ours,
or uses none. **Four roads.** The sentence above requires one of them.

So v0.01 cannot be closed by a demo on any machine that does not run a model
locally, which — as the owner puts it — is almost every machine a customer has.

And `docs/hardware.md` sizes the certified machine entirely on the road that
sentence requires:

> **32 GB** — A 7B model at four bits is ~5 GB of weights and needs the OS, the
> agent and a browser beside it. On a 15.5 GB machine the same model ran against
> swap at a quarter of a token a second and the guest went down under it.

That measurement is sound and it is a measurement **about local inference**.
Nothing has measured what an OS, an agent and a browser need when the model is
somebody else's API.

## The decision

**1. The demo and the common case are the API road.** A person signs in, asks
alo to do something, approves it in one click and reads afterwards what it did —
with the answer coming from an API. That is what is shown, and it is what most
customers will run.

**2. The local-model road stays first-class and is what the 32 GB machine is
for.** It is not deprecated, not a future and not a tier. ADR 0095's four roads
stand, and *a model the customer owns* remains the product's first-class
workload per `CLAUDE.md`'s opening sentence.

**3. v0.01's sentence splits rather than loosens.** The clause *and the model
that proposed it ran on the customer's own machine* is **not deleted** — it
becomes its own promise, provable on the 32 GB machine, while the rest of the
sentence is provable on any certified machine. One sentence that two different
machines prove in part is a sentence nobody can tick.

**4. The certified machine's memory number does not move here.** 32 GB is still
what to buy, because it is the only machine that can prove promise 3, and
because the measurement behind it is real. **What changes is the reason written
beside it**: it sizes the local-model road, and nothing yet measures the API
road's needs.

## What this unblocks, and what it does not

**Unblocks:** phase 8 on hardware that exists. Twenty-nine promises are waiting
on *a person sat at a machine watching it work*, and until now the only machine
that could was one nobody has bought.

**Does not unblock** the local-model promise. That still needs the 32 GB
machine, and saying so is the point of splitting the sentence rather than
rewriting it.

**And it does not make the testing NUC a certified machine.** That machine is
8 GB, a 120 GB SATA disk, an Atom-class 6th-generation processor and not a
laptop. It is the **installation** machine, and the right one: it arrives with
Windows, it has one disk, and that is the hard case for *alo OS beside Windows*.
What it can prove about the product is a separate question that the measurement
in rule 5 answers.

**5. One measurement is owed before any machine is called certified for the API
road.** What an OS, an agent and a browser need with no local model has never
been measured. Nobody may infer it from the 32 GB figure, in either direction —
that number sizes weights, and weights are the thing being removed. Until it is
measured, the API road has no certified machine and `docs/hardware.md` says so
rather than implying 8 GB is fine or that 32 GB is needed.

## Why the API road may be the better demo, which is not why it was chosen

It was chosen because it is what customers will run. But it is also where the
first law becomes visible:

> **Nothing leaves silently.** Every network egress an agent causes is visible at
> the moment it happens and afterwards in a record. On a machine sold on
> sovereignty the indicator is a feature, not a diagnostic. With a local model a
> working day produces **zero** inference egress.

**A local model gives the indicator nothing to show.** An API demo is the only
one where a person watches egress happen, see it named, and find it afterwards
in a record. The law's harder half — *visible* — is demonstrable only on the
road this decision picks.

That is an argument for the demo, not for the road. The zero-egress measurement
stays a published claim about the local-model road and is not weakened by a demo
that takes a different one.

## What would make this wrong

A demo that shows the API road and a product that only works well on it. The
test is whether the local-model promise keeps a named machine, a named owner and
a date — if it becomes *the thing we will get to*, this decision quietly deleted
the first clause of what alo OS is, which is a model the customer owns.

## Options rejected

**Delete the clause from v0.01's sentence.** It is the product's distinguishing
claim; a milestone that does not require it anywhere is a milestone that does
not test it.

**Leave the sentence and wait for the 32 GB machine.** Twenty-nine promises
blocked on one purchase, when twenty-eight of them do not need it.

**Call the testing NUC certified.** It fails four rows of `docs/hardware.md` and
the installer itself tells its user *alo OS is made for 16 GB or more, and is
slow with less*. Certifying a machine the product warns about would make the
honest sentence on screen a lie about our own test machine.
