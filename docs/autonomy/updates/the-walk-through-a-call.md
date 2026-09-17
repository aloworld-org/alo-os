# Every sentence, and the walk through a call

**Date:** 2026-09-17
**Workstream:** v0.5 — capture, and the room you are sitting in
**Task:** *Every sentence, and the walk through a call*
(`docs/autonomy/v0-5-capture-and-the-room-plan.md`, task 7)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
tested and linted in the Lima VM (Ubuntu 24.04 aarch64).
**Egress:** none.
**Status:** done. **The plan is finished.**

## The walk, as a person meets it

Read out of the vocabulary the whole machine loads, in order — a screenshot with
a password blurred, a call, and thirty seconds recorded with the microphone:

| What they are doing | What they read |
|---|---|
| they pick the blur tool | *Hide this — it cannot be brought back once you save* |
| before they share anything | *This call cannot see your screen* |
| they share their whole screen | *This call can see your whole screen, including anything that appears on it* |
| they choose what the recording will hear | *The microphone — this records anyone speaking near this machine, not only you* |
| the screen is being read | *The screen is in use by org.alo.Recorder* |
| the microphone is listening | *The microphone is in use by org.alo.Recorder* |

## What reading it through showed

Each of these passed its own test before today. In order, they hold together in
a way worth stating:

- **The two hardest sentences are the two about other people.** The microphone
  line and the whole-screen line both warn about somebody who is not looking at
  the screen — the person in the room, and the person whose message would arrive
  during the meeting. They are the only two sentences in this walk that carry a
  clause after a dash or a comma, and in both cases that clause is the whole
  point.
- **Nothing in the walk is a dead end.** The blur says what it costs *before* it
  is used; the share says what can be seen while it is happening; the indicator
  names the application rather than saying *something*.
- **The indicator says who**, not what kind of thing. *In use by
  org.alo.Recorder* is a name a person can act on; *screen in use* is a light
  they would learn to ignore.

## Nothing names the machinery

A second test reads every sentence both crates declare for twelve words:
*PipeWire*, *portal*, *Wayland*, *VP9*, *AV1*, *Opus*, *Matroska*, *codec*,
*encoder*, *ffmpeg*, *GStreamer*, *XDG*. None appears.

A person sharing a window has no use for the word *PipeWire*, and a person
reading *VP9* learns only that somebody left a note to themselves in the
product. What they need is what it will do and whom it reaches, which is what
the sentences say.

## Every sentence has a translator's note

Both crates, every word. The notes carry what the sentences must not lose in
translation — that the blur is not reversible, that *your voice* is the wrong
rendering of a microphone, that the promise in *and nothing else on your screen*
is why somebody picked a window at all.

## The plan is finished

| | |
|---|---|
| 1, 2 | done before this lane took the plan |
| 3 | annotation, and a blur destroyed into the file |
| 4 | a recording that begins silent, and refuses before it starts if the machine cannot keep up |
| 5 | a call sees what was picked this time; no notification on a shared screen |
| 6 | the agent's one road to the screen, with the finding that no such verb exists |
| 7 | this |

And on the way, **[ADR 0051](../../decisions/0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md)**
— taken from the devices plan because it was blocking tasks 4, 5 and 7 here and
no machine held it.

## What this lane leaves open, for whoever comes next

- **The decoding half of ADR 0051** is the owner's: whether the image may carry
  H.264, HEVC and AAC decoders in the EU. `alo-playing`'s closed list waits on
  it, and so does the rest of the devices plan's task 1.
- **No verb captures the screen** (task 6's finding). The road is built and
  refuses everything else; declaring the verb is `alo-capability`'s and the
  agent-verbs contract's.
- **A recording that fails mid-way keeps what it had** — the value is here, the
  running recording is the shell plan's surface.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, with the test gate narrowed to the crates this lane touched.
