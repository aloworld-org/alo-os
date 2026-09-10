# v0.01, lane B — accounts and session entry

The second loop's plan. `docs/autonomy/v0-01-delivery-plan.md` is the spine of
v0.01; this file carves out the one chain in it that is independent of the
overlay work, so two loops in two checkouts can build at once without ever
choosing the same task.

Point a loop at it with `ALO_LOOP_PLAN=docs/autonomy/v0-01-lane-b-plan.md`, from
a checkout of its own. **Never from a checkout another loop is in** — the lock
forbids it, and the lock is right.

## Why a second file rather than a shared plan

The supervisor has no task-claiming: two loops reading one plan both select the
same next task, launch two workers at it, and produce two competing
implementations racing to publish. Partitioning by file is the claiming
mechanism — a lane owns every task in its file, and the main plan marks these
two as lane B's so the first loop steps over them.

**The cost is a protocol, and it is written here so it is not folklore:** when a
task finishes in this file, the same handoff marks the matching task done in
`v0-01-delivery-plan.md`, in the same commit. The main plan's task 10 depends on
session work it can only see there; a lane that finished quietly would leave the
image task waiting on work that is already done.

## Rules

Everything `v0-01-delivery-plan.md` holds itself to, unchanged: nothing ticked
from a fixture, no release verdicts, tasks sized to a forty-five-minute worker,
and whoever finishes a task writes the next one in the same change.

## Tasks

### 1. The local account that needs no tenant

**Status:** ready. **Depends on:** nothing.

Phase 4's smaller half, and the one the exit gate actually requires: v0.01
opens with *sign in*, and nothing in this repository signs anybody in. A local
account, against the machine's own store — no identity provider, no tenant, no
network. Those are the other half of phase 4 and are not this task.

- **Acceptance:** a local account is created and authenticated against the
  machine's own store; a wrong password is refused in words and is not
  distinguishable by timing from an unknown user; and the session that results
  carries the uid `alo-agentd` is told about, so the machine description and
  the running session cannot disagree about who is signed in.
- **Constraint:** nothing in `crates/alo-shell` — the entry *surface* is the
  compositor lane's; this is the account and the authentication it will call.

**Done, 2026-09-10.** `crates/alo-accounts`: the store at
`/etc/alo/accounts.toml`, Argon2id behind one refusal that costs the same for
a wrong password and an unknown name, and `Session::opened`, which cannot
carry a uid the machine description does not name. Measured in
`tests/a_person_signs_in.rs`; the report is
`docs/autonomy/updates/the-local-account-that-needs-no-tenant.md`. Task 2 is
the next task and was already written.

### 2. The daemon's environment is the session's

**Status:** ready. **Depends on:** 1.

`alo-agentd` runs as the signed-in person and finds their bus at
`/run/user/<uid>`. That is measured (`a_session_that_really_ended.rs`) and not
wired: nothing starts the daemon into a real session with the right environment.

- **Acceptance:** the daemon starts under the session task 1 creates, reaches
  that session's bus, and stops when the session ends — the three states already
  measured, reached from a sign-in rather than from a test harness.
- **On finishing:** mark tasks 4 and 5 done in `v0-01-delivery-plan.md`, in the
  same handoff, so the image task there stops waiting on work that is done.

**Done, 2026-09-10.** `crates/alo-entering` derives what a session hands a
process — `/run/user/<uid>`, the bus address, and `user@<uid>.service` — from
the `Session` task 1 opens, so the sign-in and the machine cannot spell it two
ways. `image/usr/lib/systemd/system/alo-agentd.service` is wired to that
session rather than to `multi-user.target`: pulled in by the person's own
manager, `BindsTo=` it so signing out stops it, ordered after it, and given the
two variables. `crates/alo-image` checks every one of those lines against the
number the machine description names, and `crates/alo-agentd/src/session.rs`
refuses at start-up an environment naming another login's session — it checks
the variables and still never reads one to decide where to connect. The three
states from a real sign-in are
`crates/alo-entering/tests/a_daemon_in_the_persons_session.rs`, `#[ignore]`d for
`a_session_that_really_ended.rs`'s reasons. Report:
`docs/autonomy/updates/the-daemons-environment-is-the-sessions.md`. Task 3 below
is the next task and was written in the same change.

### 3. Where a machine keeps its grants between one sign-in and the next

**Status:** ready. **Depends on:** 2.

`docs/features.md` promises for v0.01: *Grants: pick a folder, see what is
granted, revoke it, and it expires*. A person can now make one —
`crates/alo-picking` — and `alo-agentd` honours the ones it is holding. **Nothing
keeps them.** `crates/alo-agentd/src/starting.rs` says so in as many words: the
service begins with no grants at all, because where a machine's grants live is a
question nobody has answered, and a list read from a file would be a list nothing
writes. So a grant made this morning is gone at the next sign-in, and *see what
is granted* has nothing to show.

This is lane B's because it is session-shaped rather than surface-shaped: a
grant belongs to the person who made it, it is written under their own
authority, and its life is measured from one sign-in to the next. The
**surface** that lists and revokes them is the compositor lane's, as picking's
was.

- **Acceptance:** a grant a person made survives a restart of the daemon and is
  honoured afterwards; a revoked grant does not come back; an expired one is
  gone when it is read rather than being read and then filtered; the file is the
  person's alone and a store somebody else could write is refused in words, as
  `crates/alo-accounts`' store already is; and nothing an agent can send over the
  socket writes a byte of it.
- **Constraint:** no new surface, and nothing in `crates/alo-shell`. This is
  where the list lives and what may write it.

**Done, 2026-09-10.** `crates/alo-remembering`: `/var/lib/alo/grants.toml`, in
the folder the image already makes `0700` for the person, believed under
`alo-accounts`' three rules — not a link, root's or the person's, nobody else
able to write it — and replaced whole or not at all. Every grant is built again
by `alo_capability::Grant::checked` on the way in, so a file hand-edited into
granting `/` is refused by the crate that owns that rule; an expired one is
dropped before the list exists rather than filtered afterwards; and
`alo_capability::Grants::remembered` refuses a file whose handles would collide,
so a revoke cannot land on the wrong grant after a restart.
`crates/alo-agentd/src/main.rs` reads it once, before the socket exists, and
hands `starting::until_stopped` a value with no path in it — which is what makes
*nothing an agent sends writes a byte of it* the shape of the crate rather than
a rule. A file that is there and is not believable stops the service
(`NotStarted::NoGrants`); a machine that has simply never been granted anything
starts and refuses everything, as before. Measured in
`crates/alo-remembering/tests/the_grants_a_machine_keeps.rs` and in
`crates/alo-agentd/src/starting.rs`. Report:
`docs/autonomy/updates/where-a-machine-keeps-its-grants.md`. No task in
`v0-01-delivery-plan.md` matched this one, so nothing was marked there. Task 4
below is the next task and was written in the same change.

### 4. A grant made now reaches the daemon now

**Status:** ready. **Depends on:** 3.

Task 3 gave the machine somewhere to keep its grants, and `alo-agentd` reads
them when it starts. What it cannot do is hear about one made **while it is
running**: a person picks a folder at eleven, the file on the disk says so, and
the service holding the turn is still serving under the list it read at sign-in.
The gap is narrow and honest — the daemon is bound to the person's session, so
the grant applies at the next sign-in — but `docs/features.md` promises *pick a
folder*, and *pick a folder and sign out again* is not that promise.

This is lane B's for task 3's reason: the message is the person's, made under
their own authority on their own door, and its lifetime is the session's. The
**surface** that lists and revokes is still the compositor lane's.

The shape that keeps law 2 and ADR 0001 §5 true is a **knock rather than a
payload**: the person's side says only *the grants have changed*, and the daemon
re-reads its own file under the same believing rules. Nothing on the wire
carries a grant, a path or a duration, so a request that arrived from anywhere
else could still not widen anything — and the kernel already says which door a
caller is on.

- **Acceptance:** a grant made while the daemon is running is honoured in the
  same session without a restart, and a revocation takes effect on the next
  question asked; the request carries no grant, no path and no duration, so
  re-reading the person's own file is the only thing it can cause; the same
  request on the agent's door is refused in words and the refusal is written
  down; and a grants file that has become unbelievable while the service is
  running leaves the grants it already had rather than emptying them, with the
  refusal in the record — a machine that forgot what was granted because
  somebody chmodded a file is a machine that went silent.
- **Constraint:** additive to `docs/contracts/daemon-protocol.md`, which is a
  public surface (*contracts outlive code*); no new surface, and nothing in
  `crates/alo-shell`.
