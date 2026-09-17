# ADR 0051 — What this machine encodes is royalty-free; what it plays is a separate question

**Status:** **accepted** for encoding — what alo OS produces. **The decoding
half is proposed and put to the owner**, because it needs a legal answer rather
than a technical one; *what is left to the owner* below is the question, the
options and what each costs.
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

## What is left to the owner: decoding

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

**This lane does not choose.** It is a legal position with a commercial
consequence, of the same kind as the speech engine (ADR 0050) and the converter
(ADR 0039) — decisions the owner takes. What this ADR settles is that **nothing
in the image encodes an encumbered format**, which is true whichever way
decoding goes, and which is what the capture plan's recording tasks were waiting
for.

Until the owner answers, a format this machine cannot play is reported through
`alo-opening`'s *cannot open* road with what would play it — never a player that
shows nothing, which is the failure mode this repository already refuses.

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
- **Deciding decoding here anyway.** The temptation is to write *ship them, it
  is fine* and move on. It is not this lane's to write, and a wrong answer is
  discovered by a letter rather than by a test.
