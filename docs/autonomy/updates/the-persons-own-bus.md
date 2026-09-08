# The person's own bus

- Date: 2026-09-08
- Workstream: model selection and configuration (`alo-secrets`)
- Contributor: Claude Code
- Task: Record ADR 0022's acceptance and implement the backend, step one
- Status: **ADR 0022 is accepted and recorded.** The first backend step is built
  and tested. The lookup itself is blocked on a build-machine dependency, named
  below.

## Acceptance recorded

[ADR 0022](../../decisions/0022-where-a-providers-key-is-kept.md) is **accepted**
as of 2026-09-08: a provider's key lives in the **Secret Service**, and
`alo-agentd` reaches it over the person's own session bus at
`/run/user/<uid>/bus`, derived from the daemon's own uid.

Written into the ADR alongside it, so it cannot be read wider later: **no release
tier moves, no interim store, no credential-transfer protocol, no ADR 0017
amendment, and no acceptance of ADR 0021.**

## Step one: where the bus is, and whether it is the person's

`crates/alo-secrets` — Linux only, like `alo-bounding`, because everything in it
is a Unix socket and a uid.

**`TheBus::of(uid)` takes a uid and nothing else.** There is no parameter through
which `DBUS_SESSION_BUS_ADDRESS` could arrive, which makes *the environment does
not choose the daemon's keyring* a property of the signature rather than a
promise in a comment. That matters because the agent is a different login
(ADR 0001 §5) and a variable is something a session sets: on a machine where
anything an agent influenced reached this decision, the agent would be choosing
which keyring the daemon opened.

The bus is per **user**, not per session, so a person signed in on two seats has
one bus and *which session* is a question this crate never has to get wrong.

**Three checks, each with a reason:** it is there (before sign-in `logind` has
made nothing); it is a socket (a regular file at that name is a machine that is
wrong, and opening it would be believing a name); and **it is that uid's** —
asked with `symlink_metadata`, of the name itself, because following a symlink
and then checking the far end answers a question about the wrong file.

**`TheBus::as_an_address()`** is what a client is *handed*. ADR 0022's *verify the
bus actually selected* starts here: a binding given an address cannot silently
use another one. It is finished when a library is holding it, which is the next
step.

`NotStored` carries the four states — unavailable, locked, missing, denied —
told apart because they are four different things for a person to do about, and
with `nothing_was_sent` as a method rather than a comment.

## Evidence

`crates/alo-secrets`, **8 tests, all passing**:

| Test | What it proves |
|---|---|
| `where_a_bus_is_comes_from_a_uid_and_nothing_else` | the path is the uid's. **On this machine `DBUS_SESSION_BUS_ADDRESS` really is set — to root's bus — and reaches none of it**, which is the assertion that fails the day somebody reads a variable here |
| `a_bus_that_belongs_to_somebody_else_is_refused` | **cross-user refusal** — a socket owned by one uid is refused when asked whether it is another's |
| `a_uid_that_has_not_signed_in_has_no_bus` | **before login** — `Unavailable`, through a temporary path and through the real `/run/user` |
| `a_name_that_is_not_a_socket_is_not_a_bus` | a regular file at that name is refused |
| `a_symlink_pointing_at_a_socket_is_not_a_bus` | the check is on the name, not on what it points at |
| `a_client_is_handed_this_bus_rather_than_left_to_choose` | the address handed outward is this bus |
| `no_refusal_here_has_already_sent_anything` | all four states, in a match with **no wildcard** — a fifth fails to compile it |
| `the_four_are_four_and_not_one` | the four are told apart rather than collapsed |

Gates: `cargo fmt --check`, `clippy --workspace --all-targets -D warnings`,
`cargo test --workspace`, `RUSTDOCFLAGS="-D warnings" cargo doc`, the
supervisor's own tests, and the BPF target's fmt and clippy on the pinned
nightly, with kernel tests taking the shared lock.

## The blocker for step two, and it is not a decision

**`libsecret-1` is not installed on the machine this repository is gated on.**
Measured: `pkg-config --exists libsecret-1` answers no; `glib-2.0` is present.

Adding the dependency before the C library is there would break `cargo build` for
**everybody sharing that machine**, including the desktop worker's gate runs. So
it is a coordination step: the library has to be installed on the build machine
before the binding is committed. Nothing was stubbed in the meantime — there is no
placeholder implementation and no trait with an empty body, because what is
finished is what shipped.

## Coordination with the desktop worker

Three things, none of them started here and none of them edited in their
checkout:

1. **`libsecret-1` on the build machine** — needed before the binding can be
   committed and gated. This is the immediate one.
2. **A Secret Service package and unit in the image** — the store has to exist on
   a machine for the daemon to reach it.
3. **Sign-in**, so there is a session with something to unlock the keyring with.

Items 2 and 3 belong to their delivery phase; item 1 blocks the next backend step
here.

## What is preserved, and the limitations kept explicit

- **Agent and daemon identity separation** — untouched. `SO_PEERCRED` and two
  logins are what stop the agent reaching a credential, and
  `the_four_things_this_daemon_says_to_an_agent_carry_no_credential` (`97bd490`)
  holds the protocol half down.
- **Socket permissions** — `/run/alo/<uid>/agentd.sock` and every check in
  `place.rs` are untouched; nothing here goes near the inbound door.
- **Redaction** — `41c9f1e` stands; nothing in `alo-secrets` renders a key, and
  `NotStored` deliberately carries neither a credential nor what a store said.
- **Refusal without fallback** — four states, and `nothing_was_sent` says so for
  all of them.
- **Same-process limitation, kept explicit**: anything running as the person can
  retrieve the credential, including a compromised `alo-agentd`. `Secret`'s shape
  prevents accidents, not an attacker with code in the daemon.
- **Shared-connection limitation, kept explicit**: libsecret's service proxy is a
  shared singleton on a process-wide GDBus connection, so the daemon will hold a
  bus connection across turns — the case
  `a_connection_made_before_the_boundary_stays_usable_inside_it` already
  reproduces. Neither limitation is hidden by anything in this crate.

## Still to do, and unchanged from the acceptance plan

Authenticated HTTPS through the production daemon path — owned TLS server, a real
store, synthetic credentials, credential asserted **on the wire** — plus the
locked, missing and denied states against a real store, concurrent retrievals
over the shared connection, logout behaviour, and the measurement of what the bus
connection actually does. All of it waits on step two.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible yet.

**ROADMAP.md** — no tick, **no tier moved**.

**docs/autonomy/QUEUE.md** — the credential-store item is unblocked as a decision
and blocked as a dependency: `libsecret-1` on the build machine.

**docs/autonomy/STATE.md** — three facts. ADR 0022 is **accepted**: the Secret
Service over `/run/user/<uid>/bus`, derived from the daemon's own uid. Step one is
built and tested — where the bus is, whether it is the person's, and the four
refusal states, with cross-user refusal and environment-independence asserted.
And step two is blocked on `libsecret-1` being installed on the shared build
machine, which is a coordination item rather than a decision.
