# ADR 0077 — A machine signs its own account of what left it

**Status:** proposed, 2026-09-29. The owner directed that where other systems have
already answered these questions, alo OS follows them rather than inventing — which
settles the machine with no security chip (Windows' shape) and the separate key
(the TPM's own endorsement-key/attestation-key split). The one thing still open when
this was written — where the *what your proof is worth* sentence is said — **was
answered on 2026-09-29 and is at the bottom.** Nothing is asked of the owner now.
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
hardware that cannot export it; the artifact names which kind of hardware that
was; and a machine without such hardware can still attest, saying so in the signed
bytes, but only after somebody deliberately turns it on.**

Three answers, and only the first is this product's own. The second is what
Windows, Android and Apple all already do — name the kind rather than claim a
boolean — and the third is the shape Windows chose for BitLocker on a machine with
no TPM. Where somebody shipping at scale has already answered one of these, this
record follows them rather than inventing.

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

## What Windows and macOS do, because two of these questions are already answered elsewhere

This record reasons from this product's own principles, and then checks whether
anybody shipping at scale reached a different answer. On two of the three, they did
not — which is worth knowing before somebody relitigates them.

**Windows names the kinds rather than asking whether.** Microsoft's own
documentation distinguishes discrete, firmware and integrated TPMs from a
software-emulated one, which is *"well-suited for prototyping or testing"* but does
*"not provide the same level of security."* Four kinds, not a boolean — the same
shape as `HeldBy`.

**And Microsoft's stated reason for key attestation is the reason this record
keeps.** TPM key attestation is *"proof that a private key was generated and
remains managed inside a TPM in a non-exportable form, rather than merely being
stored in software where it may be copied or stolen."* That is non-exportability as
the guarantee, in the vendor's own words, and it is why the capability-model
argument was rejected above rather than merely deprecated. The failure Microsoft
names is ours exactly: without attestation, *"someone can easily spoof a software
KSP as a TPM KSP with local administrator credentials."*

**The endorsement key is not the attestation key.** *"Every TPM ships with a unique
asymmetric key called the endorsement key, burned by the manufacturer"* — and
attestation is performed with a separate key rather than that one. The hardware
designers split identity from statement-signing for the blast-radius reason this
record gives, which settles §2 below by convention rather than by argument.

**Apple binds the key to the Secure Enclave and states the same bounded claim.** A
key generated there *"is tied to the Secure Enclave and is therefore available only
on a specific device"*, and the enclave *"has very strong protections against key
extraction, even in the case of a compromised Application Processor."* Compromise
the software and you get signatures made while compromised, not a stolen key — the
limit this record states about itself.

**But Apple's attestation is Apple-signed, and that part is deliberately not
copied.** The enclave key is certified by Apple's servers, which makes the vendor a
participant in every attestation. It is right for Apple because their claim is
*this is a genuine Apple device*, which only Apple can vouch for. Ours is *this is
my machine's account of what left it*, which is the customer's to make — so vendor
participation buys nothing and costs exactly the sovereignty this product is sold
on. Same mechanism, different claim, opposite conclusion.

## The machine with nowhere safe to keep a key

**It attests, saying `a-file-on-disk` — but only after somebody deliberately turns
it on, and the artifact loses the guarantee rather than appearing to keep it.**

This follows Windows rather than being invented here. BitLocker faces the same
question and does not refuse: without a TPM it still encrypts, but an administrator
must first enable *"Allow BitLocker without a compatible TPM"*, the machine then
demands a PIN or a USB startup key at every boot, and Microsoft states plainly that
it *"does not provide the pre-startup system integrity verification offered by
BitLocker with a TPM."* Weaker mode available, **off by default, behind an explicit
act, with the lost guarantee named rather than glossed.**

Refusing outright was rejected: it denies a real capability to a customer whose
hardware is older and who may understand the limits perfectly well, and it is not
what anybody shipping at this scale does.

Attesting silently was also rejected, and this is the part the Windows precedent
sharpens. An artifact identical in every respect but one line is exactly the shape
a reader skims past, so the weaker mode is not something a machine falls into — it
is something somebody asks for.

**And the compliance point makes this urgent rather than academic.** PCI DSS, HIPAA
and ISO 27001 commonly require *hardware-backed* attestation, so a software-key
attestation may not satisfy the very auditor it was produced for. A person must not
discover that from the auditor. Whatever asks them to turn this on has to say it.

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

## A separate key from the image's, settled by convention rather than by argument

A per-machine attestation key is **not** ADR 0036's image key.

The image key's leak lets anybody hand every machine ever installed software that
verifies. An attestation key's leak forges one machine's statements. Sharing them
would give the smaller job the larger blast radius — and would put the image key on
every machine, which is the exact thing ADR 0036 exists to prevent.

This is not a judgement call. A TPM's endorsement key is burned in by the
manufacturer and attestation uses a *separate* key; Android separates its
attestation key from app signing and verified-boot keys; Apple separates code
signing from per-device enclave keys. Every system that does both does them with
different keys. The burden of proof sits on anybody proposing to merge them.

## The one thing with no prior art, which is why it stays

**The person is told at setup what their proof is worth**, in their own language.

Nothing above was copied for this one, because there is nothing to copy. Every
mechanism surveyed reports to a certification authority, a fleet administrator or
an application developer. Microsoft's software-TPM caveat is in documentation for
people configuring certificate services; Apple's enclave guarantees are in a
security guide. **No mainstream system tells the person using the machine, in plain
language, what their attestation is worth.**

Following the field here would mean copying the single thing the field gets wrong,
and the compliance finding is why it matters: the person most likely to be misled
is the one who hands the artifact to an auditor and learns from the auditor that it
does not meet the bar. On a product whose first law is that nothing leaves
silently, a machine quietly holding its key in a file while its owner believes
otherwise is the same failure one layer down.

## What was asked of the owner, and what they said

**Asked:** whether the *what your proof is worth* sentence belongs in setup's own
vocabulary, translated with the rest — and if so, where in setup it appears and how
much it says.

**Answered 2026-09-29, relayed through the laptop lane.** The path is recorded because
it is how the answer reached this record, and a relayed answer written as a direct one
is the same fault this repository spent a day finding in its checks: a true-looking
signal whose subject is not the one a reader would assume.

> **Yes — setup's own vocabulary, translated with the rest.** One short line at the
> moment of switching on. The long explanation lives in Settings. **Not a warning box,
> and not a paragraph at setup.**

### One short line, at the moment of switching on

Not later, because the person who most needs it is the one who will hand the artifact
to an auditor, and they form their belief about what it is worth on the first day. Not
longer, because a paragraph at setup is read by nobody: setup is a corridor somebody is
walking through, and prose in a corridor is scenery.

### The long explanation in Settings

That is where a person goes **when they have a reason to ask**, which is the only time
a longer answer will be read. It also puts it beside the machine's own answer to *whose
machine is this* — a managed machine's person is told so at first sign-in and can find
the detail afterwards (ADR 0004) — and those are the same question one layer apart.

### Not a warning box, which is the half with a reason behind it

A warning box says *something is wrong, and you should stop*. **Nothing is wrong.** A
machine with no security chip still produces a true statement about what left it,
signed by a key held the best way that machine can hold one; what differs is what the
statement is worth to somebody who did not watch it being made. The sentence states
worth, and dressing a statement of worth as an alarm teaches people to dismiss it —
after which the one case that matters is dismissed along with the rest.

This is the degrade-with-friction shape the rest of this record follows, and it is also
where the field's own habit is refused rather than copied. Windows tells the
administrator configuring certificate services; Apple tells the reader of a security
guide; **neither tells the person using the machine.** The answer here is not a louder
version of that. It is a quieter one, in the place a person is already looking.

### And the i18n question is settled by this not being a special case

`CLAUDE.md` externalizes user-facing strings from day one, for all 24 official EU
languages. A line in setup is a user-facing string, so it is translated with the rest
of setup and there is nothing further to decide.

**What was open was never the i18n rule — it was whether this sentence is a setup
string at all**, and the answer is that it is one. The reverse would have been the
interesting case: a sentence held too delicate to translate is a sentence that reaches
only the people who read English, on a product built in Europe, about a proof whose
entire purpose is to be handed to somebody in the language they work in.

## What is still asked of the owner

Nothing. This record is answered.
