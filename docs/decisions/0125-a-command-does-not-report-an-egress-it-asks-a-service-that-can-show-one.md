# ADR 0125 — A command does not report an egress; it asks a service that can show one

**Status:** **proposed, 2026-10-18, by the Mac lane. The owner decides.**
Nothing is built in the change that adds this.
**Context:** law 1; [ADR 0008](0008-where-inference-happens.md) (*a hosted API is
a supported first-class option and never a silent fallback*);
[ADR 0109](0109-a-persons-words-reach-the-agent-through-a-socket-systemd-opens-as-the-agent.md)
(the socket a person's process may open);
[ADR 0124](0124-one-command-asks-alo-from-the-terminal-and-asking-means-acting.md),
whose provider half this blocks; `docs/contracts/daemon-protocol.md`;
`docs/contracts/native-egress-indicator.md`; `crates/alo-egress`,
`crates/alo-ask`, `crates/alo-agentd`, `crates/alo-agent`.

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

## The decision proposed, in one line

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
  answer. This ADR proposes a request the daemon answers *for the person*,
  beside the agent road rather than through it. **Marked because it is the part
  most likely to be got wrong by whoever builds it.**
- **What `alo-ask` does today, which is worse than refusing.** Measured the same
  day: with a provider in their settings and one local model installed, a person
  is answered locally and told only *on this machine*. Whether that is corrected
  by refusing, by saying plainly that the chosen provider was not used, or by
  waiting for this road is a trade between ADR 0008's *never silent* and law 5's
  *never withdraw* — the owner's, and recorded as task 11 of
  `docs/autonomy/the-terminal-in-full-plan.md`.

## The test that decides whether this works

A person with a provider in their settings types a question at a shell, and the
answer comes back **with the egress on the indicator while it is in flight and
an entry in the record afterwards** — on a machine, with the daemon running.
And with the daemon stopped, the same person is refused in words and nothing
leaves. The second half is the one worth writing first: it is the half that
proves the indicator was not optional.
