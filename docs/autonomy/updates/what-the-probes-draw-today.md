# What the probes draw today

**Seventeen probes exist in `crates/alo-shell/examples/`. Thirteen of them drew
on this machine, under WSLg's Wayland parent, on 2026-10-08.** The other four
declined for one reason, and it is the right reason.

This is not new work. Every one of these was written by somebody and then not
run, because the lane that owns the compositor is on a Mac and **a Mac has no
Wayland parent**. The third PC has one, so *which of these draw* is a question
only this lane could answer.

## Why it was worth asking

`docs/autonomy/evidence-it-boots-and-the-agent-acts.md` says, twice:

> no pixel of any of that has ever been on a screen

> putting any of it on a screen is the compositor's, in `crates/alo-shell`,
> **which no machine has displayed**

Both sentences are true and both are narrower than they read. What has never
happened is **a real display driven by a modeset, and a person looking at it.**
What happens today, on demand, in about two minutes, is the drawing path running
nested and submitting frames.

Read as *nothing can be seen yet*, those sentences stop somebody running the
fixture that would show them the dock.

## What drew

| probe | what it put on a surface |
|---|---|
| `desktop_check` | the ordinary desktop: the dock, the dock with a question leaving, what is running, what is filling a folder, both windows, a folder that is gone — **33 frames** |
| `egress_status_check` | the indicator the first law rests on |
| `approval_check` | a question, an approval, one execution, and a refusal with no grant |
| `sign_in_check` | the sign-in screen, standing — **13 frames**, nothing proceeding on silence |
| `lock_screen` | ten opaque lock frames, private notifications held |
| `control_labels_check`, `control_label_pages_check`, `control_reader_chrome_check` | the window controls and what a reader is told about them |
| `window_controls_check`, `nested_check`, `readback_check`, `record_check`, `the_walk` | drew |

**Three of those are laws of this product, drawn rather than described.**

`egress_status_check` put real sentences on a surface:

> @mail is asking a question of alo, in the EU
> @mail is asking a question of workstation-2, on your network

and refused a frame whose indicator **had not been told what is leaving** — the
first law enforced by the drawing path rather than by a reviewer.

`approval_check` drew the whole of the second and third laws in four frames: a
verb asking, one approval causing exactly one execution, and then

> @files has not been granted …/invoices/april.pdf — **grants are made by
> picking a folder, never by asking for one**

## What did not, and why it is the right reason

`atomic_output_check`, `direct_output_check`, `scanout_check` and
`session_device_check` each exited with the same shape of message:

```
usage: direct_output_check /dev/dri/cardN
```

**They want a graphics card.** WSLg has no DRM node, so they declined rather
than failed. That is exactly the line between what this machine can show and
what it cannot, and the four that stopped are the four that would have had to.

## What this does and does not change

**It does not move *proved on a laptop*.** That column is 0 of 29 and stays
there. Nobody has sat at a certified machine and watched this, and
`desktop_check`'s own last line says so better than this document could:

> a virtual output proves the drawing path, not a panel: a certified machine
> has not seen this desktop.

**It does change what *screens owed* means.** Thirteen promises across phases 5
and 6 are recorded as *code written, screens owed*. For some of them a screen
exists and nobody has looked; for others nothing draws yet. Those are different
debts and the board does not tell them apart.

**And it is a software-rendered nested surface.** Mesa fell back — `failed to
get driver name for fd -1`, `ZINK: failed to choose pdev` — so what ran is the
layout and the drawing path, not the graphics path a certified machine would
take. `docs/booting.md` already says a virtual adapter does not stand in for
*the GPU works on first boot*, and this stands in for less than that.

## What is owed

A look. Thirteen probes draw and nobody has described what a person would make
of them — whether the dock reads as a dock, whether the egress line is findable,
whether the sign-in screen looks like one. That is not a test and cannot be
automated, and it is the cheapest thing on this list.

Then the four that want a card, on a machine that has one.
