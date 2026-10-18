# ADR 0125 — A command does not report an egress; it asks a service that can show one

**Status:** **accepted, 2026-10-18, by the owner.** Proposed the same day by the
Mac lane. Nothing is built in the change that accepts it; what the owner's
answer adds is in *What the owner decided* below.
**Context:** law 1; [ADR 0008](0008-where-inference-happens.md) (*a hosted API is
a supported first-class option and never a silent fallback*);
[ADR 0109](0109-a-persons-words-reach-the-agent-through-a-socket-systemd-opens-as-the-agent.md)
(the socket a person's process may open);
[ADR 0124](0124-one-command-asks-alo-from-the-terminal-and-asking-means-acting.md),
whose provider half this blocks; `docs/contracts/daemon-protocol.md`;
`docs/contracts/native-egress-indicator.md`; `crates/alo-egress`,
`crates/alo-ask`, `crates/alo-agentd`, `crates/alo-agent`.

## What the owner decided, 2026-10-18

**Accepted**, and the reason they gave widens it. Asked whether a short-lived
command should ask the daemon, they answered about
[ADR 0095](0095-the-release-carries-no-model-and-a-person-brings-their-own.md) —
the release carries no model, so a fresh machine has nothing to answer with:

> *if the user sets up in settings that he will use the open ai apis and then no
> need for all these if the use chooses for himself*

**Read as: the person's configured provider is to be used, and alo is not to
deliberate about it.** That is law 5 — *the person chooses* — applied to the
case ADR 0095 creates. A machine that ships no weights and then will not use the
account its owner configured is a machine that answers nobody. So this road is
not a precaution to be weighed against the person's choice; **it is how their
choice gets honoured at all.**

### What that does not change, said here because it is the obvious next step and it is wrong

**The indicator stays on.** It is not a protection the answer above switches
off, and law 5 says so in its own words: two kinds of protection stay on at
every setting, and the first is *those that restrict nothing (the record, undo,
the egress indicator, plain warnings)*. The indicator restricts nothing. It
costs the person nothing, refuses them nothing, and is the one thing that makes
*nothing leaves silently* checkable rather than promised. **A person who chose
OpenAI has chosen that their question leaves; they have not chosen to be unable
to see it leave**, and nobody offered them that — see law 5's own *never
offering is not taking away*.

So the person's choice decides **whether** a question leaves and **where it
goes**. This ADR decides **who shows it and who writes it down**, which is a
different question and is not theirs to have to think about.

### And it settles where a person's own sealed-box run writes its record

Asked separately where a command a person runs in a sealed box writes the record
it cannot run without — the machine's one record, which needs something
privileged, or a record of the person's own — the owner took this lane's
recommendation: **the machine's one record, by this ADR's principle.**

The two questions are the same shape, and this ADR's one-line decision already
covers it: *it asks a long-lived service … and that service shows it **and keeps
the record**.* So `try ./install.sh` asks `alo-agentd`, the daemon creates the
box, runs the command, and writes the record — rather than a second record file
in the person's home directory.

**What that keeps.** `docs/contracts/record-file.md`'s rule, *two files, because
a record file has one writer*: a person's run joins the record that already has
a writer instead of starting a fourth. And law 1's record stays one thing to
read rather than one thing per process.

**What it costs**, which is the honest half: the command's output has to travel
back from the daemon to the person's terminal, and `try` cannot run anything
while the daemon is down — the same cost this ADR already accepts for a
provider. **And the daemon runs it *for the person*, never as the agent**, which
is the distinction law 2 turns on: *the law binds the agent, never the person*.
This ADR's own *what this does not decide* already says a person's question must
not become an agent turn, and a person's command must not either.

## The question

`alo-ask` asks this machine and nowhere else. `docs/features.md` promises *a
provider they have an account with*, ADR 0008 says *most people will not* run a
local model, and [ADR 0095](0095-the-release-carries-no-model-and-a-person-brings-their-own.md)
ships no weights — so the command cannot answer for the ordinary fresh machine.
The transport exists and is not the problem. **Law 1 is**: an egress must be
*visible at the moment it happens and afterwards in a record*, and a command
that exits in a second can show nothing and keep nothing.

So: **how does a process that is neither the compositor nor a long-lived service
cause an egress?**

## What was measured, 2026-10-18

Each row was read by hand after a script got two of them wrong — the script's
three-line attribute lookback mis-attributed a neighbouring `#[cfg(test)]`, and
its first answer counted doctests as live code.

| asked | answer |
|---|---|
| live callers of `Asking::to_a_provider` | **one** — `alo-turn/src/asking.rs:249`, which takes an `indicator` from whoever drives the turn |
| live `Indicator` in a long-lived service | **`alo-agentd/src/starting.rs:250`**, inside `pub fn until_stopped`, whose own doc calls it *the single one law 1's surface is drawn from* |
| live `Indicator` in the compositor | **`alo-desktop/src/main.rs`**, twice |
| `alo-agent`, the per-turn process | **7 files, 1070 lines, zero mentions of `Indicator`** — and no `alo-egress` dependency at all |
| how `alo-agent` causes model work without one | it **asks the daemon** over a socket, one connection per turn (`AtTheDaemonsSocket`, ADR 0109) |
| what `alo-ask` calls | `to_this_machine` only, at `main.rs:246`, with no indicator anywhere |

**So the precedent already exists and is load-bearing.** The short-lived process
in this product that needs a model does not report its own egress. It asks a
service that owns the indicator.

## The decision, in one line

**A short-lived process never reports an egress it caused. It asks a long-lived
service that holds the one indicator to do the work, and that service shows it
and keeps the record.** For `alo-ask`, that service is `alo-agentd`, whose
socket is already **one per signed-in person**.

## Why not a *report your egress* interface, which is the obvious alternative

**Because this design makes it impossible on purpose, and the reason is good.**
`alo_egress::Indicator::beginning`'s own words:

> Ask the policy, and — if it permits — begin showing this egress. **The two are
> one call because they must not be two:** an egress that was permitted and not
> shown is exactly the failure law 1 exists to prevent, and separating the
> decision from the display is how that failure gets written by accident. **The
> `Departing` that comes back is what a caller must hold to open the connection,
> and it is the only thing that means so.**

Permission to open the connection *is* what the indicator hands back. An
interface that let a process report an egress **after** opening one would split
the permission from the showing — the single failure the type was shaped to
refuse. It would also let any process on the machine assert that an egress
happened, or happened somewhere it did not.

`docs/contracts/native-egress-indicator.md` already declines to be that
interface in as many words: *no protocol, agent capability, D-Bus interface or
stored format changes … this document describes only the drawing surface.* That
was read as a gap while writing this. **It is a decision, and this ADR agrees
with it.**

## What this costs, said plainly

- **`alo-ask` cannot answer from a provider while `alo-agentd` is not running.**
  That is the honest consequence and it matches what the compositor already
  does: `direct_desktop`'s own rule is *a machine with nowhere to show one may
  not start one. The lane refuses the ask and the window says so.*
- **A local answer is unaffected and stays in the command.** It causes no
  egress, so it needs no indicator — which is why `alo-ask` works today and why
  this gap went unnoticed until something had to leave.
- **The daemon protocol gains a request.** It is a public surface and changes
  additively, so an older client keeps working.

## What this does not decide

- **The message shape.** That belongs to `docs/contracts/daemon-protocol.md`.
- **Whether a person's question becomes an agent turn.** It should not — a
  question is not an agent acting, and routing one through the agent's turn
  machinery would put grants and an agent's record around a person reading an
  answer. This ADR specifies a request the daemon answers *for the person*,
  beside the agent road rather than through it. **The owner's answer about a
  sealed-box run puts a second thing on that road**, and the same rule holds
  for it: a command a person runs is not an agent acting either. **Marked because it is the part
  most likely to be got wrong by whoever builds it.**
- **What `alo-ask` does today, which is worse than refusing.** Measured the same
  day: with a provider in their settings and one local model installed, a person
  is answered locally and told only *on this machine*. Whether that is corrected
  by refusing, by saying plainly that the chosen provider was not used, or by
  waiting for this road was a trade between ADR 0008's *never silent* and law
  5's *never withdraw*.

  > **Answered the same day.** The owner chose a **setting the person sets** for
  > whether alo may use another model or API when the chosen one is
  > unavailable, and the interim sentence — *you chose {model} from {provider},
  > and this answer did not come from there* — landed as the one thing true
  > under either answer. The setting itself is not built. Recorded as task 11 of
  > `docs/autonomy/the-terminal-in-full-plan.md`, which this bullet still points
  > at for the rest.

## The test that decides whether this works

A person with a provider in their settings types a question at a shell, and the
answer comes back **with the egress on the indicator while it is in flight and
an entry in the record afterwards** — on a machine, with the daemon running.
And with the daemon stopped, the same person is refused in words and nothing
leaves. The second half is the one worth writing first: it is the half that
proves the indicator was not optional.
