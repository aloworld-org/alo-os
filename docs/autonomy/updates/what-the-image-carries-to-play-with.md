# What the image actually carries to play with: all of the sound, none of the video

**Date:** 2026-09-20
**Workstream:** v0.5 devices and media
(`docs/autonomy/v0-5-devices-and-media-plan.md`, task 1, *Which codecs this
machine carries, decided before anything plays*)
**Contributor:** Claude Code lane B in `/root/alo-os-lane-b` on the **development PC**,
for the owner
**Status:** a measurement. **No code changed and nothing is ticked.** It corrects
that task's status line, which says the remaining half is blocked on a machine
*and on nothing else*. That is not so, and the next worker would have found out
the hard way.

## Why this was measured at all

Task 1's remaining half is *a test per format that plays a real sample file
through the rented stack*, and the obvious way to get sample files is to **make**
them with the royalty-free encoders alo OS already ships rather than download
anything. Before writing that test, the question is what the image has to make
them with, and what it would play them through. So release `0.0.4` was opened and
read — the same image the installer test had just pulled, config
`74a4aa1563c0`, version label `0.0.4`, revision label `b41b4b5e`.

Everything below is `podman run` against that image on the development PC. It is
what the published bytes contain, not what the recipe appears to say.

## What is there, and what is not

[ADR 0058](../decisions/0058-which-software-decoders-the-image-ships.md) is
accepted and says which decoders the image ships. Against the image:

| ADR 0058 says | In release 0.0.4 | |
|---|---|---|
| Opus — yes, in software | `libopus.so.0` | **present** |
| Vorbis — yes, in software | `libvorbis.so.0`, `libvorbisfile.so.3` | **present** |
| FLAC — yes, in software | `libFLAC.so.12` | **present** |
| MP3 — yes, in software | `libmpg123.so.0` | **present** |
| AAC-LC — yes, in software | `libfdk-aac.so.2` | **present** |
| AV1 — yes, in software | no `libdav1d`, no `libaom` | **absent** |
| VP9, VP8 — yes, in software | no `libvpx` | **absent** |
| H.264 — hardware, then Cisco's `openh264` | no `libopenh264` | **absent** |
| HEVC — hardware only | nothing | absent, **and correct** |

**The sound half of the decision is in the image. The video half is not there at
all.** Every audio codec ADR 0058 names can be decoded by a library in the
published release; not one of the video codecs it says we ship in software can be.

## And there is nothing to play anything *through*

The bigger finding is not a missing codec but a missing stack. Searched on `PATH`
in the image:

```
absent  gst-launch-1.0   absent  gst-inspect-1.0   absent  ffmpeg   absent  ffprobe
absent  vlc              absent  mpv
PRESENT pw-cat  pw-play  pw-record  pw-dump  wpctl
```

and no `libgstreamer*` and no `libavcodec*` anywhere under `/usr`. The only media
programs in alo OS are PipeWire's own, which the recipe installs for
`crates/alo-media-server` — and `pw-cat` reads files through **libsndfile**
(`libsndfile.so.1`, linked against FLAC, Vorbis, Opus and mpg123), whose business
is sound. Its `--format` option takes `ulaw|alaw|u8|s8|s16|s32|f32|f64`: sample
formats, not containers full of video.

So *the rented stack* that task 1's remaining half is meant to play a file
through **does not exist in the image for video**, and for sound it is
libsndfile behind `pw-cat` rather than anything chosen for the job.

## The samples cannot be made the way the task assumes either

The plan's intent — make the samples with the royalty-free encoders the image
ships, and say how each was made — runs into the same wall from the other side.
The encoder *libraries* are there (`libvorbisenc`, `libmp3lame`, `libopus`), but
**there is no program in the image to drive any of them**: no `opusenc`, no
`oggenc`, no `flac`, no `lame`, no `aomenc`, no `vpxenc`, no `svt-av1`, no
`ffmpeg`. `pw-record` could write a file through libsndfile, but it records from
a device, and a build host has none.

This does not make the task's instinct wrong — making a sample beats downloading
one, and it should stay. It means the thing that makes it has to exist first.

## What this changes

Task 1's status says it is blocked on a machine *and on nothing else*. Measured,
it is blocked on two things, and the machine is the smaller one:

1. **The image ships no video decoder and no media pipeline**, so *plays a real
   sample file through the rented stack* has nothing to run for AV1, VP9 or H.264
   — on any machine, including a certified laptop.
2. **The image ships no encoder program**, so the samples cannot be made the way
   the task asks.

The audio formats are a different case and are the honest place to start: Opus,
Vorbis, FLAC, MP3 and AAC-LC all have a decoder in the released image and a
program that can reach it, so a test per audio format is writable **now**, and
only needs a machine with a sound device to run on.

**Nothing here proposes adding packages to the image.** ADR 0058 already decided
what may be shipped and this measurement does not reopen it — what it says is
that the decision is accepted and only half applied, which is a gap between a
record and a release rather than a new question. Whether the video decoders
arrive as distribution packages, and which program plays them, is a plan's
decision and an owner's, not this measurement's. It is also the same shape as two
faults this repository has already paid for: the media server that no recipe
installed until 0.0.3, and the document converter whose executable bit was
checked instead of running it until 0.0.4. In all three the crates were right and
the image was empty.

## How to repeat it

```
podman run --rm --entrypoint "" ghcr.io/aloworld-org/alo-os:0.0.4 bash -c \
  'for t in gst-launch-1.0 ffmpeg pw-cat opusenc flac aomenc vpxenc; do \
     command -v $t >/dev/null && echo "PRESENT $t" || echo "absent  $t"; done; \
   ls /usr/lib64/libdav1d* /usr/lib64/libvpx* /usr/lib64/libopenh264* 2>&1 | tail -3; \
   ls /usr/lib64/libopus.so.0 /usr/lib64/libmpg123.so.0 /usr/lib64/libfdk-aac.so.2'
```
