# The walk through a working day's devices, after a film became a film

**Date:** 2026-09-17
**Workstream:** v0.5 — devices and media (the walk), and `alo-opening` (the
change that moved a sentence in it)
**Task:** the follow-up the walk's own test asks for when a sentence along it
changes — [the first table](the-walk-through-a-working-days-devices.md) stands as
published, and this is the table that replaces it
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
built, linted and run in the Lima VM on that Mac — **Ubuntu 24.04 aarch64**.
**Egress:** none.
**Status:** done. One row changed, and it is the row that was a finding.

## Why there is a second table

`crates/alo-sound/tests/the_walk_through_a_working_days_devices.rs` reads the
table in the report rather than a copy of it, so that no sentence along the walk
can change without the table changing. Its own rule for when one does: **publish
the table again in a follow-up report and point the test at it, because a
published report is never rewritten.** This is that follow-up.

What changed is step 9, and it changed because the thing the first report called
a finding has been fixed:

> **Before.** *This machine does not recognise what this file is, so it has not
> opened it. Whoever sent it can say which program made it, or send a copy saved
> in a different format.*
>
> **Now.** *This is a Matroska video, and nothing on this machine opens it or
> converts it into something that does. What would open it is a machine with a
> program for this kind of file, or a copy saved in a different format by
> whoever sent it.*

The first is what a person reads about a file that arrived corrupt. The second is
what is true: this machine knows exactly what they have, and has nothing that
plays it.

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
| 9 | A video file is opened | This is a Matroska video, and nothing on this machine opens it or converts it into something that does What would open it is a machine with a program for this kind of file, or a copy saved in a different format by whoever sent it |
| 10 | The battery reaches a fifth | The battery is getting low |
| 11 | and what is using the machine is the person's own model | Your model is what is using this machine |
| 12 | The battery is nearly gone | The battery is nearly gone. Save what you are doing |

Eleven rows are the first table's, unchanged. Step 9's two sentences are joined
by a space in this table because that is how the walk reads what a person is
shown: `Cannot::explained` is **what it is** and then **what would open it**, in
that order, and a person meets both.

## Everything else the first report says still holds

Twelve moments, nine sentences, three that say nothing. No sentence in any of the
four crates names PipeWire, WirePlumber, BlueZ, libcamera, V4L2, ALSA, GStreamer,
ffmpeg, libaom, dav1d, power-profiles or UPower; all thirty are in the machine's
own vocabulary with a translator's note. The hardware halves are each task's own
report.
