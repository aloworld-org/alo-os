# Contract — what a person's shell sends their agent

Status: public surface. The socket and its first line are
[ADR 0109](../decisions/0109-a-persons-words-reach-the-agent-through-a-socket-systemd-opens-as-the-agent.md)
(2026-10-14); the folders line was added on 2026-10-24; the earlier
conversation was added on 2026-10-28 by
[ADR 0135](../decisions/0135-a-follow-up-is-sent-with-the-conversation-so-far.md).
This document was written that day: until then the lines were described only
in code, in two places that held each other to nothing.

## The socket

`/run/alo-asking/socket`, opened by the person's shell, accepted by
`alo-asking.socket` and handed to one `alo-agent` running as the agent's own
user. **One connection is one request and one turn.** Which person asked is the
kernel's answer (`SO_PEERCRED`), never a field.

The shell is `crates/alo-shell`'s `what_a_person_asked_goes_to_the_agent.rs`;
the agent is `crates/alo-agent`'s `what_the_person_sent.rs`.

## What the shell writes

Up to three lines, each ending in a newline, written at once:

| Line | When | Shape |
|---|---|---|
| 1 | only when the person has granted folders | `alo-may-reach:` and the folders, separated by a tab |
| 2 | always | **the request**, in the person's own words, on one line — a line break the person typed is written as a space |
| 3 | always, from a shell that knows this line | `alo-earlier:` and the conversation so far, or nothing after the marker for a first request |

The folders are **a hint for the model, never a capability**: every verb is
still decided by `alo-agentd` against the grants it reads itself.

### The conversation so far

One JSON object on one line after the marker:

```text
alo-earlier:{"turns":[{"person":"list my invoices"},{"alo":"march.pdf and april.pdf"},{"proposed":"Move march.pdf to Paid","answer":"allowed"}],"left-out":0,"cut":false}
```

| Field | Means |
|---|---|
| `turns` | What was said in that task window before the follow-up, oldest first. Each is exactly one of `{"person": …}`, `{"alo": …}`, or `{"proposed": …, "answer": …}` |
| `answer` | `allowed`, `denied`, or `not-answered` — the last for a change that lapsed or was answered somewhere else |
| `left-out` | How many of the oldest turns were left out to stay inside the bound |
| `cut` | Whether the oldest turn kept was cut because it alone was longer than the bound |

It holds what the person saw in the window and nothing else: never what a
verb read during an earlier turn, unless the person's words or alo's answer
contained it.

**Bound.** The words of all the turns together are at most **8000 bytes**
(`alo_instructing::TheConversationSoFar::THE_MOST_SENT`), and the whole line
at most six times that and 4096 more, for JSON's escaping. The shell leaves the
oldest turns out whole to stay inside it and says so in the person's window; the
agent tells the model how many were left out.

**When.** Only for a follow-up the person sent with Enter in a task window,
built at that moment. A shell never sends it on a timer, a focus change or a
keystroke.

## What the agent does with it

- Lines 1 and 2 as before.
- Line 3, on a socket, is waited for **for half a second at most**, so a shell
  from before this line still works. On a pipe or a terminal it is not read.
- A conversation is put to the model **inside the question the turn asks the
  daemon**, between what the turn may reach and the request, marked as an
  earlier exchange that is not a request now. It reaches a provider only as that
  question does — through `alo-agentd`, under the egress indicator, into the
  record (`docs/contracts/daemon-protocol.md`).
- **Refused, in words, and nothing driven:** a third line that does not begin
  with `alo-earlier:`, one that is not the JSON above, a turn or answer of a
  kind not listed here, and a conversation over the bound. A refusal is written
  to the agent's error output; the shell sees the connection close with nothing
  answered and tells the person the request stopped without finishing.

## Versioning

**Additive only.** A new line goes after the last one a current agent reads,
never before the request, because an older agent reads the request from the
position it knows and never reads past it. That is what happens to line 3
today: an agent written before 2026-10-28 reads lines 1 and 2, answers the
follow-up as a first request, and never sees the conversation.
