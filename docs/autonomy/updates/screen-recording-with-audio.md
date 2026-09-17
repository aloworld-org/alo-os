# Screen recording, with audio, to a file

**Date:** 2026-09-17
**Workstream:** v0.5 — capture, and the room you are sitting in
**Task:** *Screen recording, with audio, to a file*
(`docs/autonomy/v0-5-capture-and-the-room-plan.md`, task 4)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
tested and linted in the Lima VM (Ubuntu 24.04 aarch64). The encoder speeds this
rests on were measured for
[ADR 0051](../../decisions/0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md)
in that VM — 6 CPUs, 3 GB, no graphics card.
**Egress:** none. (ADR 0051's measurement installed ffmpeg in the VM yesterday;
nothing new was fetched here.)
**Status:** done.

## What changed, for somebody outside this repository

A recording of the screen, a window or a part of one, written to a folder the
person chose, as **VP9 and Opus in Matroska** — or AV1 where the hardware can
encode it (ADR 0051).

**And the sound is a decision, every time.**

## Sound is never the default, and never remembered

A recording begins with **no sound**. There is no constructor that starts with
the microphone on, and nothing in this crate remembers what somebody chose last
time — **because the room is not the same room.** Yesterday they were alone;
today a colleague is at the next desk.

The four choices, as a person reads them:

| | |
|---|---|
| *No sound* | the picture and nothing else |
| *This machine's sound — nobody in the room is recorded* | what the machine plays; the second half of that sentence is the point of it |
| *The microphone — this records anyone speaking near this machine, not only you* | |
| *This machine's sound and the microphone — this records anyone speaking near this machine* | |

The microphone's sentence says **where it reaches**, not whose it is. Its
translator's note forbids *your voice* and *your microphone*, which say the
opposite of what happens: a microphone records whoever is in the room, including
people who are not looking at the screen and have not agreed to anything. The
in-use indicator is honest for the person holding the machine and invisible to
everybody else in it, and no sentence can fix that — it can only avoid lying
about it.

`Sound::TheMachine` exists because it is what most screen recordings actually
need: the thing being demonstrated, and nobody in the room.

## The machine says before it starts, not after

ADR 0051's rule, and ADR 0008's *never a silent fallback*. A recording that
cannot be encoded at the screen's size and rate is **refused before anything is
written**:

> *This machine cannot record this screen as fast as it happens, so nothing was
> started. It can record a smaller picture — choose that size and start again.*

The numbers are shown beside it: the size, the frames a second it wants, the
frames a second this machine manages. Where nothing smaller would help, the
sentence says that instead.

**It offers the smaller size and does not choose it.** A recording quietly made
smaller, or quietly dropped to fifteen frames a second, hands somebody a file
whose fault they discover when they watch it back — usually once, usually when
it mattered.

## Tested directly: while it records, the indicator says so

The same discipline as task 6, for the same reason. *The microphone was on and
nothing said so* is the worst shape of this failure, because the person it fails
is the one in the room who never looked at the screen.

| Test | What it holds |
|---|---|
| recording with the microphone | the microphone is in use, and the line is the recorder's rather than the agent's |
| recording without it | **the microphone stays dark** for *no sound* and for *this machine's sound* — a light that is always on says nothing |
| every recording | the screen is in use, whatever the sound |
| the sentences | the microphone's says *anyone speaking near this machine*, and does not say *your voice*; the machine's own sound says *nobody in the room is recorded* |

## What is decided here, and what is not

**Decided:** what a recording is, what sound it carries, what it is encoded as,
where it goes, and what the machine says when it cannot keep up.

**Not here:** the encoder itself is rented and pinned (ADR 0011, ADR 0051), the
selection and the indicator are drawn by the shell plan, and stopping from the
indicator is that plan's surface acting on this crate's value. The *keeps what
it had* rule for a recording that fails mid-way belongs with the running
recording, which is the shell plan's task 12 — named here so it is not assumed
to be done.

## A third and fourth group of words

This crate's vocabulary was *told* and *refused*; task 3 added *tools*. This adds
**the four sounds** — a question with four answers, asked every time — and two
refusals. The test that walks the list walks all four groups.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, with the test gate narrowed to the crates this lane touched.
