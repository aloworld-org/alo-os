# ADR 0139 — Stop reaches alo, and cancels what it sent

**Status:** **accepted, 2026-10-29, by the owner**, who chose *Stop alo and
cancel the request* over stopping between steps alone.

**Context:** `docs/features.md`'s task window (*Stop asks for cancellation at
once and says when the work actually stopped, naming the changes already made*)
and *closing a running task asks*; law 1; *one approval causes exactly one
execution*;
[ADR 0109](0109-a-persons-words-reach-the-agent-through-a-socket-systemd-opens-as-the-agent.md)
(the asking socket); `docs/contracts/asking-socket.md`;
`docs/contracts/daemon-protocol.md`; #990, which found that nothing on the road
from the shell to the agent could stop a turn.

## What was found

A task window draws a Stop control and nothing could press it to any effect:
the shell's thread writes the request and waits, `alo-agent` (one process per
request, running as the agent's own user, which the person's shell cannot
signal) drives the turn to its end, and the daemon protocol has no way to
cancel a question already on its way to a model. A Stop built only in the
window would say *stopped* while a provider was still being asked — the one
thing the promise forbids.

## The decision

**Stop travels the road the request did, and each hop ends what it holds.**

1. **The shell half-closes the request's connection.** A new meaning on the
   asking socket, additive: a shell that shuts its sending side while the turn
   is running asks the agent to stop. A shell never did that before, and an
   agent from before this reads it as the person having answered, which ends
   the turn too.
2. **`alo-agent` watches for it**, between rounds and while a question is out.
   When it comes, the agent asks for nothing more and half-closes its own
   connection to the daemon.
3. **The daemon cancels a question in flight.** While it is putting an
   agent's question to a model, it watches that agent's connection; when the
   agent half-closes, the connection to the provider is shut at once, so
   **nothing more leaves after Stop**. The departure ends, so the egress
   indicator goes dark, and the record says the question was stopped. This is
   a new meaning on the agent's door (a half-close during a question), not a
   new message, so no client changes and nothing older breaks.
4. **The window says *stopped* only when alo has stopped**: when the agent's
   side of the connection closes after the Stop, never at the press. Between
   the two it says *stopping*.

### What Stop never does

- **It never answers a change.** A change waiting for Allow once or Deny when
  Stop is pressed is neither approved nor declined: the agent's turn ends, the
  change lapses with it, and the card says it was not answered. One approval is
  still exactly one execution.
- **It never undoes what already ran.** A change that was allowed and carried
  out before Stop stays done, and the window names it.

### Closing a running task

Closing the window of a task alo is still working on now offers **Stop and
close** and **Cancel**. *Stop and close* is Stop, and the window closes once alo
has stopped; *Cancel* closes nothing. The third choice the features list names,
**Keep running**, waits for the right-hand panel's place for a task, which is
not built; it is not offered rather than offered and broken. The design has not
drawn the sheet these choices sit on, so they are drawn as a card in the
conversation, in the approval card's shell, with its two answers — a deliberate
departure, recorded here so the design can replace it.

## A model on this machine

A question to the local runtime is not cancelled by this: it leaves nothing,
and the runtime finishes and its answer is dropped. The window still says
*stopped* only when the agent has gone.

## Costs

- The daemon runs one watcher beside a question in flight. It touches nothing
  of the machine: it reads whether the agent hung up, and shuts one socket.
- A provider that had started answering is cut off; whatever it charges for a
  request it began is charged.
