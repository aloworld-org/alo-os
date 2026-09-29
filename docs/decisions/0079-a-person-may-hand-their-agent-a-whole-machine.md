# ADR 0079 — A person may hand their agent a whole machine, and is told that is what it is

**Status:** proposed, 2026-09-29
**Date:** 2026-09-29
**Context:** `docs/design/a-persons-other-machines.md` (the design note this
settles one question from);
[ADR 0001](0001-the-capability-model.md) §3 (grants are enumerated, visible,
revocable in one action and expiring — *there is no grant to `/`*);
[ADR 0043](0043-the-terminal-is-a-persons-and-never-an-agents.md) (the terminal
is a person's and never an agent's, *because a shell is the whole machine by
another road*);
[ADR 0025](0025-the-default-is-what-a-machine-arrives-able-to-do.md) (the default
is what a machine arrives able to do, and nobody chooses for the person);
[ADR 0031](0031-the-pairing-is-the-key.md) (the pairing is the key);
[ADR 0003](0003-the-network-is-not-authority.md) (the network is not authority);
`docs/features.md` line 124 (the remote desktop portal, v1) and line 304
(cross-machine agent work, v1).

## The question in one line

**A person adds another of their machines to their canvas and can work in it.
May they also let their agent work in it — and if a pointer on another machine
is the whole of that machine, how is that not the grant to `/` this product has
twice refused?**

## What was true before this decision

Two refusals, both deliberate, both for the same reason.

ADR 0001 §3: grants are enumerated — *a list, not a rule* — **and there is no
grant to `/`**. ADR 0043 met the same principle a second time and named it: a
shell is the whole machine by another road, so an agent may not be granted the
terminal, and `alo-capability` refuses it three ways rather than once.

The design note that precedes this record found the same shape a third time. A
pointer and a keyboard on another machine's screen reach every verb that
machine's interface offers, and not one of them is on a list.

**An earlier draft of that note concluded: refuse it.** The owner overruled it.
Their standing rule is that a person may choose what they prefer, that the
danger is indicated, and that a feature is not removed to prevent the choice.

## The decision

**A person may grant their agent control of another machine they have added. It
is off until they turn it on, it is granted for one named machine at a time, it
expires and is revocable in one action — and the person is told, in the sentence
where they grant it, that it is everything that machine can do.**

## This *is* a grant to `/`, and the record says so rather than arguing otherwise

The tempting move is to find a distinction: the machine is a different one, the
person owns both, the pointer is only pixels. **All three are rejected here, and
the third is false.**

- *It is a different machine.* True and irrelevant to blast radius. An agent with
  a pointer on machine B can do on B everything B's person could do. That the
  harm lands on B rather than on A does not make it smaller; it makes it
  somewhere else.
- *The person owns both.* Ownership is not a grant — the design note rejects it
  for work, and it is rejected here for the same reason. If ownership sufficed, a
  stolen machine would carry its owner's authority everywhere its owner has a
  machine.
- *It is only a screen.* It is not. A screen with input is a machine.

So: **this grant is `/` on another machine, reached by a third road.** ADR 0043
named the second road and refused it. This record names the third and, on the
owner's direction, permits it — deliberately, visibly, and under the narrowest
terms that still leave the feature real.

A record that dressed this as something smaller would be worse than one that
refused it, because the next person to read it would not know what they were
being handed.

## What makes it survivable, and each one is load-bearing

**1. It is the only grant in this product that cannot enumerate what it
permits.** ADR 0001 §3 says grants are a list and not a rule. This one is a list
of length one whose single entry is a machine, and the verbs behind it are
unbounded. **That is stated in the record and in the words the person reads.** A
grant that looks like the others and is not is the failure this repository has
spent the week removing from its own gates.

**2. Off until turned on.** ADR 0025 holds that a default is what a machine
arrives able to do and nobody chooses for the person. Adding a machine grants the
*person* a window; the agent driving it is a second act and takes a second
decision. Nobody arrives able to be driven.

**3. One machine at a time, named.** There is no grant over *machines*. A person
with four machines who wants their agent on three makes three decisions.

**4. Expiring, and revocable in one action, reaching work already in flight.**
ADR 0031 already makes revocation immediate for proofs, because the key goes with
the row. A pointer already moving must stop on the same word, or *immediately*
means two different things in one product.

**5. The machine being driven says so, on its own screen.** A person who walks up
to machine B must see that something is working in it. This is the protection
that survives the grantor being wrong about what they granted.

**6. It does not touch ADR 0043 on any machine.** The terminal stays a person's
own, here and there. An agent driving B's screen can be refused B's terminal by
B's own `alo-capability`, exactly as now. **This is a real limit and also an
honest one: a pointer can open a terminal a person has already opened.** The
refusal holds where it can hold and is not claimed where it cannot.

**7. What is granted does not travel.** An agent granted machine B holds nothing
on machine C, and nothing back on machine A. Each machine's grants are made on
that machine. Lateral movement between a person's own machines is exactly what
ADR 0003 refuses for strangers, and being the same person's machine is not
authority either.

## What the person is told, and where

In the person's own language, in the vocabulary with its own key (ADR 0068), at
the moment of granting and not in a document:

> **This lets the agent do anything on that machine that you could do there.**
> Unlike every other permission you give, this one cannot list what it allows.

Not a warning box to click past. Not *are you sure*. The plain fact, where the
choice is made — because the person most likely to be harmed is the one who
believed they were granting something narrower.

## Consequences

- `docs/features.md` line 304's promise — an agent may **ask** a paired machine
  and act under a grant made there — stays exactly as it is. **This adds a second,
  separate thing a person may grant, and does not widen the first.**
- The word this grant refuses under, when it has not been given, is
  `NotGranted::Never` as ADR 0043 established: nothing the agent holds covers the
  machine. No new variant, for the reason ADR 0043 gives — an exhaustive match in
  a crate this change does not own.
- Remote lock and wipe (`docs/features.md`, v1) stops being optional. A lost
  machine now holds a standing grant that cannot enumerate itself, and the
  dependency belongs in the roadmap rather than in somebody's memory.
- `docs/design/who-is-acting.md` gains its hardest case. A pointer moving on
  another machine has to be attributable to the person or the agent, and the
  attribution has to survive the window filling the screen.

## What was rejected

**Refusing it outright**, which was the design note's first recommendation. The
argument for refusing is the strongest one in this record and it is not
weakened by the decision: a pointer is a general verb. It is rejected because
the owner's rule is that a person chooses and the danger is named, and because
the feature it would remove is the whole of *sit down at your other machine*.

**Enumerating the verbs instead** — a list of what the agent may do on the
remote machine. Rejected as a fiction. Input is not enumerable at the other end:
a keystroke is not a verb, and anything that claimed to enumerate keystrokes
would be a list that permits everything while reading like a list that permits
seven things. **A grant that lies about its own scope is worse than an honest
wide one.**

**Granting it once, for all machines.** Rejected: it makes the widest grant in
the product the cheapest one to make.

**Deriving it from ownership**, or from the pairing. Rejected: the pairing is
how two machines trust each other, not how a person hands their agent a machine.
Merging them would mean every pairing a person ever made silently became an
agent grant.

**Making it permanent.** Rejected by ADR 0001 §3 — there is no grant that
outlives the reason it was made, and this is the last grant that should be the
exception.

## What this record does not settle

Whether an agent driving a machine may cross from it to a *third* machine that
the second machine has a grant over. The answer here is no by construction —
what is granted does not travel — but the shape of that refusal, and where it is
enforced, belongs with the work-sending decisions the design note lists and is
not decided by this record.
