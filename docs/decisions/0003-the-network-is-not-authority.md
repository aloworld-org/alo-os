# ADR 0003 — Being on the same network is not authority

**Status:** accepted
**Date:** 2026-09-02
**Context:** machine discovery, shared local inference, self-hosted workspace
discovery, fleet enrollment; extends the grant model of
`docs/decisions/0001-the-capability-model.md` across machine boundaries

## The decision in one line

alo machines **discover** each other on a local network with no configuration,
and **trust** none of them for it: every use of another machine — its models,
its files, its printer, its agent — requires a deliberate pairing made by a
person on **both** machines, and a remote agent acts only under a grant made on
the machine it is acting upon.

## What this is for

A company buys one alo OS AI workstation. Every other machine in the office
should find it and use its models without anyone typing an address, so the
inference never leaves the building — it just moves down the corridor. An office
with no internet at all still has working AI.

The same discovery removes the most painful step in setup: a self-hosted
workspace on the LAN should be *found*, not configured with DNS.

That is the convenience. This ADR exists because of what the convenience would
cost if it were built the obvious way.

## What was actually wrong with the obvious way

"Machines on the same network can use each other" is how lateral movement works
in essentially every network breach. An attacker who reaches the office WiFi —
a guest password, a compromised laptop, a printer with old firmware — inherits
whatever the network confers.

Building that deliberately, into a product sold on sovereignty, would be the
most serious mistake available to us. And it would contradict ADR 0001 directly:
that document says reach comes from a person's deliberate act and never from
circumstance. A machine being on the same WiFi is circumstance.

## The model

**Discovery is open.** A machine advertises that it exists and what it offers —
mDNS/DNS-SD, no credentials, no configuration. Discovery reveals presence and
nothing else: no files, no records, no model access, no agent surface.

**Use requires pairing.** Before one machine may use another's inference, files,
printer or agent, a person confirms it **on both machines**. Pairing is
mutual and deliberate; it is never inferred from a subnet, a WiFi password, a
certificate authority or a "we've seen this machine before".

**Pairings behave like grants** (ADR 0001 §3), because a person should not have
to learn a second permission concept: enumerated, visible where they can be
found, revocable in one action taking effect immediately, and expiring by
default.

**A remote agent acts only under a local grant.** An agent on machine A that
reaches machine B is bound by the grants made **on B, by B's person**. A's
grants confer nothing on B. Pairing lets A ask; it never lets A act.

Concretely, when an agent on A causes something to happen on B:

- B evaluates it against B's own grants and verb list, not A's;
- B's person approves any change, on B, from B's own approval surface;
- B records it, with A named as the origin;
- and B's egress indicator treats it as egress, because it is.

**There is no trusted-network setting.** No "this is my home network" toggle, no
"trust all machines on this subnet", no enterprise mode that turns pairing off
in exchange for convenience. That switch is the whole vulnerability, and a
setting is how it would arrive.

## Consequences

- Setting up an office is a short sequence of deliberate pairings rather than
  zero clicks. That is a real cost in convenience and it is the point.
- Fleet enrollment (v1) is pairing at scale: a new machine appears to the fleet
  and **asks**, and an administrator admits it. Discovery makes enrollment
  painless; it does not make it automatic.
- Shared inference is egress. Modest egress, inside the building, to a machine
  the person paired with — but the indicator fires, because law 1 says nothing
  leaves silently and "it only went to the machine down the corridor" is exactly
  the kind of exception that erodes a guarantee.
- Everything here works with no internet, which is what makes air-gapped
  operation possible rather than merely claimed.

## Alternatives rejected

**A trusted-network setting.** Rejected: it is the vulnerability, wearing the
clothes of a preference. Anybody who wants it has been asked for it by somebody
whose real problem is that pairing is too tedious — which is a design problem in
pairing, to be fixed there.

**Certificate-based automatic trust within an organisation.** Rejected for v1:
it moves the question to who may issue certificates and how enrollment is
bootstrapped, and every answer smuggles the ambient trust back in through a
component that is harder to audit. Revisit when fleet management is real, with
its own ADR, and with pairing still underneath it.

**Treat a paired machine as an extension of this one, so an agent can act
freely across both.** Rejected: it collapses two machines' capability models
into one, and the person who owns machine B never agreed to that. Pairing lets a
machine ask. Grants on B decide what happens.
