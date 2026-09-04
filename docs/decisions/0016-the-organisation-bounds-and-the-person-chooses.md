# ADR 0016 — The organisation bounds, the person chooses, and they are two files

**Status:** accepted — resolves the tension between
[ADR 0004](0004-the-organisations-machine.md)'s `/etc/alo/agentd.toml` and
[ADR 0008](0008-where-inference-happens.md)'s *the person always knows*
**Date:** 2026-09-04
**Context:** `docs/autonomy/QUEUE.md` item 21h, `crates/alo-agentd/src/doing.rs`
(which refuses a question in words because nothing tells it what was chosen),
`crates/alo-models` (the catalogue and `Brought`), `crates/alo-appearance`,
`docs/features.md`

## The decision in one line

**Two settings, two owners, two files**: the organisation writes a *bound* into
`/etc/alo/agentd.toml`, the person writes a *choice* into
`$XDG_CONFIG_HOME/alo/`, and a choice outside the bound is **refused out loud
rather than silently replaced**.

## The tension this resolves

ADR 0004 gives `/etc/alo/agentd.toml` to the organisation. ADR 0008 puts *where
a question may be answered* with the person. Both are settled, and item 21h sat
still because they appear to want the same key in the same file.

They do not. They want two different things that read alike:

- **A bound** is what an organisation may set: *questions may not leave this
  machine*, or *only these providers*. It is a rule about the set of permitted
  answers.
- **A choice** is what a person makes inside that set: *this model, from this
  list, answers my questions*. It is one element of it.

An organisation setting a bound is policy. An organisation setting a choice
would be acting as the person, which ADR 0004 already forbids in the same
sentence that forbids making their agent act in their name.

## Where each lives

| | Owner | File | Holds |
|---|---|---|---|
| Bound | the organisation | `/etc/alo/agentd.toml` | `SourcePolicy` — which sources are permitted at all |
| Choice | the person | `$XDG_CONFIG_HOME/alo/` | which model or provider answers, and which language they read |

The person's file is **XDG-shaped and per-person**, because a machine may have
several people on it and one person's model is not another's. It is not in
`/etc`, and an administrator reading it is reading a person's preferences on
their own machine — which is why nothing else goes in it.

**Which language they read lives here too**, and that is the reason this ADR
covers two things that look unrelated. Item 21h named them as having the same
owner and it was right: both are the person's, both are per-person, both are
read before a turn can say anything, and inventing two stores for one owner is
how a settings system becomes six settings systems.

## When they disagree

**The bound wins, and the person is told who set it.** A choice that falls
outside the policy is not quietly swapped for a permitted one — it is refused,
in words, naming the policy and the fact that an administrator set it.

Silent substitution is the failure mode worth naming, because it is the
comfortable one to build: a person picks a hosted provider, policy forbids
egress, and the machine answers anyway using something local. Nothing appears
broken. The person believes they know where their question went and they are
wrong, which is the exact promise ADR 0008 exists to keep. **A refusal a person
can read is better than an answer they cannot account for.**

## The choice names its list

Item 25 made this one size larger: there are now two lists of models on a
machine — the catalogue and `alo_models::Brought` — and neither knows about the
other on purpose.

So **a choice records which list and which entry**, not a bare name. A model
called `mistral-small` in the catalogue and a file somebody brought under the
same name are two different answers to *what runs my turn*, and a setting that
cannot tell them apart would pick one by accident. The ambiguity is resolved
where it is created rather than by making the lists know about each other.

## On a machine with no organisation

ADR 0004 says a machine is either the person's or managed, and most will be the
person's. There, **the bound is simply absent** — not empty, not permissive by
default, absent — and the person chooses freely. No `/etc/alo/agentd.toml`, no
policy, no administrator to name in a refusal.

That is the common case and it must not be made to feel like the exception: a
personal machine does not get a policy file full of `true`, because a file full
of permissions is a thing somebody can later fill with prohibitions without the
person noticing it arrived.

## What we rejected

**One file with precedence rules.** It reads simpler and it fails the promise:
the person's choice becomes something an administrator can see and overwrite in
the same place they set policy, and the boundary between *bounding* and *acting
as* disappears into a merge order.

**The organisation sets a default, the person overrides it.** A default *is* a
choice, made by whoever set it, and ADR 0008 gives that choice to the person.
This also quietly answers the question on a machine where the person has not
chosen yet — which must stay unanswered and visible, because
`agentd.nothing-answers-questions` is the true sentence about a machine nobody
has configured, and it is better than a guess.

## Consequences

- `alo-agentd` reads two files rather than one, and the failure to find either
  is not an error: no policy means unbounded, no choice means nothing answers
  and the machine says so.
- A new crate or module owns the person's settings, in `alo-appearance`'s shape
  — *what the release ships kept apart from what the person changed* — because
  that separation is the same one, and it already exists there.
- `docs/features.md`'s settings surface gains the person's file; the
  organisation's is already ADR 0004's.
- Item 21h is unblocked, and what remains in it is wiring rather than deciding.
