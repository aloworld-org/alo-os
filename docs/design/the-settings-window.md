# The Settings window, in numbers

**What this is.** The figures measured off the owner's approved redesign of
Settings, so the window built here can be checked against the design rather
than matched by eye. Read live on 2026-10-31 from file `nDxyF5Ho9oC4RObjVzwBNJ`,
page `544:30910` *Proposed · Settings and the alo window · 2026-10-31*, which
the owner approved the same day.

**Two reads, because one of them is blind** — the same split
[the-ask-alo-task-window.md](the-ask-alo-task-window.md) records. The tree is
in `figma-snapshot/544-30910.xml`, from `get_metadata`: geometry and layer names
only. Everything about appearance below — fills, strokes, type, shadows — came
from `get_design_context` on each frame, the same day. The SVG artwork came from
`download_assets` on each icon's group, and is in
`crates/alo-shell/icons/settings/` with the node each came from.

**1440 × 960 is a frame, not a machine.** Every figure below is a logical
measure declared once in `crates/alo-shell/src/settings_measures.rs`, a
proportion, or a rule — never a constant that holds only at the size somebody
drew. Each goes through the person's text size **and** the display's scale, so
larger text makes the controls larger with it, which is the Accessibility
page's own footnote.

## The frames

| Frame | Node | What it shows |
|---|---|---|
| alo and AI | `550:97` | 1440 × 960, light, Indigo |
| alo and AI, scrolled to the end | `550:258` | the page scrolled under the window's top edge |
| alo and AI, 1280 × 800 | `550:419` | the same window on a smaller screen |
| alo and AI, dark | `550:580` | the dark values |
| Folders alo can use | `550:741` | a folder just given, two on the list |
| Running code | `550:874` light, `550:1025` dark | the three levels and what stays on |
| Terminal | `551:98` | two switches, one on |
| Appearance | `551:214`; `551:363` at 1280 × 800; `551:512` Moss | light and dark, the accent, the Dock |
| Region and time | `551:661`; `551:751` choosing the country | two pickers, and the country picker open |
| Keyboard and shortcuts | `551:870` | every shortcut with its keys |
| Accessibility | `551:1038` | text size and pointer size |

## The window is a rule of the display

| | 1440 × 960 (`550:98`) | 1280 × 800 (`550:420`) |
|---|---|---|
| window | 1200 × 816 at (120, 72) | 1152 × 688 at (64, 56) |
| side column | 280, its edge included | 269 |
| page padding across / above | 64 / 48 | 48 / 36 |
| the column | 720 | 720 |

**One rule draws both** (`crates/alo-shell/src/settings_frame.rs`, and a test
holds it to both frames to the unit):

- the window is **at most 1200 × 816**, and never closer to the screen's edge
  than **64 across and 56 down** — which is what the smaller frame leaves;
- it is **centred** in what is left;
- the side column is **seven thirtieths of the window's inside**, rounded up:
  1198 × 7 / 30 is 279.5 and the design draws 280; 1150 × 7 / 30 is 268.3 and
  it draws 269. Two frames, one proportion;
- the page's padding is **64 / 48 while the window is at its full width and
  48 / 36 once it is not**. This is the one difference between the two frames
  that no proportion explains, so it is written as the rule it is rather than
  as a scale;
- the page's column is **at most 720**, narrower when the window is.

**A screen with no room for a 240 column is refused** rather than drawn
cramped (`RenderError::SettingsScene`), which is the old window's rule for a
frame too small to hold its focused row.

## The window

- Surface `bg/surface`, border 1 `border/default`, radius `radius/lg` 18,
  shadow `Elevation/Floating` — `#07131F` at 14 %, falling 16, blurring 40,
  spreading −4. The same effect the task window's popovers carry, and drawn by
  the same figures (`HowItIsRaised::FLOATING`).
- The side column is `bg/canvas` with a 1 `border/default` edge on its right.
- The page is `bg/surface` and **scrolls**: frame `550:258` shows its rows
  passing under the window's top edge, cut there, and the last card 48 above the
  window's foot — the page's padding is kept below it as well as above.

## The side column

Padding 16, and 2 between every item.

| Item | Measure | Type |
|---|---|---|
| Heading | padding 12, 4 between the lines | *Settings* Heading/Medium 22 / 30, `text/primary`; the line under it Body/Medium 14 / 22, `text/muted` |
| Group | 14 above, 6 below, 12 across | Label/Small 11 / 16, `text/muted` |
| Category | 48 tall, 12 across, icon 20, 12 to the name, radius 10 | Body/Medium 14 / 22, `text/secondary` |
| Chosen category | the accent at **12 %** over `bg/canvas` (`rgba(58,90,168,0.12)` is Indigo's), icon in the accent | Body/Medium **SemiBold**, `text/primary` |
| Footer | 12 across, 8 below, at the foot | Label/Small, `text/muted` |

**Dark draws the chosen category at 22 %** — `rgba(138,160,230,0.22)` in
`550:588`, Indigo's dark value — and that is recorded here for the change that
builds dark.

## A page

Heading/Large 28 / 36 `text/primary`, 8, then the introduction Body/Large
16 / 25 `text/secondary`. Then the page's blocks, with a gap between each that
**three figures disagree about**: alo and AI draws **32**, Appearance and
Keyboard **28**, and the other five **36**. Each page is drawn as its own frame
draws it (`settings_measures::between_blocks_on`), and the disagreement is the
designer's to settle — recorded rather than tidied, because picking one would
be deciding something the design did not.

### A section

A heading (4 across, Label/Medium 13 / 18, `text/secondary`, sentence case),
10, the card, 10, and a footnote (4 across, Body/Medium, `text/muted`).

**The card**: `bg/surface`, border 1 `border/default`, radius 12. Its rows are
**at least 64** tall, 20 across and 16 down inside, 16 between the mark, the
words and the control; a hairline `border/default` above every row but the
first. Three sums that had to agree, and did: in `550:419` the rows are at y 1,
81, 162, 243, 358, 451 and 507 inside a card 589 tall — each row the one
before's bottom plus its own top hairline, and the card's own border at each
end.

### The row kinds

| Kind | Node | What is drawn |
|---|---|---|
| A choice | `550:170`, `550:184` | ring 22: not chosen, 1.5 at radius 9.25 in `text/muted` on `bg/surface`; chosen, 2 at radius 9 in the accent with a dot of radius 5 |
| A choice under another | `550:192`, `550:208` | starts 58 in, 12 above and below, 12 to its words; ring 18 at 7.25 / 7 with a dot of 4 |
| Name and line | — | name Body/Medium SemiBold 14 / 22 `text/primary`; 4; the line Body/Medium `text/muted` |
| A note | `550:199` | 4 below; icon 14, 6, Label/Small — `status/warning` `#8C5A0A` when it asks for attention |
| A switch | `550:256` off, `551:196` on | its word (Label/Medium `text/muted`), 10, a track 44 × 26: off is `bg/cool` with a 1.5 `text/muted` edge and a knob of radius 10 at the start in `text/muted`; on is the accent with the knob in `bg/surface` at the end |
| What it risks | `550:246` | `bg/canvas`, radius 10, 12 across, 10 down; warning sign 18, 10; *What it risks* Label/Medium `status/warning`, 2, the sentence Body/Medium `text/primary` |
| Adding | `550:219` | at least 56; a plus 16 in an 18 box where the ring would be; a chevron 20 at the end |
| A Quiet button | `550:205` | 48 tall, radius 12, border `border/default`, Label/Medium `text/primary` |

**A Quiet button's width.** The frames set each button's width by hand — 174
for *Say where it runs*, 115 for *Take back*, 176 for *Give alo a folder*. The
window draws **its label plus 26 each side**, which comes within a few pixels of
all three; recorded as the rule chosen, because three hand-set widths cannot
follow a translation.

## Colours

| Token | Light | Dark (`550:580`) |
|---|---|---|
| `bg/surface` | white | `#102A43` |
| `bg/canvas` | `#FAF7F2` | `#07131F` |
| `bg/cool` | `#EEF2F4` | `#183B56` |
| `border/default` | `#E7EBEF` | `#274C68` |
| `text/primary` | `#102A43` | `#FFFCF8` |
| `text/secondary` | `#274C68` | `#CBD5E1` |
| `text/muted` | `#596B78` | `#A4B4C2` |
| `status/warning` | `#8C5A0A` | `#E8AD50` |
| `status/positive` | `#146C43` | — not read |
| Indigo, the accent | `#3A5AA8` | `#8AA0E6` |

**The dark column is read from one node and is mode-coherent there**:
`get_variable_defs` on `550:580` returned every token in its dark value. That
call is known to mix modes inside one answer on some nodes
(`figma-snapshot/variables.toml` records it), so the column is read from the
dark frame's root and from nothing else. The light values agree with
`alo_appearance::Role` to the byte.

**Selections and switches are the person's accent** (the owner, 2026-10-31):
a chosen ring, a switch that is on, the chosen category's ground. The accents
are `alo_appearance::Accent::ALL` — Indigo, Violet, Moss, Rose — whose light and
dark values are exactly the ones the frames draw (`551:317` Violet is
`#7A4E99`). **The keyboard's outline is not the accent**: it is `text/primary`,
as the task window's is, so where the keyboard is reads the same whatever
colour a person chose.

## Type

All Manrope. Heading/Large 28 / 36 SemiBold; Heading/Medium 22 / 30 SemiBold;
Body/Large 16 / 25 Regular; Body/Medium 14 / 22 Regular, and SemiBold for a
row's name and the chosen category; Label/Medium 13 / 18 SemiBold; Label/Small
11 / 16 SemiBold; System/Mono 12 / 18 Geist Mono Medium for a file's name. A
picker's value is **Manrope Medium 14 / 22**, and the chosen accent's name
**Bold**, which match no named style and are recorded as they are.

**Letter-spacing is not drawn**: −0.4 on Heading/Large to +0.3 on Label/Small.
Nothing in this crate's shaper takes tracking, and that is a departure rather
than a measurement.

## Words

**Sentence case**, the owner's decision of 2026-10-31. The four ways and the
three levels of running code are their own crates' words, written in lower case
because setup draws them inside a sentence; the window draws them as names
with a capital on the first letter (`settings_type::as_a_name`) rather than
changing the words setup uses. **The frames end each line with a full stop and
those crates' words do not**, which is the one visible difference that leaves.

## The keyboard

Not drawn in the design; decided here and written down so it can be disagreed
with:

- The window opens on **alo and AI with the keyboard in the side column**.
  **Up and Down** choose the page — the page follows the keyboard — **Home and
  End** the first and last, and **Tab, Enter or Space** go into the page.
- In the page, **Up and Down** move through every row and button in reading
  order, **Home and End** go to the first and last — **End reaches the last row
  of the page**, which a walk relies on — **Tab** goes to the next section and
  from the last back to the side column, **Shift+Tab** the other way, and
  **Enter or Space** act.
- **Escape** gives up a folder being picked and leaves Settings open, as
  before; otherwise it closes Settings, from either column.
- **A walk that gave alo a folder by pressing End in the old one-list window
  now presses Down once** (to *Folders alo can use*), **Enter** (into the page)
  and **Enter** on *Give alo a folder*, which is the first row there while
  nothing is given.

## What is drawn and not built

The design draws choices this computer does not have yet. **They are drawn as
the design draws them, and choosing one says in words that nothing changed**
(`the_settings_words::NOT_ON_THIS_COMPUTER_YET`). None of them pretends.

- **alo's own service**, under the providers (`550:207`). ADR 0106; nothing on
  this computer can reach it yet. *Answered by alo, using Mistral, in France*,
  the line under it in the frame, **is not drawn**: it is a statement of fact
  about a service that does not exist yet, and this window draws no fact it
  cannot check.
- **Say where it runs** (`550:205`). A provider's stated region is kept
  (`alo_models::Region`) and drawn — *Has not said where it runs* in the
  warning colour, or *Says it runs in …* — but there is no form to state it from
  Settings yet.

## What the export does not carry, and what is still to build

1. **Focus, hover and pressed states** are not in the frames. The keyboard's
   outline above is this repository's existing rule, not a measurement.
2. **A provider row's line in the frame** says *key kept in this computer's
   keyring*; the window says that only when the provider's key reference is
   set, which is what the settings know.
3. **The scroll position is not stored**: the page is scrolled as little as
   keeps the keyboard's row whole and its section's heading above it where
   both fit, recomputed each frame. The frames have no scroll bar and none is
   drawn.
4. **Built in later changes**, each with its own record: the icons beside the
   categories and on the rows (waiting on the vector renderer the task window's
   reply box is adding); the other seven pages to their frames — light and dark
   and the Dock's edge as segmented choices, text size, pointer size, keys as
   keycaps, the lead icons and *Always on*; the country picker with search,
   which replaces the 250-row list; and dark.

## Questions for the owner

Each was decided one way to build this, and is listed so it can be answered:

1. **A person who has never said whether alo may use another model** is drawn
   with the switch's word *Not set*, not *Off*. The frame draws *Off*; but such
   a machine answers where it can and says so (`docs/features.md`, 2026-10-18),
   which is not what *Off* says. Enter on *Not set* turns it on.
2. **Computers paired with this one** are drawn on *Folders alo can use* under a
   heading of their own. They were in the old window's one list of things
   granted, and the redesign draws them nowhere.
3. **The sealed box is not chosen for a person who has never chosen**, though
   the frame draws it chosen; `crate::how_code_runs_in_settings` holds why
   *nobody has chosen* is a state of its own.
4. **The four ways are drawn always**, as the frames draw them, where the old
   window left out a way with nothing under it. Choosing an empty way says why
   there is nothing there and changes nothing.
5. **The gap between a page's blocks** is 32, 28 or 36 depending on the frame.
