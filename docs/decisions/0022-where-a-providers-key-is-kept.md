# ADR 0022 — Where a provider's key is kept

**Status:** **PROPOSED — not accepted, and nothing in it is built.** It asks the
repository owner one question, because the accepted documents answer it two
different ways.
**Date:** 2026-09-08. Revised twice: after the owner declined the release-scope
change this ADR first proposed, and again after two of its own claims turned out
to be wrong — what ADR 0017 forbids, and what protects a credential.
**Proposed by:** the model-selection workstream
**Context:** `docs/features.md` (two lines, below),
[ADR 0005](0005-applications-are-sandboxed-and-ask.md) (applications are
sandboxed and ask), [ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md),
[ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md),
`crates/alo-models/src/secret.rs`, `crates/alo-models/src/provider.rs`,
`docs/contracts/person-settings.md`

## The question in one line

**Where does a provider's key actually live, and what may read it?** The
release-scope half is settled below — a v0.01 store and the v0.5 Secret portal
are **distinct deliverables**, and nothing moves. What is left is the mechanism,
its dependencies, and one decision that blocks them.

## The apparent disagreement, and why it is not one

`docs/features.md`, *Add your own provider in Settings*:

> `[v0.01] ★` … **The key goes to the keyring, never into a settings file**, so
> it cannot leak through a backup or a support bundle.

and, in the desktop section:

> `[v0.5]` **Secret storage** — one keyring behind the Secret portal, so
> applications stop inventing credential storage.

The first revision of this ADR read those as one deliverable promised twice, and
proposed moving the v0.01 clause. **That was wrong, the owner declined it, and
the reason it was wrong is [ADR 0005](0005-applications-are-sandboxed-and-ask.md).**

**They are distinct deliverables.** ADR 0005 says third-party applications
install sandboxed and reach the system **through portals** — *file access,
screenshots, camera, printing, notifications, secrets* — and that a portal
request is ADR 0001's grant model with "application" in place of "agent". So the
v0.5 line is about the **portal**: the interface a Flatpak uses, so that
applications stop each keeping their own credentials.

`alo-agentd` is not a sandboxed application. It is the system, running as the
person. **A store for the person's own provider keys at v0.01 does not need the
portal, and the portal at v0.5 does not deliver one.** One is an interface for
other people's software; the other is where alo keeps a key somebody typed into
its own settings panel.

That settles the release-scope half: **nothing moves.**

## What specifically prevents the desktop secret service from shipping at v0.01

Not "it does not exist". Three named dependencies, each with an owner.

**1. The image ships no Secret Service implementation.** `image/Containerfile`
is `fedora-bootc:42` plus two binaries, two units, two directories and one
description — *"There is no compositor in it, no desktop, no wallpaper and
nothing to sign in at."* A keyring is a package and a unit that nobody has added.

**2. The image has no session integration** — which is **not** the same thing as
an accepted prohibition, and the previous revision of this ADR confused the two.

The correction, cited rather than paraphrased.
[ADR 0017](0017-the-agents-door-is-ours-and-not-in-the-session.md) decides one
thing: *"The daemon's socket moves out of the person's session directory to
`/run/alo/<uid>/agentd.sock`."* Its reason is **reachability of the
agent-facing socket**, and it is specific:

> `logind` creates `$XDG_RUNTIME_DIR` — `/run/user/<uid>` — as **`0700`, owned
> by the person**. The agent is a different user (ADR 0001 §5), so it is refused
> by the *parent* directory before either of our two modes is ever consulted.

Its consequence clause is scoped the same way: *"`session.rs` stops reading
`$XDG_RUNTIME_DIR` **for this purpose**"* — the purpose being where to put the
socket. And its rejected alternative is about identity, not about D-Bus:
*"**Running the agent as the person.** … it would delete the boundary the whole
design rests on: `SO_PEERCRED` answers which of two users is on this
connection."*

**There is no clause forbidding the daemon from making outbound connections, to
the session bus or anywhere else.** ADR 0017 is about an inbound socket's
address. It says nothing about this, and this ADR was wrong to say it did.

**And the obstacle it names does not apply here.** `/run/user/<uid>` is `0700`
owned by the person; the *agent* cannot enter it, which is why the agent's door
moved. But `alo-agentd` runs as **`User=alo`** — the person — so the person's own
runtime directory is exactly the one directory it may open. The thing ADR 0017
could not do for the agent is the thing the daemon can do for itself.

So what blocks this is the image: **no Secret Service package, no unit, no
desktop and nothing to sign in at.** That is work with an owner, not a decision
about architecture.

**3. Unlock is tied to signing in.** A keyring unlocked by the login password
needs the sign-in path to exist. `docs/features.md` promises `[v0.01] Boots on
one certified machine, firmware to sign-in`, and that work belongs to the
compositor and session workstream. Until a person signs in, there is no secret
with which to unlock anything.

## Whether an existing implementation can satisfy it

Yes, and the field is small. Nothing below is a store alo would be inventing.

| | Fits? | Why |
|---|---|---|
| **Secret Service** (`libsecret`; gnome-keyring or another provider) | **yes**, with dependencies 1–3 | The standard, and what *keyring* means on Linux. Locked, unavailable and denied are native states with real meanings rather than ones we invent |
| **`systemd-creds`** | **no**, and worth saying why | It is already in the base and needs no session or bus — but it is for credentials an **administrator provisions to a unit**, decrypted by systemd at unit start from a root-only host key or the TPM. A person adding a provider at runtime cannot write one, and a service running as the person cannot decrypt one. Named here so it is not re-proposed |
| **The kernel keyring** (`@u`) | **partly** | No daemon, no bus, no session — it works today for a system unit running as the person. **Nothing survives a reboot**, so the key is typed again after every restart. A real answer with a real price |
| A file alo owns | — | The thing not to invent |

## The trust boundary, measured rather than assumed

**The first revision of this ADR said a D-Bus or kernel-keyring store "lands on
neither" of the boundary questions a file store lands on. That was wrong**, and
the correction is the most useful thing here.

`crates/alo-bounding/tests/what_a_bound_turn_can_still_reach.rs`, against the
real loaded programme:

- **`a_bound_turn_may_reach_a_unix_socket_nobody_showed_it`** — a turn bound to
  one folder and shown no destination is refused a non-loopback address with
  `EACCES` and **connects to a Unix socket in the same breath**. Deliberate:
  `deciding.rs` permits it because *a Unix socket is not egress, and refusing it
  would be enforcing something no policy claims*. So **a bound turn can reach
  the session bus, or a keyring daemon's own socket, directly.**
- **`a_connection_made_before_the_boundary_stays_usable_inside_it`** — a
  connection opened before the turn stays usable inside it. A pooled D-Bus
  connection held across a turn is that case exactly.
- And `keyctl` is a **syscall**, which this boundary does not hook at all.

**So none of the three mechanisms is isolated from a turn by the kernel
boundary, and choosing between them on that basis would be choosing on a
mistake.**

### What is kernel-enforced, and what is not

The previous revision listed `Secret`'s API and the timing of retrieval as what
protects a credential. **They are application-level safeguards and this ADR was
wrong to present them as isolation.** Separating the two is the point of this
section.

**Kernel-enforced — two logins, and one question the kernel answers.**
ADR 0001 §5 gives the agent a login of its own: `alo-agent`, uid 60989, while
`alo-agentd` runs as `alo`. What stops the agent reaching a credential is
therefore not a type:

- **The agent is on the other side of a socket**, and `SO_PEERCRED` is what says
  which of two users is on it — ADR 0017 rejected running the agent as the person
  precisely because that *"would delete the boundary the whole design rests
  on"*.
- **The agent cannot reach the store at all.** The person's session bus lives at
  `/run/user/1000/bus` inside a `0700` directory owned by the person. A process
  running as `alo-agent` is refused by that directory — the same refusal ADR 0017
  measured, working in our favour this time.
- **The protocol has no verb that returns a credential.** `ToAnAgent` carries an
  answer, a refusal or a proposal, and nothing that a key could travel in.

**Application-level — worth having, and not isolation.** `Secret` has no
accessor, no `Display`, no `Serialize`, no `Clone` and a hand-written `Debug`
that says nothing. That prevents a credential reaching a log, a record, a
support bundle or a serialised structure **by accident**, which is a real class
of failure and the one `41c9f1e` closed for settings refusals. It prevents
nothing on purpose.

Fetching outside the turn's boundary narrows *when* a bounded turn's own code
has the value in reach. It does not isolate the daemon's address space from
itself.

### The limitation this ADR keeps, explicitly

**Anything running as the person can retrieve the credential**, including
`alo-agentd` itself if it is compromised. A process that can execute code in the
daemon can open the same bus, ask the same service and read the same key —
`Secret`'s shape does not stop it, and the kernel boundary does not either,
because the daemon *is* the person.

What the design buys is that a compromise of **the agent** is not a compromise
of the credential, and that an accident anywhere is not one either. What it does
not buy is protection against a compromised daemon, and no arrangement of these
three mechanisms would.

## The recommended architecture

One, concrete, and it adds **no protocol**. The daemon makes an outbound
connection; nothing new is spoken to the agent, and nothing hands a credential
between processes.

**Store.** The **Secret Service**, through `libsecret`, under the attributes alo
already derives — `provider/<the person's own name for it>`. `SecretRef` is
unchanged: this puts an implementation behind it and alters no abstraction.

**Who reaches it.** `alo-agentd`, as `User=alo` — the person it already runs as.
No new login, no capability, `CapabilityBoundingSet=` stays empty.

**How the session is found, and it is not an environment variable.** The bus is
at **`/run/user/<uid>/bus`**, with `<uid>` taken from the daemon's own
`getuid()` — never from `DBUS_SESSION_BUS_ADDRESS`, never from anything an agent
or an application can set, and never announced. It is per **user**, not per
session, which is what makes the concurrent case have one answer.

That directory is `0700` and the person's. ADR 0017 measured that as the reason
the *agent* could not be given a door there; the daemon runs as the person, so
for it the same permission is the discovery working. Where a stronger answer is
wanted later, `sd-login` (`sd_uid_get_sessions`) asks logind directly rather than
trusting a path — named as an option, not required.

**What is preserved, unchanged.** `/run/alo/<uid>/agentd.sock` and every check in
`place.rs` — this touches the *inbound* door not at all. The separate agent
identity and `SO_PEERCRED`. The record, the indicator, and ADR 0020's
request-scoped destinations and hostname verification.

**Lifecycle, stated for each state rather than assumed:**

| | What the daemon finds | What happens |
|---|---|---|
| **Before login** | no `/run/user/<uid>/bus` | **unavailable** — refused, nothing sent |
| **Signed in, keyring locked** | a bus, a service, a locked collection | **locked** — refused, and it says *locked*, which is a different thing to do about it |
| **Signed in, no entry** | a bus, an unlocked collection, nothing under `provider/<name>` | **missing** — refused, naming that this provider has no key here |
| **Denied** | the service refuses this caller | **denied** — refused, and **never retried against another store** |
| **After logout** | the bus is gone; a held connection breaks | **unavailable** — refused. `/run/alo/<uid>` is removed at sign-out, which ADR 0017 already made this daemon's business |
| **Concurrent sessions** | one bus per uid, whichever seats are open | one answer, no ambiguity, no choosing between sessions |

**None of the six falls back** — not to plaintext, not to another provider, not
to another model.

### Connection ownership — a claim withdrawn

The previous revision said the store handle would be *"opened and closed per
retrieval"*, so that nothing was held across a turn. **That is withdrawn, because
libsecret is unlikely to be able to honour it and this ADR had not checked.**

What is documented, and is documentation rather than a measurement this
repository has made:

- `secret_service_get_sync()` returns a **shared singleton** service proxy — the
  default service object is not per-caller.
- It is built on GDBus, and `g_bus_get_sync(G_BUS_TYPE_SESSION, …)` returns a
  **shared, process-wide** connection.
- A Rust wrapper's `Drop` unrefs a GObject. **Dropping it is not evidence that
  the underlying bus connection closed**, and with a singleton it is evidence of
  the opposite.

**So the daemon will hold a session-bus connection across turns**, and that is
exactly `a_connection_made_before_the_boundary_stays_usable_inside_it` — the
inherited-socket case this workstream reproduced. Concurrent retrievals share
that one connection, and a turn is a thread of the same process, so a turn shares
it too.

**This does not sink the architecture; it removes a reason that was never load-
bearing.** What protects the credential is the two logins, not the lifetime of a
socket. But the claim has to go, and its verification is in the acceptance plan
below as a measurement rather than an assumption: this ADR does not add a
dependency in order to check a proposal.

### Dependencies, in the order they block

1. **A Secret Service in the image** — a package and a unit. **Desktop worker's**,
   and to be scheduled with them rather than assumed.
2. **Sign-in**, so there is something to unlock with. **Desktop worker's.**
3. **A `libsecret` binding.** C behind a Rust crate — the same shape as `aya` or
   `ureq`, and *not* a third language in this repository, which is what
   `CLAUDE.md` forbids. Named so it is decided rather than assumed.

**There is no fourth**, and in particular no ADR amendment: the previous revision
listed one and it was based on the misreading corrected above.

## The acceptance plan

Nothing here is done until all of it passes. **The published routing test does
not count toward it**: `the_key_reaches_the_chosen_one_and_the_other_hears_nothing`
used the **local-service door** on loopback, because a plain-HTTP provider cannot
be constructed. It establishes the pairing and it establishes **nothing about
authenticated HTTPS operation through the daemon.**

| Must be proved | Shape |
|---|---|
| **An authenticated HTTPS provider request through the production daemon path** | an owned TLS server, a synthetic credential in a real store, the question driven through `alo-agentd`, and the credential asserted **on the wire** at the far end |
| The credential comes from the store, not from anywhere else | the store empty ⇒ refused; the store holding it ⇒ sent |
| Unavailable / locked / missing / denied | four refusals, four sentences, **nothing sent in any of them** |
| No fallback | in each of the four, no other provider and no other model is asked |
| **What the bus connection actually does** — measured, not assumed | count the daemon's open Unix sockets across several turns (`/proc/<pid>/fd`, `ss -x`) and record whether the connection persists, is shared, and survives dropping the wrapper. This replaces the withdrawn *opened and closed per retrieval* claim, and its answer changes the documentation rather than the design |
| **Concurrent retrievals** | two questions in flight sharing one connection: both get their own credential, neither gets the other's, and neither blocks on the other past the timeout |
| The agent cannot reach the store | a process as `alo-agent` is refused `/run/user/<uid>/bus` by the directory — the kernel-enforced half, on a real machine with two logins, since a root test box cannot show it |
| The published guarantees still hold | settings refusals carry no credential (`41c9f1e`); a key is never rendered (`secret.rs`) |

## What this ADR does not decide

- **Nothing about ADR 0021**, which stays PROPOSED.
- **Nothing about the three choices.** Local models, your own API provider, and
  alo remain exactly as `docs/features.md` states them.
- **Nothing about alo's own endpoint**, which does not exist and is not invented
  here.

## The approval requested

One thing, and nothing else in this ADR is being asked for:

> **Approve the Secret Service — reached by `alo-agentd` over the person's own
> session bus at `/run/user/<uid>/bus`, discovered from the daemon's own uid — as
> the v0.01 store for provider keys**, and with it the scheduling of its two
> image dependencies with the desktop worker: a Secret Service in the image, and
> sign-in.

What that approval does **not** include, and what nobody should read into it:
no release tier moves; no temporary or interim store is created; no
credential-transfer protocol is added; ADR 0017 is not amended; ADR 0021 is
untouched; and this ADR stays **PROPOSED** until the approval is given.

**If the two image dependencies cannot land inside v0.01**, that is a scheduling
question to answer then — with the kernel keyring as the mechanism that needs
neither of them, at the cost of a key re-entered after every reboot. **It is not
a reason to move the promise**, and this ADR does not propose moving it.

Until this is approved, a provider that needs a credential is chosen, persisted,
and refused at the moment of asking — which is what this machine does today, and
is the honest behaviour rather than a placeholder.
