# ADR 0008 — Where inference happens is a choice, and the person always knows

**Status:** accepted — extends [ADR 0007](0007-the-cpu-is-the-default.md) and
carries law 1 into it
**Date:** 2026-09-02
**Context:** `crates/alo-models`, `docs/features.md` (sovereignty as testable
claims), [ADR 0003](0003-the-network-is-not-authority.md),
[ADR 0004](0004-the-organisations-machine.md); `alo-workplace`'s `AiConfig`,
which already carries a base URL and an optional key

## The decision in one line

A model may run **on this machine**, **on a machine on this network**, or
**behind somebody's API** — the person choosing is told which, in those words,
and the egress indicator tells the truth about each; a hosted API is a supported
first-class option and never a silent fallback.

## What was actually wrong

ADR 0007 made the CPU the default, which was right and still left a gap: it
assumed everybody runs a model. Most people will not. A thin laptop cannot
comfortably run even a small model, plenty of organisations would rather buy
inference than operate it, and some will use a provider they already have a
contract with.

Meanwhile law 1 says nothing leaves silently. Sending a question to a hosted API
is the largest egress this product will ever cause — it is somebody's records
leaving the building — so "supporting APIs" without deciding how that is
surfaced would have hollowed out the sovereignty claim by accident, one
convenient default at a time.

Both problems have the same answer: **make the location of inference an explicit,
visible property rather than a configuration detail.**

## The three places

**This machine.** The weights are local, the answer never leaves. Expected
inference egress over a working day is **zero**, measured at the network
boundary, and that measurement is what `docs/features.md` publishes.

**A machine on this network.** ADR 0003's one-GPU-box-serving-an-office. The
question leaves this machine and does not leave the building. The indicator
fires — "it only went down the corridor" is exactly the exception that would
erode the guarantee — and says which paired machine answered.

**A hosted API.** alo's EU service, the customer's own hosted endpoint, or a
third-party provider. The question leaves the building. The indicator fires, and
says **who** and **where**: a provider that will not say where it runs is
reported as unknown rather than assumed to be nearby.

## The rules

**Never a silent fallback.** A local model that fails does not quietly become an
API call. Failing to answer is recoverable; a person's records leaving the
building because a download was corrupt is not.

**The source is named where the answer appears**, not in a settings page.
Somebody about to paste a contract into a question is entitled to know where it
is going before they paste it, not afterwards.

**An organisation can forbid the ones it does not want** (ADR 0004): a managed
machine may permit local only, or anything inside a region it names, or a named
provider. The policy is stated in words a person on that machine can read.

**Where a provider runs is a stated fact, not an inference.** A region is
something a provider declares and we record; a provider that has not declared it
is `unknown`, and unknown never satisfies a policy that names a region. Guessing
from a domain name would put a customer in breach while showing them a
reassuring label.

**We ship no default that chooses a provider, and no region of our own.**
Whether somebody uses Mistral, alo, their own endpoint or nothing at all is
their decision. The policy type exists so an organisation with a rule can state
it — "in the EU", "in Switzerland", "in the United States" — and have it
enforced; it does not exist so alo OS can have a rule. A product built in Europe
that hardcoded Europe would make everybody else a special case in their own
operating system.

## Consequences

- alo OS becomes usable on hardware that can run no model at all, which widens
  the reachable fleet considerably beyond ADR 0007.
- The sovereignty story gets *stronger*, not weaker: "your data need never
  leave" is a claim you can only make credibly if you also let people choose
  otherwise and show them what they chose.
- The catalogue in `crates/alo-models` remains what it is — a list of **local**
  models with their licences and costs. A hosted model is not a download and has
  no licence to gate; it is a service with a name, a provider and a location.
- `alo-workplace` needs no change to consume this: `AiConfig` has carried a base
  URL and an optional key since 2025. What is new is that alo OS knows *what
  kind of place* that URL is, and says so.

## Alternatives rejected

**Local only, for purity.** Rejected: it excludes every machine too small to run
a model and every organisation that would rather buy inference. Purity that
excludes most of the market is not a principle, it is a market decision wearing
one.

**Treat a hosted API as just another endpoint.** Rejected: it is technically
true and is exactly how the guarantee would be lost. If the three places look
identical in the product, the indicator becomes decoration and law 1 becomes a
sentence in a document.

**Fall back to a hosted API when the local model fails.** Rejected outright, and
worth naming because it is the single most tempting convenience here. It would
mean somebody's records leave the building at the moment of a failure they never
saw, which is the precise opposite of "nothing leaves silently".
