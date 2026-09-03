# ADR 0013 — The grant is enforced by the kernel, not promised by the daemon

**Status:** accepted — direction, ordered behind `alo-agentd` and the turn
**Date:** 2026-09-03
**Context:** [ADR 0001](0001-the-capability-model.md) (the capability model),
`crates/alo-capability`, `crates/alo-record`, `crates/alo-keeping`,
`crates/alo-egress`, `ROADMAP.md` (v0.01's *enforcement at the network
boundary*, v0.5's *undo what the agent did*),
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md),
`CLAUDE.md` ("engines are configured, never patched")

## The decision in one line

A grant stops being a rule the daemon agrees to follow and becomes a **boundary
the kernel imposes**: for the duration of one turn, everything outside the grant
is unreachable to the process doing the work — so a verb that tried to overreach
would fail at the syscall rather than be refused by our own code.

## What is actually wrong today

`alo-record` records what `alo-agentd` reports. That is an **audit log, not a
guarantee**, and the difference is the whole product.

If a verb has a bug, if an argument is mis-parsed, if the daemon is ever
compromised, the record faithfully writes down a lie — and it writes it in the
person's own language, with a grant id beside it, looking exactly like the
truth. Everything else in this repository is built on the premise that a person
can find out what happened on their machine. That premise currently rests on one
program's honest account of itself.

**Nothing about the design is wrong. The enforcement point is.**

## What this is not

**Not a model in the kernel.** Inference in kernel space has no memory
protection, no fault isolation and no upside; a crash would take the machine
down. The model stays exactly where it is.

**Not a kernel of our own.** *No kernel* remains a non-goal, for the reasons
`docs/features.md` already gives: a kernel is thirty years of device drivers,
and it buys nothing an agent needs.

**Not a patch to Linux.** Everything below uses Linux's own extension points
from userspace, which is `CLAUDE.md`'s *configured, never patched* exactly.

## The decision

**A turn runs inside a kernel-enforced boundary built from primitives Linux
already ships.** None of these is new; the composition is.

| Primitive | What it enforces |
|---|---|
| **Landlock** | the filesystem the turn can reach — the grant, and nothing else. Applied irrevocably to the process before any verb runs |
| **seccomp** | which syscalls exist at all for that process |
| **cgroup v2 + eBPF** | which sockets may be opened, and attribution of every one to the turn that caused it |
| **namespaces** | what the turn can see |
| **A snapshot at turn start** | what *undo* rewinds to |

Three consequences follow, and they are the point:

**1. The grant becomes unreachable rather than refused.** A verb given a path
outside its grant does not get a polite refusal from `alo-capability` — it gets
`EACCES` from the kernel. Our refusal stays, because a person deserves a sentence
they can read; but it stops being the only thing standing there.

**2. The record becomes an observation, not a claim.** Every file opened and
socket connected during a turn is attributed by the kernel to that turn's cgroup.
*What did the agent do this afternoon* stops being the daemon's account and
becomes what the machine watched happen.

**3. Undo becomes exact.** A snapshot taken when the turn begins makes *"undo
everything it did"* precise rather than best-effort, and it is the same
mechanism ADR 0011 already brings for the image.

## The boundary of the boundary, stated honestly

**This protects what `alo-agentd` does directly. It does not protect what an
application it drives does.** When an agent operates Blender or a mail client
through the accessibility tree, that application acts with its own permissions,
in its own sandbox, and our Landlock ruleset has nothing to say about it.

That is not a hole to be quietly hoped over. It is where ADR 0005 already
lives — applications are sandboxed as Flatpaks and reach the system through
portals — and the two boundaries have to meet: **the agent's reach is bounded by
the kernel; the application's reach is bounded by its own sandbox.** Anyone
implementing this must say, in the record, which of the two a given action went
through, because "the agent did it" and "the agent asked an application to do
it" are different claims about who could see what.

## Why this is worth doing

**It converts the central promise from a log into a guarantee**, and those sell
differently. *We record what the agent did* invites a security team to ask what
happens when the recorder is wrong. *The kernel would not have permitted
otherwise* does not.

**And it is the part that is genuinely hard to copy.** The code in this
repository is GPL and anybody may take it. Reproducing this requires engineers
who work comfortably in LSM, eBPF and Landlock — a skill set an
application-layer competitor hires over years rather than months, and one most
of them do not know they need until they try to make the same claim.

## Consequences

- **A new crate, and it is the daemon's floor rather than a library beside it.**
  Everything it does happens before the first verb of a turn runs.
- **Ordering: after `alo-agentd` and after the turn.** There is no turn to
  enforce yet. This ADR is a direction so that the turn is built with a boundary
  in mind rather than retrofitted into one — the same reason ADR 0005 was
  written before the first hostile page in `alo-browser`.
- **It gives v0.01's owed line its answer.** *Enforcement at the network
  boundary, without which all of this describes only the code that asked* — that
  is an eBPF program attached to the turn's cgroup, and now it is named.
- **It raises the floor for the kernel version**, which is a fact about the base
  (ADR 0011) rather than a new dependency: Landlock has been upstream since 5.13
  and gained network rules in 6.7. A Fedora-derived image is comfortably past
  both. `docs/hardware.md` gains a line if that ever stops being true.
- **A turn that cannot be sandboxed does not run.** If the boundary cannot be
  applied, that is a refusal, not a warning — the same rule `alo-egress` already
  follows when policy cannot be evaluated.

## Alternatives rejected

**Leave enforcement in the daemon.** Rejected: it is the current state, and it
makes the record only as trustworthy as the program writing it.

**A model, or agent logic, in kernel space.** Rejected on every ground —
stability, security, and no benefit whatsoever.

**Wait until somebody asks for it.** Rejected on ordering rather than merit. A
turn built without a boundary is a turn that assumes ambient authority in a
hundred small places, and retrofitting that is the expensive kind of rewrite.
This ADR exists now precisely so it is cheap later.

**Write our own LSM.** Not rejected, but not chosen: an out-of-tree security
module is a maintenance burden across kernel versions, and Landlock plus seccomp
plus eBPF covers what is needed today. If a gap appears that they genuinely
cannot express, that is a new ADR with the gap named in it.
