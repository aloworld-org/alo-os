# ADR 0031 — The pairing is the key

**Status:** accepted
**Date:** 2026-09-13
**Context:** [ADR 0003](0003-the-network-is-not-authority.md) (being on the
same network is not authority — discovery is open, use requires mutual
pairing, no certificate authority, no trusted-network setting),
[ADR 0001](0001-the-capability-model.md) §3 (grants are enumerated, revocable
and expiring), task 6 of `docs/autonomy/v0-5-the-local-network-plan.md`,
`crates/alo-nearby`, `crates/alo-asking`, `crates/alo-turn`

## The question in one line

**When a question or a verb arrives from the network naming a paired machine,
what makes it true that it came from that machine — with no certificate
authority, no trusted network, and two machines that had never met before two
people paired them?**

## What was true before this decision

Tasks 2 to 5 of the local-network plan built pairing, the corridor a question
travels down, the door a verb arrives at, and the measurement that an office
with no internet still works. Every one of their reports said the same thing
outright: *there is no cryptography here and none is claimed.* A question
reached the studio over plain HTTP from whoever held the address, and the
studio wrote down *the reception machine* because its pairing named one — not
because the connection proved it.

That is a gap ADR 0003 cannot tolerate once anything carries a verb. A stranger
on the office WiFi who read a `MachineId` off a discovery packet — which is
open by design — could present it and act under every grant B's person had made
to A. That is the lateral movement ADR 0003 exists to refuse, arriving through
the one component built to refuse it.

## The decision

**A pairing leaves each machine holding a secret made at the moment both people
confirmed and held by nobody else, and every message that claims to come from a
paired machine carries a proof made with that secret.** The proof is checked
against the pairing at the moment the message arrives — so a pairing revoked or
expired stops proofs at once — and a proof seen before is refused.

Concretely, in `crates/alo-nearby`:

1. **Agreement at pairing time — X25519.** Each side makes a fresh key pair for
   the proposal (`Keying`). The asking machine's public half travels inside the
   `Proposal`; the asked machine's public half travels back with its answer.
   Only public halves cross the wire. Each side computes the same shared secret
   from its own private half and the other's public half, and the private
   halves are consumed by the agreement and never written anywhere.
2. **A per-pairing key — HKDF-SHA256** over the shared secret, salted with the
   transcript of what the two people agreed: both identities, both public
   halves, the enumerated list, and the duration. The key is held on the
   `Pairing` row and is gone the moment the row is — which is what makes
   *revocable in one action, taking effect immediately* true for proofs as well
   as for the list.
3. **A code both people compare.** Six digits derived from the two identities
   and the two public halves, shown beside the confirmation on both machines.
   Two people who see the same six digits are not being intercepted at the
   moment of pairing. This is Bluetooth's numeric comparison and Signal's safety
   number, and it is the only defence against an active attacker at pairing
   time that does not smuggle a certificate authority back in: the thing that
   stands in for the authority is the two people, which is exactly what ADR
   0003 asked for.
4. **A proof per message — HMAC-SHA256** with the pairing key over: the sending
   machine, the receiving machine, the moment, and the SHA-256 of what the
   message carries. The tag is unique per message, so it is also what the
   receiver remembers to refuse a replay; there is no separate nonce and no
   randomness in a proof.
5. **Checking a proof** happens in one function, `Proven::checked`, which asks
   the receiving machine's own pairings at the moment, refuses a proof for
   another machine, verifies the tag in constant time, refuses a moment more
   than two minutes from now, and refuses a tag it has seen within that window.
   `Origin` — the only thing a remote turn can begin from — is now made only
   from a `Proven`, so a verb cannot reach the grants without the connection
   having proved where it came from.
6. **The primitives are rented, never written here.** `ring` 0.17, which this
   workspace already pins and compiles for every build through `ureq`'s
   `rustls` — BoringSSL's code, independently audited, and one crate rather
   than four. Nothing in `alo-nearby` implements a hash, a MAC, a curve or a
   key derivation. Its own randomness comes through `ring`'s system generator,
   which is `getrandom` underneath — the same kernel door `MachineId` uses.

## What this does not decide

- **It does not encrypt the corridor.** A proof says *who* sent a message and
  that it was not altered; it does not hide the question from the office
  switch. Confidentiality on the local network is a separate decision with its
  own trade-offs, and nothing here forecloses it: the pairing key is exactly
  what a later channel would be keyed from.
- **It does not authenticate discovery.** Presence stays open and unsigned
  (ADR 0003); a signed advertisement would only tell a watching network which
  machine was which more reliably.
- **It does not make a machine's identity a public key.** `MachineId` stays
  sixteen random bytes in a file, for the reasons `machine.rs` gives. The
  public halves here are per pairing and ephemeral, so two pairings of one
  machine cannot be linked by a third party through them.

## Alternatives rejected

**A shared secret sent over the wire at pairing time.** Simpler, and the whole
of it is readable by anybody on the network during pairing. The proposal
crosses the wire in the clear by design; a secret inside it would be no secret.

**A long-term signing key per machine (Ed25519), with the identity being its
fingerprint.** Cleanly authenticates without a shared secret and gives
non-repudiation the record does not need. Rejected because a long-term key is
a tracker across pairings and across networks — the thing `machine.rs` refuses
a serial for — and because it would make `MachineId`, a public surface, into a
different kind of value.

**TLS with self-signed certificates and trust-on-first-use.** It is the same
key agreement wearing heavier clothes, plus a certificate machinery that ADR
0003 rejected because every answer to *who issues them* smuggles ambient trust
back in. Numeric comparison at pairing time is what trust-on-first-use lacks.

**A nonce in every proof.** Standard, and it needs randomness on every message
and a failure path for a machine that cannot make any. A tag over a moment and
a body hash is already unique per message and is what the receiver remembers;
two identical messages in one nanosecond do not happen.

## Consequences

- A pairing's `Deliberating` is no longer one value both people agree on; it
  is one value **per machine**, each holding its own private half, and the
  asked machine's public half is carried back to the asking one. Tests that
  built both sides from one value now build both sides the way two machines
  would.
- `alo_nearby::Origin::paired` is gone; `Origin::proven` takes a proof and the
  bytes it is about. A turn crate that could begin a remote turn without a
  proof cannot compile.
- Whatever carries a question or a verb between machines — the corridor today,
  the verb wire when it is built — puts the proof in a header and the receiving
  end calls `Proven::checked` before anything else, per message.
- A surface that confirms a pairing shows the six-digit code beside the
  confirmation. `alo-shell` is outside the local-network plan; the code is a
  value the surface reads, and the rustdoc on `Deliberating::code` says what
  the surface owes.
- Clocks on two paired machines must agree to within two minutes for a proof
  to stand. An office whose clocks disagree by more is told so by the refusal
  rather than by nothing working.
