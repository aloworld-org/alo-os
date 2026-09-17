# Sound out and in, and switching mid-call

**Date:** 2026-09-17
**Workstream:** v0.5 — devices and media
**Task:** [task 2](../v0-5-devices-and-media-plan.md) — sound out and in, and
switching mid-call
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2.
Everything measured and every gate run in the Lima VM on that Mac: **Ubuntu 24.04
aarch64, 6 CPUs, 3 GB of memory**, PipeWire 1.0.5 and WirePlumber 0.4.17.
**Egress:** `apt-get install pipewire pipewire-audio-client-libraries wireplumber
pipewire-pulse dbus-x11` from Ubuntu's own archive into the VM, on 2026-09-17 —
the media server this machine did not have until now. Nothing else was fetched,
and nothing left the machine.
**Status:** done. The code, and the parts a machine can show, taken on a machine
with a real media server. **Not** ticked *on the machine* in `ROADMAP.md`: a Mac
with a Linux virtual machine on it is not certified hardware, and no headset has
been plugged into one of these.

## What this is

`crates/alo-sound`: the outputs and inputs the rented audio server reports, which
one each is in use for, what a person set on each of them, and **which device they
meant**. The last of those is the only thing here that is ours. PipeWire routes and
WirePlumber decides policy, both rented and neither patched (ADR 0011). What
neither of them holds is that this person always uses the desk speakers, and that
is the whole crate.

| File | What it is for |
|---|---|
| `device.rs` | a device's identity, kind, and how loud it is |
| `mute.rs` | muted or not, as a value |
| `pinned.rs` | what a person pinned, per kind |
| `keeping.rs` | `sound.toml`: volume, mute and pins, per device (ADR 0038) |
| `heard.rs` | the server's own record, read |
| `asking.rs` | what a machine's sound can be asked and told |
| `server.rs` | this machine's media server, reached through its own tools |
| `meant.rs` | which device a person meant, and why |
| `applying.rs` | the difference between the two, put on the machine |
| `words.rs` | the four sentences this crate can say |
| `testing.rs` | machines to decide against, so every decision is tested anywhere |

Twenty-eight tests in the crate, and two on a machine.

## The two things that had to be taken on a machine

### A call moves to a device plugged in mid-call, and does not drop

`tests/a_headset_plugged_in_mid_call_takes_the_call.rs`. A tone plays through the
machine's chosen output. A second sound card is **plugged in for real** — the
kernel is told to take back a card it had let go of, which is the road a cable
takes — and the person's pin says that card is the one they meant. `bring_into_line`
tells the server, the server moves the stream, and the test then asks the question
people actually mean: **is the program still playing**. It is. Then the card is
unplugged, the sound comes back to the laptop's speakers, the program is still
playing, and the one sentence about a pinned device being away is carried back
rather than left for a person to work out.

And the identity claim, measured rather than asserted: after the replug the device
is found again **under the identity the pin is kept under**, and the number the
server knows it by is a different number. That is the whole reason a pin survives
somebody moving a cable.

### A mute is silence, not a low volume

`tests/a_mute_is_silence_not_a_low_volume.rs`, and it is the test in this crate
worth the most, because it is the claim a person cannot check for themselves. A
volume slider at zero and a muted microphone look identical on a screen and sound
identical to whoever is listening, which is nobody — and they are not the same
thing. One is a microphone that is still listening.

So the test plays a tone into a loopback card and **reads what the microphone's
stream carries**: loudest sample 20,000 of 32,767. It mutes the microphone through
this crate, reads the stream again, and finds **zero** — not quiet, empty. It then
reads the volume back and finds the number a person set, unmoved: a mute
implemented as *take the gain to zero* would have passed the first half and fails
this one. Unmuting brings the sound back, which is what says the silence was the
mute and not a broken fixture.

**Which output feeds which input is found, not assumed** — play into each, listen
on each, use the pair that hears itself. A fixture that hardcoded the pairing would
pass on the machine it was written on and quietly test nothing anywhere else.

### Where they run, and where they skip

Both skip themselves, loudly, on a machine with no media server — the gate runs on
machines that have none. **That means they do not run in the workspace gate unless
a session is up**, and a skip nobody reads is the same colour as a pass, so: they
were run here against a live PipeWire, in the VM, and passed in 1.5 s and 12.4 s.
Anyone running them needs a session, and the kernel's loopback cards
(`modprobe snd-aloop`).

## What is ours, and what is the server's

- **The server's:** which devices exist, which are in use, moving a running stream
  when the chosen device changes. A version of this crate that relinked the graph
  itself would be a second policy quietly disagreeing with the one the machine
  ships.
- **Ours:** a pin, and what it means — *this device whenever it is here*, and
  **nothing at all when it is not**. Somebody who pinned their desk speakers and
  then took the laptop to a café wants whatever is there, not silence. A pin that
  forgets is worse than no pin, because a person stops checking, and then one day
  the call goes to the laptop speakers.
- **Ours:** volume and mute per device, kept by identity so that a headset muted
  today is muted when it is plugged in tomorrow (ADR 0038, `sound.toml`).

## Three things found on the way

1. **A virtual source made by the server's own loopback tool cannot be recorded
   from** (`no more input formats`). In `docs/quirks.md`. It is why these tests use
   the kernel's loopback cards rather than the server's own virtual ones — a test
   that must read a stream needs a device the rest of the machine treats as real.
2. **A card put in another profile comes back in its default one after a replug**,
   with different node names. Also in `docs/quirks.md`, and worth knowing beyond a
   test: it is the one case where a device's identity does not survive a replug.
   The identity is stable across a replug of the same card in its own profile,
   which is what a person's cable does.
3. **WirePlumber here is 0.4.17, not 0.5.** Its configuration is Lua, and the
   `wireplumber.conf.d/*.conf` rules written for 0.5 are read by nothing and
   reported as nothing — a file that is silently ignored. Worth knowing before
   anybody configures policy for the image: the version in Ubuntu 24.04 is not the
   version most of the documentation on the internet is about.

## For other lanes

- **`alo-in-use` clears the environment** before running the media server's tool,
  and does not pass `XDG_RUNTIME_DIR` through. On an ordinary machine that is
  harmless — the server falls back to `/run/user/<uid>`, which is where a session
  puts it. It is not harmless where a server listens anywhere else, which is every
  test session and every machine with more than one. `alo-sound` passes it through
  and says why in `server.rs`. Not changed here: `alo-in-use` belongs to the
  capture plan, and this plan reads it and does not edit it.
- **Devices task 4** (the camera and the microphone) wants both this crate's list
  and `alo-in-use`'s indicator. A microphone muted here is still a microphone an
  application holds open, and the indicator must keep saying so: *muted* and *not
  in use* are different sentences, and this crate deliberately says only the first.

## What is not done

- No sentence here has been through the walk in task 6, which is where the four of
  them meet the rest.
- Per-application volume is not in this crate and not in this task's acceptance.
  It is the server's, and where alo OS wants a view about it, it is a later task.
- Nothing about Bluetooth audio: task 3's, and a Bluetooth audio device joins this
  crate's list when it exists.
