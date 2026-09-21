# The status area's four

**Date:** 2026-09-21
**Workstream:** v0.5 — the shell, task 7
**Task:** 7, *The status area's clock, battery, network and volume.* **Done.**
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** Apple M3 with 8 GB unified memory; built, linted and tested in the
Lima VM — Ubuntu 24.04.4 aarch64. `alo-shell` does not build on macOS, so every
compile here happened in the VM. Nothing is ticked *on the machine*.
**Egress:** `git fetch` and `git push` against `github.com/aloworld-org/alo-os`.

## What is here

Two files, and the wiring that makes them appear.

`status_items.rs` holds the four **as the crates that own them said them**:

| | Whose answer it is |
|---|---|
| the clock | `alo_formats::Regionally::time` wrote the text |
| the battery | `alo_power::Reading` — its own charge, its own charging state |
| the network | `alo_networks::Reaching` — `HowFar::reaches_anything` is the one place *connected* is decided |
| the volume | `alo_sound::Volume` |

`status_items_raster.rs` places them inside `dock_raster`'s status area, and
`desktop_paint` paints them onto the band.

**Nothing is measured in the shell.** The readings arrive on `DesktopFrame` —
the same arrangement `crate::lock_battery` is already under. A compositor that
opened `/sys` to read a battery would be a compositor measuring, and the status
area *shows*.

## The two things the task said the drawing must not undo

**A machine with no battery has none.** A desktop gets **three** cells rather
than four with one blank, and the three are wider for it. A battery drawn at
zero says the machine is about to die; a desktop is not dying.

**A machine told nothing about metering is holding off.** `should_hold_off`
counts *nothing said* as metered, and the network mark asks that rather than
reading the variant — so *nothing said* and *metered* are drawn identically, and
neither is drawn as a free connection. A status area that showed *not metered*
where the machine does not know would be adding a claim about somebody's data
allowance.

Both are tests, not sentences.

## Four things I got wrong, and how each was caught

**The battery outline was a filled block.** I drew ink, then the fill — so a
flat battery and a full one rendered as the same solid rectangle. My own test
did not catch it because it **counted the shapes drawn**, which is a stand-in
for the picture rather than the picture. It now counts ink, subtracting ground
painted over ink, and asserts a flat battery and a full one differ. The outline
is hollowed the way `lock_battery` hollows its own.

**`HowFar::reported(70)` does not mean "reaching".** Anything outside 1–4 is
`NotSaid` — *the service said something we do not understand*. My fixture meant
*connected* and was testing the opposite. The fix was to stop using a magic
number and name the variant. The one test that could not be wrong about this is
the one that compares against the crate's own answer for every variant, which
is why it is written that way.

**`Volume::of(3500)` is louder than loud.** It is hundredths out of a hundred,
not out of ten thousand, and `LOUDEST` is 100. The test now walks `0`, `1`, `35`
and `Volume::LOUDEST` rather than numbers I assumed.

**Charging and metered were computed and never drawn.** Clippy found them:
`is_charging` and `should_hold_off` were *never used*. I had written the
accessors, documented them, and drawn neither. Both are now marks — a bar across
the battery for power going in, a notch out of the network mark for holding off
— and both have tests, because a thing drawn without a test is a thing I cannot
honestly say is drawn.

## One promise I nearly left behind

`desktop_raster_tests::every_colour` exists to hold *not one pixel of the
desktop is terracotta* — terracotta means the agent, and the desktop is not the
agent. It walks the dock and both windows. **It did not walk the status area**,
so the four new items could have painted the agent's colour and that test would
have passed.

They are in the walk now. The guarantee was not mine to invent and was very easy
not to extend.

## What is still owed here

**Brightness.** `docs/features.md` lists it in the status area and the task's own
constraint holds it back: *brightness waits for the display work that owns it*.
Nothing here reserves a cell for it or pretends it is coming.

**The readings' source on a running machine.** `DesktopFrame` now carries them
and `examples/desktop_check.rs` hands over **fixed** ones, labelled as fixed in
the source: that probe asks whether the compositor draws a frame on a real
display, not what this machine's battery is. Whoever stands the shell up on a
certified machine supplies the real four from the four crates. That is not this
task's — this task is the drawing — and it is named here rather than left for
somebody to discover.

## Gates

Nine in the Lima VM.

## Crates touched

`crates/alo-shell` only. It gained `alo-power`, `alo-sound` and `alo-networks`
as dependencies, for their value types alone — no call in the shell measures
anything.
