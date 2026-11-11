# The alo Bar when it is open

## Round 3: typing in the Dock's own field — a departure from `561:275` / `561:370`

**The owner's decision of 2026-11-11**, in their words: *"Clicking 'Find, open,
or ask…' on the Dock should not open the same panel as Super+Space. It should
allow user to text inside it"* — then, of the matches, **rise just above the
field** (*"a small list grows upward from the Dock field as you type — same
matches and 'Ask alo' row, anchored to the field, nothing in the middle of the
screen; Enter opens the highlighted one"*), and of Super+Space, **the same as
clicking the field** (*"Super+Space puts the cursor in the Dock field too — one
place to type, by mouse or keyboard"*).

**So a click on the field, Super+Space and Super+A no longer open the centred
panel of Round 2.** The design file has no frame for this yet; the rules below
are taken from what it does have, and each is for the owner to overrule.

| | the rule | taken from |
|---|---|---|
| the field | the Dock's composer, unchanged — the typed words replace the prompt and the key in `TEXT_AT_ORDINARY`, the **end** of them when they are wider than the field, and a 2-wide caret after them while it has the keyboard; with nothing typed, the prompt stays and the caret sits just before it | the Dock's own field |
| the list | **448 wide** (`More apps`), starting where the field starts; **12 from the bar** (`BESIDE_THE_DOCK`); air inside **8** (`GAP`) | *More apps*' placement, ADR 0102/0103 |
| where | above a bottom Dock, below a top one, inward from a side one; inside the display and clear of the put-aside panel's column | `docs/features.md`'s *More apps* line, now one function for both |
| rows | Round 2's rows exactly: an application's 56, the hairline, the row that asks's 64, 4 between, the chosen row in the accent at 12 % | `561:324`, `561:339` |
| how tall | what it holds, up to **three fifths** of the display and never past the room between the bar and the far edge; the rows scroll from there, keeping the chosen one in view; the row that asks is never scrolled away | Round 2's own bound |
| panel | white, 1 `border/default`, corner 18, `Elevation/Floating` — Round 2's panel | `561:276` |
| behind it | **nothing dimmed**: the list is non-modal, as *More apps* is | *More apps* |

**Readings taken where the decision is silent:**

- **A side Dock's field is a 48-wide slot** and cannot hold a line of text, so on
  the left and right Docks the list opens the moment the field takes the
  keyboard, with what is typed on its first row in Round 2's field style (the
  cool well, the 2 outline, the search mark, the caret).
- **Nothing typed on a bottom or top Dock is no list** — Round 2's own rule, so a
  chord does not flash every application on the machine.
- **A press outside the list closes it and does the thing it landed on**, which is
  what every list opened from the Dock does; the words stay in the field, as they
  do when a person clicks a window. Round 2's *a press on the dimmed screen is
  the panel's* does not carry over, because there is no dimmed screen.
- **Escape empties the field and gives the keyboard back** in one press, as it
  did in Round 2.
- **Over a full-screen window the Dock is shown** while its field has the
  keyboard, since that is where the words are.
- **A screen reader** meets the field as the text entry it already was, with the
  matches as a list under it, the chosen row *selected* — which Round 2 noted as
  missing.

**The centred panel of Round 2 is not removed.** *Help with this window* still
opens it, headed with that name; another change is moving that road into the
window's own panel, and the panel can go when nothing opens it.

## Round 2: the launcher the owner approved on 2026-11-01

**This section supersedes the tables below where they disagree.** `561:275`
(1440 × 960) and `561:370` (1280 × 800), *Find, open, or ask · the launcher*,
read off the live file on 2026-11-01.

| | 1440 | 1280 | the rule |
|---|---|---|---|
| panel | 680 at (380, 208), **360 tall** | 604 at (338, 174), **357 tall** | 47.2 % of the width, top at 21.7 %; **its height holds what it shows**, up to three fifths of the display, and the list scrolls from there |
| air inside | 26 | 23 | the panel's width over 26 |
| heading | 40, title Manrope SemiBold 22 / 30 at +5 | the same | a measure |
| Close | 48 × 40, `bg/cool`, corner 12, a 16 cross in `text/primary` | the same | a measure |
| field | 56, `bg/cool`, corner 12, a **2 `text/primary` outline**; search mark 20 at (18, 18) in `text/secondary`; typed text Regular 16 / 25 at 50 | the same | a measure |
| line under it | 12 below, SemiBold 13 / 18, `text/muted`, **on the panel's white** | the same | |
| an application's row | 12 below, 56 tall, corner 12; chosen: the accent at **12 %**; icon tile 32 at (12, 12), corner 8; name SemiBold 14 / 22 at 56; *· kind* Regular 14 `text/muted` 6 after; chosen: *Open* SemiBold 11 and an Enter key cap 26 × 18 ending 14 from the row's end | the same | rows 4 apart |
| hairline | 4 below the last row, 1, `border/default` | the same | |
| the row that asks | 4 below, **64** tall; alo's mark on an `accent/soft` tile 32 at (12, 16); *Ask alo "…"* SemiBold 14 and *· a new task* Regular 14 at (56, 12); a cloud 14 and where the answer would come from, SemiBold 11 `text/muted`, at (56, 36) | the same | |
| keys | 12 below, 28: ↑ ↓ caps *choose*, ↵ *open*, Esc *close*; caps white, a 1 `border/strong` line, corner 6 | the same | |
| under the keys | 15 | 15 | a measure, not a share |
| backdrop | `bg/canvas` at **45 %** over the whole screen, blur 4 | the same | |
| panel | white, 1 `border/default`, corner 18, `Elevation/Floating` | the same | |

**What was wrong, and is put right.** The grey band: every line on the panel was
inked on the field's grey and pasted onto the white panel, so the line under
the field sat in a grey rectangle. Words are now shaped on the ground they are
drawn on. **No heading, no Close, no backdrop, no key hints, and no way to
choose a row with the arrows** — all five are drawn and reached now.

**Readings taken, each for the owner to overrule:**

- **The blur is not drawn**, only the dimming. Nothing in this shell reads back
  what is behind a surface to blur it; that is a renderer change of its own.
- **Where the answer would come from is `alo-models`' own sentence** — the one a
  task window shows (*by openai, which has not said where it runs*), not the
  frame's *Answered by …*. Two sentences about where a person's words go is the
  duplication law 1 cannot afford; changing the shared sentence changes every
  surface at once, which is the owner's call.
- **A row's kind** is *Application*, followed by what the application says it is
  (its entry's `GenericName`, in its own words): *Application · File Manager*
  where the frame draws *Application · Files*.
- **An application this machine has no icon for** shows the cool tile empty, as
  the Dock's fallback ground, with the name beside it.
- **A press anywhere on the dimmed screen closes the launcher** as Escape does,
  and reaches nothing behind it; a press on a row does what Enter does on it.
- **No screen reader is told the launcher's rows yet**: `alo_access::Surface`
  has no surface for it, as it had none before. That is its own change.

**Read off `alo Bar / Open` (`81:890`), inside `09 · Find, open, or ask`
(`78:1671`), on 2026-10-20.** Every number here was taken off the frame; none
was matched by eye.

The frame is 1440 × 960, which is the design's reference size and **not a
screen anybody has**. So the table below gives each number twice: as the frame
draws it, and as the rule a layout follows — `CLAUDE.md`: *a figure taken from a
frame becomes a proportion or a named rule, never a constant that holds only at
the size somebody drew.*

## What it is

A panel **over the canvas**, not the Dock's field growing upward. The Dock
stays where it is; this opens in front of it with a backdrop behind
(`Search backdrop`, a sibling frame).

## The numbers, and the rule each becomes

| | at 1440 × 960 | the rule |
|---|---|---|
| panel | 680 × 390 at (380, 208) | **47.2% of width**, centred across; top at **21.7% of height** |
| inset, all round | 25 | panel width ÷ 27.2 — one measure, used on every side |
| heading | 22 px, at y 30 | **5.6% of panel height** |
| close control | 48 × 40, radius 12, at the heading's right | a target, flush to the inset |
| field | 632 × 56, radius 12, at y 77 | panel width less two insets; **a target's height** |
| what is typed | 16 px, inset 16 into the field | |
| the line beneath | 13 px, at y 145 | one line, a **gap of 12** under the field |
| a row | 632 × 56, radius 12 | the field's width and height exactly |
| row to row | 68 | a row plus **12** |
| first row | y 175 | a **gap of 30** under the line |

**The rows are 632 × 56 — the same box as the field.** That is the design
saying a row is a thing you act on, drawn at the size of the thing you type
into, and it is why a row's height is a named target rather than a number of
its own.

## The one number that is not a size

**Four rows are drawn and the panel is 390 tall.** The fourth sits at y 379, so
its bottom is 435 — past the panel. That is deliberate: **the list scrolls**,
and the design shows it clipped rather than pretending a result count fits.
Nothing should cap the number of results to what is visible.

## What a row says

`Name · where it is` — *Launch strategy · Docs*, *Launch website · Browser*,
*Opening scene · Blender*. **The second half is the kind of thing**, so an
application and a document are told apart at a glance. The separator is a
middle dot with two spaces each side, the same on every row.

The line under the field is *On this computer · 3 open windows* — **where the
results came from, and how many.**

## What this means for the installer's screens

Nothing. They are a separate surface with their own frames
(`530:30834`) and their own derivation. This file is the canvas's Bar.

## What is not here

**Which rows a search finds, and in what order.** That is
`alo_shell::the_bar_finds`, which decides and says, and has no geometry in it
at all.

**The keyboard.** How a row is reached, what Enter does on it and what Escape
closes are `docs/design/the-shortcuts-and-the-edges.md`'s subject, and the
frame shows no focus state — so the ring is `a_focus_ring`'s rule, as it is
everywhere else on the Dock.
