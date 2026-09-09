# Connections come and go

- Date: 2026-09-09
- Workstream: model selection and configuration (`alo-secrets`)
- Contributor: Claude Code
- Task: Connection lifetime, concurrent retrievals, logout
- Status: **measured.** The answer is not libsecret's, and it was not assumed to
  be either way.

## The question ADR 0022 left open

The ADR records a **shared-singleton** limitation and is careful to record it as
*libsecret's*: `secret_service_get_sync()` hands back one service proxy for the
whole process, so dropping a wrapper is no evidence the bus connection closed.
The 2026-09-08 amendment replaced libsecret with a client this crate constructs
per connection, and said in as many words that this makes connection lifetime
**something to measure rather than inherit**.

This is that measurement.

## What was measured

Counted by asking the **bus** rather than the crate: a unique name — `:1.7` and
the like — is one per client connection, so `ListNames` counts connections. The
counting connection is opened once and kept, because counting from a connection
made and dropped per count would measure the measuring.

### Each keyring is a connection of its own, and giving it up closes it

Four `TheKeyring`s opened at once are **four** connections on the bus. Dropping
them returns the count to exactly where it started.

**So there is no shared singleton here**, and dropping a `TheKeyring` really does
stop holding a session on the person's bus. That is the opposite of the
libsecret behaviour, which is why the ADR refused to inherit the answer.

The count is polled for a bounded moment rather than read instantly: a
connection closing is the client's socket shutting *and* the bus noticing, which
is two things. Asserting immediately would have made it a test about scheduling.

### Eight retrievals at once each get the key

Eight threads, each opening its own keyring and asking for the same reference
against one `gnome-keyring-daemon`. All eight succeed. Since the encrypted
session is negotiated per connection, this is also eight `Dh` handshakes at once.

### A keyring whose session ended refuses rather than answering

Logout is `logind` taking `/run/user/<uid>` away, and what a daemon holds is a
connection to a bus inside it. The test proves the key is reachable **while
signed in**, then ends the session exactly as logout does, then asks again:

- it **refuses** — a keyring does not go on serving a person who has gone;
- it refuses **promptly**, which is asserted with a loose bound. A stale
  connection that waited out a method timeout would be a daemon that appears to
  have hung, and this crate has already been bitten once by exactly that.
- and it hands back **no key**.

## What this changes, and what it does not

Nothing in the production path changed. This is three tests and a fact that was
previously unknown.

**Proposed for ADR 0022, not applied here.** Its consequences section poses the
lifetime question and leaves it open; it can now record the answer — *a client
constructed per connection is one connection per client, closed when dropped;
the shared-singleton limitation is libsecret's alone and does not apply.* That
is recording a measurement rather than changing a decision, but the ADR is
accepted and this report will not edit it unasked.

The same-process limitation the ADR records is **untouched and still true**: code
running inside `alo-agentd` can ask for a key while a lookup is legitimate. What
protects the credential is the two logins, not the lifetime of a connection.

## Still open

Authenticated HTTPS acceptance remains blocked on the trust-anchor decision in
`the-daemon-fetches-and-connects.md` — A (enable `platform-verifier`, recommended),
B (an operator-configurable extra root), or C (leave it, and keep the narrower
claim). Nothing here depends on that answer.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.

**docs/autonomy/QUEUE.md** — credential store: lifetime, concurrency and logout
measured; only the HTTPS trust decision outstanding.

**docs/autonomy/STATE.md** — a keyring is one bus connection per client and
closes when dropped, so libsecret's shared-singleton limitation does not apply
here; eight concurrent retrievals all succeed; a keyring whose session has ended
refuses promptly and hands back no key.
