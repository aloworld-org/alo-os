# Sharing the screen in a call

**Date:** 2026-09-17
**Workstream:** v0.5 — capture, and the room you are sitting in
**Task:** *Sharing the screen in a call*
(`docs/autonomy/v0-5-capture-and-the-room-plan.md`, task 5)
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
tested and linted in the Lima VM (Ubuntu 24.04 aarch64).
**Egress:** none.
**Status:** done.

## What changed, for somebody outside this repository

When a call application asks to see the screen, it gets **exactly what the
person picked at that moment** — the whole screen, one window, a part of one, or
nothing — and:

- **the picking cannot be answered in advance**, by any setting;
- **the indicator names what it can see**, not merely that something is shared;
- **no notification is drawn while a screen is shared**;
- **ending it is one act**, after which the call sees nothing.

## Why there is no "always allow this application"

A grant lets an application **ask**. What it may see is picked every time,
because the thing being shared is not a capability — it is a moment. The screen
at half past three, with whatever happens to be on it. **A person who agreed
once agreed to what was there then**, and they cannot have agreed to the mail
that arrives during the meeting.

So `Shared` has no constructor that takes a remembered answer, and
`tests/a_share_is_picked_every_time.rs` reads this crate's own source for eight
ways one could creep in — `always_allow`, `remembered`, `do_not_ask_again`,
`last_time` and the rest — in code rather than comments.

## A window is not the screen, and the sentences say which

Sharing one window is a **promise about everything else**: that the other
meeting in another window, the mail, the file with somebody's name in it, are
not travelling. That promise is why a person picks a window rather than the
screen, so it is in the sentence and a test holds it there:

| What is picked | What the person reads |
|---|---|
| nothing | *This call cannot see your screen* |
| one window | *This call can see one window, and nothing else on your screen* |
| a part of the screen | *This call can see one part of your screen, and nothing outside it* |
| the whole screen | *This call can see your whole screen, **including anything that appears on it*** |

The last one is the warning the other three do not need: a person sharing
everything pictured the document they meant to show, and what they are actually
sharing is whatever opens next.

## No notification arrives on a shared screen

A notification during a shared screen is **somebody else's message read aloud to
a meeting they are not in**. They did not agree, they are not in the room, and
they will never know it happened — the same shape as the microphone recording
whoever is speaking near the machine, which is this plan's other rule about
people who are not looking at the screen.

`Shared::notifications_may_be_shown` is false for the whole of a share, **whatever
was picked**. Sharing one window is not a defence: notifications are drawn by the
shell over whatever is there, so *only a window is shared* would not have saved
anybody.

## What is here and what is not

**Here:** what a call may see, what the indicator names, whether a notification
may be drawn, and how a share ends.

**Not here:** the network half of the call is the application's. The picker, the
indicator and the drawing are the shell plan's surfaces, acting on these values —
and `alo-portals` judges the grant that lets the application ask at all. This
crate decides what may be seen and makes the seeing visible; it opens nothing.

## Gates

The nine commands of `tools/kernel-loop`'s `EVERY_GATE`, verbatim, in the Lima VM
as root, with the test gate narrowed to the crates this lane touched.
