# The shortcuts, and the edges

The owner's specification of 2026-09-27. **It is a design specification, not a
claim that any of it is implemented.** What exists is recorded against the tasks
in `the-smallest-canvas-worth-showing.md`, and nothing here should be read as
shipped.

`⊞` is the Windows logo key on a PC keyboard; Command serves the same role on a
connected Mac keyboard.

## Moving and resizing, and why the pointer is the whole affordance

**The edge and the corners resize. The name above the frame moves it.** This is
what Windows and macOS have always done, and it is what a person arrives
knowing.

**The pointer is what teaches it**, which is the reason it needs no furniture:
over an edge or a corner the cursor becomes the **double-headed arrow** for that
direction, and over the name it does not. A frame therefore shows nothing but
its content while somebody works, and still tells them what each part does the
moment they reach it.

This settles the collision in
[ADR 0065](../decisions/0065-the-interface-a-world-its-places-and-the-objects-in-them.md):
one paragraph says dragging an edge or a corner resizes, and the table two
paragraphs later calls the edge a handle. **The table is the loose one.** The
name is the handle for moving; the edge and corners are the handles for
resizing.

Inside the frame, every click still belongs to the application.

## The shortcuts to teach first

| Action | Shortcut |
|---|---|
| Open the alo Bar — find, open, create or ask | `⊞` |
| Zoom the canvas in | `⊞` + Plus |
| Zoom the canvas out | `⊞` + Minus |
| Show all windows on the canvas | `⊞` + 0 |
| Return the canvas to 100% | `⊞` + 1 |
| Focus the selected window | `⊞` + F |
| Switch between open windows | Alt + Tab |
| Enter or leave full screen | `⊞` + Enter |
| Open History | `⊞` + H |
| See decisions in *Requires you* | `⊞` + N |

**Plus and Minus mean the keys that produce `+` and `−` on the person's own
keyboard.** The shortcut guide shows the right keys for German, Finnish and
every other layout rather than the ones a designer had.

## Moving around the canvas

| Action | Mouse or trackpad | Keyboard |
|---|---|---|
| Pan in any direction | two-finger scroll, or Space + drag | arrows while the canvas is focused |
| Pan faster | faster two-finger scroll | Shift + arrow |
| Pan sideways with a mouse | Shift + wheel on empty canvas | ← / → while the canvas is focused |
| Zoom toward the pointer | pinch, or Ctrl + wheel on empty canvas | `⊞` + Plus / Minus |
| Fit the selected window in view | double-click its name | `⊞` + 2 |
| Show the whole canvas | the dock's *Show all* | `⊞` + 0 |

**Scrolling inside an application scrolls that application. Scrolling over empty
canvas moves the canvas.** Space + drag deliberately takes panning even when the
pointer is over a window.

## Working with windows

| Action | Shortcut |
|---|---|
| Switch forward or backward | Alt + Tab / Alt + Shift + Tab |
| Focus the selected window without hiding the canvas | `⊞` + F |
| Enter or leave full screen | `⊞` + Enter |
| Minimize to the fixed panel | `⊞` + M |
| Restore the last minimized window | `⊞` + Shift + M |
| Collapse or expand the minimized-window panel | `⊞` + P |
| Arrange the selected windows | `⊞` + A |
| Place the selected window at the left or right edge of the view | `⊞` + ← / → |
| Move a window precisely with the keyboard | Alt + F7, arrows, Enter |
| Resize a window precisely with the keyboard | Alt + F8, arrows, Enter |
| Close the active window | Alt + F4 |

Move and Resize modes **show an outline before committing**, and Esc cancels
either and returns the window to where it was.

## Selecting several windows

| Action | Shortcut or gesture |
|---|---|
| Add or remove one window from the selection | Ctrl + click |
| Select a group | drag a selection area from empty canvas |
| Select every visible canvas window | Ctrl + A while the canvas is focused |
| Reach windows without a pointer | F6 to focus the canvas, then Tab |
| Toggle a focused window in the selection | Space |
| Arrange the selected windows | `⊞` + A |
| Give the selected windows to alo | `⊞` + D |

**Ctrl + A, Tab and Space keep their ordinary meanings inside an application.**
Their canvas meanings apply only when the canvas itself has focus.

## alo, decisions and undo

| Action | Shortcut |
|---|---|
| Open the alo Bar | `⊞` |
| Offer the selection to alo and review its plan | `⊞` + D |
| Stop alo on the selected work | `⊞` + Shift + D |
| Open *Requires you* | `⊞` + N |
| Open History — inspect, explain, undo | `⊞` + H |
| Undo a canvas action, such as moving or arranging | `⊞` + Z |
| Redo it | `⊞` + Shift + Z |
| Undo inside the active application | Ctrl + Z |

**`⊞` + D never silently starts an agent.** It opens the scope and the plan for
review. An alo change is undone from History, where the person sees precisely
what will be reversed.

## Recording and everyday controls

| Action | Shortcut |
|---|---|
| Open the screen-recording controls | `⊞` + R |
| Choose a screenshot area | `⊞` + Shift + S |
| Go Home | `⊞` + Home |
| Open the shortcut guide | F1 while the canvas is focused |
| Move among canvas, dock, panel, notifications and status | F6 / Shift + F6 |
| Move among controls in the focused area | Tab / Shift + Tab |
| Activate a focused control | Enter |
| Close a menu or a card | Esc |

Inside Blender, a browser or any other application, Ctrl + C, Ctrl + V,
Ctrl + S, Ctrl + F and Ctrl + Z **remain that application's commands**. Full
screen does not take over an application's wheel scrolling or its Esc key.

## Three collisions with what people arrive knowing, not yet decided

`docs/features.md` promises that **the habits people arrive with still work**,
so these are named rather than left for somebody to meet:

- **`⊞` + D is *show desktop* on Windows.** Here it hands work to alo. Somebody
  reaching for their old habit would offer their windows to an agent instead of
  clearing the screen. It is the highest-consequence of the three.
- **`⊞` + M minimises everything on Windows** and one window here.
- **`⊞` alone opens the alo Bar**, where Windows opens its own menu. That one is
  probably correct — it is the same *kind* of thing in the same place — and is
  listed for completeness.

Each is a decision about whether a familiar key keeps its familiar meaning or
earns a better one. None is settled here.
