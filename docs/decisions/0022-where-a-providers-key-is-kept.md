# ADR 0022 — Where a provider's key is kept

**Status:** **PROPOSED — not accepted, and nothing in it is built.** It asks the
repository owner one question, because the accepted documents answer it two
different ways.
**Date:** 2026-09-08
**Proposed by:** the model-selection workstream
**Context:** `docs/features.md` (two lines, below),
[ADR 0005](0005-applications-are-sandboxed-and-ask.md) (applications are
sandboxed and ask), [ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md),
[ADR 0016](0016-the-organisation-bounds-and-the-person-chooses.md),
`crates/alo-models/src/secret.rs`, `crates/alo-models/src/provider.rs`,
`docs/contracts/person-settings.md`

## The question in one line

**Where does a provider's key actually live, and what may read it?** Nothing
accepted says, and the two lines that come closest disagree about when it
exists.

## The disagreement, quoted

`docs/features.md`, *Add your own provider in Settings*:

> `[v0.01] ★` … **The key goes to the keyring, never into a settings file**, so
> it cannot leak through a backup or a support bundle.

and, in the desktop section:

> `[v0.5]` **Secret storage** — one keyring behind the Secret portal, so
> applications stop inventing credential storage.

So a v0.01 promise depends on a keyring, and the keyring is v0.5. Both can be
true only if they are **two different things**: alo's own store for the
providers a person adds, now; and the portal-backed one that other applications
may use, later. That reading is reasonable and **it is not written anywhere**,
which is why this is an ADR rather than a commit.

## What exists today

- `alo_models::SecretRef` — *where a key lives in the keyring*, an opaque handle
  that is deliberately not a key.
- `alo_models::Secret` — a key for the length of one call: no accessor, no
  `Display`, no `Serialize`, no `Clone`, a hand-written `Debug` that says
  nothing, and one use — `carried_by`, which takes a request and gives it back
  with the key on it.
- `docs/contracts/person-settings.md` — the settings file has **no `key` field**
  and the keyring name is derived as `provider/<name>`.
- **Nothing behind the handle.** No store, no reader, no writer. A provider that
  needs a key is chosen, persisted, and refused at the moment of asking.

The abstraction is right and is not what is missing. What is missing is one
implementation of it, and where that implementation puts bytes is a decision
with consequences that outlive it.

## What is not settled, and cannot be guessed

**1. Where the bytes are.** A file alo owns, the desktop secret service over
D-Bus, or the kernel keyring. These differ in what *locked*, *unavailable* and
*denied* even mean.

**2. What protects them at rest.** Full-disk encryption is `[v0.5] Full-disk
encryption, enrolled at install`. Until then a file store is protected by file
permissions and nothing else. That may be acceptable — it is what most systems
do — but it has to be **said**, because the v0.01 line's own justification is
*so it cannot leak through a backup or a support bundle*, and a mode-0600 file
in a home directory is exactly what a backup takes.

**3. What may read it — and this one is not obvious.** `alo-agentd` runs as the
person, and a turn is one of its threads. [ADR 0013](0013-the-grant-is-enforced-by-the-kernel.md)
means a bound turn may open only what its grant names, so:

- opening a key file **inside** a turn is refused by the kernel — correctly, and
  it would break every keyed provider;
- opening it **before** the turn and holding the descriptor is the
  inherited-descriptor gap this workstream has already reproduced
  (`what_a_turn_inherits.rs`), which is the one gap that moves contents past a
  grant.

**A file store therefore lands on an open security question.** A D-Bus or kernel
keyring store does not: neither is a `file_open`, and a Unix socket is not egress
(`deciding.rs` permits it explicitly). That is a real argument about mechanism
rather than taste, and it is the single most useful thing in this ADR.

## The options

**A — a file alo owns.** `$XDG_DATA_HOME/alo/credentials`, mode 0600, read by
the daemon. Cheapest, works with no session bus and no desktop, and it is the
one that meets the v0.01 line as written. Costs: the boundary question above,
protection by file permissions only until full-disk encryption, and it is
precisely the *inventing credential storage* the v0.5 line exists to stop.

**B — the desktop secret service, behind a portal.** What the v0.5 line
describes, and what ADR 0005's sandboxed-and-ask shape already implies.
*Locked*, *unavailable* and *denied* are native states with real meanings rather
than ones we invent. It sidesteps the boundary question. Costs: it does not
exist, it needs something running in the session, and a headless machine has no
session at all — which matters because `alo-agentd` is a service.

**C — the kernel keyring.** No file, cleared on logout, nothing to back up.
Costs: it is a third mechanism nobody has asked for, it needs syscalls this
workspace cannot make without `unsafe` or a new dependency, and a key that
disappears on logout has to be typed again on every login.

## Recommendation

**B, and therefore a decision about the v0.01 line.**

The v0.5 line is right that applications inventing credential storage is the
problem, and A is alo inventing one. B also avoids putting a credential file
inside the reach of a question this workstream has open.

But B cannot ship for v0.01, so **the v0.01 provider line needs an answer of its
own**, and there are two honest ones:

1. **Ship A as a stated interim** — alo's own store, named as such, with the
   protection it really has written down, replaced by B when B exists. The
   boundary question must be answered first, because a keyed provider that works
   only outside a turn is not a working provider.
2. **Move the clause.** Keep `Add your own provider` at v0.01 for providers that
   need no credential — which **works today** — and move *the key goes to the
   keyring* to v0.5 beside the store that makes it true. Nothing is promised
   that does not exist, and nothing is built that has to be replaced.

Between those two the second is the one that promises only what it delivers, and
the first is the one that gets a person using OpenAI on a v0.01 machine. That is
a product call rather than a technical one.

## What acceptance would need, whichever is chosen

Named now so the work is finishable rather than open-ended. All four states, and
none of them may fall back:

| State | What must happen |
|---|---|
| **Unavailable** — no store on this machine | the question is refused, saying so; nothing is sent |
| **Locked** — the store exists and will not open | refused, saying it is locked, which is a different thing to do about it |
| **Missing** — a store, and no entry for this provider | refused, naming that this provider has no key here |
| **Denied** — the store refuses this reader | refused, and **never retried against a different store** |

And in every one of them: **no plaintext fallback, no other provider, no other
model.** A refusal that quietly asked somewhere else would be the worst reading
of ADR 0008's *never a silent fallback*, with a credential attached.

Plus: a key never reaches a settings file, a record, a log, or any `Debug` —
which `secret.rs` makes structural and which a store must not undo by holding
bytes anywhere else.

## What this ADR does not decide

- **Nothing about ADR 0021**, which stays PROPOSED.
- **Nothing about the three choices.** Local models, your own API provider, and
  alo remain exactly as `docs/features.md` states them.
- **Nothing about alo's own endpoint**, which does not exist and is not invented
  here.

## The decision the owner must make

> **1. Is a provider's key kept in a store alo owns (A), or behind the desktop
> secret service (B)?** Recommended: B.
>
> **2. Since B cannot ship for v0.01 — does `Add your own provider` ship with a
> stated interim store, or does *the key goes to the keyring* move to v0.5 with
> the store that makes it true?** Recommended: the second, and it is a product
> call.
>
> **3. If a file store is chosen: may `alo-agentd` hold a credential descriptor
> across a turn?** It is the inherited-descriptor gap, and a keyed provider
> cannot work without an answer.

Until these are answered, a provider that needs a credential is chosen,
persisted, and refused at the moment of asking — which is what this machine does
today, and is the honest behaviour rather than a placeholder.
