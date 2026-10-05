# More than one display

**What this is.** The compositor draws to **one** output, by design and in
writing, and three promises in `docs/features.md` need it to draw to more. This
plan is that work: discovery, globals, presentation, frames, layout, popups and
— last, and the only part anybody had written down — a camera per viewport.

**Why it exists as a plan rather than as a task.** Canvas task 9, *every screen
is a view onto the canvas*, was written as though a camera per viewport were the
whole of it. Picking it up on 2026-10-04 found that the substrate underneath is
single-output throughout:
[`the-canvas-and-its-places.md`](the-canvas-and-its-places.md) task 9's section
*What this is actually blocked on* carries the six citations. The owner decided
on 2026-10-05 that the remaining work is this lane's, queued behind
[`access-and-language-plan.md`](access-and-language-plan.md) and
[`models-a-person-adapts-and-subscribes-to-plan.md`](models-a-person-adapts-and-subscribes-to-plan.md).

**Crates this plan owns:** `alo-shell`. It reads `alo-displays` and
`alo-desktops` and re-decides nothing in either — see *What this plan does not
do*.

**The honest limit, at the top rather than discovered at the end.** Every
acceptance below ends in *two displays*. **The lane that owns this plan has one
laptop and no second display.** Every task here can be built and can have its
code half ticked on this machine; **not one of them can have `On the machine.`
ticked here.** That is not a reason to move the plan — the code is real work and
somebody has to do it — but a plan whose every acceptance needs hardware its
owner lacks should say so in its first paragraph, and the lane that eventually
shows it will be a different one.

## What exists, measured 2026-10-05 rather than assumed

| | |
|---|---|
| A model of more than one display | **`alo-displays` already has it**, and `ScreenPlace` gives each its own room |
| Per-display divisions | **`alo-dividing` already has them** |
| The camera's home | **settled 2026-10-04** — one home, on `Surfaces`, and its note names this work as the reason it had to be |
| Output discovery | `direct_output.rs:87` — `select` sorts the ports and returns **the first usable one** |
| The atomic route | `atomic_output.rs:16` — one `AtomicOutput`: one connector, one CRTC, one plane, three property maps |
| Presentation | `presentation.rs:264` — `output: Option<Output>`, one global, one metadata, one `entered` list |
| The display lifecycle | `display_lifecycle.rs:45` — `const THE_DISPLAY = DisplayId::from_compositor(1)` |
| Files saying one output in prose | **18**, under `crates/alo-shell/src/` |

**Nothing in that list is a defect.** A single-output compositor was the right
thing to build first, and `display_lifecycle.rs`'s own note says the constant
goes the moment a second output is advertised.

## What this plan does not do

- **It does not decide what a person sees on a second screen.** That is the
  canvas's, and canvas task 9 is its acceptance.
- **It does not re-model displays.** `alo-displays` and `alo-dividing` already
  do, and this plan reads them.
- **It does not add hotplug beyond the shape that exists.**
  `display_lifecycle.rs` already treats a changed extent as *leaving and
  returning at the new size*, which is the road a screen unplugged at one desk
  and plugged in at another already travels. A second display joins and leaves
  by that same road unless a task below says otherwise.
- **It does not advertise anything before it can serve it.** The output global
  is created after the first successful frame with valid metadata, per
  `presentation.rs`. That rule holds per display; nothing here advertises two
  outputs at birth.

## Tasks

### 1. Discovery answers with every display, not the first

**Status:** **Done, 2026-10-05: the code.** `direct_output.rs`'s `select_every`
answers with every usable display in the order a single-output shell already
preferred them, and `discover_output` keeps its signature as the first of that
list, so no existing caller moved. Five tests in `direct_output_tests.rs`.

**It was not *stop returning early*, and the part that was not is worth the
plan knowing.** A `Port` carries the **union** of the CRTCs its encoders allow,
and those unions overlap. Asking each port independently for its lowest
compatible CRTC — exactly what the single-output selection did, correctly, for
one port — **hands the same CRTC to two displays**, which is not a
configuration any kernel accepts. So a CRTC is taken as it is assigned and the
next display chooses from what is left.

**And the mode is now chosen before the CRTC**, which is the other way round
from the version this replaces. With one display the order could not matter: a
port missing either is skipped either way. With several it does — reserving a
CRTC for a port then skipped for having no supported mode takes that CRTC away
from a display that could have used it.

Both faults are invisible with one display and neither needed hardware to
find: the refusal tests already went through `Port`, which is a list, so *two
displays* is a fixture rather than a second machine. **This is the shape the
rest of the plan should expect** — the single-output code is not wrong, it is
correct for one, and each task's work is the part that stops being true at two.

**Owner:** the Mac. **Depends on:** nothing.

`select` sorts ports — internal panel first, then lowest connector ID — and
returns the first that has a usable mode and a compatible CRTC. The sort is
right and the `return` is what has to go.

- **Acceptance:** on a machine with two connected displays, discovery answers
  with **two**, each carrying its own connector, CRTC, mode and physical size,
  in the existing preference order. Every refusal path still refuses by name: no
  connected port, no usable mode, no compatible CRTC, a resource list exceeding
  the 32-bit encoder mask.
- **Constraint:** this adds nothing to what discovery may do. `discover_output`'s
  own note lists what it may not — no path opening, no master acquisition, no
  capability changes, no force-probe, no modesetting — and that list is
  unchanged.
- **Measured:** the refusal tests go through `Port`, which is already a list, so
  the fixtures for *two displays* exist without a second machine.

### 2. The atomic route is discovered per display

**Status:** ready. **Owner:** the Mac. **Depends on:** 1.

`AtomicOutput` is one connector, one CRTC, one plane and three property maps.
Two displays need two, and the per-display failure has to be per display.

- **Acceptance:** two displays produce two atomic routes with **distinct planes
  and CRTCs**. A display whose plane advertises no usable format is refused **by
  itself**, and the other display still comes up — a machine with one working
  output and one broken one is a machine a person can use.
- **Constraint:** the kernel `TEST_ONLY` rule stands per display. This schema
  check still cannot establish that hardware supports a proposed configuration,
  and nothing here starts claiming it can for two.

### 3. A session holds one presentation per display

**Status:** ready. **Owner:** the Mac. **Depends on:** 2.

- **Acceptance:** two `wl_output` globals, each with its own mode, scale and
  metadata. A client is told which of its surfaces entered which output.
  Retiring one display withdraws **its** global and leaves the other's standing,
  and the surfaces on the one that left are not reported as having entered the
  one that stayed.
- **Constraint:** a global is still created only after that display's first
  successful frame with valid metadata and a usable size. Advertising an output
  this compositor cannot yet serve is the fault `presentation.rs` already
  refuses for one.

### 4. A frame is drawn per display

**Status:** ready. **Owner:** the Mac. **Depends on:** 3.

- **Acceptance:** both displays paint at their own mode. A client drawn on a
  display gets its frame callback **from the display that drew it**. A display
  that fails to submit does not stop the other from painting, and says which one
  failed.
- **Constraint:** `render_frame`'s standing promise — every client that was
  drawn gets its callback, membership is kept, the output is published — holds
  **per display**. A desktop that bypassed it would be a compositor whose
  clients slowly stopped drawing, which is the fault `direct_desktop.rs`'s header
  already names for one.

### 5. The desktop is laid out for each display it is drawn on

**Status:** ready. **Owner:** the Mac. **Depends on:** 4.
**Likely to be two tasks** — recorded now rather than discovered.

The dock, the status area and the put-aside panel are laid out once, from one
size, and handed to one draw. Each display needs its own layout at its own size
**and its own scale**.

- **Acceptance:** a dock on each display, at that display's scale. A window
  moved from one display to the other is drawn at the right size on both. The
  fixed-control bounds recorded by `the_fixed_controls_were_drawn` are the ones
  for the display being drawn, so `canvas_never_lost`'s rule is asked about the
  right rectangles.
- **Constraint:** the standing rule, unchanged and load-bearing here more than
  anywhere: **no figure reaches a display without passing through that display's
  scale.** Two displays at different densities is the case that rule was written
  for, and this is the first code in the repository that can actually get it
  wrong.
- **The hazard this task carries:** `canvas_never_lost` currently asks its
  question against one set of control bounds. With two displays there are two
  sets, and a frame reachable on one display may be unreachable on the other.
  Which display's bounds a frame is held to is a question this task answers
  rather than inherits.

### 6. A popup is constrained to the screen it is on

**Status:** ready. **Owner:** the Mac. **Depends on:** 3.

`Surfaces::camera`'s own note says this is why the camera's home had to be
settled first: *which display's plane constrains a popup is a question the old
shape could not be asked.* Now it can be.

- **Acceptance:** a popup opened near the inner edge of the left display flips
  against **that display's** edge, not against the union of both and not against
  the other's. A popup on a display that is unplugged mid-grab is dismissed
  rather than left constrained to a screen that is gone.

### 7. A camera per viewport

**Status:** ready. **Owner:** the Mac. **Depends on:** 5 and 6.

**The only part canvas task 9's text describes**, and the last thing this plan
does. Its acceptance is that task's, inherited word for word so the two cannot
drift.

- **Acceptance:** two displays, each at its own zoom, each a view onto the same
  canvas — a frame moved on one appears moved on the other, and neither display
  is a second desktop with its own arrangement.
- **Constraint:** no second camera copy. The duplicate that existed was collapsed
  on 2026-10-04 precisely so this task would not be built over it; adding one
  back per display, kept in step by hand, is the shape that was removed.

### 8. The display number comes from whatever advertises it

**Status:** ready. **Owner:** the Mac. **Depends on:** 3.

`display_lifecycle.rs:45` hard-codes `DisplayId::from_compositor(1)` and its own
note says the constant goes the moment a second output is advertised. Task 3 is
that moment.

- **Acceptance:** two displays produce two `DisplayId`s, and `alo-dividing`'s
  per-display divisions follow the right one. The arrival and departure
  lifecycle runs per display, and a remembered division restored onto a display
  that came back is restored onto **that** display.
- **Constraint:** `alo-desktops` and `alo-dividing` are read, not re-decided.
  What changes is who supplies the number.
- **Why it is its own task:** the note that should have caught this could not —
  it is attached to a `const` with one reader, and nothing makes a second output
  appear to trigger it. See
  [`a-guard-that-cannot-fire-is-a-comment.md`](../misreadings/a-guard-that-cannot-fire-is-a-comment.md).
  **This task's own completion should leave behind something that fails** if a
  third display's assumptions are ever hard-coded the same way.

## What closes this plan

Canvas task 9's acceptance, shown on a machine with two outputs. Until then the
code halves close and the machine halves do not, and the plan says which is
which rather than letting the distinction blur.
