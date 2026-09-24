# The desktop a session stands up

**Date:** 2026-09-24
**Workstream:** v0.01 — delivery, task 39
**Task:** 39, *After sign-in, the session stands the desktop up.*
**Not done.** Built and landed; what is owed is somebody seeing it on a display,
and a session that can hold one.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** Apple M3 with 8 GB; built, linted and tested in the Lima VM —
Ubuntu 24.04.4 aarch64. `alo-shell` does not build on macOS. Nothing is ticked
*on the machine*.
**Egress:** `git fetch` and `git push` against `github.com/aloworld-org/alo-os`.

## The inventory first

Taken before any code was written, because four tasks in two days turned out to
be largely built already. It is in the plan under task 39 as well as here.

- **Nothing outside `crates/alo-shell` constructs a `DesktopFrame`.** The only
  constructor anywhere was `examples/desktop_check.rs`, the display probe. The
  gap the task named was real and was the whole of it.
- **`frame_pictures` was already backend-independent** — it turns a
  `DesktopFrame` into every picture one frame needs with no backend in sight. It
  is reused, not rewritten.
- **The direct seam carried one scene and no layers.** Widening it was the first
  piece, and the same shape of change as task 38's.
- **Every native layer paints through `crate::painted::paint`** — solids and
  inked, nothing imported. So the software painter draws the whole desktop, and
  the renderer wall task 38 found blocks only a client application's window.

That last one is why this task was reachable at all this week.

## What is landed

`b3608a5` — **the seam carries every layer, not one scene.** A desktop is not a
scene: it is a dock and its windows above whatever clients are mapped, with the
record, Settings, a question and the egress indicator above that. The order is
`scene_drawing::paint`'s own, layer for layer, because two orders would be two
answers to what covers what — and one of the two would let a window cover what
is leaving this machine.

`3d4b390` — **the desktop on a real display.** The same pictures the nested path
draws, submitted through `Server::render_frame` exactly as an ordinary frame is,
so the output is published, membership is kept and every client drawn gets its
frame callback. A desktop that bypassed that would be a compositor whose clients
slowly stopped drawing, for a reason nobody would find.

`c875455` — **the process a session starts.** `stand_the_desktop_up` is the
order — bind the socket, build the keymap, load the font, take the display,
draw — and `alo-desktop` runs it. **Two programs rather than one with a mode**,
because the greeter and the desktop belong to different people: one is the
machine's, standing in front of somebody who has not signed in, and the other is
the person's.

## What is proved, and how

Without a display, and on a machine with no graphics card at all:

- **A dock reaches the bytes a card is handed.** Read back from the injected DRM
  device every other direct-target test uses — not one flat colour, which a
  frame with no dock on it would be.
- **What is leaving this machine changes those bytes.** The one surface this
  product may not ship without, checked at the pixels rather than at the call.
  An indicator laid out and then dropped on the way to the card passes every
  other test and fails this one.
- **The egress indicator is drawn above the screen underneath it**, and a frame
  carrying a layer but no scene is drawn rather than refused for having no
  scene.
- `alo-desktop` on the gate machine reads its configuration, binds its socket,
  builds its keymap, loads its font and this machine's vocabulary, makes every
  picture a desktop needs, reaches the login seat, and fails on the card by
  name. Every step but the one that needs hardware.

**What none of that proves** is that a person sees a desktop. Nobody has.

## Three things named rather than hidden

### The volume is a claim, and the type leaves no way to avoid making it

`StatusItems` can say *nothing said* about the network and [`None`] about the
battery. It has **no absent case for the volume**, so a desktop that has asked
nothing still shows one, and silence is the least wrong thing to show. It is
said in the code, in the line the service writes to the log, and here.

Giving that reading an absent case belongs with **task 15 of the shell plan**,
whose own acceptance is that *the absent cases are real rather than defaults* —
and this is the one reading that cannot meet it yet. Task 15's acceptance is
amended to carry it.

Why the clock is handed over and the other three are not: a clock that never
moves is a machine a person can see is dead; a battery reading that never moves
is a machine lying about how much time they have left.

### A client's window cannot be drawn, and this task does not claim one

The software painter imports nothing, so every frame carrying a mapped window is
refused by name. The desktop itself needs no importing, which is why it stands
up before that is answered. The question — import shared memory on the processor
too, or an ADR about renting a safe door to the graphics card — is written into
the plan under this task.

### A session with no seat cannot hold a display

**This is a finding in another lane's crate, and it is left as one.**

`alo-sessiond` passes an empty seat and a zero VT to `CreateSession`, on purpose.
Its own words: *a seat and a VT are facts about what draws, and this component
must not draw… a later task that starts a compositor is the one that knows which
VT it is on, and it will be changing an argument here rather than adding a door.*

That task is this one, and the argument is still theirs to change. A seatless
session cannot take a display through libseat, so `alo-desktop` cannot run **as
the person** until it does. It works on a gate machine today only because
libseat's builtin backend lets root take a card outside any logind session,
which is not how a person's session works.

So *started by the session* — this task's own acceptance — cannot be shown yet,
and it needs two things that are not mine: the seat and VT on `CreateSession`,
and the unit in `image/`, which is task 40.

## What is owed

- **A photograph of a screen**, on the development PC's VM with a real display
  device or on the laptop. The command is task 38's report's, with
  `alo-desktop` in place of `alo-compositor` and no `ALO_PERSON`:

  ```
  sudo env ALO_DISPLAY=/dev/dri/card0 ALO_KEYBOARD=gb \
           RUNTIME_DIRECTORY=/run/alo-desktop \
           ./target/release/alo-desktop
  ```

  `/run/alo-desktop` must exist and be mode 0700. **What the photograph must
  show:** the dock on the edge `alo-dock` ships it on, the status area at the
  far end of it with a clock that is this machine's, and the egress indicator
  drawn there when something is leaving. Say which of those were reached.
- **The four readings** — task 15 of the shell plan.
- **A session that can hold a display** — the seat and VT above, and task 40's
  unit.

## Gates

Nine in the Lima VM on the merged tree. `alo-shell`: 466 unit tests and its
integration suites green, `cargo clippy --all-targets -D warnings` clean,
`cargo fmt` clean.

The accepted failing set on main is **0**, where it has been since #103.

## Crates touched

`crates/alo-shell` only. No new dependencies.
