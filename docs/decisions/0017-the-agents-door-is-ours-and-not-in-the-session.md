# ADR 0017 — The agent's door is ours, and it is not inside the person's session

**Status:** accepted — moves the transport in
[`docs/contracts/daemon-protocol.md`](../contracts/daemon-protocol.md), a public
surface
**Date:** 2026-09-04
**Context:** `docs/autonomy/QUEUE.md` items 21j and 21c, `docs/quirks.md` (the
measurement), `crates/alo-agentd`'s `place.rs` and `session.rs`,
[ADR 0001](0001-the-capability-model.md) §5 (the agent is a user of its own)

## The decision in one line

The daemon's socket moves out of the person's session directory to
**`/run/alo/<uid>/agentd.sock`**, whose parent the image creates and whose
per-person directory the daemon creates when a session starts and removes when
it ends.

## Why it has to move

The contract said `$XDG_RUNTIME_DIR/alo/agentd.sock`, and `place.rs` implements
that correctly: the directory is `0750`, the socket is `0660`, both handed to
the agent's group. **None of it is reachable.**

`logind` creates `$XDG_RUNTIME_DIR` — `/run/user/<uid>` — as **`0700`, owned by
the person**. The agent is a different user (ADR 0001 §5), so it is refused by
the *parent* directory before either of our two modes is ever consulted. A
correct `0750` directory inside a `0700` one is a locked room inside a locked
building.

This was found by running the process rather than by reading anything, and it
means the thing the contract describes has **never worked on a real machine**.
The person's own door works, because the person owns the building. The agent's
could not be reached at all.

## The shape of the answer

```
/run/alo/                 0755  root:root      the image makes it, at boot
/run/alo/<uid>/           0750  person:agent   the daemon makes it, per session
/run/alo/<uid>/agentd.sock 0660 person:agent   the socket
```

- **The parent is the image's**, through `tmpfiles.d`. It is one directory, it
  has no per-person knowledge in it, and it exists before anybody signs in.
- **The per-person directory is the daemon's**, made when that person's session
  starts and removed when it ends. It is named by `uid` because a machine may
  have several people on it and one person's door is not another's.
- **Every rule `place.rs` already enforces carries over unchanged**: the mode is
  set from the moment the directory exists, and the daemon refuses to start
  rather than use a directory that is a symbolic link, is not a directory, or
  belongs to somebody else. Whoever owns the directory a socket lives in can
  replace the socket, and every client would then be talking to them.

## Why we may make this directory when we would not make `/run/user/<uid>`

Item 21j rejected having the daemon create `/run/user/<uid>` itself, and that
was right: it would be **standing in for a session that has not started**, doing
`logind`'s job with none of `logind`'s knowledge about when it ends.

`/run/alo` is not `logind`'s. It is ours, it is named after us, and nothing else
on the machine has an opinion about what is in it. Creating our own directory in
response to a session that **already exists** is not inventing a session; it is
reacting to one. The distinction is the whole of why this is allowed and that
was not, and it is worth keeping in mind the next time something is tempting to
create early.

## What we rejected

**Leaving it in `$XDG_RUNTIME_DIR`.** It cannot be made to work without changing
the mode of a directory `logind` owns, on a machine where other things live in
it. Not ours to change, and it would weaken a boundary the rest of the system
relies on to fix one of ours.

**`/tmp/alo/…`.** `/tmp` is anybody's. A directory there can be created first by
somebody else, and the daemon would be refusing to start on a machine an
attacker has merely been present on.

**Running the agent as the person.** It would make every path problem here
disappear, and it would delete the boundary the whole design rests on:
`SO_PEERCRED` answers *which of two users is on this connection*, and with one
user there is no question to answer and no door to be on the wrong side of.

**A network port.** There is no port, no announcement and nothing to discover;
that sentence is in the contract on purpose and is not weakened by this change.

## The contract change

`docs/contracts/daemon-protocol.md`'s *The transport* section changes: the path
is `/run/alo/<uid>/agentd.sock`, and the names `alo` and `agentd.sock` remain
part of the contract as before.

**This is a break, and it is taken now because nothing has shipped.** alo OS is
pre-v0.01, no client outside this repository speaks this protocol, and there is
nothing deployed to deprecate against. The rule in `CLAUDE.md` — additive on
live surfaces, versioning and a deprecation note for a break — is not being
bent; it is that this surface is not live yet, and **this is the last cheap
moment to move it**. After v0.01 the same change would cost a version and a
migration.

## Consequences

- `place.rs` keeps its checks and changes where it points; `session.rs` stops
  reading `$XDG_RUNTIME_DIR` for this purpose. **Built on 2026-09-04**, queue
  item 21j, and `session.rs` turned out to have no other purpose: it is deleted
  rather than left reading a variable nothing asks about.
- The image gains a `tmpfiles.d` entry, which is the first thing this repository
  asks of the image for the daemon rather than for the desktop.
- The daemon gains a session lifetime it did not have: something must remove
  `/run/alo/<uid>` when the person signs out, and *the socket outliving the
  session* is now a thing that can go wrong and therefore a thing to test.
- Item 21j is unblocked. Item 21c's *a connection from a second user has never
  been made* stays true until this lands and somebody makes one — this ADR is
  why it was never made, not a claim that it now works.
