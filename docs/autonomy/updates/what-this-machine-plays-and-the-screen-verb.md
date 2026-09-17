# What this machine plays, and the verb that sees the screen

**Date:** 2026-09-17
**Workstream:** v0.5 — devices and media; v0.5 — capture, and the room you are
sitting in
**Task:** the decoding half of
[ADR 0051](../../decisions/0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md)
(devices task 1), and the screen verb that capture task 6 found missing
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
tested and linted in the Lima VM (Ubuntu 24.04 aarch64).
**Egress:** none.
**Status:** done. **One question inside the amendment is open and marked for
counsel**, with a shipment as its deadline.

## The amendment: what this machine plays

The owner decided it; this writes it down, in ADR 0051 rather than a second
decision record, because it is the other half of one question.

**The rule.** alo OS plays what a person was already sent — refusing their own
video on their own machine is what ADR 0039 refused to do for documents. **And
the right to decode comes from the hardware, or from somebody who licensed
redistribution, never from shipping a decoder we have no right to distribute.**
A machine may play an H.264 file because the chip in it holds a licence, or
because a company that paid for redistribution gave us a binary to pass on. It
may not play one because we compiled a decoder and hoped.

**In this order**, and the order is not a preference — it is the order in which
the right to decode exists at all:

1. the machine's own hardware decoder (VA-API, V4L2);
2. a redistributable licensed decoder, such as Cisco's `openh264`;
3. a plain refusal, naming the format and what would play it.

**The refusal is the one a person already knows.** `alo-opening`'s
`Cannot::NothingHereOpens(Kind)` — *it is recognised, and nothing on this machine
opens or converts it* — and **not a second shape for video**. One sentence for *I
cannot open this*, whether it was a document or a film. A refusal written
specially for media would drift from the first, and a person would have to learn
two ways of being told the same thing.

## What is open, and the deadline that is not a version number

**Which software decoders may ship in the image.** Written precisely in the ADR:
*for a machine sold and distributed in the EU, which of AAC-LC, baseline H.264,
main-profile H.264 and HEVC may alo OS include as a software decoder, and under
what attribution or notice?*

The technical facts are stated and the legal ones are not guessed: AAC-LC and
baseline H.264 are near or past expiry depending on filings and jurisdiction —
*near or past* being a position nobody here may assume in either direction — and
HEVC is not close, with licensing split across more than one pool.

The ADR tables what turns on each answer: whether the common case (a phone video
on a machine with no hardware decoder) plays at all, and whether the image takes
on an attribution obligation it must then keep for ever, including in derived
images.

**The deadline is a shipment: before the certified laptop goes to anybody
outside this team** — the first moment alo OS is distributed to a person who did
not build it, and therefore the first moment a licensing position is something we
hold rather than something we are thinking about. *Before v1* is not a date
anybody can act on.

Until then the image ships **no software decoder for an encumbered format**:
steps 1 and 3 are live, step 2 where the publisher's terms are unambiguous, and
anything else is refused by name.

## The screen verb, declared

Capture task 6 found that `docs/contracts/agent-verbs.md` listed no verb that
captures the screen, built the road such a verb would come down, and did not add
the verb. The owner's instruction was to open it now rather than leave it for
somebody in a hurry — so:

| Verb | Effect | Arguments | Sentence |
|---|---|---|---|
| `picture_of_the_screen` | change | none | *Take a picture of your screen* |

Declared in the contract, in `docs/by-hand.md` (what a person does instead:
press the screenshot key, and blur what should not be sent — *by hand is not
worse here, it is slower and strictly more private*), and in code as
`alo_capturing::verbs::declare_into`.

**It takes no arguments** because there is no version of this narrower than
*your screen*. An argument naming a region would read as *a picture of this
corner* while the machine held a picture of everything.

## What changed while declaring it, and why it is better

I wrote it first as *requires the `screen-once` facility*. **`alo-capability`
refuses that**, and the refusal is a decision: ADR 0040 keeps facilities — the
camera, the screen, notifications — to **applications and never to agents**,
because *a durable grant to the camera would be a background reader by another
name*. `GrantError::NotForAnAgent` is that sentence in code.

Which is the owner's *no remembered answer* rule, already decided, in a place I
had not looked. Task 5 refuses *always allow this application to see my screen*;
ADR 0040 refuses the same thing one layer down for agents. They agree, and the
agreement is why the verb now rests on nothing standing at all:

- **an agent holds nothing over the screen**, ever;
- **the approval of the sentence is the whole authority**, one picture at a time;
- `ForTheAgent::approved` refuses an authority from any other verb, refuses one
  that **did not come from an approval**, and takes the authority **by value and
  spends it** — so a second picture needs a second approval, and the compiler
  says so rather than a comment.

It is the second verb in the contract requiring no grant, for a different reason
from the first: `install_application` needs none because there is nothing on the
machine yet to grant over; this needs none because what a grant would be over is
something an agent may never hold.

## What the tests hold

| | |
|---|---|
| the verb | a change, no arguments, and the written reason names ADR 0040 and what checks it |
| another verb's authority | refused — an approval for some other change is not an approval for a picture of everything |
| an authority with no approval behind it | refused |
| one approval | one picture: the authority is moved into the call and cannot be used twice |
| the indicator | the screen, **by the agent**, terracotta with ADR 0010's mark — and an application's line is not terracotta |
| the daemon's crates | no road to a picture of the screen except this one |

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, with the test gate narrowed to the crates this lane touched.
`alo-citing`'s pointer tests pass.
