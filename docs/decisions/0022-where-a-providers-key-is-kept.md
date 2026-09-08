# ADR 0022 — Where a provider's key is kept

**Status:** **PROPOSED — not accepted, and nothing in it is built.** It asks the
repository owner one question, because the accepted documents answer it two
different ways.
**Date:** 2026-09-08, revised the same day after the owner declined the
release-scope change this ADR first proposed, and after a measurement corrected
its central claim about D-Bus.
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

**2. `alo-agentd` is a system unit, and a Secret Service is a session one.**
`image/usr/lib/systemd/system/alo-agentd.service` is `WantedBy=multi-user.target`,
started at boot, `User=alo`, `Group=alo`, `SupplementaryGroups=alo-agent`. It
runs **as the person** and has no session, no session bus and no `DBUS_SESSION_BUS_ADDRESS`.
[ADR 0017](0017-the-agents-door-is-ours-and-not-in-the-session.md) put the
agent's door outside the session **on purpose**, so this is a deliberate
property rather than an oversight — and it is the dependency that needs a
decision rather than work.

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
mistake.** What actually protects a credential is three things, and all of them
are about *when* rather than *where*:

1. it is fetched **outside** the turn's boundary, in the daemon, exactly where
   the endpoint is already resolved
   ([ADR 0020](0020-a-question-is-carried-out-inside-the-turns-boundary.md));
2. what crosses into the turn is an `alo_models::Secret` — no accessor, no
   `Display`, no `Serialize`, no `Clone`, and a hand-written `Debug` that says
   nothing;
3. **the store handle is not held open across a turn**, which is the same
   discipline ADR 0020 already applies to the HTTP client and to DNS.

Point 3 is the one an implementation can get wrong quietly, and it is the one
that must be tested rather than intended.

## The recommended architecture

One, concrete.

- **Store:** the **Secret Service**, through `libsecret`, under the attributes
  alo already derives — `provider/<the person's own name for it>`. Standard, not
  invented, and what the v0.01 line's own word means.
- **Who reaches it:** `alo-agentd`, as the person it already runs as. No new
  login, no new capability; `CapabilityBoundingSet=` stays empty.
- **When:** immediately before the question and **outside**
  `carrying_out_a_departure` — beside the resolution ADR 0020 already does
  there. **Opened and closed per retrieval.** Nothing is held across a turn.
- **What crosses:** a `Secret`, and nothing else. `SecretRef` is unchanged; this
  ADR adds an implementation behind it and changes no abstraction.
- **Unlock:** at sign-in, by the login password, as on every other Linux desktop.
- **Failure:** unavailable, locked, missing and denied each refuse in their own
  words, and **none falls back** — not to plaintext, not to another provider,
  not to another model.

### Dependencies, in the order they block

1. **A Secret Service in the image** — a package and a unit. Image work.
2. **A session bus the daemon can reach.** This **collides with ADR 0017** and
   is the one that needs a decision: either that ADR is amended to let the
   daemon reach the person's session bus, or something in the session hands the
   credential to the daemon over the door it already has at `/run/alo/1000`.
   Both are defensible and they are different products.
3. **Sign-in**, so that there is something to unlock with. Compositor and
   session work.
4. **A `libsecret` binding.** `libsecret` is C, behind a Rust crate — the same
   shape as `aya` or `ureq`, and *not* a third language in this repository,
   which is what `CLAUDE.md` forbids. Named so it is decided rather than
   assumed.

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
| The store handle is not held across a turn | asserted, because it is the one thing an implementation gets wrong quietly |
| The published guarantees still hold | settings refusals carry no credential (`41c9f1e`); a key is never rendered (`secret.rs`) |

## What this ADR does not decide

- **Nothing about ADR 0021**, which stays PROPOSED.
- **Nothing about the three choices.** Local models, your own API provider, and
  alo remain exactly as `docs/features.md` states them.
- **Nothing about alo's own endpoint**, which does not exist and is not invented
  here.

## The decision the owner must make

> **1. Is the Secret Service the v0.01 store**, with dependencies 1–4 above —
> a keyring in the image, a session bus the daemon can reach, sign-in, and a
> `libsecret` binding? Recommended: yes.
>
> **2. How does the daemon reach it, given ADR 0017?** Either that ADR is
> amended so `alo-agentd` may reach the person's session bus, or the session
> hands the credential to the daemon over the door it already has. **This is
> the decision that blocks the work**; the other three dependencies are
> ordinary tasks with owners.
>
> **3. If sign-in will not land in time for v0.01**, does the kernel keyring
> hold the key in the meantime — a key re-entered after every reboot — or do
> provider keys wait? **Not** a release-tier change either way: the promise
> stays at v0.01 and the question is only which existing mechanism keeps it.

Until these are answered, a provider that needs a credential is chosen,
persisted, and refused at the moment of asking — which is what this machine does
today, and is the honest behaviour rather than a placeholder.
