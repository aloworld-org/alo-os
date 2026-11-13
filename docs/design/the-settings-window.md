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
| Terminal | `551:98` | two switches, one on — a third built since; see *Built and not drawn* |
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

### Where the side column does not fit, it scrolls

The design draws the column at 1440 × 960, where every entry fits with the
footer at its foot. It does not say what happens on a shorter screen, and with a
tenth entry the column needs about 46 more pixels than a 1280 × 800 window
leaves it. **The owner, 2026-11-13: *"The side column scrolls"*** — asked with
pictures, against hiding an entry or drawing the entries smaller. So:

- **Nothing in it is hidden or made smaller.** Every entry keeps its 48, its
  icon and its words at every screen size and text size.
- **Where everything fits, it is drawn exactly as before**, footer at the foot.
  Where it does not, the footer follows the last entry, because a footer pinned
  at the foot would be drawn over an entry.
- **A wheel over it moves it**, one notch an entry and the gap after it, and
  never past either end. While it scrolls, the wheel is the column's and never
  pans the canvas behind it, even at an end; over a column that fits, the wheel
  pans the canvas as it always did.
- **Up, Down, Home, End, *Go to* and a press bring the chosen entry whole into
  view**, scrolling as little as they must, from wherever the wheel left it. A
  wheel leaves it where it put it, even with the chosen entry out of view, so a
  person reading down the list is not pulled back.
- **An entry scrolled under the window's edge cannot be pressed there**: only
  what is seen of it is a target. A screen reader is told every entry whatever
  is scrolled, because it reads the list, not the picture.

`crate::settings_drawn_side_column` and `crate::settings_side_column_scrolls`
in `alo-shell`; held by their tests at 1280 × 800 and 1440 × 960, at one and at
twice the density, and down the pointer road in
`a_wheel_reaches_settings_tests.rs`.

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
  **Enter or Space** act. **Left and Right** make the text size smaller and
  larger on Accessibility, **move along a row of choices**, and do nothing
  anywhere else.
- **A row of choices** — a segmented choice (*Pointer size*, *Pointer shape*,
  *Pointer colour*, *Light and dark*, *Where the Dock sits*) or the accent's
  colours — is walked by **Left and Right** as well as by Down and Up. The
  owner, 2026-11-08: *"Yes, also Left/Right"*, asked after a walk pressed Right
  on *Pointer size* and nothing moved. Down and Enter work as before, so nothing
  is taken away. Left and Right **stop at the row's first and last choice** —
  Down is the key that leaves a row — and **move without choosing**: Enter or
  Space chooses, as it does after Down. That reading of *"move between the
  options"* is this lane's. Left goes towards the row's first choice; every
  machine reads left to right today, and a right-to-left reading would want it
  the other way round, which nothing builds yet — the text size's Left and Right
  have the same gap.
- **A list** — the countries, the time zones — has the keyboard while it is
  open: letters type into its search, Backspace takes one back, Up, Down, Home
  and End move, Enter chooses and closes it, and Escape closes it and leaves
  Settings open.
- **Escape** gives up a folder being picked and leaves Settings open, as
  before; otherwise it closes Settings, from either column.
- **A walk that gave alo a folder by pressing End in the old one-list window
  now presses Down once** (to *Folders alo can use*), **Enter** (into the page)
  and **Enter** on *Give alo a folder*, which is the first row there while
  nothing is given.

## On the canvas, with the pointer — built 2026-11-01

[ADR 0145](../decisions/0145-alos-own-windows-live-on-the-canvas.md), stage 3:
Settings is a window on the canvas like an application's.

- **Its size is still the rule above**, for the display it is on and the
  person's text size — so a larger text size still makes it larger. **Its place
  is the canvas's**: the first frame after it opens puts it where a new window
  goes (`where-a-new-window-opens.md`) on the Place being looked at, and there
  it stays. It pans and zooms with the canvas; on another Place it is not drawn,
  not pressed and takes no keys; *Show all* and Super+2 count it; the chord, the
  Dock and Super+Tab bring it in front and take the view to it (its Place
  first) without moving it.
- **It is drawn among the windows**, under the Dock and every fixed control, in
  the order the person put the windows in. The egress indicator stays above
  everything at the top left.
- **It takes the keys while it is the window in front** — opening it brings it
  in front, so a person who opens it types into it exactly as before; one put
  behind an application gives that application the keys. A chord with Control,
  Alt or the logo key, which means nothing to Settings, now reaches its action
  from inside it, so Super+Tab leaves it — except while Settings waits for a
  chord to bind, when it is Settings'.
- **A press reaches it.** A category shows its page with the keyboard on it, as
  Up and Down do; a row or a list entry takes the keyboard and is **chosen as
  Enter chooses it**, by the steps a key takes (`a_key_reaches_settings.rs`), so
  a click and a key cannot disagree about a row; a press beside an open list
  closes it, as Escape does. Presses are found in the positions each frame
  already recorded — categories, rows in view, list entries — carried back
  through the camera, so what is pressed is what is seen at any zoom. **While a
  provider is being filled in** the page is the form, which is the keyboard's;
  a press there brings Settings in front and does nothing else.
- **It wears the window edge** every one of alo's windows wears
  (`an_alo_windows_edge.rs`): *Settings* and **Close**, in the focused look
  while it has the keys. Close shuts it as Escape does, giving up a provider
  form that is open; nothing is written by closing it. *Readings, for the owner
  to overrule:* **the edge has no icon** — the design draws none for Settings,
  and the grip's mark stands in its place as for an application whose icon this
  machine does not know — and **no Maximise**, which nothing can do to Settings
  yet. *Minimise was added on 2026-11-11 with design `27:2`: Settings goes into
  the right-hand panel, keeping what was typed in it
  (`docs/design/the-windows-put-aside.md`).*

## Built and not drawn: letting alo act without asking

*The owner's decision of 2026-11-06, added after the frames were drawn.* The
Terminal frame (`551:98`) has two switches; the page now has a third, **Let alo
act without asking**, under them in *In the terminal*. The design is silent on
it, so it is made of two things the design already has, with their measures:

- **the switch row** of `551:168` — name, the line under it, the switch and its
  word at the end; and
- **the *What it risks* box** of the Running code page (`550:874`), which is
  where this design puts a cost in plain words. It holds the owner's sentence,
  *alo will send, change and delete what you ask for without showing you first*,
  and is drawn **whichever way the switch is**, so it is read before switching
  on rather than after.

The page's line under its heading, *Two things alo can do in your terminal*,
reads *What alo can do in your terminal* for the same reason. **Departures, and
recorded here for the next person to take to the design.**

## Built and not drawn: where alo works in an application

*The owner's answer A of 2026-11-10 (ADR 0143 Q6), decided after the frames
were drawn.* **Folders alo can use** ends with a section **In applications**:
two choices, *alo works in its own copy of an application* (tagged *Default*,
chosen as a machine arrives) and *alo may work in your own window*, with the
*What it risks* box under the second in ADR 0143's words. The design is silent,
so it is built from the Running code page's pieces (`550:874`) — a choice per
answer and the risk box drawn whichever is chosen. It is on this page because
ADR 0143 Q10 puts letting alo use an application beside *Give alo a folder*,
and this is a question about what alo reaches once it is let in. **A departure,
recorded here for the next person to take to the design.**

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
   **Since 2026-11-01 it is drawn on the edge of what it marks**, two thick in
   navy, rather than two units outside it: standing off, it was eight units
   wider than the chosen category's pill and reached the side column's edge — an
   oversized halo on the first thing a person sees when Settings opens, which
   the frames do not draw. Waiting for a chord is still a doubled outline, the
   second one ring's width inside the first.
2. **A provider row's line in the frame** says *key kept in this computer's
   keyring*; the window says that only when the provider's key reference is
   set, which is what the settings know.
3. **The scroll position is not stored**: the page is scrolled as little as
   keeps the keyboard's row whole and its section's heading above it where
   both fit, recomputed each frame. The frames have no scroll bar and none is
   drawn.
4. **The icons** are drawn from the committed SVG by `alo_vector_shapes`, the
   renderer the reply box brought in (#1040).
5. **The chosen part of a segmented choice has no shadow here**: the frames
   give it `Elevation/Quiet` (0, 8, 24, 8 %), and a shadow under a part inside
   a box is not drawn yet.
6. **The chosen accent's name is SemiBold, not Bold**: this machine bundles
   Manrope at Regular and SemiBold only, and a Bold asked for is drawn in
   whatever face answers.
7. **The time zone is named by its city**, as `alo_formats` names it; the
   frame's *Central European Time* is a name nothing on this machine holds, and
   its line *It is 14:32 here now* is a clock this window does not draw.
8. **The text size slider runs 75 to 300 per cent**, `alo_appearance`'s range,
   so 100 % sits near its start; the frame draws it a third of the way along.
9. **A screen reader is told each page as it is drawn** (since 2026-11-12, the
   owner's *"1. Live, keep names"*): the side column's pages as tabs, the
   page's heading and its sections' at levels 1 and 2, every row by its drawn
   name with its lines as description, which choice is chosen and its place
   among its set, an open list's entries, and the provider form's fields —
   from `crate::settings_page::the_page`, the list the drawing reads
   (`crates/alo-shell/src/settings_as_read.rs`). *Until then the tree was
   `alo_access::Surface::Settings`'s fixed list and static.* Not verified with
   Orca.
10. **In the form that adds a provider, the field with the keyboard is marked
    only by its caret.** The old window drew a filled square before it; the
    page draws a form's fields as plain rows, which have none, and no frame
    draws the form. Lost in the first stage of this redesign and found when
    the old window's line-builder was deleted in the fourth.

## Dark

From `550:580` and `550:1025`, read with `get_variable_defs` on each frame's
root: `bg/surface` #102A43, `bg/canvas` #07131F, `bg/cool` #183B56,
`border/default` #274C68, `text/primary` #FFFCF8, `text/secondary` #CBD5E1,
`text/muted` #A4B4C2, `status/warning` #E8AD50, `status/positive` #7ED3A6, and
the chosen category the accent at 22 % (`550:588`). Each accent is drawn in its
dark value. High contrast keeps its own palette in either scheme.

## The list a picker opens

`551:841`: 360 across and at most 480 down, `bg/surface`, radius 18, the
floating shadow, 8 inside and 2 between its items; its right edge on its
button's and 8 under it. The search is 48 tall in `bg/cool`, radius 12, with a
2 `text/primary` outline while it has the keyboard, its sign 18 at 12 in and 10
before the words. A heading — *3 matches*, counted the way the language counts,
or *All countries and regions* — is 12 above and 4 below, Label/Medium
`text/secondary`. An entry is 48 tall, 12 inside across, radius 10; the chosen
one on the accent at 12 % over `bg/surface`, SemiBold, with a check of 18 in the
accent at its end.

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
6. **The pointer's size is built** (`551:1136`), since 2026-11-06: *Large* is
   half as large again and *Larger* twice as large, which the frame does not
   say. **Its colour is built too**, since the owner's yes of 2026-11-06 to
   three high-contrast choices rather than the accents: *Navy*, which ships;
   *White* with a navy edge, for dark backgrounds; and *Black* with a white edge.
   **Never deep teal**, which is the agent's, and a test holds every choice to
   that. No frame draws it either, so *Pointer colour* is a third row in the
   same section, with the same segmented choice.
7. **The pointer's shape is a second row in the Pointer section**, *Pointer
   shape*, with the design's own segmented choice: *Kite*, which ships, and
   *Classic*, the familiar arrow. `docs/features.md` promises *the familiar arrow
   stays a choice in Settings* (the owner, 2026-11-06); `551:1038` draws only
   *Pointer size* here, so where the choice sits is this lane's, built
   2026-11-06. It is kept in `appearance.toml` and offered only where that file
   was read.8. **"When there's no room for a new window" is a section of the Appearance
   page, under the Windows section**, not a page of its own. The owner's words
   of 2026-11-07 are *Settings → Windows*; there is no Windows page, and the
   Appearance page already has a section headed *Windows* (the title switch),
   so this lane read *Windows* as that heading. The new section sits directly
   beneath it, with the design's own choice rows (`550:170`, `550:184`): a
   ring, the name, and a line under it. *Glide to empty space* is first and
   chosen on a machine nobody has changed; *Open in front, slightly offset* is
   second. No frame draws either. It is kept in `appearance.toml` as
   `when_there_is_no_room`, beside `window_titles`, and offered only where that
   file was read. **If the owner meant a page of its own**, a ninth category
   in the side column needs an icon the design file does not have, and both
   window choices would move to it.
9. **Reduced motion is a third section on the Accessibility page**, headed
   *Motion*, with one switch — *Reduced motion* — in the design's own switch row
   (`550:256` off, `551:196` on) and the line *On, the view jumps to where it is
   going instead of gliding there.* The owner's yes of 2026-11-07; `551:1038`
   draws only Text and Pointer, so the section, its heading and its words are
   this lane's. It is **off** as a machine comes. It is kept in the person's own
   `access.toml` (`alo_access::Setting::ReducedMotion`), not `appearance.toml`,
   because that is the file alo-access owns and sign-in already reads; the rest
   of that file is written back as it was. A file that does not read is said
   above the section and the switch is not offered, so one switch never writes
   over a person's broken file.
10. **"Apps describe themselves to alo and screen readers" is a fourth section on
   the Accessibility page**, headed *Applications*, with one switch in the
   design's own switch row (`550:256` off, `551:196` on) and two lines under it:
   what it does and what it costs — *a little more memory and processor time* —
   and that a change reaches the apps opened after it. The owner's decision of
   2026-11-11, *on by default, switch in Settings*, and the cost in the words the owner approved.
   No frame draws it, so the section, its heading and its words are this
   lane's. It is **on** as a machine comes, the other way round from every
   other switch on the page, and is kept in the same `access.toml` as *Reduced
   motion*, so it is offered only where that file read.
11. **"Let alo work fully inside apps like VSCodium" is a second switch under
   *Applications***, below the first, in the same switch row, with two lines
   under it: what it does and what it changes in those apps — *their editor
   switches into screen reader mode: it behaves differently, and says Screen
   Reader Optimized at the bottom of the window* — and the same line as the
   first switch, that a change reaches the apps opened after it. The owner's
   decision of 2026-11-12, *"Person's choice, off (Recommended)"*: it is
   **off** as a machine comes. No frame draws it, so its words are this
   lane's. It is kept in the same `access.toml`, as a word in its `on` list,
   and offered only where that file read.
12. **"Screen reader" is the first section on the Accessibility page**, headed
   *Reading aloud*, with one switch in the design's own switch row (`550:256`
   off, `551:196` on) and the line *On, what is on the screen and where the
   keyboard is are read aloud, in the language you read, now and each time you
   sign in. The voice is made on this computer, and nothing it says is sent
   anywhere.* The owner's *"Ship Orca"* of 2026-11-12
   ([ADR 0151](../decisions/0151-alo-ships-a-screen-reader-and-its-voices-stay-on-the-machine.md)).
   **First rather than after *Applications***, because the person who needs it
   is the one who cannot look further down the page for it; `551:1038` draws
   only Text and Pointer, so the place, the heading and the words are this
   lane's. It is **off** as a machine comes and kept in the same `access.toml`
   (`alo_access::Setting::ScreenReader`), so offered only where that file read.
   **Where the reader could not do what the switch says**, one sentence says so
   in the section's footnote — not on this computer, did not start, or did not
   stop — and the switch stays as the person set it. The setup screens' own
   screen-reader row (`375:30435`, `384:26446`) and its `Ctrl`+`Alt`+`S` are
   not this window and are not built.
