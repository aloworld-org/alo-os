# ADR 0014 — alo's own model is a provider like any other, and gets no exemption

**Status:** accepted — makes the commercial shape of
[ADR 0008](0008-where-inference-happens.md)'s *alo's EU service* explicit, and
names the conflict of interest it creates
**Date:** 2026-09-03
**Context:** [ADR 0008](0008-where-inference-happens.md) (where inference
happens), [ADR 0009](0009-a-good-computer-without-the-agent.md) (a good computer
without the agent, and the money case), `crates/alo-models`, `crates/alo-egress`,
`docs/features.md`

## The decision in one line

alo will host models in the EU and sell access to them by subscription — and
that service is treated by this operating system as **exactly one more provider**:
same egress indicator, same provenance line, same policy, same refusals, same
behaviour when the money runs out, and **no default, no pre-selection and no
special case anywhere in the code**.

## What this adds, and what it does not

ADR 0008 already listed *alo's EU service* among the hosted options, in the
paragraph that says *"the question leaves the building. The indicator fires."*
So the option is not new. What is new is the commercial layer — a subscription,
a balance, an account — and the pressure that comes with it.

## The conflict of interest, stated plainly

**If alo sells inference, alo profits when questions leave the machine.** That
is in direct tension with the promise this product is built on, and it will not
go away by being unmentioned. Every previous decision here was made by a company
with nothing to gain from either answer; this is the first one where we do.

The pressures are predictable, so they are written down before they arrive:

- make our own service the default, because it converts;
- make it look safer than a third-party provider, because it is ours;
- have the indicator treat it gently, because we know where it runs;
- let a failing local model quietly become a paid call;
- remind somebody about a subscription when the agent cannot answer.

**Every one of those would be a lie the product tells to make a sale.** The
rules below exist to make them structurally impossible rather than merely
discouraged.

## The rules

**1. No exemption from the indicator.** A question sent to alo's service leaves
the machine, so the egress indicator fires exactly as it would for Mistral or
anyone else. *Nothing has left this machine* must never quietly mean *nothing
has left this machine except to us.*

**2. It says who and where, in the same words.** "Answered by alo, in
Frankfurt" — the same provenance sentence any other provider gets, in the
reader's own language. Not "answered by alo" with the location left off because
we are trusted.

**3. It is a `Hosted` source with a region, and nothing else.** No variant of
its own in `InferenceSource`, no branch in `SourcePolicy`, no special case in
`alo-asking`. **A machine set to keep questions in the building refuses ours too**,
and that refusal reads no differently.

**4. No default and no pre-selection.** Setup's four choices stay the same size,
in the same order, with the same weight — including *not at all*. Our service
appears inside *with a provider you add*, listed beside the others.

**5. Never a silent fallback, including to us.** ADR 0008's rule runs both ways
and this is the case it will be tested by. A local model that fails does not
become a paid call to alo. Asking somewhere else is one question a person
approves, once.

**6. Running out behaves as ADR 0009 requires.** An empty balance is not an
error, is said once where it happened, does not degrade the machine, and **does
not nag**. A subscription reminder is the greyed-out panel ADR 0009 refused,
wearing a price tag.

**7. The account is not the machine's.** Signing in to alo OS does not sign
somebody in to a paid model, and cancelling does not touch anything else. A
person who stops paying has an operating system that works, not a trial that
expired.

## Why sell it at all

Not every machine can run a model worth using. ADR 0007 put the CPU first and
the catalogue states honestly what a machine can manage — but an eight-year-old
laptop with 8 GB will run a small model slowly, and for that person a hosted
option is the difference between an agent and none.

It also serves the case ADR 0008 already names: the organisation that would
rather buy inference than operate it, in the EU, from a company subject to
European law. That is a genuine sovereign offer, and refusing to make it would
push those customers to a US provider — which is worse for them and worse for
the thesis.

## Consequences

- **`alo-models` gains nothing.** Our service is configured as a provider with
  an address, a key and a region. If it needs a new type, the design is wrong.
- **The account, the balance and the subscription live outside this repository**
  — this is an operating system, not a billing system. What alo OS knows is what
  it knows about any provider: an address, a key held in the keyring, a region,
  and whether the last request was accepted.
- **Queue item 22 covers our own service too.** Running out on alo's plan reads
  identically to running out on Mistral's.
- **A test is owed that nobody can add a special case**: our address is not
  privileged in `alo-egress`, and a policy that refuses hosted inference refuses
  ours.
- The commercial detail — price, tiers, whether it is a subscription or a
  balance — is not this repository's and is deliberately not decided here.

## Alternatives rejected

**Do not sell inference at all.** Rejected: it abandons the person whose machine
cannot run a model and hands the sovereign-hosting market to somebody else. The
conflict of interest is real, and is better managed in the open than avoided.

**Sell it, and make it the default.** Rejected: it is the single change that
would do most damage to the product's credibility, and it is the one every
commercial instinct will argue for.

**Give our own service a quieter indicator.** Rejected outright. The indicator's
worth is that it has no exceptions. One exception, and it measures our
convenience rather than the person's exposure.
