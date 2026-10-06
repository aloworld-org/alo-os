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

**Status:** **Done, 2026-10-05: the code.** `discover_every` builds a route per
display and `discover_atomic_output` is the first of them, so no caller moved.
Four tests.

**Task 1's fault again, one level down, and it was not foreseen here either.** A
plane's `possible_crtcs` is a bitmask, so the **same plane is compatible with
several CRTCs** — and asking each display independently for its lowest
compatible primary plane hands one plane to two displays. A plane scans out for
one CRTC at a time. Planes are now taken as they are assigned, exactly as CRTCs
are.

**And a refusal is now per display where the thing refusing is the display's.**
A connector, a CRTC, a plane list or a property schema that will not read takes
that display out and the run carries on; the first such refusal is kept and
returned only if no display survives. A capability the kernel would not enable,
or a resource list that would not read, still refuses everything — none of that
is about one display. **One working output and one broken one is a machine a
person can use**, and the old code could only answer *this device has no
route*.

**Owner:** the Mac. **Depends on:** 1.

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

**Status:** **Done, 2026-10-05: the code.** `Server::presentations` is a map
keyed by `OutputMetadata::name`, which is session-unique and so is what tells
two displays apart. Six tests.

**A second display was refused, not merely undrawn, and that is the finding.**
One `Presentation` holds the identity of the display it has seen, and
`validate_target` answers `OutputIdentityChanged` for any other — so handing
the server a second display's frame was **turned away by a check written for a
different purpose**. That check keeps its real job: a connector reporting a
different monitor under the same name is still a fault, and there is a test for
it.

**And retirement split in two.** `target.retire()` is the **backend** going
away; a `wl_output` global is a **display's**. They were one call, which is
correct for one display and wrong for two — retiring per presentation would ask
the same backend twice and be refused the second time. The backend is now asked
once and every display withdraws its own global.

A target naming no display this session presented is refused. With one display
that was *the identity changed*; with several it is *that is not one of mine*,
and both are the same mistake.

**What this task does not do, so task 4 is not read as smaller than it is.**
Nothing here draws a second frame. `popups.output_size`, the camera handed to
`look_at`, and `update_window_mode_output` are still one display's and are
overwritten by whichever frame ran last — tasks 4, 5 and 7. What is per display
now is the protocol state: the global, the mode, the metadata and the entered
set.

**Owner:** the Mac. **Depends on:** 2.

- **Acceptance:** two `wl_output` globals, each with its own mode, scale and
  metadata. A client is told which of its surfaces entered which output.
  Retiring one display withdraws **its** global and leaves the other's standing,
  and the surfaces on the one that left are not reported as having entered the
  one that stayed.
- **Constraint:** a global is still created only after that display's first
  successful frame with valid metadata and a usable size. Advertising an output
  this compositor cannot yet serve is the fault `presentation.rs` already
  refuses for one.

### 3a. The session holds the arrangement of its screens

**Status:** **Done, 2026-10-06: the code.** The compositor describes every
display it has presented in `alo-displays`' own words
(`Server::the_displays_as_reported`), `TheDesktop::the_screens_of` turns those
into an arrangement, and `Server::the_screens` holds it. Re-asked whenever the
descriptions change — not per frame — by the same comparison the layout save
uses. Four tests.

**A display's identity is its socket, and that was not the obvious choice.**
`Panel::of` wants a make, a model and a **serial**, and `OutputMetadata`
carries no serial. A panel built from make and model alone is not an identity:
`Identity` prefers the panel when one is given, so **two identical monitors
became one screen** — measured, not predicted, by a test that expected to see
both named and saw one identity twice.

So no panel is offered and the connector identifies the screen. **What that
costs is real and is the lesser loss:** a monitor moved from one port to
another is a different screen to the arrangement, so what a person arranged
for it is not found again. A forgotten arrangement is a person dragging a
window back; two live displays collapsed into one is an arrangement that
cannot describe the desk at all. The fix is upstream — a serial reaches this
crate only if the compositor reads EDID, which it does not — and inventing one
would be worse, making the two monitors *stably* one screen rather than
visibly one.

**The arrangement has no reader yet**, and that is deliberate rather than
overlooked: tasks 5, 6 and 7 are its readers. It is kept live from the first
frame so those tasks find it already true instead of each wiring it again,
which is the whole reason this task was split out.

**Owner:** the Mac. **Depends on:** 3.
**Added 2026-10-05, while starting task 6**, which could not be done without
it — and nor can 5 or 7.

**What was found.** `crates/alo-shell/src/screens.rs` already has the type the
rest of this plan needs. `ScreenPlace` carries `at: Position` — *its corner on
the desk the arrangement makes* — alongside its pixels, its room and its scale.
**Nothing in production builds a `Screens`.** The only caller of `Screens::of`
is `screens_testing.rs`.

```
$ grep -rn "Screens::of" crates/alo-shell/src/ crates/alo-desktop/src/
crates/alo-shell/src/screens_testing.rs:59
```

So the compositor knows how big each display is — task 3 gave it that — and
**does not know where any of them is.** Two displays are two sizes and no
geometry.

**Why three tasks need it and the plan did not say so.**

- **Task 5** lays the desktop out per display. Which display a window is on is
  a question about position, not size.
- **Task 6** constrains a popup to the screen it is on. *The left display's
  inner edge* is a position.
- **Task 7** gives each viewport a camera. A viewport is a rectangle on the
  plane, and a display with no corner has no rectangle.

Each would otherwise discover it separately, which is three readings of the
same gap — the thing this plan's task 1 and 2 notes were written to stop.

- **Acceptance:** the session holds a `Screens` built from the displays it has
  actually presented, and asking which screen a point is on answers from the
  arrangement rather than from the one viewport. A display that arrives or
  leaves changes it.
- **Constraint:** `alo-displays` decides the arrangement and this reads it.
  `Screens::of` wants `Attached::now(reported, remembered, support)` plus an
  `Appearance` and a `Tonight` — the first comes from the per-display metadata
  task 3 now keeps, and the other three are the desktop's to supply, by the
  same road the canvas layout and the shortcuts already travel
  (`TheDesktop`). **The shell shows and never measures**, so it does not read
  the kept display settings itself.
- **What it does not do:** decide anything about arrangement. Where a person
  puts their screens is `alo-displays`', and this task is the wiring that was
  never done.

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

**Depends on:** 4, and **3a** — laying a desktop out per display needs to know where each display is, not only how big it is.
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

**Depends on:** 3, and **3a** — *the left display's inner edge* is a position, and the session has none until 3a.

`Surfaces::camera`'s own note says this is why the camera's home had to be
settled first: *which display's plane constrains a popup is a question the old
shape could not be asked.* Now it can be.

- **Acceptance:** a popup opened near the inner edge of the left display flips
  against **that display's** edge, not against the union of both and not against
  the other's. A popup on a display that is unplugged mid-grab is dismissed
  rather than left constrained to a screen that is gone.

### 7. A camera per viewport

**Status:** ready. **Owner:** the Mac. **Depends on:** 5 and 6.

**Depends on:** 5 and 6, and so **3a** through both — a viewport is a rectangle on the plane, and a display with no corner has no rectangle.

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

**Status:** **Done, 2026-10-05: the code.** `Server::the_number_for` assigns a
`DisplayId` per display name on first sight and keeps it in
`Server::display_numbers`; `THE_DISPLAY` is gone. Four tests.

**Taken out of order, and the order is the plan's own.** This depends on 3 and
not on 4, so it was done while task 4 — the largest in the plan — was still
ahead. Task 6 is in the same position.

**The note fired, and it could not have fired by itself.** `THE_DISPLAY`'s own
sentence said *the moment a second output is advertised, the number comes from
whatever advertises it and this constant goes* — correct, carefully argued, and
attached to a `const` with one reader that nothing could trigger. Task 3 was
that moment and a person had to notice.

**So the replacement is deliberately not another constant with a sentence
beside it.** A number that comes from a map keyed by the display's own name
cannot quietly go back to meaning *the one display*: the next assumption is
wrong where the compiler can see it.

**Two things the implementation decided that the task did not say.**

**A number is never reissued.** The next one is *one past the highest given
out*, not the count — with two displays and one removed, a count hands the next
arrival the number the survivor is still using, and two screens would share a
division.

**And `the_display_retired` now means every display**, because that is what its
callers mean: the session's backend is going away. One display leaving is
`the_display_left(which)`, which is also what a display arriving at an
impossible size now retires — itself, rather than whichever display happened to
be numbered 1.

**Owner:** the Mac. **Depends on:** 3.

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
