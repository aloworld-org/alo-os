# ADR 0077 — A machine signs its own account of what left it

**Status:** proposed, 2026-09-29. **Awaiting the owner.**
**Date:** 2026-09-29
**Context:** `docs/features.md` promises at tier v1 *a signed, printable statement
of exactly what left this machine in a period — the artifact an auditor asks for
and nobody can currently produce*;
[ADR 0036](0036-the-image-is-signed-by-a-key-a-person-holds.md) (the image is
signed by a key a person holds, **by digest**, no transparency log);
[ADR 0003](0003-the-network-is-not-authority.md) (the network is not authority);
[ADR 0008](0008-where-inference-happens.md) (where inference happens);
`CLAUDE.md` law 1 (*nothing leaves silently*) and its standing rule that a
private signing key never lands on a machine an agent runs on;
`crates/alo-attesting`, which produces the payload and holds no key.

## The question in one line

**An attestation is a statement about one machine over one period, so there are as
many of them as machines times periods. Who signs them, with what key, and what
does a machine do when it has nowhere safe to keep one?**

## What was true before this decision

`crates/alo-attesting` landed as #267 and produces the payload: the counts, the
departures, the refusals, the errands, law 1's inference-egress measurement, what
the period could not cover, and a format number. It **signs nothing and holds no
key**, with four tests enforcing that, because the question below was open.

ADR 0036 settles signing for **the image** — one artifact per release, signed by
the owner, by digest. It does not reach this: an image key answers *is this
software genuine alo OS*, and an attestation key would answer *is this machine's
account of itself genuine*. Different claim, different audience, different blast
radius when it leaks.

## The decision in one line

**The machine signs its own attestation, with a key generated on that machine in
hardware that cannot export it, and the artifact names which kind of hardware
that was.**

## Why the vendor cannot sign, which is a disqualification and not a preference

The obvious alternative is that the owner signs, as with the image. It does not
survive one question: **what would have to travel?**

A vendor-signed attestation requires the machine's record of what left it to reach
the vendor. On a product sold on sovereignty, the flagship compliance artifact
would be one **whose own production is an egress to us** — law 1's indicator would
fire to make it, and the record of that egress would appear in the next period's
attestation. A customer would be sending us, as a matter of routine, the exact
document that says where their data went.

**And the escape hatch does not work.** *Send only the digest, so the record never
leaves* rescues the sovereignty objection and destroys the artifact: a signature
over bytes the signer has never seen attests *somebody handed me these bytes*, not
*this machine's account is genuine*. That is notarisation wearing an attestation's
clothes. It also leaves a customer's compliance artifact depending on our uptime
and their connectivity — an auditor's request answerable only when the vendor is
reachable is not an artifact the customer holds.

The user-facing form of the same point is the one that decides it: **an auditor
asks the customer, and the customer answers.** Any scheme where they must ask us
makes *what left my machine* a question only the vendor can answer about the
customer's own machine.

## Why this does not break the standing rule, and the sturdy reason rather than the clever one

`CLAUDE.md`: *never write the private signing key to any machine an agent runs on
— the owner signs.*

There is a tempting argument that the rule is satisfied because an agent has no
path to the key: law 2 allows no arbitrary command, the capability model
enumerates what an agent may reach, and a hardware key is not on that list.
**That argument is rejected here even though its conclusion is right.** It rests on
a policy staying as strict as it is today, and a policy is a thing somebody widens
in a change that looks unrelated to keys. A guarantee that depends on a list nobody
is watching is the failure this repository spent 2026-09-28 and 29 removing from
its own gates.

The sturdy reason does not mention agents at all: **the key is generated on the
machine in hardware that cannot export it.** Non-exportability is a property of
the silicon rather than of a rule. It holds if a future capability model is
sloppier than this one, and it holds against a total compromise of the software
stack — which then yields *signatures made while compromised*, not a stolen key.
An attacker cannot take it with them, cannot forge attestations after they are
evicted, and cannot produce one anywhere but on that machine.

**And the rule was never about this key.** ADR 0036's key is the one whose leak
lets anybody hand every machine everywhere software that verifies. A per-machine
attestation key's leak forges one machine's attestations. Noticing that the rule
has a subject is not a convenient reading of it.

## What an attestation claims, stated so nobody reads it as more

**An attestation says *this machine said this*. It never says *this is true*.**

A compromised machine signs false attestations for as long as it is compromised,
and **no scheme fixes that** — the vendor branch would have us signing a
compromised machine's record with a straight face. What the hardware buys is
bounded: the key does not leave, so forgery ends when the compromise does and
cannot happen elsewhere.

An ADR that does not say this invites a reader to treat a signature as proof of
the contents rather than proof of origin, and the whole artifact's value depends
on that distinction being understood by the person relying on it.

## Which kind of hardware, because "is it in hardware" has no true answer

*Is the key in hardware* is a yes-or-no whose real answer is not one. A discrete
security chip, a secure element inside the processor, the operating system's
keystore and a file on disk are four different promises. A verifier that treats
them alike **accepts the weakest while believing it accepted the strongest.**

So the artifact names which, and `alo_attesting::HeldBy` already carries the
names: `a-security-chip`, `a-secure-element`, `the-system-keystore`,
`a-file-on-disk`, and `something-else <what>` so a machine with protection nobody
anticipated can say what it used rather than claim one of the four or say nothing.

**It is inside the signed bytes.** Beside the artifact it could be dropped, edited
or lost in transit and the attestation would still verify — a document whose
strength claim can be stripped without breaking its signature overstates itself by
default. In the bytes, removing it breaks the signature.

**It reports and never judges.** Whoever checks an attestation decides what they
accept: a regulator, a customer's security team and a person curious about their
own laptop are not owed the same bar, and this repository is not the right place to
set one. The corollary is a demand on whoever builds the verifier — **a field
nothing reads is a field that reassures without doing anything**, so a rule about
these names is owed alongside it.

## The machine picks, and the person is told at setup

Which protection a machine uses is **not a setting somebody has to go and find.**
The machine picks the best it has, by itself.

And it **says so at setup**, in the person's own words, so they learn what their
proof is worth while deciding whether to trust it — rather than months later from
an auditor telling them it is weaker than they assumed. A machine that quietly
used a file on disk while a person believed it had a chip would be the kind of
silence this product exists to not have.

## The machine with nowhere safe to keep a key

**This is the part that is asked of the owner rather than answered here**, because
both answers are defensible and the choice is a product decision.

- **Refuse to attest.** The artifact then always means at least *a key that cannot
  be exported*, and a machine that cannot make that claim makes none. Clean, and
  it denies a real capability to a machine whose owner may need it and understand
  the limits perfectly well.
- **Attest, and say `a-file-on-disk` in the signed bytes.** The person keeps the
  artifact, the weaker protection is stated rather than implied, and a verifier can
  refuse it on its own terms. Also the answer that lets a customer with older
  hardware produce anything at all — but it ships an artifact that looks identical
  in every respect except one line, which is exactly the shape a reader skims past.

`HeldBy::AFileOnDisk` exists so the second is **expressible**, which is not the
same as it being decided. Silence is the one answer that is not available: it will
be discovered by the first machine without a security chip, and discovered is the
worst way for it to be settled.

## What this does not decide

**Whether a person's name may appear in an attestation.** The owner decided
separately that it is each person's own choice, off until they make it, and #267
renders no grantee at all. What that choice needs — a per-person setting nobody
else can set for you, an aggregate for those who did not choose because *a blank
identifies*, and a sentence saying it works forwards only — is its own change.

**The translated document a person reads.** The payload is a data format and its
field names are identifiers, as `format` is in `dock.toml`; the page a reader sees
is a translated rendering of it and the payload is what is signed. That page must
carry the payload's digest **and say that it is a rendering and not the record**,
because a well-typeset page in a reader's language carrying a real digest looks
exactly as convincing whether or not it was rendered honestly.

**The verifier.** Nothing in this repository yet checks an attestation. Until
something does, `held-by` is a field nobody reads.

## Rejected

**The owner signs every machine's every period.** Disqualified above: it makes the
artifact's production an egress to us, and makes a customer's compliance document
depend on our reachability.

**The vendor signs a digest it has not seen.** Notarisation, not attestation.

**One key for the image and the attestations.** The image key's leak compromises
every machine ever installed; an attestation key's leak forges one machine's
statements. Sharing them would give the smaller job the larger blast radius, and
would put the image key on every machine — the exact thing ADR 0036 exists to
prevent.

**A boolean *in hardware: yes/no*.** Four different promises reported as one, so a
verifier accepts the weakest believing it accepted the strongest.

**Leaving the protection out of the signed bytes.** A strength claim that can be
removed without breaking the signature is a claim the artifact does not really
make.

## What is asked of the owner

1. **The machine with no such hardware**: refuse to attest, or attest saying
   `a-file-on-disk`.
2. Confirmation that a per-machine attestation key is **a different key from ADR
   0036's**, which this record assumes and argues for but does not have the
   standing to settle.
3. Whether the *say so at setup* sentence belongs in setup's own words — it is a
   sentence a person reads, so it needs translating with the rest.
