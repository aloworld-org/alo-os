# ADR 0145 — alo's own windows live on the canvas

**Status:** **accepted, 2026-11-01**, by the owner's answer to the question put
the same day: *A. All on the canvas.*

## What it changes for the person

On a booted machine on 2026-11-01 (image of `c4e6cee0`, qemu) the person zoomed
the canvas out twice. Dolphin shrank with it. **alo's own terminal stayed full
size, in place, drawn on top of Dolphin.** *Show all* then fitted Dolphin and
ignored the terminal. That is a large part of the owner's complaint that *the
screens overlap each other and seem the same*: the windows alo draws itself
were not on the canvas at all, so the canvas could not arrange them, zoom them
or find them.

With this decision, **alo's terminal, its task windows, Settings and the
record are canvas windows like any application's.** They:

- pan and zoom with the canvas;
- are counted by *Show all* and reached by *return to 100%*;
- stack with the other windows the way the person puts them, instead of always
  floating above everything else;
- belong to a Place, as `docs/features.md` already promises of *every
  application*.

**Nothing is taken from the person.** None of these windows offered a choice
about floating; that was an unmade decision, recorded as one in
`crates/alo-shell/src/where_a_task_window_opens.rs`.

## The decision

[ADR 0065](0065-the-interface-a-world-its-places-and-the-objects-in-them.md)
already says it in two sentences: *an object opens as a panel on the canvas* —
it names *a task* among the objects — and *applications open on the canvas
too … not a window floating over a desktop*. alo's terminal is an application
the person opened, a task window is the task object, and Settings and the
record are the two applications alo itself ships. This ADR applies 0065 to
them instead of leaving them as the one exception nobody chose.

**What stays fixed to the screen** is what was always the viewport layer's
(`the-canvas-as-a-workspace.md` §2): the Dock, the put-aside panel, the top
controls and **the egress indicator**, which stays above everything, by design,
at the top left. Nothing here moves it, and it must never be covered by a
window, whether that window is alo's or an application's.

## The options that were weighed

- **A. All four on the canvas** — chosen.
- **B. All four fixed to the screen, as before.** They never move away from the
  person, and they keep covering zoomed work. That covering is the fault the
  walk showed.
- **C. A split**: the terminal, task windows and Settings on the canvas, the
  record fixed. Rejected because a fixed record is a window that covers work in
  the same way, and *see what alo did* is something a person reads beside the
  work it describes.

## What has to be true when it is built

- **Opening one is opening a window** (`docs/design/where-a-new-window-opens.md`):
  a new one opens in the current view, beside the active window, and never
  exactly on top of another one.
- **Going to one that is already open travels to it**: the chord, the Dock or
  Alt+Tab brings it in front, and the view goes to where it is rather than
  pulling it to where the person is looking (`the-canvas-as-a-workspace.md`
  §11). *The window you go to comes in front* (#1044) stays true.
- **The keyboard rules stay**: a window of alo's takes the keys when it is in
  front, exactly as now.
- **The window look applies**: the border, the title and the focused look that
  canvas windows have also apply to alo's windows.
- **Recovery holds**: *Show all* reaches them, and a fixed control may not
  completely cover their handle, as for any frame.

## How it is built, and what is known before it starts

In stages, one window per change, each of which leaves the machine working:
the terminal, then task windows, then Settings, then the record. A read of the
code on 2026-11-01 found what each stage has to change. Today each window's
rectangle is in display pixels, and each window is drawn as a layer over the
plane rather than among the windows on it. Two of them have no pointer hit
test at all. Several of the pictures are clipped to the display, and a canvas
window must be able to sit partly off it.

**Found in the same read, and reported rather than assumed:** the external
window edge drawn around *application* windows reads a window's position on the
plane as though it were a position on the screen, so it may not follow the
canvas when it is panned or zoomed. That belongs to the lane building window
borders and titles, and it is the edge alo's windows will share.

## What this does not decide

- How alo's windows look at a far zoom. *A frame simplifies as it shrinks* is
  its own promise, and it applies to them as to any frame.
- Where they go on a second display. There is one canvas with a camera per
  display; the windows are placed where the person is working, by the same rule
  as any new window.
