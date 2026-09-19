# ADR 0051 — What this machine encodes is royalty-free; what it plays is a separate question

**Status:** **accepted**, both halves. The encoding half was accepted on
2026-09-17; **the decoding half was decided by the owner the same day** and is
written below as *the amendment: what this machine plays*. One question inside it
is **open and marked for counsel**, with a deadline that is a shipment rather
than a version number.
**Date:** 2026-09-17
**Context:** [ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md)
(engines are rented, configured, never patched),
[ADR 0008](0008-where-inference-happens.md) (never a silent fallback: a machine
says what it could not do rather than quietly doing something else), `docs/autonomy/v0-5-devices-and-media-plan.md` task 1,
`docs/autonomy/v0-5-capture-and-the-room-plan.md` tasks 4, 5 and 7, which wait
on this, and `docs/features.md`'s *Capture: screen recording with audio*

## The question in one line

**Which audio and video formats does an operating system distributed across the
EU encode, and which does it play?**

## Encoding and decoding are two decisions, and answering them together is the mistake

They look like one question — *which codecs?* — and they are not.

**What we encode, we choose.** A screen recording is a file this machine makes
from nothing. Nobody is waiting for it in a particular format, and every
constraint is ours to set.

**What we decode, somebody already sent.** A person has a video in their mail, on
a disk, from a camera, from a colleague. Refusing to play it is refusing them
their own file on their own machine — the same failure as refusing to open their
`.docx`, and this repository already decided how that one goes (ADR 0039: the
document is converted by an engine that can reach nothing, because *not opening
it* was not an option).

One answer covering both means either shipping patent-encumbered encoders we do
not need, or refusing to play what people actually have. So: two answers.

## The decision, for what this machine produces

**1. Royalty-free only, for everything alo OS encodes.**

- **Video: AV1, or VP9 where AV1 cannot be encoded fast enough.**
- **Audio: Opus.**
- **Container: Matroska (`.mkv`), or WebM where a file is meant for the web.**

No H.264, no HEVC, no AAC in anything this machine writes.

**Why, beyond the licence fee.** An image distributed across the EU that carries
patent-encumbered *encoders* ships a licensing liability to every person who
installs it. And a product sold on sovereignty that owes a per-unit royalty to a
patent pool is **selling something it does not own**: the claim and the invoice
contradict each other. The fee is the smaller half of that.

**2. AV1 where the hardware can encode it; VP9 where it cannot — and which one
is a measurement, not a preference.**

Measured on 2026-09-17, in this lane's gate machine — Ubuntu 24.04 aarch64, **6
CPUs, 3 GB, no graphics card** — encoding ten seconds of 1920×1080 at 30 frames
a second, with ffmpeg 6.1.1:

| Encoder | Took | Against real time | Size |
|---|---|---|---|
| SVT-AV1, preset 8 | **39.7 s** | 4.0× too slow | 6.5 MB |
| SVT-AV1, preset 10 | **14.9 s** | 1.5× too slow | 6.9 MB |
| VP9, realtime, cpu-used 8 | **1.5 s** | **6.5× faster than needed** | 11.2 MB |
| SVT-AV1, preset 10, at 1280×720 | 8.3 s | 1.2× too slow | — |
| Opus at 96 kbit/s, ten seconds | 0.09 s | — | 144 kB |

So on a machine with no AV1 encoder in hardware, **AV1 cannot record a screen
live at all** — not even at 720p, not at its fastest preset. VP9 does it with
room to spare, at about 1.7× the file size. That is the whole of the rule: AV1
when hardware encodes it, VP9 when software must.

**3. Audio is Opus at every bitrate we produce**, and this is not a close call:
it is royalty-free, it is better than AAC at the rates a screen recording uses,
and every browser plays it.

## When the machine cannot keep up, it says so

A screen recorder that quietly drops to fifteen frames a second, or to a smaller
picture, produces a file whose fault the person discovers **when they watch it
back** — usually when it matters, usually once. ADR 0008's rule applies exactly — **never a silent fallback**: the machine says
what it cannot do, in words, at the time, rather than quietly doing something
lesser.

So a recording that cannot be encoded at the screen's size and rate **is
refused before it starts**, with what would work — *this machine can record this
screen at 1280×720, or at 1920×1080 if you record without sound* — and never
begun-and-degraded. If a running recording falls behind, it stops and keeps what
it had, which is this plan's own rule for a failed recording.

The same instinct as the conversion losses in the documents work: **say what it
cost.**

## The room the microphone is in

Recording the screen with sound records **whoever is speaking near the machine**
— a colleague at the next desk, somebody on a call across the room, a child in
the background. None of them is looking at the screen. None of them agreed to
anything, and the in-use indicator that makes this honest for the person holding
the machine is invisible to everybody else in the room.

This is a constraint on what is built, not only a sentence in a dialogue:

- **Sound is never the default in a recording.** It is chosen, each time, before
  the recording starts — not remembered from last time, because the room is not
  the same room.
- **The choice is which sources**, named: the microphone, the machine's own
  sound, both, or neither. *The machine's own sound* records no one in the room
  and is what most screen recordings actually need.
- **The sentence says where it reaches**, not what it captures: *this will record
  anyone speaking near this machine, not only you.*
- **No transcription, no keyword detection, nothing that listens between
  recordings.** The microphone is on while a recording is running and at no other
  time, and `alo-in-use` shows it.

## What was left to the owner: decoding — **answered, and see the amendment**

**The question:** may alo OS ship decoders for H.264, HEVC and AAC in the image
it distributes in the EU?

These are what people actually have. A phone video is H.264 or HEVC; a
television recording, a camera, a colleague's screen capture, almost any video
attached to an email. **A machine that cannot play them is a machine somebody
returns.**

The options, with what each costs:

| | What it means | What it costs |
|---|---|---|
| **Ship them in the image** | the image carries the decoders | patent licensing for every unit distributed; the terms differ per codec and per country, and this is a lawyer's answer rather than an engineer's |
| **Ship through a rented runtime an application brings** | the browser or a media application carries its own licensed decoders | works today for most people; the machine itself still cannot play a file that somebody double-clicks |
| **A licensed binary from a vendor** | e.g. a browser vendor's codec bundle, under their licence | legally settled by somebody else; a dependency on their terms and their continued willingness |
| **Do not ship** | the machine plays only royalty-free formats | honest, and the machine cannot play a phone video, which most people will read as broken |

**This lane did not choose.** It is a legal position with a commercial
consequence, of the same kind as the speech engine (ADR 0050) and the converter
(ADR 0039). **The owner decided it on 2026-09-17** — see *the amendment* below,
which keeps one question open for counsel and names the shipment it must be
answered before.

## The amendment: what this machine plays

**Decided by the owner, 2026-09-17**, and written here rather than in a second
ADR because it is the other half of one question.

### The rule

**alo OS plays what a person was already sent.** A video in their mail, from
their phone, from a camera, from a colleague — refusing it is refusing them
their own file on their own machine, which this repository already refused to do
for documents (ADR 0039).

**And the right to decode comes from the hardware, or from somebody who licensed
redistribution — never from shipping a decoder we have no right to distribute.**
Those are different things, and the difference is the whole of this half: a
machine may play an H.264 file because the chip in it holds a licence, or
because a company that paid for redistribution gave us a binary to pass on. It
may not play one because we compiled a decoder and hoped.

### In this order

1. **The machine's own hardware decoder** — VA-API on a graphics card, V4L2 on
   an embedded decoder. The licence was paid for with the silicon, and the file
   never touches a decoder we distributed.
2. **A redistributable licensed decoder**, such as Cisco's `openh264`, which is
   published under terms that let it be passed on with the royalty already paid
   by its publisher.
3. **A plain refusal**, naming the format and what would play it.

The order is not a preference. It is the order in which the right to decode
exists at all, and each step is only reached because the one above it was absent.

### The refusal is the one a person already knows

**`alo-opening`'s `Cannot::NothingHereOpens(Kind)`** — *it is recognised, and
nothing on this machine opens or converts it* — and **not a second shape for
video.** A person meets one sentence for *I cannot open this*, whether what they
double-clicked was a document or a film, with the same shape of *and here is what
would*.

A second refusal written for media would drift from the first, and the person
reading it would have to learn two ways of being told the same thing.

### What was open, and for counsel — **answered on 2026-09-19**

> **This section is history**, kept as written. The shape of the question is
> what made the answer cheap, and the reasoning below is what
> [ADR 0058](0058-which-software-decoders-the-image-ships.md) is a reply to.
> **The list the image actually ships is in ADR 0058**, and
> `crates/alo-playing` carries it as types. What remains for a lawyer is one
> sentence about AAC-LC rather than the four-format question posed here, and
> the deadline stated at the end of this section is unchanged and still governs
> it.

**Which software decoders may ship in the image, and where.**

Patent positions differ by codec, by filing and by jurisdiction:

- **AAC-LC** and **baseline H.264**: the core patents are near or past expiry
  depending on which filings are counted and in which country. *Near or past* is
  not a position a repository may assume in either direction.
- **HEVC** is not close, and its licensing is split across more than one pool,
  which is a different problem from an expired patent and a worse one.

**The question, precisely:** *for a machine sold and distributed in the EU, which
of AAC-LC, baseline H.264, main-profile H.264 and HEVC may alo OS include as a
software decoder in the image it ships, and under what attribution or notice?*

**What turns on each answer:**

| If the answer is | Then |
|---|---|
| **none may ship** | the image carries no software decoder; step 1 and step 2 above are the whole of playback, and a machine with no hardware decoder and no `openh264` refuses an H.264 file — which most people will meet at least once |
| **the expired ones may ship** | AAC-LC and baseline H.264 decode in software everywhere, and the common case — a phone video, an old recording — plays on every machine without hardware help |
| **all may ship under notice** | playback is a solved problem and the image carries an attribution obligation it must then keep correctly, for ever, including in derived images |

**The deadline is a shipment, not a version.** This must be answered **before
the certified laptop goes to anybody outside this team** — the first moment alo
OS is distributed to a person who did not build it, and therefore the first
moment a licensing position is something we hold rather than something we are
thinking about. *Before v1* is not a date anybody can act on.

Until it is answered, the image ships **no software decoder for an encumbered
format**: steps 1 and 3 are live, step 2 is live where the publisher's terms are
unambiguous, and a format that cannot be played is refused by name. That is the
conservative reading, it is honest to a person, and it is the only one that
cannot become a letter.

**What was answered.** ADR 0058 took the second row of the table above — *the
expired ones may ship* — and drew the line at what nobody can charge for: free
by design, or past term, or somebody else's paid-for binary. In practice that
is AAC-LC and MP3 in software, H.264 to the silicon or to `openh264`, and HEVC
to the silicon alone.

## What it costs

- **Bigger files.** VP9 at real time is about 1.7× the size of AV1 at a preset
  no machine here can reach live. A person recording a long session notices.
- **Files some tools will not open.** A `.mkv` of VP9 and Opus plays in every
  browser and in VLC, and does not open in some older editing software. The
  alternative was encoding in a format we do not own the right to produce.
- **Two paths to maintain** where hardware AV1 exists and where it does not,
  which is a measurement at first use rather than a setting a person answers.

## Rejected

- **H.264 for recording, "because everything plays it".** It is the most
  compatible format and we would be paying, per machine, to produce something we
  could produce for nothing. Compatibility is the decoder's problem, and that is
  the half the owner decides.
- **AV1 everywhere, on principle.** Measured above: it cannot record a screen
  live on hardware without an AV1 encoder, which is most hardware today. A rule
  that produces unusable recordings is not a principle.
- **Recording at a lower frame rate when the machine cannot keep up.** That is
  the silent degradation this ADR exists to forbid.
- **Deciding decoding here anyway.** The temptation was to write *ship them, it
  is fine* and move on. It was not this lane's to write, and a wrong answer is
  discovered by a letter rather than by a test. The owner answered it the same
  day, and the one part that still needs a lawyer is marked as needing one —
  which is a different thing from a lane guessing.
- **A second refusal shape for video.** *This film cannot be played* beside *this
  document cannot be opened* is two ways of telling somebody the same thing, and
  the second one drifts. `alo-opening`'s road carries both.

### The road now exists (noted 2026-09-17)

When this amendment was written, `alo-opening`'s `Cannot::NothingHereOpens(Kind)`
was the road it chose and **`alo_opening::Kind` had no media in it** — eighteen
kinds, all documents, images and a zip archive — so a film reached a person as
*this machine does not recognise what this file is*, which is what a corrupt file
reads like.

`alo-opening` now carries nine media kinds, read from the containers people are
actually sent: Matroska, WebM, the container MP4 names (split into a film and a
sound recording with no picture), AVI, Ogg, MP3, WAVE and FLAC. A film this
machine cannot play is therefore already reported the way this amendment says,
and `alo-playing` inherits the sentence rather than inventing one.

**A kind is the wrapping, not the codec**, which is the split this decision
needs: one Matroska file holds AV1 that this machine plays and the next holds
something it may not. `alo-playing` reads the tracks inside against the list
above; `alo_opening::Kind::is_played` says which kinds it is asked about.
