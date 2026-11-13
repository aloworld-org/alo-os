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
| Zoom the canvas in | `⊞` + Plus, or `⊞` + = |
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

**And `⊞` + = zooms in too — the owner's yes of 2026-11-07.** On a US layout
the main block prints `+` only with Shift, so without a numpad `⊞` + Plus
could not be pressed at all (`docs/quirks/a-shortcut-naming-a-character-is-unreachable-where-the-layout-shifts-it.md`).
`=` is the key `+` shares there. It is a **second** shortcut beside the first,
not a replacement: `⊞` + Plus and the numpad's plus still work. It follows the
action's own shortcut — a person who changes *Zoom the canvas in* has changed
both, and a person who gives `⊞` + = to something else gets their own — and
Settings → Keyboard and shortcuts says *Also ⊞ + =* under the action's name
(`alo_shortcuts::Shortcuts::second_chord_for`).

## Moving around the canvas

| Action | Mouse or trackpad | Keyboard |
|---|---|---|
| Pan in any direction | two-finger scroll, or Space + drag | arrows while the canvas is focused |
| Grab the canvas from anywhere, over a window too | `⊞` + left-drag | — |
| Pan faster | faster two-finger scroll | Shift + arrow |
| Pan sideways with a mouse | Shift + wheel on empty canvas | ← / → while the canvas is focused |
| Zoom toward the pointer | pinch, or Ctrl + wheel on empty canvas | `⊞` + Plus / Minus |
| Fit the selected window in view | — *(double-click on the name maximises since 2026-11-09; see below the table)* | `⊞` + 2 |
| Show the whole canvas | the dock's *Show all* | `⊞` + 0 |

*Double-click its name* was this row's pointer road until 2026-11-09, and it was never built. The owner's window-controls specification of that day gives the double-click on a window's band to **maximise and restore** ([the window controls](the-window-controls.md)); `⊞` + 2 is unchanged.

**Scrolling inside an application scrolls that application. Scrolling over empty
canvas moves the canvas.** Space + drag deliberately takes panning even when the
pointer is over a window.

### `⊞` + drag grabs the canvas from anywhere — the owner, 2026-11-13

Asked *"many canvas apps also let you grab from anywhere, even over a window, by
holding a key such as Super while dragging. Would you like it?"*, the owner
answered **"yes add it please"**. With several windows covering the screen there
is no empty canvas to start a wheel or trackpad pan on.

- **Holding `⊞`, a left press anywhere on the canvas takes hold of all of it** —
  over an application, its band, its edge or empty canvas alike. Dragging pans
  every window together, one for one with the hand in screen pixels, at every
  zoom. Letting go of the button, or of `⊞`, ends it, and nothing glides on
  afterwards, so there is nothing for reduced motion to change.
- **The window under the pointer is told nothing** — not the press, not the
  movement, not the release — and the keyboard stays where it was.
- **The Dock, what it opened, the Bar's matches, the put-aside panel and the
  egress indicator are not canvas**: a press on them is theirs, `⊞` or not.
  Sheets take the whole seat before a press is routed. The top controls' room is
  reserved and nothing is drawn in it yet, so a press there is on the window the
  person can see, and it grabs.
- **The view the press landed on is the one that pans**, to the end of the
  stroke, even if the pointer crosses onto another display on the way.
- **The pointer is an open hand while `⊞` is held** where a press would take
  hold, **and a closed hand while it is held** — so the gesture announces itself
  the moment the key goes down.
- **It is not a row in Settings → Keyboard and shortcuts.** Every row there is an
  action with a chord a person can change, and a chord is modifiers and a key;
  a held key and a drag is not one, which is also why Space + drag and the arrows
  are not listed there.
- **Checked before it was built: `⊞` + drag was free.** Nothing in the shell, the
  shipped shortcuts or the design gave it to moving or resizing a window, as some
  desktops do; here a window moves by its band and resizes by its edges (above).
  **`⊞` alone is the Bar's** in the table above and is not built yet; when it is,
  a `⊞` that took hold of the canvas was not pressed alone and must not open the
  Bar on its release.

Built in `crates/alo-shell/src/canvas_grab_anywhere.rs`.

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
  **Shipped since 2026-11-01, by the owner's choice that day**: minimise
  moved from `⊞` + Down, the Windows habit for one window, to `⊞` + M as this
  table says. One action holds one shipped key, so keeping both would have
  needed a second action or a change to the settings file's format. `⊞` + Down
  now does nothing by default, and anyone can set it again in Settings.
- **`⊞` alone opens the alo Bar**, where Windows opens its own menu. That one is
  probably correct — it is the same *kind* of thing in the same place — and is
  listed for completeness.

Each is a decision about whether a familiar key keeps its familiar meaning or
earns a better one. None is settled here.
