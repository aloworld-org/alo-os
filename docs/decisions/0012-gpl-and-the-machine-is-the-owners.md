# ADR 0012 — GPL-3.0 stays, and its lock-out clause is a promise rather than a problem

**Status:** accepted — confirms this repository's licence and settles the
conflict between GPL-3.0 §6 and signed images, before either is built
**Date:** 2026-09-03
**Context:** `LICENSE` (GPL-3.0), `README.md`,
[ADR 0004](0004-the-organisations-machine.md) (personal or managed),
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) (the image
is a bootable container), `ROADMAP.md` v1 — *signed images verified before boot;
Secure Boot with our key* — and `alo-browser`'s ADR 0009, which relicensed the
engine on the same day for the opposite reason

## The decision in one line

alo OS stays **GPL-3.0**, and its anti-tivoization clause — which requires that
a person be able to install a modified version on a device they own — is
**adopted deliberately as an expression of the product's own promise**, not
worked around.

## Why the question came up

The browser moved from Apache-2.0 to MPL-2.0 on the same day, so the obvious
next question is whether this repository's licence is right either. It is, and
for a reason that is the mirror image of the browser's.

**The browser needed to be embeddable.** Its value is adoption: an engine nobody
can put inside their product is an engine nobody uses, which is why file-level
copyleft was the answer there.

**Nobody embeds an operating system.** What this repository needs is the
opposite guarantee — that the parts which make it *alo* stay open if somebody
ships them. `alo-capability`'s grant model, `alo-record`'s record,
`alo-agentd`, the shell: those are what a competitor would want, and GPL-3.0 is
what stops them being enclosed.

- **MPL would be too weak here.** A competitor could build a proprietary
  operating system around the capability model and publish only the files they
  happened to edit.
- **AGPL would add nothing.** Its network clause protects hosted services; a
  desktop is not one. It would buy no protection and cost real enterprise
  goodwill, since procurement departments have been trained to fear those four
  letters.
- **Permissive is not on the table.** It hands over the whole argument.

## The clause worth deciding on purpose

GPL-3.0 §6 requires that when GPL-3 software is conveyed inside a **User
Product**, the recipient gets the Installation Information needed to run a
*modified* version on that device. It is the anti-tivoization clause, and it is
the reason the Linux kernel deliberately stayed on GPL-2.0 — Torvalds wanted
hardware vendors to be able to lock devices.

It appears to collide with `ROADMAP.md`'s v1 line: *signed images verified
before boot; Secure Boot with our key.*

**It does not collide. It says the same thing this product already says.**

ADR 0004 divides every machine in two, and the division answers this cleanly:

- **A personal machine** — *"no policy, no escrow, no remote wipe, nobody above
  the person."* Here GPL-3.0 §6 is simply the licence restating that promise in
  legal language. A person who owns an alo OS machine can replace what runs on
  it. Signing exists so they can tell whether an image is ours, **never so that
  only ours will run.** A locked personal machine would violate ADR 0004 before
  it violated the licence.
- **A managed machine** — the organisation owns it, sets policy, holds the
  recovery key, and **the person is told exactly that from the first sign-in.**
  Locking here is the owner's own choice about its own property, and the honesty
  requirement is already ADR 0004's and is stricter than any licence.

So the rule is: **Secure Boot verifies provenance; it never enforces
exclusivity.** Custom key enrolment stays available on a personal machine, and
where an organisation removes it, ADR 0004 requires the person be told.

**This is the rare case where a licence obligation and the product thesis are
the same sentence.** Sovereignty means the owner controls the machine. GPL-3.0
§6 means the owner controls the machine. Nothing needs reconciling; it needs
writing down before Secure Boot is implemented, which is what this ADR is.

## Consequences

- The licence stack is settled across all three repositories, each for its own
  reason rather than by default:

  | | Licence | What it protects |
  |---|---|---|
  | `alo-browser` | MPL-2.0 | embeddable, but engine improvements come back |
  | `alo-os` | GPL-3.0 | the capability model cannot be enclosed |
  | `alo-workplace` | AGPL-3.0 | closes the hosting loophole |

- **The image is an aggregate and stays one** (`README.md`): every rented
  component keeps its own licence and each image publishes an SBOM naming them.
  ADR 0011's bootc base does not change that.
- **Whoever implements Secure Boot inherits a constraint from here**: key
  enrolment is not to be disabled on a personal machine. If that turns out to be
  impossible on some hardware, that is a fact about the hardware and belongs in
  `docs/hardware.md` — such a machine is not certified.
- **Trademarks are separate and still worth taking.** No open licence lets
  anyone call their fork "alo OS".

## Alternatives rejected

**GPL-2.0, to avoid §6.** Rejected, and it is worth being explicit: the only
thing this buys is the ability to lock a person out of a machine they own, which
is the one thing this product exists to refuse. It would also be incompatible
with Apache-2.0 dependencies that GPL-3.0 accepts.

**AGPL-3.0.** Rejected: no protection gained on a desktop, real cost in
enterprise procurement.

**MPL-2.0, matching the browser.** Rejected: the browser needs adoption and this
needs integrity. Same family, opposite requirement, and copying the answer
across would be the mistake.
