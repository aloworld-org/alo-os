# The Dock's tooltip

What a person sees when they point at a control on the Dock, or the keyboard
is on it: the control's name, in a small panel beside it.

## What the owner decided, 2026-10-29

Asked after the design was found to have no tooltip on the Dock, and nothing
anywhere about how long a tooltip waits:

1. **No delay.** It appears straight away, as the window title's tooltip does
   (`crates/alo-shell/src/window_edge_tooltip.rs`). There is no timer.
2. **The selection bar's tooltip, reused.** Node `166:8846` (*Tooltip ·
   Arrange*) in `nDxyF5Ho9oC4RObjVzwBNJ`, read on 2026-10-28:

   | | |
   |---|---|
   | fill | `#102A43` (`text/primary` in the light theme; the node binds no variable) |
   | corner | 7 |
   | words | Manrope SemiBold 11 / 18, white, tracking 0.1 |
   | padding | 8 either side, 4 above and below |
   | shadow, hairline | none |

   Scaled per display, like everything on the Dock.
3. **At most the owner's width for a name beside an icon**: the words wrap at
   `alo_dock::measures::A_NAME_BESIDE_AN_ICON` (200), two lines, then an
   ellipsis. So the panel is at most 216 wide, inside the 224 that figure
   allowed with its own 12 padding. The selection bar's 8 replaces that 12,
   because the owner chose that look.
4. **The composer's spoken name stays its visible words**, *Find, open, or
   ask…*.

## What the lane decided where neither the design nor the owner did

Written here so they can be disagreed with:

- **Which controls get one: those that draw an icon or a mark.** Applications,
  Settings, Applications, *Show more apps*, and History on a Dock down either
  side, where it is the design's icon (`348:23633`) rather than the word. The
  composer, *Show all* and History as a word (on the top and bottom Docks, by the
  owner's instruction) draw their own words already, and
  the window title's tooltip already holds that a panel repeating what is on the
  screen says nothing.
- **The gap between the bar and the panel is 8**, the Dock's own spacing unit
  (`alo_dock::measures::GAP`). The design's only tooltip sits 4 inside a
  blurred backdrop on a different bar. That backdrop (*Local popup / blurred
  canvas*, 120 × 36) is not drawn.
- **Toward the canvas**: above a bottom Dock, below a top one, beside one down
  either side. It is centred on the control, then moved to stay on the display.
- **The words follow the person's text size** (11 / 18 at the ordinary size),
  because the Dock already holds that a name is a sentence and must scale. The
  width does not, so a larger text size reaches the ellipsis sooner. A screen
  reader is given the whole name either way.
- **Pointer first**: when the pointer is on one control and the keyboard on
  another, the pointer's is shown.

## Accessibility, 2026-10-29

The two WCAG gaps and the right-to-left one were bugs by `CLAUDE.md` and were
fixed rather than listed (`crates/alo-shell/src/the_docks_name_is_held.rs`):

- **Escape dismisses it, and does nothing else** (WCAG 1.4.13, *dismissible*).
  Focus does not move, the pointer does not move, and nothing on the Dock
  opens or closes. The name stays away while the same control is asked about
  and comes back once the pointer or the keyboard has moved to something else.
  Escape is taken only while a name is showing, and after every window that
  holds the keyboard (Settings, the market, the Bar, a task window), because
  each is in front of the Dock and Escape there cancels that.
- **The pointer can move onto it** (WCAG 1.4.13, *hoverable*). It stays while
  the pointer is on the control, on the name, or in the 8 between them.
- **In a right-to-left language each line starts from the panel's right**,
  inside the same 8, and a shorter line is flush right. Where the panel sits
  does not change: it is centred on the control, or beside a side Dock toward
  the canvas.

## Not done

- **The navy is the same in the dark theme.** A known gap, not a choice: the
  Dock's palette has not adopted dark values for these roles, and the design
  lane has not given them. No change until it does.
- A task window drawn over that part of the canvas covers the tooltip, as it
  already covers the *More apps* list.
