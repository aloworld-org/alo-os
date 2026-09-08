# What the agent cannot reach

- Date: 2026-09-08
- Workstream: model selection and configuration (`alo-protocol`, ADR 0022)
- Contributor: Claude Code
- Task: Refine ADR 0022 before requesting acceptance
- Status: **corrected, and one approval requested.**
  [ADR 0022](../../decisions/0022-where-a-providers-key-is-kept.md) stays
  **PROPOSED**. Nothing implemented, no tier moved, no protocol added.

## Four corrections, three of them to my own claims

**1. ADR 0017 does not forbid outbound session-bus access, and I said it did.**

Cited precisely this time. ADR 0017 decides *"The daemon's socket moves out of
the person's session directory to `/run/alo/<uid>/agentd.sock`."* Its reason is
the reachability of the **agent-facing** socket: *"`logind` creates
`$XDG_RUNTIME_DIR` … as `0700`, owned by the person. The agent is a different
user (ADR 0001 §5), so it is refused by the parent directory."* Its consequence
clause is scoped — *"`session.rs` stops reading `$XDG_RUNTIME_DIR` **for this
purpose**"* — and its rejected alternative is about identity: *"Running the agent
as the person … would delete the boundary the whole design rests on."*

**No clause forbids the daemon making an outbound connection.** What blocks the
Secret Service is the **image's missing session integration** — no keyring
package, no unit, nothing to sign in at — which is work with an owner, not an
accepted prohibition.

**And the obstacle ADR 0017 names points the other way here.** `/run/user/<uid>`
is `0700` and the person's: the *agent* cannot enter it, which is why the agent's
door moved — and `alo-agentd` runs as `User=alo`, so it is the one process for
which that directory is its own. The thing ADR 0017 could not do for the agent is
the thing the daemon can do for itself.

**2. `Secret`'s API and retrieval timing are application-level safeguards, not
isolation.** I presented them as what protects a credential. They are not.

What is **kernel-enforced**: two logins. The agent is `alo-agent` (ADR 0001 §5),
the daemon is `alo`, and `SO_PEERCRED` answers which of the two is on the
connection. A process running as the agent is refused `/run/user/1000/bus` by the
directory — the same refusal ADR 0017 measured, working in our favour. And the
protocol has no verb a key could travel in.

What is **application-level**: `Secret` has no accessor, no `Display`, no
`Serialize`, no `Clone`, and a `Debug` that says nothing. That prevents a
credential reaching a log or a support bundle **by accident** — a real class of
failure, and the one `41c9f1e` closed. It prevents nothing on purpose.

**The limitation is now stated rather than implied: anything running as the
person can retrieve the credential, including a compromised `alo-agentd`.** No
arrangement of these mechanisms changes that, because the daemon *is* the person.

**3. "Opened and closed per retrieval" is withdrawn.** libsecret's
`secret_service_get_sync()` returns a **shared singleton** service proxy, on a
GDBus **process-wide** session connection. A Rust wrapper's `Drop` unrefs a
GObject; **dropping it is not evidence the bus connection closed**, and with a
singleton it is evidence of the opposite.

So the daemon **will** hold a bus connection across turns — which is exactly
`a_connection_made_before_the_boundary_stays_usable_inside_it`, already
reproduced. Concurrent retrievals share that connection, and a turn is a thread
of the same process.

**This removes a reason that was never load-bearing** rather than sinking the
architecture: the protection is the two logins. But the claim had to go, and it
is replaced in the acceptance plan by a **measurement** — counting the daemon's
open Unix sockets across turns — rather than an assumption. **I did not add a
dependency in order to check a proposal**, so this is documented behaviour, not
something this repository has measured.

**4. The credential-transfer protocol is dropped.** The previous revision offered
"the session hands the credential to the daemon over the door" as an alternative
to a restriction that, per correction 1, does not exist. Inventing a protocol to
avoid an imaginary prohibition is worse than either option.

## The architecture, as it now stands

The Secret Service through `libsecret`, reached by `alo-agentd` as the person.
The bus is found at **`/run/user/<uid>/bus`**, with the uid taken from the
daemon's own `getuid()` — **never** from `DBUS_SESSION_BUS_ADDRESS` or anything
an agent or application can set. It is per **user**, not per session, which is
what gives the concurrent case one answer.

Unchanged and untouched: `/run/alo/<uid>/agentd.sock` and every check in
`place.rs`; the separate agent identity and `SO_PEERCRED`; the record, the
indicator, and ADR 0020.

Six states, each stated: before login (**unavailable**), locked (**locked**), no
entry (**missing**), refused caller (**denied**), after logout (**unavailable**;
`/run/alo/<uid>` already goes at sign-out), concurrent sessions (one bus per uid,
no ambiguity). **None falls back** — not to plaintext, not to another provider,
not to another model.

Dependencies: a Secret Service in the image, and sign-in — **both the desktop
worker's, to be scheduled with them** — plus a `libsecret` binding, which is C
behind a Rust crate, the same shape as `aya` or `ureq`, and not a third language
in this repository. **There is no fourth**, and in particular no ADR amendment.

## Evidence

**`the_four_things_this_daemon_says_to_an_agent_carry_no_credential`** —
`alo-protocol`, new. The match over `ToAnAgent` has **no wildcard**: a fifth
variant does not fail this at runtime, it fails to compile it, so whoever adds
one comes here and decides what it may carry. Same shape as `alo-bounding`'s
*the programme has nowhere to write what it sees*, which names its two maps
exactly rather than counting them.

That is the half of correction 2 this repository can hold down. **The other half
— that a process as `alo-agent` is refused the person's bus — is a property of
two real logins on a real machine, and a root test box cannot show it.** It is in
the acceptance plan rather than asserted here.

Gates: `cargo fmt --check`, `clippy --workspace --all-targets -D warnings`,
`cargo test --workspace`, `RUSTDOCFLAGS="-D warnings" cargo doc`, the
supervisor's own tests, and the BPF target's fmt and clippy on the pinned
nightly, with kernel tests taking the shared lock. WSL certifies no hardware.

## Coordination with the desktop worker

Two items belong to them and are **not** being started here: a Secret Service
package and unit in the image, and sign-in. This report is the handoff; the
scheduling is theirs, and ADR 0022 names them as dependencies rather than
assuming them.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible.

**ROADMAP.md** — no tick, **no tier moved**. *Add your own provider* stays v0.01.

**docs/autonomy/QUEUE.md** — the credential-store item stays, now blocked on one
approval rather than on an open architecture.

**docs/autonomy/STATE.md** — three facts. ADR 0017 does not forbid the daemon
making outbound connections; what blocks the Secret Service is the image's
missing session integration. What protects a provider key is **two logins**, not
`Secret`'s API — and a compromised daemon can retrieve it, which is now stated.
And ADR 0022 asks one approval: the Secret Service over `/run/user/<uid>/bus` as
the v0.01 store, with its two image dependencies scheduled with the desktop
worker.
