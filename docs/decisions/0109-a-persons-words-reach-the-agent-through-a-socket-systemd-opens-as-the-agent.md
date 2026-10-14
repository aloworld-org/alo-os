# ADR 0109 — A person's words reach the agent through a socket systemd opens as the agent

**Status:** proposed, 2026-10-14, by the Mac lane.

## What is stuck

`docs/features.md` and `ROADMAP.md`'s first launch item promise *a person asks
the agent in plain words, and the agent does it.* Everything behind that works
and is measured: a turn, a sentence a person approves, the verbs, and since
#682 a process that drives one request to its end.

**And a person cannot reach it.** The composer they type in is drawn and reads
keys; nothing joins it to the agent. Today the only road is a terminal:
`alo-agent "rename march.pdf to march-final.pdf"`.

## The thing in the way is not wiring

The shell runs as **the person** (`alo-desktop.service`, `User=alo`). The agent
is **its own Unix login** — ADR 0001 §5 — and that is load-bearing rather than
tidy: `docs/contracts/daemon-protocol.md` decides which of the two doors a
connection is on by `SO_PEERCRED`, and *a machine on which the person and the
agent are one login has no socket at all — the daemon refuses to start*, because
both doors would be one and **the side that proposed a change could approve it.**

So the shell cannot run the loop itself. Nor can it start a process as the
agent: it is not root, and the daemon is not either (ADR 0001 §2 — it refuses to
run as root, and runs as `alo`).

**Whoever starts the agent as the agent has to be something with the privilege
to change user.** On this machine that is exactly one thing already trusted with
it, and it is not ours: systemd.

## The decision

**A socket the person may write to, which systemd accepts and hands to one
`alo-agent` process per connection, running as `alo-agent`.**

```
/run/alo-asking/socket     SocketUser=root  SocketGroup=alo  SocketMode=0660
alo-agent@.service         User=alo-agent   Accept=yes       StandardInput=socket
```

- **The person writes one line**: their request, as they typed it.
- **systemd accepts** and starts `alo-agent@<n>.service` as the agent's user,
  with the connection as that process's input and output.
- **The agent drives one turn** and writes back what happened — a sentence that
  is waiting, an answer in words, or a refusal.
- **The connection closes.** One request, one process, one turn.

### Why socket activation and not a service that waits

`alo_agent::AtTheDaemonsSocket`'s own rule is that **one connection to the
daemon is one turn**, because `daemon-protocol.md` says *nothing names a turn* —
which turn a message belongs to is answered by the connection it arrived on. A
long-running agent would either hold one turn open with nobody asking anything,
or manage several and need a name for each, which is the number an agent could
change.

`Accept=yes` makes the one-process-per-request rule systemd's to keep rather
than ours to remember. It is also how `alo-convertd` already works here
(`Accept=no` there because the converter wants the listener itself), so this is
a second instance of an established arrangement rather than a new one.

### Why this is not a way for anything to make the agent act

**Only the person's group may open it**, by the mode, checked by the kernel
before a byte is read. That is the same protection `daemon-protocol.md` relies
on one door along, in the same direction: there, the agent's group may reach the
daemon; here, the person's group may reach the agent.

And it grants nothing. What arrives is **one line of text** — a request, not a
command. The agent reads it, shows it to a model, and asks the daemon for verbs
by name with typed arguments. Law 2 is untouched: there is no field for a
command to arrive in, at either door.

### Which person asked is the kernel's answer

A machine can have two signed-in people, and the agent must drive the request
against the **asking** person's grants. Getting that wrong is the one mistake
here nobody could detect from the outcome: the turn would succeed, the record
would be the other person's, and the answer would look right.

With `Accept=yes` the agent holds the connection, so it asks `SO_PEERCRED` for
the uid at the other end and opens `/run/alo/<uid>/agentd.sock` — **that**
person's daemon. Not a path it was told, not a directory it guessed. This also
settles a question #682 left open: its `--at` discovery refuses when it finds two
doors, and under this road it never has to choose, because the kernel already
said whose words these are.

## What this does not decide

**Not what the composer looks like while a turn runs.** A person who has pressed
Enter is owed something on the screen, and what that is belongs to the design
file and `docs/design/`. This decides the road, not the drawing.

**Not a conversation.** One line is one request, and a turn ends at a sentence
waiting for an approval. Whether a person's next sentence continues the last
turn is a separate question with ADR 0001 §4's answer behind it — *an approval is
never a session* — and nothing here presumes it.

**Not the terminal road's removal.** `alo-agent <request…>` keeps working, for
whoever is building or debugging. A machine that offers only a graphical road is
a machine nobody can diagnose.

## What it costs if it is wrong

If the owner would rather the daemon started the agent, the change is to give
the daemon a privileged helper that can change user — which is a new privileged
component, and ADR 0001 §2's whole argument is that there is one of those and
adding a second is expensive. This decision is the version that adds none.

## Not relitigated

ADR 0001 §2's unprivileged daemon and §5's separate agent login,
`daemon-protocol.md`'s two doors and its `SO_PEERCRED` rule, and law 2's
enumerated verbs. This asks for no exception to any of them and adds no
privileged code.
