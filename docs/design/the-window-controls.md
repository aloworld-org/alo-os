# The window controls

**Status: the owner's specification of 2026-11-09, with every figure measured off
the design file.** Recorded before implementation, which is `CLAUDE.md`'s first
standing rule. **Nothing in this note is a claim that any of it is built**; what
is built is said, with its pull request, in *What is built* at the end.

**The design:** file `XbSmsBYix0TrcQ0Qzaup64` — a new file, not
`nDxyF5Ho9oC4RObjVzwBNJ`. Six implementation frames, read live on 2026-11-09 and
committed at
[`figma-snapshot/window-controls-XbSmsBYix0TrcQ0Qzaup64.xml`](figma-snapshot/window-controls-XbSmsBYix0TrcQ0Qzaup64.xml):

| frame | what it shows |
|---|---|
| `11:2` | Resting window — title and application name, no controls |
| `11:24` | Controls revealed, and the window panel open |
| `11:84` | Placement preview — *Left* previewed, nothing committed |
| `11:146` | alo working in this window, with Stop |
| `19:2` | Capture & export, the visible window as an image |
| `19:96` | Capture & export, a full document as a PDF |

The boards *Floating capsule*, *Bare symbols* and *Soft squares* in the same file
are exploration history and **not** the target. The frames are visual states,
not a wired prototype.

**It supersedes** the four-control edge of
[the external window edge](the-external-window-edge.md) — Move, Minimise,
Maximise and Close on a 32-tall strip in a 44-tall region — and the 2026-11-01
round-2 edge's application icon. That note keeps what it says about decoration
ownership, content never moving and the long-title ruling of 2026-10-10, and
says at its head which parts this one replaced.

## The owner's two rulings, 2026-11-09

Asked in a question dialog the same day, because the specification and the
standing promises disagreed:

- **No hide wait.** *"No wait, continuous path (Recommended)."* Controls stay
  while the pointer is in the band, on a control, or travelling into the panel,
  and fade as soon as it truly leaves — **about 160 ms out, about 120 ms in, and
  immediately under reduced motion.** There is no 400 ms wait. The standing
  promise *no reveal or hide delays* (`docs/features.md`, the Dock's reveal) is
  unchanged: a fade is how a change is shown, not a delay before it happens.
- **The old title choice is replaced.** *"Replace it."* The title is always
  visible; Settings offers only **Always show window controls**. The choice
  *Show the title only when I point at it*, promised on 2026-11-01, is
  **withdrawn by the owner** — law 5 lets the owner withdraw what was offered,
  and `docs/features.md` records it with these words.

## What the person sees

### The band, at rest (`11:2`)

The title, then the application name in quieter text — **The next chapter ·
Notes**. Both are visible whatever the pointer is doing; an inactive window's
softens and stays readable. With no meaningful title, the application name only,
never the same word twice. A long title is cut with an ellipsis before the area
the controls take, so **revealing the controls never moves the title**, and the
whole title is shown on hover or keyboard focus (the 2026-10-10 ruling, kept).

**No controls are drawn at rest**, and no application icon: `11:2` draws none.

### Revealed (`11:24`)

Exactly **three** controls, at the trailing end of the band, each a 44 × 44
target:

1. **Window actions** — the layout mark, no word on it; tooltip and accessible
   name *Window actions*; pressing it opens and closes the window panel, and
   while the panel is open it wears the open look (navy square, white mark).
2. **Minimise** — puts this window aside, travelling toward the panel of
   put-aside windows on the right; restoring returns it to its Place and its
   size.
3. **Close** — asks this window to close; the application handles unsaved work;
   never forced. A restrained red on hover and press. It sits on the far side of
   Minimise from the panel, so the pointer that opened the panel does not land
   on it.

**There is no full-screen button on the band.** Full screen is in the panel.

**When they appear:** as the pointer enters **anywhere in the band** — on an
inactive window too, **without activating or raising it**. They stay while the
pointer is in the band or on a control, while the panel or anything it opened is
open, and while the keyboard is on them. Pointing at Window actions does not
open the panel; pressing it does. **Always show window controls** in Settings
keeps them shown. A control that cannot be seen cannot be pressed, and the
keyboard never rests on one that cannot be seen.

### The band as a handle

**Dragging the band moves the window** — it is the only free-move handle (ADR
0071, unchanged). Buttons and panel contents never start a drag. **Double-click
maximises**, and double-clicking again restores the size it had. **Right-click
opens the same window panel.** Clicking the title never renames anything.

*This replaces one clause of [the shortcuts and the edges](the-shortcuts-and-the-edges.md)*:
its table gave *double-click its name* to *Fit the selected window in view*.
That pointer road was never built (`canvas_fit_the_window_in_front.rs` says so);
`⊞` + 2 still fits the window in view, and the double-click is now maximise, by
this specification.

### The window panel (`11:24`)

Heading **Window**, then:

- three placement choices drawn as pictures — **Left**, **Maximise** (which
  reads **Restore** while the window is maximised) and **Right**;
- **Full screen** (reads **Exit full screen** while it is);
- **Move to…** — the person's Places, and displays only when more than one is
  connected;
- **Help with this window** — asks alo, with this window as the context;
- **Capture & export** — opens export choices in the panel's place.

No promotional text and no duplicates: Minimise and Close are on the band and
are not repeated here. **A choice that does not work yet is not drawn** — not
greyed, not shown as coming soon.

**Left and Right** preview where the window would go when pointed at or reached
by keyboard (`11:84`: a 2-wide navy outline of the destination with *Left half ·
preview* in its corner). **The preview never moves or resizes the real
window.** Click or Enter commits; Escape or dismissing cancels. Other windows
are not touched. The destination is the display's available work area less what
the shell reserves, and the window's minimum size is respected; if it cannot
fit, the person is told rather than given a smaller half.

**Maximise / Restore** fills the work area and keeps the band and the shell's
areas; Restore returns the size and place it had before. **Full screen** fills
the display and hides the band and the shell's areas; **leaving it restores the
state before** — a maximised window comes back maximised. In full screen a
12-unit zone at the top of the display reveals the title and controls together
as an overlay that does not resize or push anything.

**Move to…** maps onto the existing canvas: each display is a view onto the
canvas, a Place is where windows live, and there is no second model of
workspaces. The current destination is marked. A moved window is never left off
every screen.

**Undo placement**: after a committed placement, a brief accessible **Undo**
that puts **this window** back where and how it was — never another window's
change and never the Place's whole history. Built only with its own per-window
history; *a Place remembers time* is a different, whole-Place series.

### alo working in the window (`11:146`)

*Help with this window* opens an alo request with the window identified;
opening it starts nothing. The existing permission levels apply, including full
access acting without the OK (ADR 0149); nothing already granted is asked again
and nothing is widened silently. **Hidden when AI is switched off.** While alo
works in the window, a teal band across its foot says what it is doing in plain
words, with **Stop** always available. Stop prevents further actions and says
what state things were left in; stopping is not undoing. Navy stays the colour
of the manual controls; teal never decorates an idle entry; colour is never the
only signal.

### Capture & export (`19:2`, `19:96`)

Choosing it replaces the panel's contents; the back arrow returns, keeping what
was chosen.

- **What to include:** *Visible window* (the default — the application's
  content only: no band, no panel, no tooltip, never another window covering
  it), *Select an area…* (the panel steps aside, the person drags a boundary
  inside the window or uses the keyboard, Escape returns), and **Full document /
  Full page only where the application provides it**, labelled for its content
  and never silently replaced by a screenshot. The *Supported* mark is per
  application, never hard-coded.
- **Format:** only those that scope supports — PNG, JPEG, PDF. A PDF of a
  screenshot says so (*PDF here saves a screenshot on a PDF page*); selectable
  text and layout are promised only where the application's own export does
  them.
- **Save to:** the folder shown (Downloads by default), **Change…** opens the
  chooser; an editable file name from the title, its extension following the
  format; invalid names and collisions through the normal save flow; **never
  overwritten silently**.
- **Copy image** (image scopes only) puts it on the clipboard with a brief,
  accessible confirmation. **Save image** / **Save PDF** saves to the folder
  shown without a second chooser, then offers **Open** and **Show in folder**;
  a failure keeps the choices and offers to try again; a slow export shows
  progress and can be cancelled.

## Panel behaviour and accessibility

Anchored to Window actions and kept on the display; on a small window, a large
scale or a short display it scrolls inside rather than being cut off. Presses
inside never reach the application. A click outside dismisses without applying
a preview. Escape closes the deepest layer first. Focus returns to the control
that opened it, starts somewhere safe, and moves with the arrow keys, Enter and
Space, always visibly. A screen reader hears names, roles, and the selected and
expanded states, and a placement preview has a spoken equivalent. Icon-only
controls have tooltips. Contrast holds in the active and inactive looks.

**Client-drawn title bars** keep the repository's ruling of 2026-10-09: the
application's own header stays, and alo's band adds **Window actions only** —
no second Minimise or Close. Its limits are named per application rather than
hidden.

## Measured, 2026-11-09

All logical units. `get_design_context` on `11:30` (the window), `11:51` (the
panel), `11:90` (the preview) and `11:173` (the working state); no variables
are bound in this file — `get_variable_defs` on `11:24` returns nothing — so
every colour below is a literal, mapped to the nearest of the twelve roles where
the role is what the code already draws with.

### The band

| | figure | node |
|---|---|---|
| band | **54** tall, the window's full width | `11:31` |
| title | x **22**, y **18**; Manrope SemiBold **14**, `#102a43` | `11:32` |
| application name | **24** after the title (x 160 after 114 wide); Regular **11**, `#829096` | `11:33` |
| controls group | **140 × 44** at y **5**, its trailing edge **18** in from the window's | `11:41` |
| each control | **44 × 44**, **4** apart | `11:42`, `11:45`, `11:49` |
| Window actions, revealed | a `#f0f1ed` square filling the 44 target, corner **8**, the mark in navy | `11:164` |
| Window actions, open | navy `#102a43` square, corner **8**, the mark in white | `11:42` |
| Minimise, Close | no square at rest; the mark in navy | `11:45`, `11:49` |
| the marks | stroke **1.4**, round caps and joins: Minimise `M17 22H27`, Close `M17 17L27 27M27 17L17 27`, layout `M23 16V28M16 16H28V28H16V16Z`, all in a 44 box | asset SVGs |
| window card | corner **16**, line 1 `#d9dfd9`, shadow `0 16 40` at 10 % of `#102a43`, surface `#fdfefc` | `11:30` |

**The controls are centred down the band** — `(54 − 44) / 2 = 5` — and counted
from the trailing edge: the last ends 18 in, each 4 after the one before. `1056`
is the specimen's width, not a window's.

### The panel

| | figure | node |
|---|---|---|
| panel | **348** wide (397 tall with every row), corner **16**, line 1 `#e1e4e0`, white, shadow as the window's | `11:51` |
| where | **8** below the band; its leading edge **168** before Window actions' (730 against 898) | `11:51`, `11:42` |
| heading *Window* | (22, 20), SemiBold 14, navy | `11:52` |
| placement choices | three **94 × 94**, **10** apart, from (22, 50); corner 10, `#f5f6f3` | `11:54`–`11:63` |
| inside each | a 46 × 30 outline at (24, 16), line 1 `#a8b4ba`, corner 4; the placement inside, corner 2 — Left 18 × 22 at (4, 4), Maximise 38 × 22, Right at (24, 4); label Medium 12, navy, at y 62 | `11:56`–`11:66` |
| placement colour | `#667d8d` for Left and Right, navy `#102a43` for Maximise as drawn | `11:57`, `11:61` |
| divider | 304 × 1 at y 160, `#e8eae6` | `11:67` |
| Full screen | 304 × 40 at y 172; the expand mark 12 × 12 at (2, 12); label Regular 13 at x 30 | `11:68` |
| Move to… | 304 × 40 at y 216; Regular 13; `›` at x 284 in `#7a878f` | `11:72` |
| divider | at y 267 | `11:75` |
| Help with this window | 304 × 52 at y 284, `#f4f7f5`, corner 9; *alo* SemiBold 15 at x 12, label Regular 13 at x 51 | `11:76` |
| Capture & export | Regular 12 `#647582` at y 356; `›` at x 302 | `11:79`, `11:80` |

### The preview, the working state and export

| | figure | node |
|---|---|---|
| placement preview | line **2**, navy, corner 15; *Left half · preview* Regular 12 `#647582`, 20 in and 30 up from the bottom-left | `11:90` |
| working band | 1016 × 48, 20 in from each side, 20 above the window's foot; `#e8f2ef`, corner 10; text Medium 13 teal `#0f6b72` at (16, 14) | `11:173` |
| Stop | 64 × 30 at (940, 9), white, corner 7; SemiBold 12 teal | `11:175` |
| export panel | 380 wide, its contents 336 from (22, 20); rows 36, formats three of 106.6 × 36, destination and file name 40, actions 132 and 196 × 42 | `19:59`, `19:153` |

### Readings taken, each for the owner to overrule

- **The application name is drawn in `text/muted` (`#596b78`), not `#829096`.**
  `#829096` on the band is about 3.3 : 1, under the 4.5 : 1 that 11-unit text
  needs, and the specification asks for contrast in both looks. It stays
  quieter than the title by size and weight.
- **Literal colours map to the twelve roles** where one is within a step:
  surface `#fdfefc` → `bg/surface`, Window actions' resting square `#f0f1ed` →
  `bg/cool`, lines `#d9dfd9`, `#e1e4e0` and `#e8eae6` →
  `border/default`, the placement tiles `#f5f6f3` → `bg/canvas`, the outline
  `#a8b4ba` and grey placement `#667d8d` → `text/muted`. *Shared tokens, not
  duplicated styling* is the specification's own rule.
- **The navy placement marks the choice in hand.** `11:24` and `11:84` both draw
  Maximise navy and the others grey, while `11:84` previews Left — so navy is not
  the hovered tile there. Read as *the choice the pointer or keyboard is on*,
  navy, and the others grey, because a picture that is navy for no reason a
  person can find is decoration.
- **The full-title road by touch.** The 2026-10-10 ruling put the whole title in
  the window menu so it could be read by touch. The panel has no title block,
  and **this shell receives no touch input at all** (no touch event is handled
  anywhere in `alo-shell`), so *controls visible for touch* and a touch road to
  the full title are both blocked on touch input existing. Named, not
  substituted.
- **Move and Resize rows leave.** The old window menu listed *Move* and
  *Resize*; pressing either did nothing (`carry_out_the_row` matched them to an
  empty arm). They are not in the panel. Moving is the band; resizing is the
  edges and corners (ADR 0071).

### Readings taken while building, each for the owner to overrule

- **The application's name is its desktop entry's `Name`, in the person's
  language** — `Name[fr]` and the rest, read in the order `GenericName` already
  is — from the one map the alo Bar shows, so the Bar and the band never call
  one application two things. A window of `org.gnome.TextEditor` says *Text
  Editor*. The `app_id` is shown only where no installed entry names the
  application, and a window with no title shows the name once, never its
  `app_id` beside it. *(Until the change after #1138 the band showed the
  `app_id`.)*
- **A double-click is two presses within 500 ms and 4 units**, the longest of
  the common desktop defaults; the design names none. It is never the only
  road: the panel's Maximise and `Super+Up` do the same.
- **The band out over full screen has a ground of its own**, drawn over the
  application's top in the band's look and line, with square corners because
  it runs edge to edge; a band at rest has its ground under its window, on the
  card. It **starts after the egress indicator**, which keeps the top-left
  corner over everything (law 1), so the title is never under it. **Choosing
  Full screen by key gives the keys back to the application** rather than to
  Window actions, because Window actions is what the choice hides; `Alt+F3` or
  the top edge brings the band back.
- **Tab is not taken inside the panel**: `crate::window_menu_keys` had already
  declined to decide the whole focus order there. The arrows, Home, End, Enter,
  Space and Escape walk it.
- **No band carries an icon**, alo's own windows (the terminal, the record)
  included, so every band is the one band the frames draw.
- **The work area includes the top controls' 84-unit band.** The shell
  reserves it whether or not anything is drawn in it (`crate::top_controls_region`),
  so a maximised window starts under it, with its band; a window that took that
  room would be under the top controls once they are drawn.
- **A display too short for a band and a window under it refuses Maximise**
  (`OutputUnavailable`), rather than maximising with the band off the screen.
- **Left and Right are not the division.** `Super+Left` and `Super+Right` ask
  `alo-dividing` to divide the window's share with the next window, which
  moves that one too; the panel's choices are promised to move no other
  window, so they give this window a share of its own, half the work area of
  the display it is on. **The half is still `alo-dividing`'s**: it answers one
  window alone on a side (`alo_dividing::alone_on`) with the cut the chords
  use, so both give the same half, and the odd unit is the right half's —
  there is one layout decider (`tests/one_layout_decider.rs`).
- **A window on a half reads Restore** in the middle picture, as a maximised
  one does: a window in a share cannot be dragged, so Restore is how it goes
  back to the size and place it had.
- **The refusal is said in the preview**, before anything is chosen, because
  choosing closes the panel: pointing at Left on a window too wide for half
  the screen shows *This window needs more room than half the screen* in the
  outline, and choosing it then changes nothing.
- **The preview's words sit on a small `bg/surface` ground**, 6 round, so
  they read over whatever window is under the outline; the frame draws them
  over an empty canvas.
- **Opening a panel does not bring its window forward**; the panel is drawn
  above every window instead, so it is never covered and a press on it never
  reaches what is behind.
- **Minimise's travel is an outline, 300 ms, eased in and out, to the
  window's own card.** The design says *travelling toward the panel* and
  names no duration, easing or path. The window is hidden at once, as it was,
  so nothing a person can press changes mid-travel (*the-windows-put-aside.md*:
  animations must not change hit targets midway through a pointer move). An
  outline of where it was — the cards' accent, the focus ring's weight — moves
  and shrinks onto the card the panel would draw for it revealed, on the
  panel's edge (left for a right-to-left reader), and is gone when it arrives.
  An outline and not the window's pixels, because the painter has no
  per-window transform. 300 ms and the curve are the view glide's
  (*where-a-new-window-opens.md* rule 4). Under reduced motion nothing travels,
  and turning it on mid-travel ends the travel; a window with no card (no room
  for the panel, a rail too short) is put aside without travelling; a client
  that minimises itself never travels; a window brought back mid-travel stops.
  `crate::a_window_travels_aside`, `crate::where_a_travelling_window_lands`.

## What is built

*Updated with each stage as it lands.* Until a row here names a pull request,
that part is **not built**.

| stage | what | state |
|---|---|---|
| 1a | the band (title and application name always, 54 tall), the three controls revealed on approach with the fade, *Always show window controls*, the panel with Maximise / Restore and Full screen / Exit full screen, right-click and double-click on the band, `Alt+F3` and the keys into the band and panel, expanded and selected states for a reader | #1138 |
| 1b | Maximise keeping the band: the work area of the window's display (through that display's camera, less the Dock's band, the panel's column, the top controls and the indicator), the band at its top; full screen filling that display; a restore on the Place already shown leaves the view where it is while the window is in sight | #1141 |
| 1c | Minimise travelling to the side panel (the outline above); restoring was already returning a window to its Place and its size, and now has a test that drives it from the panel | #1144 |
| 1d | Left and Right with the preview: this window alone on half its display's work area under its band, the `11:84` outline and its words drawn under the panel while either is in hand, the same words for a reader, *This window needs more room than half the screen* where its minimum does not fit | this change |
| 1e | the full-screen top-edge reveal | not built |
| 2 | Move to…, Undo placement | not built |
| 3 | Help with this window, the working state and Stop | not built |
| 4 | Capture & export | not built |
