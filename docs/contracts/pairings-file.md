# Contract — the file a machine keeps its pairings in

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is the file that says which machines this one is paired with, what each
may ask it for, when each pairing was made and ends, and the key the two
machines agreed. It is written by `alo-agentd` at the moment a pairing is kept
or revoked, and read by it once, at start — so that a machine switched off at
night is paired with the same machines in the morning, for exactly as long as
it was. Nothing else in alo OS writes it.

Read `docs/decisions/0003-the-network-is-not-authority.md` first and
`docs/decisions/0031-the-pairing-is-the-key.md` beside it: what a pairing *is*
and what the key proves belong to those. `docs/contracts/local-network-wire.md`
is how one is made across the network, and `docs/contracts/daemon-protocol.md`
is how the person here proposes, confirms and revokes one.
`crates/alo-nearby`'s `keeping.rs` is the shape and `crates/alo-remembering`'s
`pairings.rs` is the file.

## Where it is

```
/var/lib/alo/pairings.toml
```

Beside the grants (`grants.toml`) and the machine's own identity
(`machine-id`), in the folder the image makes for what an agent did on this
machine. The same folder for the same reason, and one more: the identity the
key is paired **to** is already there, and a pairing kept in one store and an
identity in another would be a machine that could wake with one and not the
other.

The path is a constant. Where a machine keeps its pairings is nobody's policy,
and a second copy of the answer would be a second file for somebody to point
somewhere the daemon is not reading.

## What it looks like

```toml
format = 1

[[pairing]]
with = "0f1e2d3c4b5a69788796a5b4c3d2e1f0"
may = ["models"]
made = 1760000000
ends = 1760086400
key = "…sixty-four lowercase hexadecimal characters…"
```

| Key | Meaning |
|---|---|
| `format` | Which shape the file is in. Required. `1` today. |
| `with` | The other machine, by the identity discovery found it by. |
| `may` | What it may ask this machine for: `models`, `workspace`, as the wire spells them. Never empty. |
| `made` | When the two people agreed, in whole seconds since 1970. |
| `ends` | When the pairing stops, in whole seconds since 1970. Never absent. |
| `key` | The key both machines hold and nobody else does (ADR 0031), as hexadecimal. |

**Five fields, and every one of them is something the two people were shown
or agreed.** There is no field for an address, for a name a person gave the
machine, or for *trusted* — a row with any of those would be a pairing on terms
nobody confirmed. A key nobody declared is refused rather than read around.
There is at most one row per machine.

## The rules it is held to

- **Every row is made again on the way in**, through the same constructor a
  pairing is made under, after the checks a proposal is held to: the list must
  permit something and name only arms that exist, the pairing must last whole
  seconds and no longer than thirty days, the key must be exactly the bytes a
  key is. A row that fails any of them refuses the **whole** file — a list that
  silently dropped the machine a person is looking for, or kept a key it could
  not check, would lie about who this machine is paired with.
- **A pairing that has ended is not on the list that comes back.** The reader
  drops every row whose `ends` is at or before the moment it reads, before
  anybody holds the list; the writer writes only what stands. A pairing that
  ran out while the machine was switched off is gone when it wakes, and a key
  never outlives the row it was made for. This is what makes *a pairing's
  expiry survives a restart* a property of the file.
- **Who may have written it** is the grants file's three rules, asked of the
  open file: not a symbolic link, owned by root or by the login reading it,
  and writable by nobody else. The file is written `0600`. A file that fails
  any of them stops `alo-agentd` from starting, exactly as an unbelievable
  grants file does; a file that is simply not there is a machine that has never
  paired, and starts.
- **Replaced whole or not at all**: written to a sibling, synced, and renamed
  over the real file, so a machine that loses power mid-write keeps the
  pairings it had.

## Why the key is here

The key is what makes every proof from a paired machine true (ADR 0031), and
the plan asked which store it goes in, with the care a credential gets. It goes
in this file and not in the keyring a provider's key lives in (ADR 0022), for
three reasons in order of weight. The identity the key is paired to is already a
file in this folder. A keyring can be locked when the service starts, and a
machine that reads its pairings at start would then be paired with nothing until
somebody typed a password — which is the *paired with nothing in the morning*
this file exists to end, wearing a prompt. And what the key lets its holder do
is bounded on the other machine by that machine's own grants and by this row's
expiry, so it is exactly as sensitive as the grants file beside it: whoever can
read this file could already rewrite what this machine's agent may reach.

## Versioning

`format` is `1`. Anything that would stop this version reading a file
correctly raises it; anything additive does not. A file with a higher number is
refused as written for an alo OS this is not, before any row is looked at.
