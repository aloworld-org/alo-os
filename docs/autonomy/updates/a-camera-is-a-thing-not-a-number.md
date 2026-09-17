# A camera is a thing, not a number — and off is below the door

**Date:** 2026-09-17
**Workstream:** v0.5 — devices and media
**Task:** [task 4](../v0-5-devices-and-media-plan.md) — the camera and the
microphone
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2.
Everything measured in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6 CPUs,
3 GB of memory**, PipeWire 1.0.5, WirePlumber 0.4.17, kernel 7.0.0-31.
**Egress:** into the VM on 2026-09-17, from Ubuntu's own archive:
`gstreamer1.0-tools`, `gstreamer1.0-pipewire`, `gstreamer1.0-plugins-base` (a
client that can open a camera through the media server — see below for what
happened), and an attempt at `v4l2loopback-dkms` + `v4l-utils` +
`linux-headers-7.0.0-31-generic` which **failed to build** against this kernel and
was not needed: the kernel's own `vivid` module was used instead, which is in
`linux-modules-extra` and was already here.
**Status:** the code, and four of the five acceptances taken on a machine. The
fifth is stated plainly below and is not claimed.

## What this is

`crates/alo-cameras`, and the first thing to say about it is that it is a **fifth
crate in a plan that named four**. The plan's crate list — `alo-sound`,
`alo-bluetooth`, `alo-playing`, `alo-power` — was written before task 4 was, and
task 4 names no crate at all. The camera list had to live somewhere that is this
plan's, and the switch that turns *the camera or the microphone* off for everyone
is one mechanism, so splitting it across two crates would have made two switches.
If the owner would rather it were called something else or lived elsewhere, it is
a rename.

| File | What it is for |
|---|---|
| `reported.rs` | what a camera is, and what it has of its own |
| `seen.rs` | the media server's record, read for cameras |
| `the_media_server.rs` | asking this machine |
| `switch.rs` | the switch, and the refusal that names which one |
| `letting_go.rs` | telling the kernel to let go of a device |
| `turning_off.rs` | the two halves of *off*, as one act |
| `keeping.rs` | the two switches, kept (ADR 0038) |
| `words.rs` | the six sentences |

Nineteen tests, two of which are on a machine.

## A camera is never `/dev/video0`

`CameraId::reported` **refuses** a device file. The number the kernel hands a
camera is the order it happened to find it in: plug in a second camera and
yesterday's `video0` is today's `video2`, with a grant still pointing at it. That
is ADR 0040's own argument, and it is the failure that is invisible until the day
it happens.

So the identity is what the device says about itself by way of the media server —
here `v4l2_input.platform-vivid.0` — and `tests/a_camera_is_named_by_what_it_is_not_by_a_number.rs`
takes it on a machine the way a cable does: the camera is taken away from the
kernel, given back, and the machine is asked again.

> this machine's camera(s) came back under the same name(s): `v4l2_input.platform-vivid.0`

The device number is used in exactly one place — as a road to the hardware when
the machine is letting go of it — and is never remembered.

## Off is held below the door

A switch checked where grants are checked covers the programs that ask politely.
alo OS ships a terminal on purpose, so a person's machine has programs on it that
never asked anybody. **Turning the camera off therefore takes the device away
from the kernel**: the driver is told to let go, and there is no device file left
for anything at all to open.

`tests/off_means_there_is_nothing_left_to_open.rs` measures exactly that, by
looking for the device file afterwards:

> this machine let go of 1 camera(s); while it had, there was no device file for
> anything to open

And where a machine cannot release a device — a driver holding several at once, a
microphone on a card that also plays sound — the crate **says so** and the
sentence a person sees changes with it: *this machine cannot release this device,
so off is kept at the door. A program you run yourself outside the workspace could
still reach it.* An honest limit, in the vocabulary, rather than a promise quietly
not kept.

The other half is the door itself, and it is held against the real machinery:
`tests/an_application_with_a_grant_gets_nothing.rs` makes a real
`alo-capability` grant of `Facility::Camera` to an application, has the real
`alo-portals` judge a real camera request and allow it — and then the switch says
no anyway. The grant is not revoked, hidden or weighed: **the switch is not a
permission, it is the machine**, and when it goes back on the application has what
it had.

## What was not taken, and why

The acceptance *a test opens the camera through the portal and finds it listed on
the in-use indicator* is **not** done, and it is not done for a reason that is
worth more than the test would have been:

**On WirePlumber 0.4.17, nothing can open a camera through the media server.** The
camera is in the graph, complete with its formats, its device file and its serial
— and `pw-cat --record --media-type Video` answers `no target node available`
while `gst-launch-1.0 pipewiresrc` answers `stream error: target not found`, by
name, by serial, and with no target at all. Both are the server's own tools
talking to its own node. Ubuntu 24.04 ships 0.4.17; the video policy everybody
writes about is 0.5's. **The certified image should pin WirePlumber 0.5
deliberately**, and this test is the reason.

And a second measurement, which is the more important of the two:

**A program that opens a camera directly does not appear on the indicator.** With
`v4l2-ctl --stream-mmap` pulling frames at five a second from `/dev/video0`, the
media server's own record still said the camera's node was `suspended` — so
`alo-in-use`, which counts running sources, has nothing to show. Both are in
`docs/quirks.md`.

That is not a bug in the indicator; it is the honest shape of the claim. An
application on alo OS is sandboxed (ADR 0005) and has no other road, so *every
application's use appears*. A program a person runs in their own terminal has
another road, deliberately. **It is also exactly why this crate holds off below
the door**: the one thing that covers a program nobody vouched for is there being
no device.

## What this machine could not be asked

- **A hardware privacy shutter or light** is reported as
  `OfItsOwn::ThisMachineCannotTell`, on every camera, here. That is honest rather
  than lazy: neither the kernel nor the media server says anything about either,
  and a machine claiming to know about a camera's light would be claiming the one
  thing a person can check with their own eyes. Where a device does report a
  privacy control, reading it is an `ioctl` and wants a machine with such a camera
  to write it against.
- **The microphone half of the switch** is held at the door only. Letting go of a
  microphone means letting go of the sound card it is on, which takes a person's
  speakers with it — so the crate does not, and says which.

## For other lanes

- **Three crates now reach the media server through their own copy of the same
  twenty lines**: `alo-in-use`, `alo-sound` and `alo-cameras`. None of them owns
  *reaching the media server*. A small crate that did would be a better home for
  it than three copies, and it is a proposal for whoever holds the media stack
  rather than a change to make from inside one plan.
- **The portal's implementation must ask this crate's switch**, after it has
  judged the request and before it hands anything over. The test above holds the
  two together from this side; the call itself belongs to whoever writes the
  portal service.
- **The image should pin WirePlumber 0.5**, per the quirk.
