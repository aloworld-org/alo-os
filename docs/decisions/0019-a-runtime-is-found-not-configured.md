# ADR 0019 — A runtime is found, not configured; and the person's list is the person's

**Status:** accepted — answers what
[`docs/autonomy/QUEUE.md`](../autonomy/QUEUE.md) item 21k could not build
without, and keeps [ADR 0006](0006-ollama-is-the-pinned-model-runtime.md)'s
one-file rule and [ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md)'s
bound-versus-choice line intact
**Date:** 2026-09-05
**Context:** item 21k; `crates/alo-choosing`, `crates/alo-models`,
`crates/alo-turn`; `docs/contracts/machine-description.md`

## The decision in one line

**A local runtime is discovered by its adapter at an address the adapter alone
knows** — no contract, description or settings file names one. The person's
brought weights are listed in the person's own settings, and the daemon reads
those from its own environment **because it runs as that person**.

## The gap this closes

`alo_choosing::Chosen` says *this model, from this list*. `alo_turn::Turning::asking`
wants an `Answers`. Between them, three facts nothing on a machine states:

1. where a runtime on this machine is,
2. where weights somebody brought are,
3. whose settings are being read.

Item 21k could build all of it once those are decided, and correctly refused to
guess at the first, because guessing wrong there breaks two settled ADRs at once.

## 1. Where a runtime is: found, never named

**The adapter finds it. Nothing else may name it.**

The tempting answer is a key — `runtime_address` in
`docs/contracts/machine-description.md`, set in `/etc/alo/agentd.toml`. It is
wrong twice over, and each reason alone is sufficient.

**It breaks ADR 0006's one-file rule.** That ADR says *nothing outside the
adapter names an Ollama endpoint* and asks that *a reviewer should be able to
find every mention of Ollama in one file*. An address key is an Ollama endpoint
sitting in a public contract, wearing a generic name. The rule would then be
true of the source and false of the system, which is the worst place for a rule
to be true.

**It breaks ADR 0016's line.** `/etc/alo/agentd.toml` is the organisation's, and
ADR 0016 has just finished saying an organisation sets a **bound** and never a
**choice**. *Which runtime answers* is a choice. Putting its address in the
organisation's file is the organisation choosing, one indirection away from the
thing ADR 0016 forbids.

So: `alo_models`'s adapter — the one file allowed to know Ollama exists — also
knows where Ollama listens on a machine that has one. **A local runtime is at a
local address**, and that is a fact about the runtime, not about this
deployment. There is nothing for an operator to configure and nothing for a
contract to carry.

**What a machine says when it finds none is unchanged**: `agentd.nothing-answers-questions`,
which item 21k rightly calls true rather than a placeholder. Discovery that
finds nothing is an answer.

**The escape hatch we are not building.** No override key, no environment
variable, no *advanced* address field. The moment one exists, the organisation
can point every person's agent at a machine of its choosing, and the egress
indicator (ADR 0003) would be telling the truth about a destination nobody
chose. If a deployment ever genuinely needs a remote runtime, that is a
**provider** — which alo already models, shows in the indicator, and lets the
person pick under a bound.

## 2. Where brought weights are: with the person

`Which::Brought` names an entry in `alo_models::Brought`, a list this machine
keeps nowhere. It goes in **the person's settings**, beside the model or
provider they chose — the `$XDG_CONFIG_HOME/alo/` file ADR 0016 established.

The weights are the person's: they fetched them, the licence they accepted is
theirs, and ADR 0016 already put *which model answers* in that file. A list of
weights somebody brought is the same kind of fact about the same person, and
splitting it into a second store would be inventing a second settings system for
one owner — the thing ADR 0016 declined to do.

The organisation's bound still applies. A brought model is a **local** source,
so a policy permitting only local answering permits it, and one forbidding
local answering forbids it. Nothing new is needed for that: the bound is about
where a question is answered, and the list does not change where.

## 3. Whose settings: the daemon's own, because the daemon is the person

`alo_choosing::where_it_is` takes both environment variables as arguments rather
than reading them, which item 21k notes was done for exactly this question.

The answer today is **the daemon's own environment**, because `alo-agentd` runs
as the signed-in person (ADR 0001 §2) and systemd starts one per login. The
process and the person are the same account, so `$XDG_CONFIG_HOME` from its own
environment *is* that person's.

**This is a condition, not an assumption, and it is written down so it can be
checked rather than remembered.** It holds only while one daemon serves one
login. The day anything serves two people from one process — a machine-wide
service, a session broker, a daemon that outlives a login — this becomes wrong
silently, and the wrongness is *one person reading another's choices*. That the
function takes both variables as arguments is what makes the fix a caller change
rather than a rewrite, and it should stay that way.

## What we rejected

**An address key in the machine description.** Breaks ADR 0006's one-file rule
and ADR 0016's bound-versus-choice line. Rejected on either ground alone.

**A key in the person's settings instead.** It keeps the organisation out of it,
and still puts an Ollama endpoint in a file a person edits — ADR 0006's rule is
about the system, not about who owns the file.

**Discovery by scanning ports.** A runtime is at a known address or it is not
there. Probing a range to see what answers is how a daemon ends up talking to
something that merely resembles a runtime.

**Keeping brought weights in the organisation's file.** They are the person's,
and an organisation that could add entries could point a person's agent at
weights it chose.

## Consequences

- Item 21k is unblocked and is now wiring rather than deciding: `Chosen` →
  `Answers` via the adapter's discovery for a catalogued model, and via the
  person's list for a brought one.
- `docs/contracts/machine-description.md` gains **nothing**. That is the point,
  and it is worth stating positively so a later reader does not add the key
  believing it was an oversight.
- The person's settings file gains the brought-weights list; ADR 0016's shape
  covers it and no second store appears.
- Items 21l and 26f, which waited on 21k, are freed with it.
