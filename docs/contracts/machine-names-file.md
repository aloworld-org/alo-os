# Contract — the file a machine keeps the names of paired machines in

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code". Added 2026-09-14.

This is the file that says what the person on this machine called each machine
this one is paired with — so that the egress indicator, the record, the list of
pairings and the machine chosen to answer questions say *the studio machine*
rather than thirty-two hexadecimal characters. It is written by `alo-agentd` when
the person names a machine or clears a name on their own door
(`name-machine`, `clear-machine-name`, `docs/contracts/daemon-protocol.md`) and
when a pairing is revoked or kept afresh, and read by it once, at start. Nothing
else in alo OS writes it.

Read `docs/decisions/0003-the-network-is-not-authority.md` first:
**a name decides nothing.** The identity is what a pairing, a proof and a grant
are about (`docs/decisions/0031-the-pairing-is-the-key.md`); a name is only ever
shown to the person who gave it. `crates/alo-remembering`'s `named.rs` is the
rule a name is held to, `names.rs` is the shape, and `machine_names.rs` is the
file.

## Where it is

```
/var/lib/alo/machine-names.toml
```

Beside the pairings (`pairings.toml`) and the grants, in the folder the image
makes for what an agent did on this machine. **Beside the pairings file, never in
it**: a pairings row is exactly what two people made, and a name is what one of
them calls the other machine. The path is a constant, for the pairings file's
reason.

## What it looks like

```toml
format = 1

[[machine]]
identity = "0f1e2d3c4b5a69788796a5b4c3d2e1f0"
called = "the reception machine"
```

| Key | Meaning |
|---|---|
| `format` | Which shape the file is in. Required. `1` today. |
| `identity` | The machine, by the identity discovery found it by. |
| `called` | What the person here calls it. |

A key nobody declared is refused rather than read around. There is at most one
row per machine.

## The rules it is held to

- **Every name is checked again on the way in**, by the rule the person's door
  holds a name to: something in it once trimmed; at most 64 characters; no line
  break, tab or other control character; and not spelt as a machine's identity,
  bare or as a grantee (`machine:<identity>`), in any case. A row that fails
  refuses the **whole** file.
- **A name outlives nothing its pairing does not.** The reader keeps a name only
  for a machine a pairing with stands at the moment it reads, measured against
  the pairings file read just before; the writer writes only names of machines
  paired at that moment. A name whose pairing was revoked, or ran out while the
  machine was switched off, is gone when it wakes. A pairing kept afresh with a
  machine starts with no name.
- **Who may have written it** is the pairings file's three rules, asked of the
  open file: not a symbolic link, owned by root or by the login reading it,
  writable by nobody else. The file is written `0600`. A file that fails any of
  them stops `alo-agentd` from starting — whoever could write it could put one
  machine's name on another machine's evidence — and a file that is simply not
  there is a person who has named nothing, and starts.
- **Replaced whole or not at all**: written to a sibling, synced, and renamed
  over the real file.

## What a name is never used for

Nothing reads this file to find a machine on the network, to dial one, to check a
proof, or to decide a grant. No name crosses to another machine
(`docs/contracts/local-network-wire.md`). Every request on the person's door
still names a machine by its identity, and a name typed where an identity goes is
refused as not a machine.

## Versioning

`format` is `1`. Anything that would stop this version reading a file correctly
raises it; anything additive does not. A file with a higher number is refused as
written for an alo OS this is not, before any row is looked at.
