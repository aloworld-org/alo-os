# The compositor a machine boots to

**Date:** 2026-09-23
**Workstream:** v0.01 — delivery, task 38
**Task:** 38, *The compositor a machine boots to, and the one privilege it holds.*
**Not done.** Built and landed; what is owed is somebody seeing it on a display.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** Apple M3 with 8 GB; built, linted and tested in the Lima VM —
Ubuntu 24.04.4 aarch64. `alo-shell` does not build on macOS. Nothing is ticked
*on the machine*.
**Egress:** `git fetch` and `git push` against `github.com/aloworld-org/alo-os`.

## What this task turned out to be

Three things were found by reading, and each changed what the task is. They are
first because they matter more than the code.

### A compositor could not draw at all, and the reason is a law

`crates/alo-shell` had no way to make a renderer. Every road to a GLES one in
the pinned Smithay runs through `EGLDisplay::new` and `GlesRenderer::new`, both
`unsafe fn`, and the root `Cargo.toml` sets `unsafe_code = "forbid"` — a level
no crate lifts with an `allow` of its own. The nested backend never met this
wall because `winit::init` does that work behind a safe door and hands back a
renderer already made.

The door taken is the one this workspace already names for a thing whose only
spelling is `unsafe`: **rent it from somebody whose API is safe**, as
`signal-hook` is rented for `sigaction` and `getrandom` for the kernel's
randomness. Smithay's `PixmanRenderer::new` is safe, and pixman is configured
rather than written in (ADR 0011).

It costs nothing here. The sign-in screen is flat rectangles and inked text
(`crate::painted`) with no client beneath it — nobody is signed in to own a
window — and a frame has always reached the display as bytes uploaded into a
dumb buffer, which was never the graphics card's work either.

**What it cannot draw is a client's window.** That arrives as a buffer to
import, and importing is the half of a renderer this does not have. Every such
frame is refused by its own name — the window, the menu, the pointer a client
drew, the lock screen — never drawn empty. Task 39's note now carries the
question, with the two honest answers: import shared-memory buffers on the
processor too (`ImportMemWl` is implemented for the same renderer, and a client
offering only a dmabuf is then refused by name), or an ADR about how this
repository rents a safe door to the graphics card. **Not by relaxing the lint
quietly.**

### The image cannot carry the binary, and that is a task rather than a line

This is the one that reverses an instruction, so it is stated plainly.

`image/Containerfile`'s build stage compiles four packages for
`x86_64-unknown-linux-musl`, statically, and installs no development package for
Wayland, libinput, libseat, xkbcommon or pixman. A compositor cannot be built
that way at all: it links the machine's own display and input libraries, which
is the opposite of what a static musl binary is. It cannot simply be built
dynamically in the same stage either — the builder is Ubuntu 26.04 and the base
is Fedora bootc 42, so the binary would be linked against a glibc newer than the
one it must run on. And the final image installs none of those libraries, so
there would be nothing to link against.

So **I bumped the recipe to 0.0.6, declared `next = "0.0.6"` beside it, and then
reverted both** (`8a7347a`, reverted by `56f8e46`). An image built from 0.0.6
would have been 0.0.5 with a different label, and a declaration that a candidate
is being prepared would have been untrue. **I did not run the Image candidate
workflow**, for the same reason: it would have published a rebuild whose stated
purpose it did not serve.

The work is written up as **task 40** with the recipe: a build stage on the
base's own distribution, the development packages, the runtime libraries in the
final image, the unit with its three variables, and the `crates/alo-image` check
that whatever holds the privilege holds nothing else — the check task 38's
acceptance asked for and which needs a unit to be about.

### Two clauses of the acceptance belonged to somebody else

*`alo-agentd` comes up inside the session* is `alo-sessiond`'s and the session's.
This process asks for a session and is told *opened*; what systemd then starts
in it is another component's promise. The `alo-image` check moved to task 40
with the unit it is about. Both are recorded in the plan rather than dropped.

## What is landed

`e241f44` — the seam. `FrameTarget::submit_native_layers` in its general shape,
implemented for the sign-in scene, with the other five refusing by name.

`7ab4b7c` — the drawing and the lane. `software_scanout.rs` paints a native
scene on the processor and refuses everything that would have to be imported.
`direct_sign_in.rs` takes the display from the login seat, intercepts the seat's
keys and hands each to the screen, and draws it until a session opens. It is a
**second driver on the one display loop**, never a second loop: how a display is
polled, paused, retired and flushed has one answer, and two would be two ways
for a machine to end a session.

`4e231d6` — the order and the process. `booting.rs` is the composition a machine
performs at boot; `src/bin/alo-compositor.rs` is the process that runs it, and
it refuses to start without the three things a machine differs by.

`56f8e46` — the revert described above.

## What is proved, and how

**Without a display, and on a machine with no graphics card:**

- The bytes a card is handed carry the screen: read back from the injected DRM
  device every other direct-target test uses, they are not one flat colour and
  they differ with the name typed.
- **They do not differ with the password.** Two passwords of two different
  lengths are the same frame. That is the sign-in screen's own rule held where
  the pixels are, so nothing about a password reaches a screen read over a
  shoulder — or a photograph of one.
- Nothing is drawn once a session opens: the next frame would be a password
  field in front of somebody already signed in.
- Every layer the painter cannot import is refused by its own name, and the
  refusal names the layer rather than the backend's general capability.
- The binary, run on the gate machine: it reads its configuration, binds its
  socket, builds its keymap, loads its font and this machine's vocabulary,
  reaches the login seat, and fails on the card by name. Every step but the one
  that needs hardware.

**What that does not prove** is that a person sees any of it. Nobody has.

## What is owed, and exactly what would settle it

A photograph of a screen, on the development PC's VM with a real display device
or on the laptop.

**The command.** With the binary built on that machine
(`cargo build --release --bin alo-compositor`), on a VM whose display device is
virtio-gpu rather than headless, from a text console with no other compositor
running:

```
sudo env ALO_DISPLAY=/dev/dri/card0 \
         ALO_PERSON=<the uid /etc/alo/agentd.toml names> \
         ALO_KEYBOARD=gb \
         RUNTIME_DIRECTORY=/run/alo-compositor \
         ./target/release/alo-compositor
```

`/run/alo-compositor` must exist and be mode 0700 — the socket refuses a runtime
directory anybody else can read. `alo-sessiond` must be running for a password
to open anything; without it the screen still draws and says nobody answered the
door, which is itself worth photographing.

**What the photograph must show**, in the order they can be reached:

1. **The sign-in screen on the display**, drawn by the direct backend: a ground,
   two fields, the waiting one told apart by a thicker edge and a caret.
2. **A name typed appearing in the first field**, which proves the seat's keys
   reach the screen and the layout is the one named.
3. **A password typed showing one band and not a count of dots**, then Enter
   opening the session — or, with no `alo-sessiond`, the refusal in the
   machine's own words.

The third is the one that needs the opener running. **Say which of the three was
reached; do not tick the ones that were not.**

## Gates

Nine in the Lima VM. `alo-shell`: 461 unit tests and its integration suites
green, `cargo clippy --all-targets -D warnings` clean, `cargo fmt` clean.

The accepted failing set on main is unchanged at **10** — ADR 0039 §5 converter
tests needing the x86_64 engine on an aarch64 gate. None of them is mine, and
the count is reported rather than read as *none mine*.

## Crates touched

`crates/alo-shell` only, plus `Cargo.lock` for the two dependencies it gained:
`pixman` (pinned to the version Smithay's own `renderer_pixman` resolves to,
named directly because `Offscreen` must be asked for a `pixman::Image` by its
type) and `alo-saying`, which was already a dev-dependency and is now a real one
— the compositor has nobody to be handed the machine's sentences by, because it
runs before any session exists to carry words into it.

`image/` was touched and reverted, as described above; the branch leaves it
byte-for-byte as it was.
