# Pairing a device, and what it is not

**Date:** 2026-09-17
**Workstream:** v0.5 — devices and media
**Task:** [task 3](../v0-5-devices-and-media-plan.md) — Bluetooth: pairing,
audio, keyboards and mice
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
built, linted and tested in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6
CPUs, 3 GB of memory**, BlueZ 5.72.
**Egress:** `apt-get install bluez` from Ubuntu's own archive into the VM, on
2026-09-17. Nothing else was fetched.
**Status:** done, as far as a machine with no Bluetooth radio can take it — which
is further than it sounds, and stated plainly below.

## What this is

`crates/alo-bluetooth`. BlueZ is rented and unmodified (ADR 0011). What is ours
is everything a person does: **nothing pairs that they did not choose and say yes
to**, what a device asks is shown in full, forgetting is one act that takes the
keys with it, the radio switch means what it says — and **pairing a device grants
nothing**.

| File | What it is for |
|---|---|
| `reported.rs` | address, name, kind, what was found — and *no radio* as a value |
| `choosing.rs` | a device a person picked out of what was found, which is the only road to a pairing |
| `confirming.rs` | the four things a device can ask, and the digits shown in full |
| `asking_the_person.rs` | where a person is asked, and every question put to one |
| `becoming.rs` | what a paired device is for, and who holds it afterwards |
| `granting.rs` | nothing, and a whole file about why that is worth a type |
| `service.rs` | asking the service, and the four things a person can do |
| `bluez.rs` | this machine's own service, reached (Linux) |
| `pairing_agent.rs` | the object the service asks, served in the person's session (Linux) |
| `bus.rs`, `words.rs`, `testing.rs` | the names on the bus, the ten sentences, a service to decide against |

Twenty-five tests.

## The four ways a machine ends up paired with something nobody meant

Each is closed by a type rather than by a rule in a comment, and each has a test.

1. **An address nobody was shown.** [`Chosen`] cannot be made from an address, a
   name, or a device that asked — only from a [`Found`] taken out of the list the
   service reported at that moment. The only road to a pairing starts at a list a
   person was looking at.
2. **The radio off.** With it off, nothing is chosen, nothing pairs, and this
   machine is not asked to look. Off means off.
3. **A person who said no** — in every one of the four shapes, including the one
   where the device asks for nothing at all. `Asked::needs_a_person` returns true
   for all four and exists to be read.
4. **A code the machine guessed.** Older devices very often take `0000` or
   `1234`, and every pairing tool on the internet tries them. This one asks, and
   a person who gives nothing has given nothing.

And the digits are shown as six digits, leading zeros kept: `012345` shown as
`12345` is a person typing the wrong thing and being told the device refused.

## A headset is not a machine

The plan's sharpest line, and `granting.rs` is a file with no logic in it about
exactly that. alo OS uses *pairing* for two things:

| | What it is | What it grants |
|---|---|---|
| `alo-nearby`'s | two **machines**, each person agreeing on their own (ADR 0003) | an agent on one may ask the other, under grants made where it acts |
| this crate's | a **headset** | nothing |

`tests/a_bluetooth_pairing_grants_nothing.rs` pairs a device of every kind the
whole way and then asks the **real** `alo-nearby` and `alo-capability` whether
anything moved: the grants are empty, the paired machines are empty, and this
machine has not come to believe it paired with a computer. Against the real
crates rather than stand-ins, because what is held is that this crate *cannot
reach* them, and a stand-in for something you cannot reach proves nothing.

`WhatAPairingGrants` has one variant, `Nothing`, and is `#[non_exhaustive]`: a
later change that wants a device to be able to do something has to add a variant,
which is a change somebody will notice in review.

## What an audio device becomes

`alo-sound`'s, and nothing else. `Becomes::of(Kind::Audio).is_held_elsewhere()`
is the crate saying so: a Bluetooth headset joins task 2's list, with the same
pin, volume and mute as the laptop's own speakers. **This crate keeps no second
list of audio devices**, because two lists would disagree, and the day they
disagreed a person's call would come out of the wrong one.

## What the machine could show, and what it could not

The VM has no Bluetooth radio — a Lima guest on an M3 has no radio passed to it —
so what ran on a machine is the half a radioless machine can take, and it is the
half that is invisible when it is wrong:

> `this machine has no Bluetooth radio, and says so: This machine has no
> Bluetooth. Nothing found is not the same as nothing to find`

Three answers are right, and they are three different sentences: nothing owns the
service's name, the service is there with no radio under it, or a list. **The
fourth — an empty list from a machine with no radio — is the one this crate must
never give**, because a person shown *no devices found* goes and puts their
headphones into pairing mode and waits.

**What has not been taken on a machine**, and is not claimed:

- a real pairing, with a real passkey, on hardware;
- that the radio switch disconnects a connected device — BlueZ powering off does
  that, and *that it does* is the rented service's behaviour rather than
  something this crate has watched happen;
- a Bluetooth headset appearing in `alo-sound`'s list, which is the join between
  tasks 2 and 3 and needs one machine with both.

All three want the certified laptop. Nothing here ticks *on the machine*.

## For other lanes, and for whoever configures the image

- **The pairing agent has to be registered by the session**, once, at sign-in:
  `pairing_agent::registered` serves the object, registers it and asks to be the
  default agent. A machine whose agent is not the default is a machine where
  something else is answering for somebody. The surface it asks through is the
  shell's to draw — the same shape `alo-networks`'s secret agent has, and for the
  same reason.
- **BlueZ must not be left discoverable.** Nothing here makes this machine
  discoverable, and nothing should: looking is deliberate and it ends.
- **v0.5 has no Bluetooth keyboard story beyond pairing one.** What a keyboard
  that types before anybody has signed in may do is the session plan's question,
  not this one's, and it is worth somebody asking it out loud.
