# ADR 0058 — Which software decoders the image ships

**Status:** **accepted** by the owner on 2026-09-19, closing the question
[ADR 0051](0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md)
left open under *What is open, and for counsel*. **One item is marked for a
lawyer's confirmation** before the certified laptop goes to anybody outside this
team — named below, and it is a narrower question than the one 0051 asked.
**Date:** 2026-09-19
**Context:**
[ADR 0051](0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md)
(encoding is royalty-free; the right to decode comes from the silicon or from a
licence, never from hope),
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) (the base is
rented, and what it already carries we inherit rather than choose),
[ADR 0008](0008-where-inference-happens.md) (a machine says what it could not do
rather than quietly doing something else), `crates/alo-playing`, and
`docs/autonomy/v0-5-devices-and-media-plan.md` task 1, which this unblocks.

## The question in one line

**For a machine sold and distributed in the EU, which software decoders may alo
OS put in the image it ships?**

## What was already settled, and is not reopened here

ADR 0051 settled the shape and left only the list:

- **Encoding** is royalty-free and finished: AV1 or VP9, Opus, Matroska or WebM.
- **The right to decode** comes from one of three places, in this order: the
  machine's own hardware decoder, then a redistributable licensed decoder, then
  a plain refusal in the sentence a person already meets for a document.

This decision adds the fourth thing 0051 deliberately did not have: **a closed
list of decoders the image itself carries in software**, which is the step that
runs when there is no hardware decoder in the machine.

## The decision

**The image ships software decoders for formats that carry no royalty to
anybody, plus one licensed binary somebody else pays for. It ships no software
decoder we would need a licence to distribute.**

| Format | In the image | Why |
|---|---|---|
| AV1, VP9, VP8 | **yes**, in software | royalty-free by design; AV1 under the AOMedia patent licence, which is granted, not bought |
| Opus, Vorbis, FLAC | **yes**, in software | royalty-free by design |
| MP3 | **yes**, in software | the last patents expired in 2017 and Fraunhofer ended its licensing programme; there is nobody left to license from |
| AAC-LC | **yes**, in software | the core MPEG-2 AAC filings are from 1997 and are past a twenty-year term everywhere we ship. **This is the item for counsel** — see below |
| H.264 | **hardware first, then Cisco's `openh264`** | the licence is paid by the chip maker, or by Cisco for the binary it publishes. We compile no H.264 decoder of our own |
| HEVC | **hardware only** | the patents are not near expiry, the licensing is split across three pools and unpooled holders, and no redistributable binary exists. Without silicon, an HEVC file is refused by name |
| Anything else encumbered | **no** | there is no fifth step |

### The rule underneath the table

**We ship a decoder when nobody can charge us for shipping it** — because the
patents expired, because the format was made free, or because a company that
paid the royalty published a binary for us to pass on. Everything else is the
silicon's job, and where there is no silicon the machine says so.

That rule, rather than the table, is what a later codec is measured against. The
table is what the rule produces on 2026-09-19.

## What we are not doing, and why

**We are not shipping a full media stack and relying on the argument that the EU
does not enforce patents on software.** It is a common position and it is wrong
in the way that matters: the European Patent Convention excludes programs *as
such*, but the EPO grants and European courts enforce patents on technical
processes implemented in software, and codec patents are among the most
litigated of them. A distribution maintained by volunteers is not a target. **A
company selling a machine in the EU is exactly the target**, and that is what alo
OS is.

**We are not going hardware-only either**, which was the safest reading. A
machine with no hardware decoder would then fail to play an ordinary phone video
— the precise failure this product exists to argue against, and one a person
would meet on day one.

So: everything free in software, one licensed binary, silicon for the rest, and
an honest refusal at the end.

## The item for counsel, which is now a narrow question

**AAC-LC.** The core patents are from 1997 filings and are past term; the pool
that licenses AAC still exists, because it also licenses the later extensions
(HE-AAC, xHE-AAC) that are not near expiry. So the position is *the thing we
want has expired, inside a family that has not*.

We ship AAC-LC decode on that reading, and it needs confirming, because:

- It is the audio track in nearly every H.264 file a person will be sent. **A
  machine that decodes the video and refuses the sound is not better than one
  that refuses both** — it is worse, because it looks broken rather than
  principled. This is why the decision does not simply omit it.
- It is the only row in the table resting on *a patent term ran out* rather than
  on *this was never encumbered* or *somebody else paid*.

**The question for a lawyer is therefore one sentence:** *may a machine sold in
the EU include a software decoder for AAC-LC and for MP3, on the ground that
their patents have expired?* That is a cheaper and more answerable question than
the one ADR 0051 posed, and while it is open **the reading above is the one in
the code**, not a placeholder.

**The deadline is unchanged and is still a shipment, not a version: before the
certified laptop goes to anybody outside this team.** If the answer comes back
no, one row of the table changes and `alo-playing` changes with it; nothing else
in this decision depends on it.

**This decision is not legal advice** and does not pretend to be. It is a
position taken on the facts we have, written down so that a lawyer can correct
one row rather than re-derive the whole thing.

## What it costs

- **HEVC on a machine with no hardware decoder does not play.** That is a real
  person with a real file, and they get a sentence naming the format rather than
  a spinner. Most machines made after about 2016 have the silicon; the ones that
  do not are the old ones, which is the wrong way round.
- **`openh264` is a dependency on somebody else's choice to keep paying.** ADR
  0051 already gave that its own answer in `Right::could_be_withdrawn`: the right
  that came from a licence is the one that can be taken away, and the code knows
  which one it is.
- **One row rests on a patent term**, and terms are argued about. Named above so
  that it is argued about on purpose.

## How this is held

`crates/alo-playing/src/right.rs` carries the list as a type, replacing
`SoftwareDecoders::NotAnsweredByCounsel` — the one-value type ADR 0051 put there
to make the hole trip a reader. The acceptance test in
`crates/alo-playing/tests/` reads this document and fails if the table here and
the list in the code stop agreeing, in either direction.
