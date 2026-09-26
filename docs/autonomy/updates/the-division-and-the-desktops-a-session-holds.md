# The division and the desktops a session holds, and the one layout decider

**Date:** 2026-09-26
**Workstream:** v0.5 — the shell, task 16
**Task:** 16, *The division and the desktops a session holds, and the one layout
decider.*
**Done for what a session holds, what a chord does and what a swipe does.** Two
things are named rather than ticked, below.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** Apple M3 with 8 GB; built, linted and tested in the Lima VM —
Ubuntu 24.04.4 aarch64. `alo-shell` does not build on macOS. Nothing is ticked
*on the machine*.
**Egress:** `git fetch` and `git push` against `github.com/aloworld-org/alo-os`.

## The inventory first

Taken before any code was written, and written into the plan as commit
`38c1dbc`.

- **Both deciding crates already held the whole lifecycle.**
  `alo_desktops::Desktops` has `plug_in`, `unplug` and `on`;
  `alo_dividing::Divisions` has `remember`, `on`, `forget` and `restored`. So
  this task wrote no state machine and no restore — it wrote somewhere for a
  session to keep them and the roads by which things reach them.
- **The `Server` held neither**: an overlay, a press, a presentation, its
  surfaces, its socket and a switch order, exactly as task 10 found on
  2026-09-21.
- **The removal had to come second**, because `set_window_tiled(side)` can only
  become *the share the division gives for that side* once a division is
  `Server` state.

## What a session holds

`crate::server_desk::Desk` holds a division per display and the desktops on it,
and routes what happens to them. A display that arrives gets its desktops and
the division it left; one that goes has its division remembered under the
screen's own key and its desktops handed back, with nothing closed and nothing
moved. A division comes back **under the window numbers that are open now** —
`alo_dividing::Remembered::restored`'s own behaviour — so an application that is
not running is left out and its share collapses onto what is left.

Window numbers are handed out once and never again, from
`crate::window_number`. A reused number is a tree holding a share for a window
that is gone and a different window walking into it, which is the one failure
mode of this design; it is also why these are not the fresh-per-mapping
visibility identity the compositor already kept, since a window that flickers is
still the window in somebody's layout.

## The one layout decider

`crate::window_tiling` computed half an output and set a window mode from it.
The half is gone in full: the module, its tests, `Mode::Tiled`, `Anchor::Tile`,
`TileSide`, `TileGeometry`, `TileGeometryError`, `WindowCommandError::Tile`, two
display-probe stages and 807 lines of integration tests for a mechanism that no
longer exists. Nothing is deprecated and nothing is aliased.

A chord now asks `alo_dividing::Division::divide_with_next`, and every rectangle
comes back from the division's `shares()`. Each window's minimum size is read
from its own XDG state and handed over, so the crate can refuse a split that
would squeeze either window — handing over *any size* would have made that
refusal unreachable, which is a protection lost rather than a detail skipped.

`tests/one_layout_decider.rs` holds it there, **read from the crate rather than
from a list kept beside it**: how many `Mode` variants carry a rectangle, how
many files construct one, whether that one file asks `alo-dividing` or does
arithmetic of its own, and whether the half is still sitting in `src/` unused.
It was proved by putting a second builder in another module and watching it
fail, then taking it out again.

**What this design costs, stated plainly.** A chord with one window open now
refuses. A division divides *between* windows, so a single window has nothing to
share with and `alo-dividing` says so by name. The withdrawn API would have put
the only window on half a display with nothing beside it.

## Three things the removal exposed

**Nothing ever told the session it had a display.** A chord asking to divide was
answered *this session has no display to divide* on a machine that was drawing a
frame at the time. A test found it. `crate::display_lifecycle` hangs the arrival
off the submitted frame and the departure off output retirement, which are the
two moments this compositor's single output already has.

**A popup grab refused too late.** The chord used to refuse inside
`set_window_tiled`; the new road divided first and applied afterwards, so a
window that could not take its share would have left the tree divided and the
screen not — a layout right in the tree and wrong in front of the person. Both
windows are now asked **before** anything is divided, through a
`ready_for_a_mode` extracted from `plan_window_mode` so the two cannot drift.

**Membership followed what was drawn.** The once-a-frame call passed the drawn
windows, which would have meant that switching desktop told every division the
windows on the desktop being left had closed — and already meant a minimised
window quietly lost its share. It follows what is buffered now.

## What a swipe does

`libinput_routing` said *unsupported touch/tablet/gesture/switch events are
ignored*, so a three-finger swipe did nothing at all and the recogniser
`alo-desktops` ships was never handed an event. Gestures reach it first now, and
what it answers as a desktop switch is carried out on every display. Which
desktop a swipe reaches, whether there is one and what the end of the row does
are its answers: the test asserts the desktop shown afterwards is the one the
crate named, not that it is the second one.

A pinch and a scroll are **refused rather than swallowed**. Nothing in this
compositor zooms an application, so a zoom is handed back and named as a gap; a
scroll already reaches the focused client through `libinput_scroll`, and doing
it here as well would scroll twice for one movement of somebody's fingers. Both
still reach the recogniser, because each cancels a swipe it had begun.

Windows belong to desktops now: one that opens joins the desktop being looked
at, one that closes leaves every desktop, and a switch hides the rest where they
stand. `Surfaces` gained a third reason a window is not drawn, deliberately
separate from minimised — somebody who minimised a window expects to find it
minimised, and somebody who switched desktop expects everything exactly as they
left it.

## Named rather than ticked

**A real display plugged in and unplugged.** The `Desk` remembers and restores,
and its unit tests hold every branch of it: the same screen returning finds what
it left, a different screen does not inherit it, a screen whose applications
have all closed comes back undivided, and what comes back carries today's window
numbers. **None of that has been done with a real display**, because this lane
has no machine with a display to plug. The task's own constraint says so. It
needs a certified machine and a monitor.

**The agent overlay is not drawn.** `alo_desktops::Always` names three surfaces
that are on every desktop: the egress indicator, the approval surface and *the
part of the screen a person brings the agent up in*. The first two are drawn on
two desktops in pixels, with something leaving the machine and a question
waiting, and both are in both frames. The third has no frame in this compositor
at all — `DesktopFrame` carries no agent overlay — so what is held for it is
`alo-desktops`' half. The frame it needs is a task, not a tick.

## What was found, for other lanes

Three, written into `docs/quirks.md` where they belong and repeated here.

- **No backend metadata this compositor reads carries a screen's serial
  number.** A division is remembered by make, model and connector together,
  which handles a different monitor on the same port and the same monitor
  returning to it. Two *identical* monitors swapped between two ports find each
  other's arrangement. Ours; EDID serials would fix it and are not in this plan.
- **Neither `alo-desktops` nor `alo-dividing` takes a new area for a display it
  already holds.** There is no resize on either. `alo-shell` treats a changed
  extent as the display leaving and returning at the new size, which uses both
  crates' own roads and keeps the arrangement; what it costs is that a resize
  rebuilds the tree rather than adjusting it. Not this lane's crates.
- **`alo_desktops::Desktop` hands back no identity of its own.** A caller
  holding a `&Desktop` from `in_order()` cannot ask `windows_on` about it,
  because that takes a `DesktopId` and nothing on `Desktop` gives one. The test
  that walks every desktop switches through them instead. Small, and not this
  lane's crate.

## Contracts

`docs/contracts/native-window-tiling.md` is **withdrawn**, and says at the top
what went, when, and what replaced it — a contract that quietly stopped
describing the code would be worse than one that says when it stopped.
`docs/contracts/native-window-dividing.md` is new and describes what a session
holds, what a chord does and every refusal it can give.
`native-window-commands.md` and `native-window-maximize.md` follow it.

## Evidence

All of it in the Lima VM — Ubuntu 24.04.4 aarch64. Nothing was gated on macOS.

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: clean.
- `cargo test -p alo-shell --locked`: 512 unit tests and 17 test binaries, all
  green, including the five new gesture tests, the three new desktop-membership
  tests and the four in `one_layout_decider`.
- The nine gates on the exact tree merged, posted as `alo/nine-gates`.
