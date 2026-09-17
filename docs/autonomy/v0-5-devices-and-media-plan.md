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
machine plays, which codecs it carries and under what terms), and `crates/alo-power`
(the battery, power profiles, and what a person is told about both). **It reads and
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

**Status:** **the decision is written — [ADR 0051](../decisions/0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md),
2026-09-17.** Its encoding half is **accepted**: everything alo OS produces is
AV1 or VP9, Opus and Matroska, royalty-free, with AV1 only where hardware can
encode it — measured, not preferred. Its decoding half is **put to the owner**,
because whether the image may carry H.264, HEVC and AAC decoders in the EU is a
legal position rather than a technical one. **`alo-playing`'s closed list and its
per-format tests wait on that half**, so this task is done as far as a lane may
take it and the rest is the owner's. **Depends on:** nothing.

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

**Status:** blocked — on `v0-5-capture-and-the-room-plan.md` task 1, whose
`alo-in-use` is the indicator every camera and microphone use must appear on.
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

**Status:** ready. **Depends on:** nothing.

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

**Status:** ready. **Depends on:** 1, 2, 3, 4, 5.

- **Acceptance:** every sentence these crates can say is in the vocabulary with a
  translator's note; one walk — pair Bluetooth headphones, join a call on the laptop's
  speakers, switch to the headphones mid-call, turn the camera off, play a video file,
  reach low battery — produces the exact sequence a person meets, recorded as a table
  and held by one test; no sentence names PipeWire, WirePlumber, BlueZ, libcamera,
  a codec's library or a profile daemon.
- **Constraint:** nothing here re-decides what the sentences describe.
