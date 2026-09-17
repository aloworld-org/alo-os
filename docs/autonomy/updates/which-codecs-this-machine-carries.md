# Which codecs this machine carries, decided before anything plays

**Date:** 2026-09-17
**Workstream:** v0.5 — devices and media
**Task:** *Which codecs this machine carries, decided before anything plays*
(`docs/autonomy/v0-5-devices-and-media-plan.md`, task 1)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** the encoding was **measured in the Lima VM — Ubuntu 24.04 aarch64,
6 CPUs, 3 GB of memory, no graphics card** — on an Apple M3 host with 8 GB. That
machine is the point of the measurement: it is what modest hardware looks like.
**Egress:** one — `apt-get install ffmpeg` in the VM (6.1.1-3ubuntu5), to measure
encoders rather than assert their speed. Nothing was added to the image.
**Status:** done as far as a lane may take it.
**[ADR 0051](../../decisions/0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md)**
— its encoding half **accepted**, its decoding half **put to the owner**.

## Why this lane took this plan

This task was blocking three of the Mac's own capture tasks — 4, 5 and 7 — and
it sat `ready` with no machine holding it. Taking the plan unblocked this lane
rather than waiting for a machine that might never take it. Both rows in the
lane table are updated in this commit, and the capture plan now says which
decision its tasks were waiting on and who took it.

## The decision, in short

**Encoding and decoding are two questions.** Answering them together is the
mistake people make: it produces either an image shipping patent-encumbered
encoders it does not need, or a machine that refuses to play a person's own
video.

**What this machine produces is royalty-free, and that is accepted:** AV1 or
VP9, Opus, in Matroska. No H.264, no HEVC, no AAC in anything alo OS writes. The
licence fee is the smaller half of the reason — **a product sold on sovereignty
that owes a per-unit royalty to a patent pool is selling something it does not
own.**

**What this machine plays is the owner's**, because it is a legal position with
a commercial consequence, not an engineering choice. The ADR names the question,
the four options and what each costs.

## AV1 or VP9 is a measurement, not a preference

Ten seconds of 1920×1080 at 30 frames a second, ffmpeg 6.1.1, on 6 CPUs with no
graphics card:

| Encoder | Took | Against real time | Size |
|---|---|---|---|
| SVT-AV1, preset 8 | **39.7 s** | 4.0× too slow | 6.5 MB |
| SVT-AV1, preset 10 | **14.9 s** | 1.5× too slow | 6.9 MB |
| VP9, realtime, cpu-used 8 | **1.5 s** | **6.5× faster than needed** | 11.2 MB |
| SVT-AV1, preset 10, at 1280×720 | 8.3 s | 1.2× too slow | — |
| Opus at 96 kbit/s | 0.09 s | — | 144 kB |

**AV1 cannot record a screen live on a machine without an AV1 encoder in
hardware** — not at 1080p, not at 720p, not at its fastest preset. VP9 does it
with room to spare, at about 1.7× the size. So: AV1 where hardware encodes it,
VP9 where software must, and the machine measures rather than assumes.

Reproduce it with `ffmpeg -f lavfi -i testsrc2=size=1920x1080:rate=30:duration=10`
into each encoder; the commands are in this report's commit.

## When it cannot keep up, it says so before it starts

A recorder that quietly drops to fifteen frames a second gives a person a file
whose fault they discover **when they watch it back** — usually when it matters,
usually once. So a recording that cannot be encoded at the screen's size and
rate is **refused before it starts**, naming what would work; and one that falls
behind while running stops and keeps what it had. ADR 0008's never-a-silent-
fallback, and the same instinct as the documents work saying what a conversion
lost.

## The room the microphone is in — a constraint, not a dialogue

The owner's point, and it is in the ADR rather than only in a sentence: recording
the screen with sound records **whoever is speaking near the machine**. A
colleague at the next desk, somebody on a call across the room, a child in the
background. None of them is looking at the screen; none of them agreed; and the
in-use indicator that makes this honest for the person holding the machine is
invisible to everybody else in the room.

So it shapes what gets built:

- **sound is never the default**, and is chosen before each recording rather
  than remembered — the room is not the same room;
- **the sources are named** — the microphone, the machine's own sound, both, or
  neither, and *the machine's own sound* records nobody in the room;
- **the sentence says where it reaches**: *this will record anyone speaking near
  this machine, not only you*;
- **nothing listens between recordings** — no transcription, no keyword
  detection, and `alo-in-use` shows the microphone exactly while a recording
  runs.

## What is left to the owner

Whether the image may carry **H.264, HEVC and AAC decoders** in the EU. These
are what people actually have — a phone video, a camera, a colleague's capture —
and a machine that cannot play them is a machine somebody returns. The ADR sets
out four options with their costs: ship in the image; rely on a runtime an
application brings; a licensed binary from a vendor; do not ship.

**`alo-playing`'s closed list and its per-format tests wait on that half**, which
is why this task is done as far as a lane may take it. Until then, a format this
machine cannot play is reported through `alo-opening`'s *cannot open* road with
what would play it — never a player that shows nothing.

## What this unblocks

Capture tasks **4** (screen recording with audio), **5** (sharing the screen in
a call) and **7** (the walk) are `ready` again, and this lane returns to them
next.

## A citation this lane nearly repeated

This report's ADR named `0008-when-the-machine-cannot-answer.md` in its first
draft. The real file is `0008-where-inference-happens.md`. It was caught by
listing `docs/decisions/` before committing — the habit this lane adopted after
three pointers in `alo-hosted` named ADR 0014 by a filename it no longer had and
broke `alo-citing` on `main` for every machine. `alo-citing`'s ten pointer tests
pass on this tree.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, with the test gate narrowed to the crates this lane touched. This change
is documents only; `alo-citing` was run directly.
