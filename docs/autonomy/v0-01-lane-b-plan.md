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
