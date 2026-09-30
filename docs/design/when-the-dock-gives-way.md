# When the dock gives way

**Status:** a design note, not a decision. It names what has to be settled before
the last piece of *the dock's size, and whether it hides when a window needs the
room* can be written, and recommends an answer to each.
**Date:** 2026-09-30

## What is already built, measured rather than recalled

`crates/alo-dock/src/hiding.rs` — 206 lines, six tests — holds all of it:

- `Hiding`, the person's two choices, `Never` by default.
- `TheRoom`, the input: `Free` or `AWindowNeedsIt`.
- `Showing`, what the dock then does.
- `Hiding::showing(TheRoom) -> Showing`, the decision, exhaustive over both
  choices against both states, with only one of the four answers `Hidden`.

`crates/alo-shell` offers the choice: two rows in the settings window, the
sentence a person reads, and tests that changing it changes the dock.

**What is missing is one caller.** `TheRoom::` appears nowhere outside
`alo-dock`, and inside it only in tests and in the match arm. A person can choose
the behaviour, the dock knows what to do when told, and **nothing ever tells it.**

## But the caller cannot be written yet, and that is the point of this note

**Nothing in this repository says which windows count.** Measured 2026-09-30:
`docs/features.md:74` is the single line *The dock's size, and whether it hides
when a window needs the room*; `docs/design/the-alo-dock.md` does not mention it;
`TheRoom`'s own documentation defines the *type* and not the *predicate*. Every
other occurrence is the phrase quoted back.

Four readings are available and they are visibly different to a person:

| reading | what a person sees |
|---|---|
| any mapped window overlapping the dock's band | the dock goes when anything is under it |
| only a window filling the screen | the dock stays until something takes the whole display |
| only the **focused** window overlapping | the dock returns when you click away from the window under it |
| only a **maximised** window | the dock ignores a window you dragged over it |

Writing the caller means choosing one. That is an interface decision, not an
implementation detail, and it is not recorded anywhere.

## The hazard that makes this worth a note rather than a guess

**The obvious implementation oscillates.**

If the dock's presence shrinks the work area, then: the dock hides → the work
area grows → the window re-lays out → it no longer overlaps the band → the dock
returns → the work area shrinks → the window re-lays out → it overlaps again.
A dock that flickers at a few hertz, from two rules that are each correct.

This is the classic auto-hide feedback loop and it is the reason the reading
cannot be picked casually: three of the four readings above are exposed to it,
because all three are computed from a geometry the dock's own visibility
changes.

It is also, exactly, an instance of the family
[0080](../decisions/0080-a-signal-that-cannot-be-wrong-tells-you-nothing.md)
names. A predicate computed from a geometry its own answer moves cannot
distinguish *a window needs the room* from *a window needs the room because the
dock hid* — the two inputs are the same reading. A check that cannot tell its
two cases apart has one case, and here the one case is the flicker.

## What I recommend, and why

**1. The work area does not depend on the dock's setting.** When `Hiding` is
`WhenAWindowNeedsTheRoom`, windows lay out over the full screen and the dock
draws over them. The work area is then a constant, the loop above has no cycle
in it, and the dock's visibility is a pure function of where windows already are.

That is the decision the other three rest on, and it is the one worth arguing
with first.

**2. Any mapped window on that screen whose area overlaps the band** — not only
the focused one. A person who set the dock to give way and is watching something
play behind it did not ask for the dock back when they clicked elsewhere; and
*focused* would make the dock appear and disappear on focus changes that have
nothing to do with the room.

**3. Not `FillingTheScreen`**, which is already its own state in `alo_dock::HowItSits`
and bypasses the dock entirely — a window filling a display is not being shown by
the canvas at all. Reading fullscreen as the trigger would leave the ordinary
case, a window dragged over the band, doing nothing.

**4. The answer is per screen.** A window on the laptop says nothing about the
dock on the external display. `screens_raster::picture` already draws each
screen's dock from that screen's own place, so the shape is there.

**5. No timers.** `alo-dock`'s `revealing.rs` is the precedent — the edge state
machine has none, deliberately, so the Dock does not vanish under an approaching
pointer. Whatever hysteresis this needs, if any, should be a state and not a
delay, and should be measured on a real display before it is invented.

## What this note does not settle

- **Whether a window counts while it is being dragged.** Probably yes, and it is
  cheap, but it is a separate feel question and nobody has watched it.
- **Whether hysteresis is needed at all** once the work area is constant. On the
  argument above it should not be, and that is a claim to test rather than a
  thing to build in advance.

## What would have to be true before it is called done

**A real display, and a window a person drags over the band.** Nothing in this
repository has ever had one — every screen here is arithmetic and a test — so
this can earn a code box and not an *On the machine* one. The same shape as the
disk that defers the installer's alongside walk: a hardware condition, not an
effort one.
