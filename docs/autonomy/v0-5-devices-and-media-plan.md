# v0.5 — devices and media: sound, Bluetooth, the camera, playback and the battery

**Workstream:** the `ROADMAP.md` v0.5 line *Devices: audio with mid-call switching,
Bluetooth, camera, microphone, media playback, power management*, and the matching
`docs/features.md` section *Devices and media*. *Night light* is in the session and
displays plan; *sleep on lid close* is too. `docs/features.md` says why this plan
matters more than it sounds: *an operating system with a brilliant agent and no
working Bluetooth is not a product.*
**Why it exists:** written 2026-09-15 so that no v0.5 line is without a plan.

**Crates this plan owns, all new:** `crates/alo-sound` (outputs and inputs, which is
in use, and switching mid-call), `crates/alo-bluetooth` (pairing a device, what a
paired device may be, and forgetting it), `crates/alo-playing` (what media this
machine plays, which codecs it carries and under what terms), `crates/alo-power`
(the battery, power profiles, and what a person is told about both),
`crates/alo-cameras` (the cameras this machine has and the switch that turns the
camera and the microphone off for everyone — task 4 named no crate and needed one),
and **`crates/alo-media-server`** (reaching the rented media server and reading what
it says). **`alo-media-server` is read by crates this plan does not own** —
`alo-in-use` and `alo-capturing`, both the capture plan's — and that is the point of
it: four crates had their own copy of the same twenty lines, the copies drifted, and
each fix had to be made three times. It says nothing to a person and decides nothing
about what a node means; those belong to the crates that know which question is being
asked. **It reads and
never edits** `alo-portals` and `alo-capability` (camera and microphone are
grantable facilities, ADR 0040), `alo-in-use` (the capture plan's indicator — a
microphone switched mid-call is still a microphone in use), `alo-sleeping` (the
session plan's — suspend is not a power profile), `alo-nearby` (a Bluetooth pairing
is **not** an alo pairing, and the two must never be confused), `alo-opening` (a
media file is a file `alo-opening` decided something about), and `alo-saying`.
**Nothing in `crates/alo-shell`.**

**What this plan may not do:** tick anything *on the machine* — a headset that has
never switched mid-call on certified hardware is `- [x] The code.`; write an audio
server, a Bluetooth stack, a camera pipeline or a codec (PipeWire, WirePlumber, BlueZ,
libcamera and the rented codecs are configured and never patched, ADR 0011); ship a
codec whose licence or patent terms the image cannot carry without an ADR saying so;
or name any of them to a person. Before writing the next task, `git pull` and read
the plan as published.

## Tasks

### 1. Which codecs this machine carries, decided before anything plays

**Status:** blocked — **on a machine to play a real sample file on, and on
nothing else.** The question that was the other half of this block was answered
on 2026-09-19. **Everything a lane may take is taken.** A supervisor cannot start
what is left, so this says so in the one word it reads: a status it cannot parse
is a finished task it selects again for ever, which is how this line was found.

The decision is written — [ADR 0051](../decisions/0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md),
2026-09-17 — and everything that is not hardware is built. Its encoding half is
**accepted**: everything alo OS produces is AV1 or VP9, Opus and Matroska,
royalty-free, with AV1 only where hardware can encode it — measured, not
preferred. Its decoding half was answered the same day except for one question,
*which software decoders may ship in the image*, and
[ADR 0058](../decisions/0058-which-software-decoders-the-image-ships.md) closed
that on 2026-09-19: **we ship a decoder when nobody can charge us for shipping
it.** In practice AV1, VP9 and every audio codec in the list decode in software;
H.264 goes to the silicon or to Cisco's `openh264`; HEVC goes to the silicon
alone, and without it an HEVC file is refused by name.

`crates/alo-playing` holds all of it — the codecs as a closed set, split by
*free by design* against *term expired*, because only the second is arguable;
what this machine produces, tested exhaustively to be free on every machine and
for every purpose; **the order in which a right to decode exists at all** (free,
then expired, then the silicon, then a redistributable licensed decoder, then a
refusal), walked once per track because *a kind is the wrapping, not the codec*;
the list of decoders the image itself carries; and the refusal, which is
`alo_opening::Cannot::NothingHereOpens` and **not a second shape for video**.
What a machine has is handed in rather than read, so a laptop with no video
hardware and a workstation that decodes HEVC are both testable here. **One row
of the list rests on a patent term** — AAC-LC — and that is named as a type
constant and asserted in both directions, so a lawyer's answer cannot arrive and
change nothing. 43 tests. Written up in
[What this machine plays](updates/what-this-machine-plays.md).

**What is left, and only this:** a test per format that plays a real sample file
through the rented stack. It needs a machine, which cannot be written.
**Depends on:** nothing.

*Media playback, and the codecs people actually have files in.* Some of those
codecs carry patent licences, and an image distributed across the EU carries their
terms. That is a decision, not a package list, and it comes first.

- **Acceptance:** a decision record, numbered after `git pull` when it is written,
  sets out which audio and video formats alo OS plays out of the box, which it plays
  only through an application a person installs, and which it does not — with each
  codec's licence and patent position stated and its source, the options (ship in
  the image; ship through a rented runtime an application brings; a licensed binary
  such as a browser vendor's; do not ship), what each costs, and a recommendation;
  `alo-playing` holds the decided list as a closed set with a test per format that
  plays a real sample file through the rented stack; and a format this machine does
  not play is reported through `alo-opening`'s *cannot open* road with what would
  play it, never as a player that shows nothing.
- **Constraint:** the decision is proposed and marked so; tasks that play or record
  a format it has not settled wait on it. No codec is added to the image before it
  is accepted.

### 2. Sound out and in, and switching mid-call

**Status:** **Done, 2026-09-17.** `crates/alo-sound`, with the switch and the mute
taken on a machine against the kernel's own loopback cards — a call moved to a
device plugged in mid-call and never dropped, and a muted microphone's stream read
back and found empty while its volume stood where a person left it. Written up in
[Sound out and in](updates/sound-out-and-in-and-switching-mid-call.md).
**Depends on:** nothing.

- **Acceptance:** `alo-sound` holds the outputs and inputs the rented audio server
  reports, by a stable identity that survives a replug, and which one each is in use
  for; **plugging in or connecting a headset during a call moves that call's audio to
  it without the call dropping**, held by a test against the rented server's own
  loopback devices, and unplugging moves it back; a person can pin a device so it is
  always chosen when present; volume and mute per device are kept by this crate
  (ADR 0038); and a microphone mute is a real mute of the source, not a lowered
  gain, with a test that reads the stream and finds silence.
- **Constraint:** WirePlumber's policy is configured, not replaced. What is ours is
  which device a person meant, remembered.

### 3. Bluetooth: pairing, audio, keyboards and mice

**Status:** **Done, 2026-09-17.** `crates/alo-bluetooth`: nothing is paired with
that a person did not choose from what was found and say yes to, the four things
a device can ask are shown in full and asked of a person by an agent that has no
branch answering for them, forgetting is one act that takes the keys, and **a
pairing grants nothing** — held against the real `alo-nearby` and
`alo-capability`. On a machine, the part a machine with no radio can take:
*no Bluetooth here* is a different sentence from *nothing found*. Written up in
[Pairing a device, and what it is not](updates/pairing-a-device-and-what-it-is-not.md).
**Depends on:** 2.

- **Acceptance:** `alo-bluetooth` pairs a device only when a person chose it from what
  was found, and shows the passkey or confirmation the device requires in full, never
  auto-accepting; a paired device's kind — audio, keyboard, mouse, other — decides what
  it becomes, and an audio device joins task 2's list; forgetting a device is one act
  and removes its keys; **a Bluetooth pairing grants nothing on this machine in the
  sense of ADR 0001 or ADR 0031** — a test holds that `alo-nearby`'s pairings and
  `alo-capability`'s grants are untouched by pairing a device, because a headset is not
  a machine and must never look like one on the one list; and the radio can be turned
  off, and off means off for every device.
- **Constraint:** BlueZ is rented and unmodified. No automatic pairing of anything,
  including devices that ask to be paired.

### 4. The camera and the microphone

**Status:** blocked — **on reaching a camera through the portal, on a machine
with a desktop session** — *or* on why a V4L2 node never becomes a linkable,
which is a second explanation measured later the same day and is **cheaper to
decide than the first**. See *Two explanations, and how to tell them apart*
immediately below the account of the two faults. Updated 2026-09-20, and this is
the third thing this line has said it was waiting for, so what changed is
written out rather than swapped over. Two faults were found in our own code and fixed, and **neither of
them was the camera's**: a video capture that never said what kind it was, so
the session manager searched by an unknown kind and refused it with *target not
found*; and a pipeline handed to `gst-launch-1.0` with each element's settings
glued to it in one argument, which that tool reads as one word and refuses as a
syntax error — so the screenshot road had never carried a picture on any
machine. Both are proved on the development PC against a source with no camera
in it: without the kind, zero bytes by id, by name and with no target; with it,
768,000 bytes each time, and a real 320×240 PNG out of the whole screenshot
pipeline. **The camera itself is still refused** with both fixes in, on
WirePlumber 0.4.17 and 0.5.13, by the server's own `pw-cat` as well as ours.
What that re-opens is the portal: ~~*a portal cannot make a link the graph will
not make*~~ is **withdrawn** — the session manager's own
`client/access-portal.lua` gates exactly the nodes a camera is
(`media.role = Camera` with `media.class = Video/Source`) on the portal
permission store, so a portal does not link, it grants the permission without
which no link is offered. That is a reading of the engine's script and not yet
a measurement. `docs/quirks.md` carries all of it.

**Two explanations, and how to tell them apart, 2026-09-20 (later the same
day).** A second measurement in the Lima VM, on WirePlumber **0.4.17**, found
something upstream of any permission question: **the camera is not a linkable at
all.** The policy chooses a target only from `linkables_om`, and printing every
member of it at the moment of a failed attach gives the six loopback audio
nodes, a synthetic `Video/Source`, and the client — and **not** the camera.
WirePlumber never makes a session item for it: instrumented, `create-item.lua`'s
`addItem` is called for every audio node and for the synthetic video source, and
never for `v4l2_input.platform-vivid.0`, in a run where that node is present and
complete. A portal grants permission *on a node*; here nothing is choosing among
candidates that include the camera, so on this stack no permission store entry
could produce a link.

The same measurement **confirmed the undeclared-kind fault on a second stack and
through a second code path** — on 0.4.17 it is `policy-node.lua`'s `canLink()`
rejecting on `media.type` being `nil`, not 0.5's `prepare-link.lua` — so that
fault spans two major versions.

**The experiment that decides between them is cheap and needs no desktop
session:** print `linkables_om`'s members on **0.5.13** at the moment of a failed
camera attach. Absent there too, and the portal cannot be the explanation on that
stack either; present, and the portal reading stands and the above is a 0.4-only
quirk. Written up in
[The camera is not a candidate](updates/the-camera-is-not-a-candidate.md), which
also records that `vivid` being *eliminated* as a variable rested on a control
that was itself failing for the undeclared-kind fault, and is withdrawn.

**The code is
written and four of the five acceptances are taken, 2026-09-17**; the fifth was
recorded as waiting on WirePlumber 0.5, and on 2026-09-19 that was measured and
is false. Two 0.5 releases were built from upstream and run against this
machine's PipeWire 1.0.5 and `vivid`: 0.5.17 cannot activate the camera node at
all, and 0.5.2 — contemporary with this PipeWire — creates a complete, healthy
`Video/Source` node that **still nothing can attach to**, by node id, by serial,
by name or with no target. `docs/quirks.md` carries the measurement.

What the fifth acceptance actually waits on was then narrowed the same day by a
control experiment: in one session, the same client attached to an **audio**
source and captured 1.9 MB, and could not attach to the **camera** at all. So the
session, the client, the addressing and the client's permissions all work, and
**the refusal is video-specific**. That also rules out the portal — `alo-portals`
declares `Portal::Camera` and the obvious guess was that a camera is reachable
only through a portal handing over a connection, but a portal hands out a
connection and connections demonstrably work. **A portal cannot make a link the
graph will not make.**

Two candidates were left and neither was tested: whether `vivid` differs from a
real camera in a way that matters — every camera measurement here was against that
one fixture — and PipeWire 1.0.5's own V4L2 capture path. **Both were eliminated
on 2026-09-20, on the development PC** (Intel Core Ultra 7 155U), in a KVM guest
running Ubuntu 24.04.5, kernel `6.8.0-139-generic`, PipeWire 1.0.5 and
WirePlumber 0.4.17 — a different machine, a different architecture from this
lane's, and the first stock distro kernel available here that has `vivid` at all
(WSL's own kernel has no such module).

The control that settles it takes the camera out of the experiment. A synthetic
video stream was published *into* the graph with
`gst-launch-1.0 videotestsrc ! pipewiresink mode=provide` carrying
`media.class=Video/Source` — **no V4L2, no `vivid`, no kernel device anywhere in
it** — and `pipewiresrc` failed to attach to that with the identical
`stream error: target not found`, in a session where `pw-record` captured
**1,298,476 bytes** of audio. So the refusal is neither `vivid` nor the V4L2
path; **it is video in this graph**, and the fifth acceptance does not wait on a
real camera being found.

Three further measurements say what it is *not*, and each kills the obvious
answer:

- **Not permissions, and not the portal.** `pw-cli info` on the camera node
  reports the client's permissions as **`rwxm-`** — everything — which confirms
  from the server's side what the 2026-09-19 control argued from the client's.
  ~~and not the portal~~ — **withdrawn 2026-09-20.** The permissions reading
  stands; the portal conclusion drawn from it does not. See the status above.
- **Not a node that failed to form.** `vivid` under this kernel is a real
  source: raw V4L2 captured **13,824,000 bytes** at 4.99 fps from it, and the
  graph's node `v4l2_input.platform-vivid.0` advertises a full `EnumFormat` —
  YUY2 320x180 with fourteen framerates. WirePlumber **0.4.17 does create the
  node**, which narrows the sentence below from *0.4.17 cannot* to *0.4.17
  creates a node nothing attaches to* — the same sentence already recorded for
  0.5.2, which makes the version far less interesting than it looked.
- **Not a missing format on the client side**, which the logs make look likely
  and which is a trap worth writing down. `PIPEWIRE_DEBUG=3` shows
  `find_format(): no format given` immediately before
  `error (-32) target not found`, so the obvious fix is to supply one — and
  supplying the node's own advertised caps, addressed by node id, by node name,
  and with no target, **failed all three times with the same error and zero
  bytes**. The warning is not the cause, and `target not found` is a misleading
  message for whatever is.

What is left is one question inside the media server's own stream connection,
and **no further camera measurement needs a camera**. The image's 0.5 floor stays — 0.4 is
the old line and the recipe shipped no media server at all — but it was pinned
partly on this belief, and that half of the reason is withdrawn. Its blocker
cleared: the capture plan's task 1 is done. `crates/alo-cameras` (a fifth crate, and why is
in the report): cameras are listed by what they are and never by a device number,
the switch turns the camera or the microphone off for everyone, and **off is held
below the door** — the machine lets go of the hardware, measured on a machine by
looking for the device file afterwards. **What is not taken:** *an application opens the camera
through the portal and appears on the in-use indicator*. On WirePlumber 0.4.17 — what
Ubuntu 24.04 ships — **no client can attach to a camera through the media server at
all**, measured with two of the server's own tools (`docs/quirks.md`), so there is no
machine here on which the test could pass or fail honestly. ~~It wants WirePlumber 0.5,
which the image should pin deliberately.~~ **Corrected 2026-09-20: it does not want
WirePlumber 0.5.** 0.4.17 was measured on the development PC to create the camera node
perfectly well, and to refuse the attach in exactly the way 0.5.2 does — and a
*synthetic* video source with no camera in it is refused the same way. The version is
not the variable. The pin below stays for the separate and still-good reason in its own
paragraph (the recipe carried no media server at all); what is withdrawn is the belief
that 0.5 is what this acceptance is waiting for. Written up in
[A camera is a thing, not a number](updates/a-camera-is-a-thing-not-a-number.md).
**The pin is in, 2026-09-19** — `image/Containerfile` now installs `pipewire`,
`pipewire-utils` and `wireplumber` and **refuses a WirePlumber below 0.5 in the
build itself**, which also ended a larger fault: the recipe carried no media
server at all, so neither `pw-dump` nor `wpctl` was on a machine built from it
and every crate that reaches sound, cameras or the in-use indicator answered
*nothing here handles sound and video*. `image/` belongs to the installer plan
and was taken deliberately while nobody was inside it, with the reason in
[The image carries a media server](updates/the-image-carries-a-media-server.md).
**The acceptance is still not taken**: it needs a machine running a built image,
and this lane gates on aarch64 against WirePlumber 0.4.17.
**Depends on:** 2.

- **Acceptance:** cameras are listed by a stable identity through the rented camera
  stack and the media server, never by a `/dev/videoN` number (ADR 0040's reason);
  an application reaches a camera or microphone only through the portal and its grant
  (`alo-portals`), and every use appears on the capture plan's in-use indicator —
  a test opens the camera through the portal and finds it listed; a person can turn a
  camera or the microphone off for everyone, and **off is enforced below the portal**
  so an application with a grant still gets nothing; and a camera with a hardware
  privacy switch or light is reported honestly as having one or not.
- **Constraint:** no image processing, background blur or *beautify* is written here.

### 5. The battery, and power profiles

**Status:** **Done, 2026-09-17.** `crates/alo-power`: the battery is read straight
from the kernel's own files, a person is told once at low and once at nearly gone,
and **how long is left is not said at all** unless four rules hold — the rules are
in one file and each is a test that names it. Power profiles are the rented
daemon's own list, and a profile this machine's hardware does not do is absent
rather than greyed out; on a machine, a profile was chosen, read back and put
back. The battery half was taken through the kernel's own **test** battery, driven
down past both marks. Written up in
[A battery, and what a machine will not guess](updates/a-battery-and-what-a-machine-will-not-guess.md).
**Depends on:** nothing.

- **Acceptance:** `alo-power` reads the battery the kernel reports — charge, whether
  it is charging, and time remaining only when the reading is steady enough to mean
  something, with the test naming the rule — and a person is told once at low and once
  at critical, never repeatedly; power profiles are the rented daemon's closed set
  (saver, balanced, performance), chosen by a person and kept by this crate; a
  charge limit is offered where the hardware supports one and absent where it does
  not (not greyed out); and **a running local model is named** when it is what is
  draining the battery, because *why is my battery gone* is a question this machine
  can answer (`alo-measuring`, read).
- **Constraint:** no power-management daemon of our own. Nothing here suspends the
  machine; that is `alo-sleeping`'s.

### 6. Every sentence, and the walk through a working day's devices

**Status:** **Done, 2026-09-17.** One walk — pair headphones, take a call, move it
to them mid-call and back, mute, cover the camera, open a video file, run the
battery down — held against the table in
[The walk through a working day's devices](updates/the-walk-through-a-working-days-devices.md),
which the test reads rather than a copy of. Twelve moments, nine sentences and
three that say nothing; all thirty sentences of the four crates are in the
machine's vocabulary with a note, and none names the rented stack. **Step 9 was a
finding and is now fixed:** a video file read as *this machine does not recognise
what this file is*, because `alo_opening::Kind` had no media kinds. It has nine
of them since 2026-09-17, so a film reaches a person by the road ADR 0051's
amendment chose — the walk's table moved with it, in
[a follow-up](updates/the-walk-through-a-working-days-devices-after-media-kinds.md).
**Depends on:** 1, 2, 3, 4, 5.

- **Acceptance:** every sentence these crates can say is in the vocabulary with a
  translator's note; one walk — pair Bluetooth headphones, join a call on the laptop's
  speakers, switch to the headphones mid-call, turn the camera off, play a video file,
  reach low battery — produces the exact sequence a person meets, recorded as a table
  and held by one test; no sentence names PipeWire, WirePlumber, BlueZ, libcamera,
  a codec's library or a profile daemon.
- **Constraint:** nothing here re-decides what the sentences describe.
