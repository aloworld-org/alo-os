# WirePlumber 0.5 is not the blocker, and nobody had run it

**Date:** 2026-09-19
**Workstream:** v0.5 — devices and media, task 4's fifth acceptance
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** measured in the Lima VM on that Mac — **Ubuntu 24.04.4 aarch64, 6
CPUs, 4 GB**, PipeWire 1.0.5, against the kernel's own `vivid` device. Nothing is
ticked *on the machine*.
**Egress, recorded:** Ubuntu's own archive for `meson`, `ninja-build`,
`liblua5.4-dev`, `libpipewire-0.3-dev`, `libspa-0.2-dev`; and two source
tarballs from `gitlab.freedesktop.org` — WirePlumber **0.5.17**
(`13d1e4456e64bcc81111ec5a4af75d0bd6316040c5718b388b14103a8a77cc6b`) and
**0.5.2** (`24ecc2323f7c39fe577b50903c324cfcbb77b9ea2da01baffd3467c9dbad1d8a`).
Nothing else was fetched.

## What this was

Task 4's fifth acceptance — *an application opens the camera through the portal
and appears on the in-use indicator* — has been blocked since 2026-09-17 with one
stated cause: **it wants WirePlumber 0.5.** That sentence is in the plan, in
`docs/quirks.md`, in two reports, and it is half the reason I pinned a 0.5 floor
into `image/Containerfile` on 2026-09-19.

It had never been run. It was inference from release notes — *the video policy
people write about is 0.5's* — and it propagated as though it were a measurement.

So I ran it.

## How, without putting the gate machine at risk

Both builds went into their own prefix (`/opt/wireplumber-0.5.17`,
`/opt/wireplumber-0.5.2`), unmodified upstream, configured and never patched
(ADR 0011). The packaged 0.4.17 was left installed and untouched, and each test
session ran on **its own socket** (`/run/pw-05`) so the gate's session on
`/run/pw-test` was never competing for the same ALSA loopbacks.

Afterwards the session was stopped, its socket removed, and `alo-sound`,
`alo-cameras`, `alo-in-use` and `alo-media-server` were re-run: **all green.** The
machine is as it was found.

## What happened

| WirePlumber | Result |
|---|---|
| **0.5.17** | the camera node is never created — *Failed to activate V4L2 node `v4l2_input.platform-vivid.0`: enum params id:2 (Spa:Enum:ParamId:Props) failed*. Version skew: 0.5.17 is two years newer than PipeWire 1.0.5 |
| **0.5.2**, contemporary with PipeWire 1.0.5 | the node **is** created, complete and healthy — and **nothing can attach to it** |

On 0.5.2 the node is everything it should be: class `Video/Source`, backed by
`/dev/video0`, state `suspended`, with a full YUY2 `EnumFormat` running from
320×180 upwards and every framerate `vivid` offers. And:

- `gst-launch-1.0 pipewiresrc path=<node id>` — `target not found`
- `gst-launch-1.0 pipewiresrc target-object=<serial>` — `target not found`
- `gst-launch-1.0 pipewiresrc target-object=<node name>` — `target not found`
- `gst-launch-1.0 pipewiresrc` with no target at all — `target not found`
- `pw-cat --record --media-type Video` — `no target node available`, 0 bytes

**Identical to 0.4.17.** The refusal did not move.

## What this means

**The blocker is not the session manager's version**, and the plan, the quirk and
my own image report all said it was. All three now say what was measured.

What the acceptance actually waits on is **named rather than guessed**, and none
of it has been tested:

- **the `xdg-desktop-portal` camera road** — which is what the acceptance asks
  for in its own words, *an application reaches a camera only through the portal
  and its grant*, and which is not running on this machine. This is the most
  likely answer and the most likely reason direct node access is refused;
- **PipeWire's own camera path**, which is 1.0.5 here and much older than either
  WirePlumber tried;
- **`vivid` versus a real camera** — a kernel fixture may differ in a way that
  matters, and every camera measurement this repository has is against it.

## The image pin stands, on its other reasons

`image/Containerfile` pins a floor of 0.5 and refuses below it in the build. That
stays, and it is still right: 0.4 is the old line, and the larger fault the pin
fixed was that **the recipe shipped no media server at all** — neither `pw-dump`
nor `wpctl` — so every crate reaching sound or cameras answered *nothing here
handles sound and video*, correctly.

But the pin's report said the floor *makes the acceptance possible*. **That half
is withdrawn.** It was the unmeasured sentence, and I repeated it.

## What it cost, and why it was worth it

Two builds and a session, against a claim that a lane could have gone on
repeating for months. The result is a negative — nothing is unblocked — and it is
the most useful thing I could have produced for this task, because *it wants 0.5*
was about to be satisfied by the image and would then have looked solved. A
blocker that has moved into the image and quietly stayed is worse than one
everybody can see.

**The rule this is another instance of**, already adopted here on 2026-09-17: a
run that decides something must execute in the same environment as the thing it
decides about. *The release notes say 0.5 fixes it* is a judge that never saw the
machine.
