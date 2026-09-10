# An operating system that does not claim what it cannot know

- Date: 2026-09-10
- Workstream: model selection and configuration (`alo-models`, `alo-asking`)
- Contributor: Claude Code
- Task: ADR 0021 — what a service on this machine vouches for
- Status: **accepted and built.** C1, Option B, D1 with the wording corrected.

## The decision, and whose it was

The owner gave a standing delegation on 2026-09-10 — *make decisions where
needed* — against the goal of an AI-native operating system that is the best in
the world. They did not review the options one by one, and neither the ADR nor
this report pretends otherwise. It is written down so it can be overturned with
one line.

ADR 0021 had been **proposed and unaccepted since 2026-09-08**, and it was the
only thing standing between this workstream and *implementation complete for the
in-scope v0.01 requirement*. It is now accepted.

## What was wrong, in one sentence

`Served::source()` answered **where was this processed** with a fact about
**what address was contacted**, and those two differ in exactly the case that
matters: a proxy on loopback.

alo OS said *on this machine*. It knew the address was this machine's. It did
not know — could not know, cannot know — what the process behind that address did
with the question next.

## Why this was worth doing before anything more impressive

The hole itself is not closed and cannot be, by this or any operating system: a
process the person started can forward a question anywhere. **That limitation is
shared with every OS in existence.** The false label was not.

For a product whose entire claim is sovereignty, a label that is not true is
worse than the gap it papers over. The gap is a limitation. The label is a
betrayal — found once, believed never again. Nobody buys this for the feature
list; they buy it because when it says a thing, the thing is so.

## What was built

**The two kinds of local are now different sources.** The types already knew
which door an answer came through — `Answers::Runtime` against
`Answers::Service` — and only the source they reported collapsed them.

| | Says | Because |
|---|---|---|
| The runtime alo OS ships and manages | *on this machine* | no socket at all; the claim is carried by the absence of one |
| A service the person runs, at a loopback address | *by a service at this machine's address — alo cannot verify where it was processed* | an address establishes what was contacted, never what did the work |

Externalised as a word with a translator's note, like every other user-facing
string. A variant rather than a flag, so every `match` on a source has to say
which of the two it means.

**Bounded identically, described differently.** This is ADR 0021's D1 and it is
asserted as a pair, because either half without the other is a decision the
repository did not take: refusing a service under *this machine only* would break
every honest vLLM user to inconvenience nobody, since no configuration is
verifiable today. The permission and the admission ship together.

**A new miswiring became representable, so it is refused.** `Miswired::NotTheRuntime`
— the runtime answering for a service, or the reverse. It costs nobody anything,
and it is what keeps the provenance line about the thing that actually answered.

## The tests that were designed to stop me

Two tests existed specifically to pin the old behaviour so that *"whoever changes
it has to come here"*. They both failed, exactly as intended, and both are now
updated to assert the new truth:

- `this_machine_only_still_permits_a_service_that_cannot_be_verified` — still
  permitted (D1), and now told, in the same breath, what alo OS cannot check.
- `a_service_that_forwards_is_answered_as_though_it_never_left`, renamed to
  `a_service_that_forwards_no_longer_claims_the_answer_stayed_here`. It stands up
  a **real relay to a real far service** and confirms the question genuinely
  leaves. The indicator is still quiet and the record still shows no egress —
  both truthful, since no socket left this machine — and the provenance line no
  longer claims the answer stayed.

That is the gap, still open, no longer lied about.

## The promise, rewritten to match

`docs/features.md` is the only list of what gets built, so the claim changed
there too rather than quietly in code:

- The **egress indicator** line now names its exception: a service the person
  themself put on this machine is their own trust boundary, not one alo OS
  polices.
- The **zero inference egress** line now says *the runtime alo OS ships*,
  specifically — because that is the configuration the measurement can carry, and
  a service somebody else runs opens a loopback socket the measurement cannot see
  past.

## What is explicitly not promised

**An enforceable local-only guarantee.** It is qualified by **supervision, not
ownership** — a third-party runtime under alo's supervision would qualify and
alo's own outside it would not — and it needs a mechanism that does not exist.
Until then no configuration qualifies, alo OS's own included, and D4's stricter
setting is not offered. Offering an empty guarantee is the thing this ADR was
written to prevent.

## One consequence worth watching

`InferenceSource` is serialised into the record, so the new variant is a tag
(`a-service-at-this-machines-address`) that an older alo OS has never heard of.
That is the behaviour `docs/contracts/record-file.md` already describes for a
record from a newer alo OS: the line is reported as unreadable rather than
guessed at. Nothing existing is rewritten and no old record changes meaning.

## Verified

The whole workspace suite, green. Every failure along the way was a test
asserting the old claim, and each was changed deliberately with the reason
written beside it — no test was weakened to make this pass.

## Proposed integration updates

**CHANGELOG.md** — when an answer comes from an inference service you run
yourself, alo OS now says it cannot verify where the question was processed,
instead of reporting it as answered on this machine. Nothing about which models
or services you may use has changed.

**ROADMAP.md** — the *On the machine* half of the egress-indicator line is
implementation-complete for v0.01; hardware acceptance is still pending and no
machine box may be ticked.

**docs/autonomy/QUEUE.md** — ADR 0021 accepted; C1 built; the loopback gap is
accepted and stated rather than open and unstated.

**docs/autonomy/STATE.md** — `InferenceSource` distinguishes the runtime alo OS
ships from a service the person runs at a loopback address; they are bounded
identically and described differently.
