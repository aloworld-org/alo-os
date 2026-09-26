# Every new surface, walked

**Date:** 2026-09-26
**Workstream:** v0.5 — the shell, task 14
**Task:** 14, *Every new surface, walked.*
**Done for the walk and the sequence.** Four things are named rather than
ticked, below.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** Apple M3 with 8 GB; built, linted and tested in the Lima VM —
Ubuntu 24.04.4 aarch64. `alo-shell` does not build on macOS. Nothing is ticked
*on the machine*.
**Egress:** `git fetch` and `git push` against `github.com/aloworld-org/alo-os`.

## What a raster here is, and what it is not

**Every raster below is what this compositor drew. None of them is what a
parent, a compositor above it, or any panel displayed.** The frame is painted
twice by the same painter — same `GlesRenderer`, same
`crate::scene_drawing::paint`, same roots, popups, cursor and layers — once into
the parent's window, which is the submission, and once into an offscreen buffer
that can be read, because reading the window's own buffer back loses the EGL
context on this backend (`docs/quirks.md`).

The parent is `weston --backend=headless` and the renderer is Mesa's llvmpipe. A
headless parent **composites to nothing**: there is no panel, no scanout and no
photograph. Eight green steps are evidence that each surface laid itself out and
put pixels down. They are not evidence that anything showed them, and **no owed
screenshot of a real display is discharged by this report.**

## The walk, and what each step drew

Run with `cargo run -p alo-shell --example the_walk` under a Wayland parent.
Each step submits a frame through the parent's own EGL and reads back the pixels
that frame was painted into.

```
probe_should_be_7=7

the sign-in screen, before any account is chosen   1366x768, 1049088 pixels painted
a name typed at the sign-in screen                1366x768, 1049088 pixels painted
the screen divided between two windows            1366x768,     512 pixels painted
a second display docked, and the desktop after it 1366x768,  127038 pixels painted
a capture of the screen, with a blur marked on it 1366x768,  241662 pixels painted
a notification arrives on the desktop             1366x768,  173118 pixels painted
the screen is locked                              1366x768, 1049088 pixels painted
a key is pressed at the locked screen             1366x768, 1049088 pixels painted
```

The counts are the evidence, not the fact that a frame came back. The sign-in
and lock screens are the whole output and opaque, so all 1,049,088 of their
pixels are painted — those two are drawn **instead of** everything else rather
than above it, so nothing underneath can have drawn them. The desktop is 127,038;
the capture tools add about 114,000 to it and a notification about 46,000. A
surface that refused, or laid itself out off the output, comes back at the
desktop's own count or at zero, and the fixture fails on zero by name.

## The walk, sentence by sentence

Held by `crates/alo-shell/tests/every_new_surface_walked.rs`, which parses this
table and compares it against the sequence the crates produce. A sentence that
changes without this table changing fails there. Every one is said through
`alo_strings::Strings` from a declared `Word`; none is written in `alo-shell`.

| # | Moment | What a person meets |
| --- | --- | --- |
| 1 | The sign-in screen, before any account is chosen | who is signing in |
| 2 | The sign-in screen, before any account is chosen | password |
| 3 | The sign-in screen, before any account is chosen | sign in |
| 4 | The sign-in screen, before any account is chosen | settings for seeing, hearing and typing |
| 5 | A second display is docked | Built-in screen |
| 6 | A second display is docked | Your screens are arranged the way you last left them |
| 7 | The screen is divided, and the desktops a swipe reaches | Next desktop |
| 8 | The screen is divided, and the desktops a swipe reaches | Previous desktop |
| 9 | On every desktop, whatever else is | What is leaving this machine |
| 10 | On every desktop, whatever else is | What is waiting for your approval |
| 11 | On every desktop, whatever else is | The agent |
| 12 | The desktop, read as a reader reaches it | the desktop |
| 13 | The desktop, read as a reader reaches it | the windows that are open |
| 14 | The desktop, read as a reader reaches it | ask the agent |
| 15 | The status area, where what is leaving is shown | what this machine is doing |
| 16 | The status area, where what is leaving is shown | something is leaving this machine |
| 17 | The status area, where what is leaving is shown | the agent is working |
| 18 | The controls on a window | close this window |
| 19 | The controls on a window | move this window |

## Named rather than ticked

**A second display has no surface of its own.** `screens_raster::desk` produces
`ScreenPicture`s and **nothing in this compositor paints one** — there is no
`NativeLayers` slot for them and no nested submit. So the step docks a second
display through the session's own road (`Server::display_arrived`, which gives it
desktops and restores the division it left) and draws the **first** display's
frame afterwards. What a person would see of their two screens arranged is not
drawn anywhere yet.

**The divided windows keep their own size.** Both clients hold the 16×16 buffer
the shared client fixture makes, so what the raster shows is each window drawn at
**the origin its share gave it**, not a client grown into its share. Growing into
it is task 17 of the shell plan, written for exactly this gap.

**Three of the surfaces this walk draws are not in the accessibility tree.**
`alo_access::tree::Surface` knows nine and none of them is the lock screen, the
capture tools or a notification. A surface a reader cannot reach is a surface
somebody cannot use, and the lock screen is the one a person meets before they
can do anything else.

**The certified machine has seen none of it**, which is this task's own
constraint. See the first section for why a green run does not change that.

## What was found

**Two windows from two clients were one window.** The walk refused to divide the
screen with two windows mapped: `alo-dividing` answered *nothing to share with*.
`crate::window_number` keyed its map by `surface.id().protocol_id()`, which is
the number a surface's **own client** knows it by and which restarts at 1 for
every client — so two applications with one window each are both `wl_surface@3`
and were given one number between them. A division asked to divide a window from
itself refuses, correctly. **It would have happened to the second application
anybody opened**, the first time they pressed the chord. It passed nine gates
twice because every test before this one drives a single client.

The number now lives in the surface's own data map, where
`crate::window_placement` already keeps a placement. Keying by the whole
`ObjectId` also fixes the collision and is a `clippy::mutable_key_type` — that
type has interior mutability, and a key whose hash the compiler will not vouch
for is not one to build somebody's window layout on.

**A public field no caller could fill.** `DesktopFrame::capturing` is public and
its type was not, so every caller outside `alo-shell` could only ever pass
`None` — including the fixture whose whole purpose is to draw a frame with the
capture tools on it. `Capturing` and `CaptureLook` are exported.

**A client's deadline shorter than the server's work.** The walk's clients first
waited ten seconds to be released, and software-rendering a 1366×768 frame per
pump is slower than that; they disconnected, took their windows with them, and
the division then refused for a reason that had nothing to do with dividing. A
test failing about its own timing is worse than one failing about the thing.

## Evidence

All in the Lima VM — Ubuntu 24.04.4 aarch64. Nothing gated on macOS.

- `cargo test -p alo-shell --locked`: 515 unit tests and 17 test binaries green.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: clean.
- `cargo run -p alo-shell --example the_walk` under `weston --backend=headless`:
  the eight steps above.
- The nine gates on the exact tree merged, posted as `alo/nine-gates`.
