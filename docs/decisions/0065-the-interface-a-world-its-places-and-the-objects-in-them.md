# ADR 0065 — The interface: a canvas for each goal, and the objects on it

**Status:** **accepted, 2026-09-22, by the owner.** Nothing is built in the
change that adds this. Each part is its own task, and the v0.5 shell — the
dock, the status area, the sign-in, the windows being finished now — stays as
the foundation and as the option a person can keep (Law 5). This decides the
**v1** interface.
**Date:** 2026-09-22
**Proposed by:** the owner, from his own brief; written down by the dev PC
**Context:** [ADR 0002](0002-the-shell-is-native.md) (the shell is native);
[ADR 0009](0009-a-good-computer-without-the-agent.md) (anything a verb can do, a
person can do by hand); [ADR 0064](0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md)
(Law 5, and the three levels of running code);
[ADR 0001](0001-the-capability-model.md) §5 (a change waits and is written
down); `docs/features.md`'s *alo's visual language* and *What makes a person
want it*; the Screen Brief of 2026-09-12, which said the desktop should be
*conventional where it is conventional* — **this decision supersedes that for
v1**, on the owner's instruction.

## The sentence, and what it must not do

> **Work your way: do it yourself, do it with alo, or hand it all to alo.**

The owner rejected an earlier wording that said *hand **part** of it to alo*,
because it caps what alo does: **a person may hand over everything**, and the
interface has to make that as ordinary as doing it by hand. The promise beside
it is *your computer, simplified; intelligence when you want it; control
always* — where **control means the person chose the level**, including *all
of it*.

## Three levels, two tools

```
World  →  Place  →  Object
```

- **World** — every Place the person has, seen at once.
- **Place** — **a canvas**: an endless surface holding everything that belongs
  to one goal, with the people in it and the agents working in it. A Place is
  not a folder, not an application and not a window. It is the desk.
- **Object** — the thing being worked on: a document, a message, a meeting, a
  table, an image, a task. **An object opens as a panel on the canvas**, where
  the person put it, and stays there.

**Applications open on the canvas too.** A spreadsheet, a browser, Blender:
each is a panel on the surface, beside the work it belongs to, not a window
floating over a desktop. **The canvas replaces window management.** Nothing is
minimised, because nothing was ever stacked; a panel is simply somewhere else
on the surface, and the way back to it is to move there.

**A panel is a frame, and the canvas behaves the way a design canvas does.**
The owner's words: *like the one of Figma* — the opened applications are
frames, and the person changes their size by dragging. So:

- **Handles, not chrome.** A frame is the application's own picture with
  nothing around it. Selecting it shows handles; dragging an edge or a corner
  resizes it, and the application is told its new size as it happens, not after
  a border is let go.
- **Move, select, align.** Drag to move. Draw a box or hold shift to take
  several. Guides appear as edges line up, and things snap when they nearly
  meet. Several frames selected move, resize and align together.
- **Frames carry a name** — *Pricing*, *Blender — the chair*, *Anna's offer* —
  shown when the canvas is far out and out of the way when it is close.
- **The gestures a person already knows.** Pinch or wheel-with-a-modifier to
  zoom; space and drag, or two fingers, to pan; a key to fit the whole Place on
  screen; a key to fill the screen with what is selected; double-click a frame
  to work inside it. Every one of them has a keyboard form.
- **Tidy up, without losing where things are.** Alignment and distribution are
  there for a person who wants them, and *tidy this canvas* is something alo can
  be asked to do — and it is a proposal like any other, shown as a ghost before
  anything moves.
- **A frame can leave.** Drag it out of the Place to the World and drop it into
  another Place; the work goes with it.

**A frame arrives the shape its work is, and can be made small without going
away.** The owner's example is a messaging application: opening it should give
a narrow column of conversations, the shape it has on a Mac, not a large
rectangle a person has to cut down. So:

- **Every frame has a natural size and shape**, declared by the application and
  remembered per Place once a person changes it: a chat is a tall column, a
  spreadsheet is wide, a video is sixteen by nine, a terminal is as many columns
  as it says.
- **A frame has two forms.** *Full*, which is the application, and **compact**,
  a small live tile that still shows what matters — the last three messages,
  the track playing, the build at four of six — and can be acted on without
  growing back. Dragging a corner far enough moves between them; so does one
  key.
- **Compact replaces minimising.** Nothing is swallowed into a bar at the
  bottom of the screen: a small thing is still on the canvas, still alive, still
  where the person left it, and still says what it is doing.
- **An application that offers no compact form gets one made of its name, its
  frame's last picture and whatever it is doing** — never a blank square.

**Where the title bar's work goes.** The owner asked the right question — a
title bar does five things, and each needs a home before it can be taken away:

| A title bar does | On the canvas |
|---|---|
| says what this is | the frame's **name sits above it**, shown when the canvas is far out and faded while the person works close |
| gives something to grab | **the name is the handle**, and so is the frame's edge. Inside the frame every click belongs to the application, so nothing is moved by accident |
| close, minimise, maximise | **close is removing it from the canvas**, and that is in History; **minimise does not exist**, because nothing is stacked; **maximise is zooming in** |
| shows which window has focus | the **selected frame carries handles**, and what the person has zoomed into is plainly what they are in |
| holds menus | tools appear when something **inside** the content is selected |

So a frame is not chrome-less by fashion. **It shows nothing but its content
while the person works, and its name and its few controls appear when they
point at it, select it, or zoom out.** What is gone is the permanent grey strip
repeated on forty frames, carrying the same three buttons and a border that
existed to be dragged.

An application that insists on drawing its own window is given a frame holding
that window, which is the one place the old shape survives.

**Arranging is placing.** Drag a panel where it makes sense and it stays. Two
panels side by side are two panels side by side, not a split somebody
configured. Zooming into a panel gives it the whole screen; zooming out puts it
back where it sits, among the rest.

Moving between them is **one gesture, zoom**, and moving across a Place is
**pan** — by pinch and drag, by wheel with a modifier, and by key, so a mouse
and a keyboard both have all of it. Zoom out of a panel to see its Place; out
again for the World; in to fill the screen with one object. It replaces window
switching, minimising, tiling and virtual desktops, and it is **not**
*desktop → application → window → tab → folder → file*.

**A canvas has to stay a tool, not a museum**, so four rules hold it: a Place
opens where the person left it, never at a stored zoom nobody remembers; **the
Bar finds anything on the canvas** without hunting for it, and takes the person
there; frames the person has not touched for a long time are offered for tidying
rather than left to accumulate for ever; and **a frame is never lost off the
edge** — fitting the Place on screen always brings everything back into view. What is off screen costs
nothing: a panel out of view is a still picture until it is reached, so a Place
with forty things in it is not forty programs running.

Two tools are always one gesture away:

- **The alo Bar** — ask, find, open, create, or hand over. **It works with no
  model at all**: application names open applications, file names find files on
  the machine, setting names open settings, arithmetic is exact, and commands
  run. That is ADR 0009 applied to the one place everything starts.
- **History** — what happened, why, and undo. Agent actions come from the
  record the kernel already watches. **A person's own work is shown from file
  versions, never from watching them**: *it forgets everything that was not an
  agent* is unchanged.

## What keeps a canvas better than a wall of miniatures

Five things, decided with the canvas rather than after it, because each one is
where a canvas usually fails.

1. **A frame simplifies as it shrinks.** Full application, then its compact
   form, then its name and what it is doing — chosen by how large it is on
   screen, not by a setting. Zoomed out, a person reads *three new from Anna*
   and *the build at four of six*, never a wall of unreadable miniatures. **This
   is what makes the far view worth having.**
2. **Zones mean something.** A region of a canvas can be named — *drafting*,
   *waiting on Anna*, *done* — and dragging a frame into one **does** what the
   name says: a task dropped in *done* is done, a document dropped in *waiting
   on Anna* asks her. A zone can also be handed to alo whole: *everything in
   drafting, finish it.* A canvas that knows what its regions mean is a way of
   working, not a wall to pin things on.
3. **A Place remembers time, not only space.** The time ribbon runs through the
   canvas: drag it back and the Place is as it was on Tuesday — which frames
   were open, where they sat, what alo had done — built on the snapshots undo
   already takes and the record the kernel already watches.
4. **Every screen is its own view onto the canvas.** Two displays are not two
   desktops but two viewports, each at its own zoom and position, on one
   surface; pushing a frame past the edge of one puts it on the other. On a
   small screen, **focus** shows one frame at a time, and the Bar and
   next-and-previous do the moving, so a thirteen-inch laptop is not a worse
   version of the idea.
5. **The habits people arrive with still work.** The key that cycles windows
   cycles the frames of this Place; the key that closes a window removes a frame
   from the canvas; the key that switches desktops moves between Places. Nobody
   leaving Windows is stranded on the first morning, and nothing about the
   canvas has to be learnt before anything can be done.

## What is on the screen

- **Home shows three things**: the Bar, *Continue* (up to three Places), and
  *Requires you*. The time, the battery and **the privacy symbol** are always
  there. No widgets, no application grid, no sidebar, no agent panel.
- **Content is the interface.** A panel shows its content and nothing else;
  tools appear when something inside it is selected. Zoomed into an object, it
  fills the screen.
- **Anything selected can be done, or given to alo** — by dragging it to the
  alo symbol, by the alo key, or from the menu a right-click opens.
- **A whole goal can be handed over.** *Give the European launch to alo*: alo
  shows its plan in the Place, works in the background under one capsule, and
  returns only the decisions that must be the person's — money, anything
  public, anything legal, and whatever they said they would decide. The person
  may step in, take over a piece, or stop it, and everything alo did is in
  History.
- **Agents appear only where they work**: a terracotta outline on that object,
  a named cursor, and a way to stop it. With nothing running there is no agent
  interface at all.
- **Proposals are ghosts.** At *ask me* a change arrives translucent, with
  *keep*, *adjust* (say what to change) and *reject*. At *full trust* changes
  land directly and stay undoable — otherwise *hand it all to alo* would become
  a hundred approvals, which is the same refusal wearing a helpful face.
- **Notifications are decisions**: *requires you*, *working*, *finished* — and
  *finished* only for work the person started and walked away from. Nothing
  announces a connected printer.
- **The model disappears** behind four plain choices — *on this computer*, *my
  provider*, *alo in Europe*, *no AI* — and every answer still says where it
  was processed (ADR 0008).
- **With no model, nothing dramatic happens.** The alo symbol goes quiet, one
  calm line says so, and everything that does not need a model still works.

## As the owner decided it

1. **The dock goes, by default.** The owner: *Apple removed the buttons without
   asking the people who loved them; if we can remove something successfully
   and innovatively, why not.* The alo key, the bottom edge or a swipe reveals
   the **alo Edge** — *Home, Continue, Create, Applications, System*. **The
   replacement must be found by a person who has never seen it, within thirty
   seconds, with no instructions**, and that is a test with people, not an
   opinion. If they cannot, the Edge is fixed; the dock does not come back. A
   person may pin one (Law 5).
2. **The clock, the battery and the privacy symbol stay visible always**, and
   **the privacy symbol can never be hidden**: Law 1 does not bend, and it takes
   no choice away from anybody. Its states are *private*, *local activity*,
   *data leaving*, and *camera or microphone on*.
3. **One world, not two.** Every application opens as a panel on the canvas,
   **including the ones that expect ordinary windows** — Blender, a browser, a
   development tool. An application that insists on its own window gets a panel
   holding that window, on the canvas, rather than a second desktop behind the
   first. An interface where alo's own work is modern and everything else is
   exiled to an old desktop is the failure this decision must not repeat.
6. **The canvas must stay usable for somebody who cannot pan and zoom.** Every
   Place also answers as a list — its panels, in order, reachable by keyboard
   and readable by a screen reader — and the Bar reaches anything by name.
   A surface nobody can navigate without a touchpad would fail EN 301 549 and
   fail a person with a tremor.
4. **Objects begin where alo can see inside them** — alo's own applications and
   files — and widen as adapters and alo's own engine make other applications
   readable as structure.
5. **Everything hidden stays reachable.** The Edge, the tools that appear on
   selection, and zoom each have a keyboard and a screen-reader road
   (EN 301 549), and the layouts hold in German and Finnish.

## The five screens that define it

Home · Place, as a canvas (including one handed entirely to alo) · Object (a document
the person and alo are both in, with a ghost) · World · History. Installation,
Settings, Files, the App Store and recovery follow the same rules; these five
carry the model.

## The bar every part is held to

Can it be understood with no explanation? Can one more visible thing be
removed? Does the person see their work or our interface? Is there one obvious
next action? **Did the person decide how much alo does?** Does it still work
with no model? Does it feel inevitable?

## Consequences

- `docs/features.md` gains *the interface* section; the visual language and the
  wow features already landed there point at it.
- The v0.5 shell is not undone and not paused. It is the foundation, and the
  familiar layout stays available.
- **Each part is its own task**, sized separately: the Bar, Places, Objects,
  zoom, the Edge, ghosts at each trust level, handing over a goal, History,
  notifications, the privacy symbol.
- The Screen Brief's *conventional where it is conventional* holds for v0.5 and
  is superseded here for v1.
