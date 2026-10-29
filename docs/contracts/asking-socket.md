# Contract — what a person's shell sends their agent

Status: public surface. The socket and its first line are
[ADR 0109](../decisions/0109-a-persons-words-reach-the-agent-through-a-socket-systemd-opens-as-the-agent.md)
(2026-10-14); the folders line was added on 2026-10-24; the earlier
conversation was added on 2026-10-28 by
[ADR 0138](../decisions/0138-a-follow-up-is-sent-with-the-conversation-so-far.md).
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

One JSON object on one line after the marker, holding the turns and nothing
else:

```text
alo-earlier:{"turns":[{"person":"list my invoices"},{"alo":"march.pdf and april.pdf"},{"proposed":"Move march.pdf to Paid","answer":"allowed"}]}
```

| Field | Means |
|---|---|
| `turns` | **Everything** said in that task window before the follow-up, oldest first. Each is exactly one of `{"person": …}`, `{"alo": …}`, or `{"proposed": …, "answer": …}` |
| `answer` | `allowed`, `denied`, or `not-answered` — the last for a change that lapsed or was answered somewhere else |

It holds what the person saw in the window and nothing else: never what a
verb read during an earlier turn, unless the person's words or alo's answer
contained it. **Nothing is left out of it**: alo puts no limit of its own on a
conversation (ADR 0138); the model's is the only one, and a conversation too
long for the model is refused and said to be, never shortened.

**The line's bound, and its reason.** A line is at most **1 MiB**
(`alo_instructing::THE_LONGEST_LINE`), and the agent stops reading at that
bound rather than after it. The number is not chosen to fit a model: it is the
daemon's own bound on a request (`alo_protocol::LONGEST`, *Bounds* in
`docs/contracts/daemon-protocol.md`), and the conversation can only ever reach
a model inside one such request, so a longer line could never be asked. 1 MiB
is roughly a quarter of a million tokens, beyond the context of every model
alo OS can be pointed at today — so in practice the model's limit is met
first, and is the one a person hears about. The shell checks before sending
(`TheConversationSoFar::fits_in_a_question`, counted as JSON with room for the
instructions) and says the conversation is too long to send rather than send
something the daemon would refuse.

**When.** Only when the person sends a reply in a task window, built at that
moment. A reply sent while alo is still answering, or while a change waits for
Allow once or Deny, is held in the window and sent by itself once alo has
answered and nothing waits — built then, from the conversation as it then
stands. A shell never sends it on a timer, a focus change or a keystroke.

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
  kind not listed here, and a line over the bound. A refusal is written
  to the agent's error output; the shell sees the connection close with nothing
  answered and tells the person the request stopped without finishing.

## Versioning

**Additive only.** A new line goes after the last one a current agent reads,
never before the request, because an older agent reads the request from the
position it knows and never reads past it. That is what happens to line 3
today: an agent written before 2026-10-28 reads lines 1 and 2, answers the
follow-up as a first request, and never sees the conversation.

## Stop

*Added 2026-10-29, [ADR 0139](../decisions/0139-stop-reaches-alo-and-cancels-what-it-sent.md).*

**A shell that shuts its sending side while the turn is running asks the agent
to stop.** The shell writes nothing more after the request in any case, so the
half-close is the whole message. The agent then asks the daemon for nothing
more, shuts its own sending side to the daemon (which cancels a question on its
way to a provider; `docs/contracts/daemon-protocol.md`, *Stop*), and exits.
**The agent's side closing is the shell's only evidence that alo stopped**, and
the only moment a window may say so.

- **A caller that shuts its sending side as soon as it has written** — `alo-ask`
  does — has said *that was all*, and is never read as stopping: the agent sees
  that half-close already there where the third line could have been.
- **After a proposal**, the shell holding the connection open for the person's
  answer stops the turn the same way; the change on the card lapses with the
  turn and is never answered by a Stop.
- **An agent written before this** reads the half-close as the person having
  answered, and ends the turn too.
