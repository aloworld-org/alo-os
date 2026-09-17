# The walk through a working day's devices

**Date:** 2026-09-17
**Workstream:** v0.5 — devices and media
**Task:** [task 6](../v0-5-devices-and-media-plan.md) — every sentence, and the
walk through a working day's devices
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
built, linted and run in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6
CPUs, 3 GB of memory**.
**Egress:** none.
**Status:** done, with one sentence in the walk that is a finding rather than a
finished road — step 9, below.

## What this is

The five tasks before this one each end in sentences somebody reads at a moment
when something is already going on: a call, a cable, a battery. Each crate's own
tests hold its sentences one at a time. What none of them can show is **the
sequence** — whether these read as one account when they arrive one after
another, or as four crates talking past each other.

`crates/alo-sound/tests/the_walk_through_a_working_days_devices.rs` walks one
person's day through the real values — a real pairing, a real grant, a real file
— and holds the result against the table below. **The table is the test's
expectation**: a sentence that changes without this table changing fails the
gate, and so does a table edited to say something the machine does not.

## The walk, sentence by sentence

| Step | The moment | What a person reads |
|---|---|---|
| 1 | The headphones ask a person to compare six digits — 123456 | Check that the device shows these same digits, then say yes on both |
| 2 | and it is the first device this machine has paired with | This is a device, not another computer. Pairing it gives it nothing on this machine |
| 3 | A call starts, on the laptop's own speakers | (nothing is said) |
| 4 | The headphones connect mid-call, and take the call | (nothing is said) |
| 5 | The headphones are put away, and the call comes back to the speakers | The device you always use is not connected, so this machine chose another |
| 6 | The person mutes their microphone | Muted — nothing is heard from it |
| 7 | The person turns the camera off for everything | This machine has let go of the camera. There is nothing for a program to open until you turn it back on |
| 8 | An application that was granted the camera asks for it | The camera is off. Nothing on this machine can use it, whatever you have allowed it |
| 9 | A video file is opened | This machine does not recognise what this file is, so it has not opened it Whoever sent it can say which program made it, or send a copy saved in a different format |
| 10 | The battery reaches a fifth | The battery is getting low |
| 11 | and what is using the machine is the person's own model | Your model is what is using this machine |
| 12 | The battery is nearly gone | The battery is nearly gone. Save what you are doing |

## What the table shows

**Twelve moments in a working day produce nine sentences.** Three of them — a
call starting, a headset taking it, the pinned device being the one chosen — say
**nothing at all**, and they are rows in the table for that reason. A walk that
listed only the sentences would hide the best thing about this part of the
machine, which is how little of it a person has to read. The headset taking the
call mid-call is the one people expect a notification for; it does not get one,
because nothing went wrong and nothing needs deciding.

**Each sentence is about the thing in front of the person**, not about what alo
OS is doing underneath. `no_sentence_names_the_rented_stack` holds that as a
test: none of the four crates' thirty sentences contains *PipeWire*,
*WirePlumber*, *BlueZ*, *libcamera*, *V4L2*, *ALSA*, *GStreamer*, *ffmpeg*,
*libaom*, *dav1d*, *power-profiles* or *UPower*. A person pairing headphones is
pairing headphones.

**The two that are promises say so twice.** Step 7 and step 8 are the same
switch from two sides — *there is nothing for a program to open* and *nothing on
this machine can use it, whatever you have allowed it* — and both name the thing
a person would otherwise have to take on trust.

## Step 9 is a finding, and this is the whole of it

Opening a Matroska file today says *this machine does not recognise what this
file is*. That is honest — `alo-opening` really does not recognise it — but it is
**not** what [ADR 0051](../../decisions/0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md)'s
amendment says a person should meet. The amendment reuses `alo-opening`'s
`Cannot::NothingHereOpens(Kind)` — *this is a film, and nothing on this machine
opens it* — and that road needs `alo_opening::Kind` to have video and audio in
it. Its eighteen kinds today are documents, images and a zip archive.

So, for whoever owns `alo-opening` (the documents-and-paper plan) and for task 1
of this plan when its counsel answer arrives:

- **`alo_opening::Kind` needs the media kinds** before `alo-playing` can report
  anything through the road ADR 0051 chose. Until it does, a video this machine
  cannot play is indistinguishable, to a person, from a file that is corrupt.
- Nothing was changed in `alo-opening` here: it is another plan's crate, and the
  list of media kinds is a decision that belongs beside the codec list rather
  than to a walk test.

The walk records what the machine says **today**, which is the point of
recording it.

## Every sentence, collected and noted

`every_sentence_in_these_crates_is_collected_and_carries_a_note` holds the other
half of this task's acceptance over all four crates:

| Crate | Sentences |
|---|---|
| `alo-sound` | 4 |
| `alo-bluetooth` | 10 |
| `alo-cameras` | 6 |
| `alo-power` | 10 |

Every one of the thirty is in `alo-saying`'s vocabulary — the machine's own list,
not a copy — every one carries a note for whoever translates it, and none is
empty.

## What is not here

- **The hardware halves.** This walk is the decisions; the devices in it are
  values rather than a headset somebody plugged in. What has been taken on a
  machine is in each task's own report: a call moved to a card plugged in
  mid-call and did not drop, a muted microphone's stream read back empty, a
  camera released so there was nothing to open, a battery driven down past both
  marks.
- **A person meeting this walk end to end on certified hardware.** Nothing here
  ticks *on the machine* in `ROADMAP.md`'s sense, and the day somebody does walk
  it on the laptop, step 9 is the one to watch.
