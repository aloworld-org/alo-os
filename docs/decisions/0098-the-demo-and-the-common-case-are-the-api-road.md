# ADR 0098 — The demo and the common case are the API road

**Status:** **ACCEPTED, 2026-10-08**, by the owner — *"I go with your
suggestions"*, to a recommendation that it be accepted with *What sovereign
means on this road* added, which it now carries.
**Proposed by:** the owner, stated directly:
*"that is not the blocker because most of the users will use the apis as few
users have such machines that can run the models so also us we shall do the demo
with the apis not models on the local machine"*, and clarified in the same
conversation: *"having models is okay, but if they are blocked with the hardware
for testing we move on overall not all the people we target have gpus or
language machines and also in our demo video we shall use apis not models since
it is expensive for the gpus"*.
**Date:** 2026-10-08
**Amends:** `ROADMAP.md`'s v0.01 exit sentence, and the reasoning — not the
number — in `docs/hardware.md`'s memory row.
**Does not change:** [ADR 0007](0007-the-cpu-is-the-default.md),
[ADR 0095](0095-the-release-carries-no-model-and-a-person-brings-their-own.md),
or the first law.

## The question in one line

Does a person have to run the model on their own machine for v0.01 to be proved?

## Three reasons, and they are not the same reason

**1. Most of the fleet cannot.** The owner: *not all the people we target have
gpus or language machines.* `docs/hardware.md` already applies this logic to
graphics — *this document's first certified machine has no discrete GPU on
purpose: it decides whether there is a market* — and the same sentence applies
one step further to memory. **A machine sized for what the model needs rather
than for what the fleet has measures a market we are not selling to.**

**2. Running them costs money, ours included.** *It is expensive for the gpus.*
That is a different argument from scarcity and it bears on us rather than on
customers: a demo filmed against local inference needs a GPU box running while
it is filmed and re-filmed. Nothing in this repository has priced that, and
nothing needs to — the point is that the cheaper road is also the one most
customers take, so there is no trade.

**3. A promise whose proof needs hardware nobody has blocks the promises beside
it.** *If they are blocked with the hardware for testing we move on.* Twenty-nine
promises wait on phase 8, phase 8 waits on a machine, and three of the
twenty-nine are the only ones that need the machine to be large. Keeping them
together means twenty-six wait on three. **That is what rule 3 separates**, and
it is the reason this is a decision rather than a preference.

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

**Which promises those are, counted rather than estimated.** Three of v0.01's
twenty-nine name a local model:

- **Model stack** — catalogue, pull, serve, unload, remove. Its code is
  finished.
- **Agents point at the local model by default**, carrying *never a silent
  fallback* (ADR 0008).
- **Zero inference egress over a working day**, which already says in
  `ROADMAP.md` what this ADR is saying: *with a local model, which is the claim
  `docs/features.md` makes and the only one that is true. A machine using the
  office GPU box or a hosted provider has non-zero inference egress by design,
  shown on the indicator.*

**So the document already makes this distinction in one place.** This decision
extends it to the place that does not have it, rather than inventing it. An
earlier draft of this ADR said *twenty-eight of twenty-nine* without counting;
it is twenty-six.

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

## The demo video, which nothing in this repository mentions

The owner names one: *in our demo video we shall use apis not models.* A search
of every markdown file finds no *demo video* anywhere, so it is new and has no
task, no plan and no owner.

**One requirement on it falls out of the first law and is not optional.** A
video showing an agent acting over an API, without the egress indicator visible
in frame, films the thing this product criticises — an agent reaching the
network while the person watching cannot see it. The indicator is *a feature,
not a diagnostic*, and a demo is where that is either true or revealed as
decoration.

So: **whatever else the video shows, it shows the indicator**, and it shows a
person finding the egress afterwards in the record. That is the demonstrable
half of law 1, and the API road is the only road on which it can be
demonstrated at all.

## What *sovereign* means on this road

`CLAUDE.md`'s first sentence calls alo OS **a sovereign AI workstation** whose
first-class workload is *a model the customer owns*. If the demo, the common
case and the certified machine are all the API road, **the word cannot go on
meaning what it meant**, and a decision that is quiet about that lets a reader
assume it survives unchanged.

So, plainly. On the local-model road the claim is **nothing leaves**: a working
day produces zero inference egress, measured at the network boundary and
published. That claim is unaffected by this decision and is what the 32 GB
machine proves.

**On the API road the claim is the other two halves of the first two laws**, and
they are not smaller, only different:

- **Every network egress an agent causes is visible at the moment it happens and
  afterwards in a record.** Not *we do not send your data* — *you can see every
  time we do, and find it again later.*
- **No verb runs an arbitrary command.** Every capability is an enumerated verb
  with typed, validated arguments: no `exec`, no shell, no script the model
  authored, no advanced escape hatch. A model that can write code that runs has
  escaped every other control, so this is what makes the rest true rather than
  decorative.

**And the fifth law, which is what makes either of those add up to
*sovereign*.** *The person chooses. alo never takes the choice away.* Four roads
exist, the person picks one, nothing moves between them on its own in either
direction, and a machine with no model at all is a supported state that does not
nag. So on every road, ours included:

> **The person decides where their inference happens, and can see it and change
> it.**

**Auditability without that is good logging.** Containment without it is a
locked room. The three together are the product, and the third is the one a
competitor cannot copy by adding a dashboard.

**Auditability, containment and choice, rather than locality.** That is a weaker
claim about where data goes and a stronger one about what an agent may do and
who decides it — and it is the honest description of a machine using somebody
else's API.

**This is a continuation rather than a new claim, and
[ADR 0095](0095-the-release-carries-no-model-and-a-person-brings-their-own.md)
is where it was already made.** That decision says outright that *answering out
of the box will then mean inference leaving the machine*, and rules that *a
working day produces zero inference egress, measured at the network boundary*
**belongs to the bring-your-own-weights road specifically, and must not be
printed beside** a machine that answers through our API. This ADR carries that
to the demo and the certified machine; it does not invent it.

**Both are rare and the second is rarer.** A product can promise it does not
send your data and be believed or not; a product whose agent *cannot* run an
arbitrary command has made a structural choice a competitor cannot retrofit.

**What this forbids.** Saying *sovereign* on the API road without saying which
of the two claims is meant. The word is doing different work on each road, and a
page, a demo or a sentence that uses it for both is selling the locality claim
on the road that does not have it.

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
