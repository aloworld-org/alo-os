# ADR 0004 — The organisation's machine, and what the person is told about it

**Status:** accepted
**Date:** 2026-09-02
**Context:** fleet enrollment, policy, disk-encryption key escrow, remote lock
and wipe, audit export; resolves a tension between those and
`docs/decisions/0001-the-capability-model.md` and law 1

## The decision in one line

A machine is either **personal** — no policy, no escrow, no remote wipe, nobody
above the person — or **managed by an organisation**, in which case the
organisation sets policy, holds a recovery key and can lock or wipe it; and on a
managed machine **the person is told exactly that, in the shell, from the first
sign-in**, because the alternative is a product that says your machine is yours
while somebody else holds the keys.

## What was actually wrong

Large organisations cannot buy a machine they cannot manage. They need a
recovery key when an employee leaves, a wipe when a laptop is stolen, policy
over what staff may run, and the audit trail in their own security console.

Every one of those reads, at first glance, as a contradiction of what this
product sells. A repository that lists "full-disk encryption" and "nothing
leaves silently" next to "your employer can wipe this and read the disk" is
either dishonest or has not thought about it. So the resolution is written down
before the features are built, rather than discovered by a customer.

The resolution is not a compromise on sovereignty. It is a statement about
**whose machine it is**. Sovereignty in this product means *the organisation's
data stays in the organisation's building* — not that an employee's work laptop
is beyond the reach of the organisation that owns it. Those are different
claims, and conflating them would have produced a system no company could deploy
and a promise we could not keep anyway.

## The two modes

**Personal.** The default. No policy, no escrow, no remote anything. The person
is the only authority on the machine. Everything in ADR 0001 applies with nobody
above it.

**Managed.** The machine is enrolled with an organisation. Enrollment is an act,
visible and recorded; **there is no silent enrollment**, and a machine cannot be
moved into managed mode without the person at the keyboard seeing it happen.

## What a managed organisation gets

- **Policy**: which verbs, which adapters, which models are permitted, by role.
- **The record**: agent executions and refusals, exportable to their own SIEM.
- **Machine state**: image version, health, update ring, encryption status.
- **A recovery key** for the disk, escrowed at enrollment.
- **Remote lock and wipe**, both destructive-by-design and both recorded.
- **Update rings**, so a fleet is not updated all at once.

## What it does not get

- **The content of the person's files.** Escrow recovers a machine; it is not a
  console for reading someone's documents from a distance.
- **The content of agent conversations**, only the record of what was *executed*
  and what was refused.
- **Silent screen or context access.** ADR 0001 §4 holds without exception: no
  invocation, no context. A managed machine does not become a machine somebody
  can watch. If it did, the context guarantee would be a lie for exactly the
  people most likely to rely on it.
- **The ability to act as the person.** An administrator can set policy and can
  wipe a machine. An administrator cannot make that person's agent perform an
  action in their name; a grant is made by the person at the machine (ADR 0001
  §3, ADR 0003).

## What the person is always told

On a managed machine, findable without hunting and shown at first sign-in:

- that the machine is managed, and by which organisation;
- what policy is in force — which verbs, adapters and models are permitted;
- that a recovery key is escrowed, and with whom;
- that the machine can be locked or wiped remotely;
- what is exported, to where, and how often.

This is not a legal notice buried in setup. If a person cannot answer "who else
has power over this machine" in ten seconds, the design has failed.

## Law 1 still holds

Management is traffic. Policy fetches, record export and enrollment all leave
the machine, so they fire the egress indicator and appear in the record like
anything else. "It is only going to the management server" is precisely the
exception that would hollow out the guarantee, and the guarantee is the product.

The zero-inference-egress claim is unaffected: it is a claim about *inference*,
and it stays measurable and true on a managed machine with a local model.

## Alternatives rejected

**No escrow, no wipe, no policy — the purest sovereignty.** Rejected: a
fleet nobody can recover or retire is not a fleet an organisation can adopt, and
data that dies with an employee's departure is a liability, not a principle. The
purity would cost us every customer above fifty seats and would protect nobody
who actually needed protecting.

**Full remote administration, including screen access and remote control.**
Rejected: it is the ambient authority ADR 0001 exists to forbid, and it would
make the context guarantee false. Where a helpdesk genuinely needs to see a
screen, that is a session the person starts and can end — never a capability an
administrator holds.

**Silent enrollment, so IT can manage machines without bothering people.**
Rejected outright. A person who does not know their machine is managed cannot
make an informed decision about what to do on it, and a sovereignty product that
hides who holds the keys has sold the wrong thing.
