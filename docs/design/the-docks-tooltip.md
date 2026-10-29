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
  Settings, Applications and *Show more apps*. History joins them on a side
  Dock, where it becomes an icon (a separate change). The
  composer, *Show all* and History as a word draw their own words already, and
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

## Not done

- **Escape does not dismiss it**, and moving the pointer onto the panel lets it
  go. WCAG 1.4.13 asks for both. Neither was asked for here.
- In a right-to-left language the lines are drawn from the left edge of the
  panel.
- The navy is the same in the dark theme, because the Dock's palette has not
  adopted dark values for these roles.
- A task window drawn over that part of the canvas covers the tooltip, as it
  already covers the *More apps* list.
