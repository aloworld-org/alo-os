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

**This claim was a double claim until 2026-10-06**, when
[`the-shell-plan.md`](the-shell-plan.md) released the crate. This plan was
written on 2026-10-05 naming `alo-shell` while that plan still claimed it and
still had an unfinished task, which is an error in the plans and was this
lane's to make. It is recorded in both so that a reader of either finds it,
and so that the release can be reversed the same way: the shell plan takes
`alo-shell` back if a machine walk on its task 18 finds a fault there.

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

**Status:** **Done, 2026-10-06: the code.** Every display the device has is
discovered once, drawn in one pass through `render_frame`, and retired on its
own. `Server::render_each_display` answers with `DrawnPerDisplay` — what each
display drew, and which refused, each named — and the desktop lane owns the
displays beyond the loop's one target and retires them through
`LoopInput::retire_the_rest`. Nine tests. **`On the machine.` is not
ticked**: the acceptance asks for both displays painting at their own mode,
shown, and this lane has one laptop and no second display. See
[`updates/a-frame-is-drawn-per-display.md`](updates/a-frame-is-drawn-per-display.md).

**What it deliberately does not do**, so task 5 is not read as smaller than it
is: the displays beyond the first draw clients, **not** the dock or the status
area — the desktop raster is still laid out once. `popups.output_size` is
still one global, so the last display drawn wins it, which is task 6.
Discovery still happens once as the session starts, so a monitor plugged in
mid-session is not seen.

**Owner:** the Mac. **Depends on:** 3.

**Scoped 2026-10-06 by reading the setup, so the next start is not a
discovery.** The desktop lane builds exactly one of each:

```
direct_desktop.rs:275   discover_atomic_output(fd)        one AtomicOutput
direct_desktop.rs:277   SoftwarePainter::new()            one painter
direct_desktop.rs:292   RoutedInput { extent }            from that one mode
direct_desktop.rs:~300  run_with_input(server, Target::new(painter, fd, output), …)
```

**`discover_every` already exists and is private.** Task 2 built it; exposing
it is one line, and it answers with every display already in preference order
with distinct CRTCs and planes. **The discovery half of this task is done.**

**What is actually left is ownership and lifetimes**, which is why it is the
largest task here and not the obvious one:

- A `Target` owns its painter and borrows the device descriptor, so N targets
  means N painters and N borrows of one `fd`.
- `run_with_input` takes **one** target, and six `LoopInput` impls and six
  `LoopTarget` impls hang off that signature. **Changing it reaches the
  sign-in lane**, which genuinely has one display and should not pay for this.
- So the containable shape is **`Desk` owning the additional targets** and
  drawing them in its own `present`, leaving the generic loop and the greeter
  untouched. `Desk` already carries a lifetime; the extra targets add more.
- Retirement has to follow: task 3 made the backend retire once and every
  display withdraw its own global, and additional backends are not yet in
  that path.

**And the acceptance's third clause is the one to build first, not last** —
*a display that fails to submit does not stop the other from painting, and
says which one failed*. Per-display failure isolation is testable with the
fake targets the loop tests already use, and it is the part that decides
whether a machine with one broken output is usable.

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

**Status:** **Done, 2026-10-06: the code**, for the layout half. Every display
is laid out from **its own size and its own scale** instead of being handed the
first display's pictures. The scale is the person's, held in their arrangement:
`Server::the_scale_of_display` looks it up and decides nothing, because a shell
working one out from pixels and millimetres would be deciding how large this
person's interface is, in a drawing crate, which this plan's header and
`the-shell-plan.md`'s both forbid. Three tests. **`On the machine.` is not
ticked** — one laptop, no second display. See
[`updates/the-desktop-is-laid-out-per-display.md`](updates/the-desktop-is-laid-out-per-display.md).

**It did split, as this task said it would**, and the half that remains is now
task **5b** below. What is *not* done here is the third acceptance clause — the
fixed-control bounds are still the first display's.

**Owner:** the Mac. **Depends on:** 4, and **3a** — laying a desktop out per
display needs to know where each display is, not only how big it is.

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

### The one thing 5b, 6 and 7 all need, measured 2026-10-06

**No window is on a display.** Not *not yet recorded* — the opposite is
asserted, once a frame, in `crate::desktop_membership`:

```rust
let displays: Vec<DisplayId> = self.desk.displays().collect();
for surface in &open {
    let number = self.desk.number_of(surface);
    for display in &displays {
        self.desk.put_on_the_current_desktop(*display, number);
    }
}
```

**Every window joins the current desktop of every display.** With one display
that is a tautology and is why it has never been wrong. With two it is a
decision nobody made: a window opened on the laptop is also on the monitor,
and `show_the_current_desktops` then shows it on both.

So the question *which display is this window on* has no answer to give, and
three tasks need one:

- **5b** holds a frame to the fixed-control bounds of its display.
- **6** constrains a popup to the screen it is on.
- **7** gives each viewport its own camera.

**Build it once, not three times.** Each of the three would otherwise
discover this separately, which is the fault this plan's task 9 note in
`the-canvas-and-its-places.md` was written about.

**Built on 2026-10-06 as part of 5b:** `Server::the_display_a_window_is_on`
now answers *which display is this frame on*, so 6 and 7 inherit it rather
than each building it. What follows was the reasoning that got there, kept
because the correction in it is worth reading.

**The mechanism is here already; only the rule is missing.** Corrected on
2026-10-06, a few minutes after the paragraph above was first written saying
the opposite — `put_on_the_current_desktop` is **`alo-shell`'s own**
(`server_desk.rs:240`), not `alo-desktops`'. That crate is already keyed by
display (`on_mut(display)`), and `Screens::main_screen` already names a main
screen. So choosing one display instead of looping over all of them is a
small change in this crate and needs no new code anywhere else.

What is genuinely missing is the **rule**, and it is a product decision rather
than an engineering one: *which display does a new window open on?*
`docs/features.md` and `docs/design/` say nothing about it — searched, not
assumed. Three readings, with the first already implementable today:

1. **The main screen**, which the person's arrangement names.
2. **The display the pointer is on**, which is where they are looking.
3. **The display that last had focus**, which is where they were working.

**This belongs to the owner, not to a lane**, because it decides where a
person's work appears and `CLAUDE.md`'s *when the answer is not obvious, it is
scope* applies. Reported now rather than at the end of whichever of 5b, 6 or 7
reaches it first.

### 5b. The bounds a frame is held to are the ones for the display it is on

**Status:** **Done, 2026-10-06: the code.** `FixedControls` is one store per
display, keyed as the presentations are, and every consumer resolves which one
— the hidden-frames search per frame, the reachability search from the frame
it was given, the drag path from the window it holds, the pointer classifier
from the point, and *at most one surface claims any point* from **every**
display, true only if all of them hold. Eight tests. **`On the machine.` is
not ticked**: one laptop, no second display. See
[`updates/the-bounds-a-frame-is-held-to.md`](updates/the-bounds-a-frame-is-held-to.md).

**The ground it needed was built here too**, because 6 and 7 need the same
thing and this task reached it first: `Server::the_display_a_window_is_on`
answers *which display is this frame on* by greatest overlap of desk
rectangles. **No new coordinate system was introduced** — `Position` already
says a display's corner is *negative to the left of the main screen*, so the
compositor's one coordinate space already is the desk with the main screen at
its origin. What was missing was anybody asking the arrangement which of its
rectangles a point falls in.

**Owner:** the Mac. **Depends on:** 5.

**Split out of 5 on 2026-10-06**, which predicted it would be two tasks. The
layout half is done and this is the half it named: *the fixed-control bounds
recorded by `the_fixed_controls_were_drawn` are the ones for the display being
drawn, so `canvas_never_lost`'s rule is asked about the right rectangles.*

**What is true now.** `Desk::present` lays out every display, and records the
**first** display's dock band, panel column, indicator band and top controls
into the server. With one display that is right and is what shipped. With two
it means a frame is held to the furniture of a display it may not be on.

**Two things this needs that it does not have, measured rather than assumed:**

- **Nothing maps a surface to a display.** `Server::mapped_surfaces` is one
  list and every display is handed all of it, so *which display is this frame
  on* has no answer yet to hold a frame to. That is the same gap task 6 is
  blocked on and task 7 needs, and it may be worth building once for all three
  rather than three times.
- **`FixedControlsDrawn` is one value on the server.** It is replaced per
  frame; two displays would need one per display, keyed as the presentations
  are.

**What this task must not do, because it was checked and nearly got wrong.**
`the_fixed_controls_were_drawn` takes an `alo_appearance::TextScale` and **not**
a display scale. The owner ruled on 2026-10-01 that the 44 × 24 handle floor is
logical, scaled by the person's text size and nothing else, and asked that any
implication of a second conversion be removed;
`desktop_raster_tests::the_dock_band_and_the_panel_column_do_not_move_with_the_displays_scale`
measures that the rectangles do not move with the display's scale. Passing the
display scale there would re-introduce a fault that ruling exists to prevent.
**The rectangles still differ per display — because the displays differ in
size, not in scale — which is why this task is real.**

- **Acceptance:** with two displays of different sizes, a frame is held to the
  bounds of the display it is on, and a frame reachable on one display is not
  brought back because it is under the other's dock.
- **Constraint:** the handle floor stays logical and text-scaled. This task
  changes *which* rectangles are asked about, never how they are measured.

### 6. A popup is constrained to the screen it is on

**Status:** **Done, 2026-10-06: the code.** A popup is constrained to the
screen its parent is on, and a popup whose display goes is dismissed. Eight
tests — seven on the arithmetic, one driving a real client through the whole
road. **`On the machine.` is not ticked**: one laptop, no second display. See
[`updates/a-popup-is-constrained-to-its-screen.md`](updates/a-popup-is-constrained-to-its-screen.md).

**What unblocked it was 5b**, which built the surface-to-display mapping this
task's own note below says does not exist. That note is kept because its
reasoning was right: the mapping genuinely did not exist when it was written,
and task 4 is what made it possible.

**A true unplug is still not detected.** Nothing in this compositor watches
for a display arriving or leaving mid-session — discovery runs once, as the
session starts. The dismissal is wired to the *display has gone* path that
does exist and is exercised: the session's output retiring, and a display
whose area changes. **So the acceptance's words *unplugged mid-grab* are met
for every way a display can currently go, and hotplug itself is a gap this
plan does not close.** Recorded here rather than left for whoever first plugs
a monitor in.

**Owner:** the Mac. **Depends on:** 3, **3a** and **4** — not 3 and 3a alone,
which is what this line said when the plan was written.

**Why the dependency was wrong.** This task has to answer *which screen is
this popup's parent on*, and **nothing in the compositor can answer it**:

```
a surface → a display    no such mapping exists anywhere in alo-shell
popups.output_size       one global, set in render_frame from the one
                         target's size — so it is whichever display drew
                         the most recent frame, for every popup
```

`Screens` gives each display a corner and a room, which is what task 3a was
for — but a *surface* is placed by `scene::trees` against the one viewport the
compositor draws, and until a frame is drawn **per display** there is no fact
of the matter about which display a given surface is on. Task 4 is what
creates that fact.

**So task 4 is the only ready task in this plan**, and 5, 6 and 7 are all
behind it. That is worth stating plainly because the dependency list made 6
look like cheap work that could be taken while 4 waited, and two readings of
this plan have now gone that way — the first discovered 3a, the second
discovered this.

**Owner:** the Mac.

**Depends on:** 3, and **3a** — *the left display's inner edge* is a position, and the session has none until 3a.

`Surfaces::camera`'s own note says this is why the camera's home had to be
settled first: *which display's plane constrains a popup is a question the old
shape could not be asked.* Now it can be.

- **Acceptance:** a popup opened near the inner edge of the left display flips
  against **that display's** edge, not against the union of both and not against
  the other's. A popup on a display that is unplugged mid-grab is dismissed
  rather than left constrained to a screen that is gone.

### 7. A camera per viewport

**Status:** **Done, 2026-10-06: the code.** Each display holds its own
camera and is drawn through it, so two displays are two views of one canvas
rather than one view twice. Five tests, and three defects found by tests that
already existed — see the report. **`On the machine.` is not ticked**: one
laptop, no second display, which this plan's first paragraph said would be
true of every task in it. See
[`updates/a-camera-per-viewport.md`](updates/a-camera-per-viewport.md).

**Owner:** the Mac. **Depends on:** 5 and 6.

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

## What a machine actually reaches, measured 2026-10-08

**Ten tasks say *Done: the code*, and a reader takes that to mean the code
runs.** For five of them it does not. `TheDesktop::the_screens_of` answers
`None` by default and **`alo-desktop` does not override it**, so
`Server::the_screens()` is `None` on every running machine and every path that
depends on an arrangement takes its single-display fallback.

Found by the Mac lane on 2026-10-08, by reading
`alo-citing`'s `every_surface_a_person_uses_is_built_somewhere` and following
`NightLight` — one of the four surfaces it lists as reachable only by tests —
back to what builds a `Screens`. Nothing does.

| task | what a machine reaches today |
|---|---|
| 1, 2 discovery per display | **reached.** `discover_every_atomic_output` is called by the desk |
| 3 a presentation per display | **reached**, keyed by name |
| 3a the session holds the arrangement | reached, and **always `None`** — nothing answers with one |
| 4 a frame per display | **reached.** Every display is discovered, drawn and isolated |
| 5 laid out per display | **half.** Each display is laid out at its own *size*; its *scale* is 100 because the arrangement is `None` |
| 5b bounds per display | **unreached, twice** — see below |
| 6 a popup on its own screen | **unreached.** With no arrangement, `popups.screens` is one rectangle written from whichever display drew last |
| 7 a camera per viewport | **unreached.** With no arrangement every display reads the camera under the empty key, so they mirror |
| 8 the display number | **reached** |

**Task 5b is unreached for two independent reasons**, and the second is a plain
defect rather than a missing arrangement: `Desk::present` calls
`the_fixed_controls_were_drawn` **once**, for the display the loop holds. The
per-display loop beside it lays out every other display's pictures and records
none of their controls. So the store has one entry whatever the arrangement
says, and the `len() <= 1` safeguard — written to keep one display behaving as
before — returns that one entry for every window on every display.

### Why this is recorded rather than quietly wired

The cheap fix is three edits: re-export the argument type, build a `Screens`
from shipped defaults in `alo-desktop`, done in an hour. It would make
`Screens` exist in production, pay `NightLight`'s debt entry, and turn all five
rows above green.

**It would also turn an honest *I have no arrangement* into *here is the
arrangement*, and the only thing demonstrable about it would be that it
compiles.** Unreachable code would become unverified code. The debt entry whose
whole purpose is to say this is not done would be paid without the behaviour
being real.

So the rows above are written first, and the tasks below do it in the order
that keeps each claim checkable.

## Tasks that make this plan's work reach a machine

### 9. Every display records the controls it drew

**Status:** ready. **Owner:** the Mac. **Depends on:** nothing — this is a
defect in task 5b's own change and needs no arrangement.

`Desk::present` records one display's fixed controls. Every other display's
pictures are laid out and thrown away as far as the store is concerned.

- **Acceptance:** with two displays drawn, the store holds an entry per display,
  and a frame on the second display is held to the second display's dock rather
  than to the first's by the one-entry safeguard. The existing single-display
  behaviour is unchanged.
- **Constraint:** the safeguard stays. One display must keep answering exactly
  as it does now, because that is every machine this lane can test on.

### 10. The seam the desktop can actually travel

**Status:** ready. **Owner:** the Mac. **Depends on:** nothing.

`TheDesktop::the_screens_of` takes `Vec<alo_displays::Reported>` and
**`alo-desktop` has no `alo-displays` dependency**, so the only crate that must
implement it cannot name its argument. The road task 3a declared is not
travelable from the crate that must travel it.

**The patch is a re-export and the fix is the seam.** `Support` is documented as
*which sizes the compositor underneath can actually draw — asked of the
compositor, because it is a fact about the machine*. So the shell should build
the arrangement from what it already holds, and ask the desktop only for what
the desktop owns: this person's appearance, their night light, their kept
changes.

- **Acceptance:** `alo-desktop` can implement the road without naming an
  `alo-displays` type and without a new crate edge; each side hands over only
  what it owns.
- **Constraint:** a desktop that answers nothing still gets today's behaviour —
  `None` is a desktop with no arrangement to offer, not a broken one.

### 11. A machine builds the arrangement it reports

**Status:** blocked on 10. **Owner:** the Mac.

With the seam travelable, `alo-desktop` answers with an arrangement built from
what the compositor reported.

- **Acceptance:** a test asserts **production** builds one — not that it can be
  built. `every_surface_a_person_uses_is_built_somewhere`'s `NightLight` entry
  is removed in the same change, because that list fails in both directions and
  a paid debt must not be left listed.
- **Constraint:** this ticks **nothing** about more than one display working. It
  makes the five rows above reachable and verifies only that. Two displays
  remain unshown, as the plan's first paragraph said they would.

### 12. The default stops being silent when a person can choose

**Status:** blocked on 11. **Owner:** the Mac.

Task 11 passes `Changes::untouched()` — *the person has changed nothing* —
which is **literally true on 2026-10-08**: nothing in production writes a
display arrangement, so nobody can have arranged one. Verified by searching for
production writers and finding none.

**It stops being true the moment the settings road lands, and it stops
silently.** A person who arranges their displays would have it ignored while the
code reads as though it were honouring them.

- **Acceptance:** a test fails when anything in production writes a display
  arrangement while the desktop still passes `untouched`, naming
  `where-a-persons-settings-are-kept-plan`'s task 8 as what changes it.
- **Constraint:** it must fail for the right reason. A test asserting *nothing
  writes an arrangement* passes forever and says nothing; the pair is **if
  something writes one, the desktop must stop passing `untouched`**.

## What closes this plan

Canvas task 9's acceptance, shown on a machine with two outputs. Until then the
code halves close and the machine halves do not, and the plan says which is
which rather than letting the distinction blur.
