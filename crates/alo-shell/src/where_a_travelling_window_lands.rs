//! **Where a window travelling aside lands, and how its outline is drawn.**
//!
//! A window put aside travels to its own card in the panel
//! (`crate::a_window_travels_aside`). This file answers the two questions that
//! travel has about the screen: where each card is, and what the outline
//! looks like on the way there.
//!
//! # The card as the panel lays it out revealed
//!
//! The panel is usually concealed when a window is minimised — nobody has
//! reached for it — and a concealed panel lays out no cards
//! (`crate::panel_raster::picture`). So the card is the one the panel
//! **would** draw revealed: on the panel's own edge, right for a person who
//! reads left to right and left for one who reads right to left, at this
//! display's scale and the person's text size. Laid out by the panel's own
//! `picture`, never worked out again here, so the outline ends where a card is
//! drawn when the person reaches for the panel.
//!
//! The newest card is the panel's first (`alo_put_aside::Panel::put_aside`
//! puts it at the top), and the panel's slots are in its own order, so the
//! card of the window at index *n* of `Panel::previews` is slot *n*. **A
//! window with no slot has no card**: no panel column fits this display, or
//! the rail was too short to hold that window's card. Then nothing travels.
//!
//! # The outline: the card's colour, the focus ring's weight
//!
//! **No new figure.** The outline is drawn in the accent, the colour every
//! card in the panel is drawn in, so the thing that travels and the thing it
//! becomes are one colour. Its weight is `alo_dock::measures::THE_FOCUS_RING_IS_THICK`,
//! the line this desktop already draws round the thing a person is on, through
//! the display's scale and the person's text size as every panel measure is.
//! An outline rather than a filled shape, because a filled shape the size of a
//! window would cover what is behind it for the length of the travel.
//!
//! # Furniture, not authority
//!
//! Nothing here decides whether a window travels or what the panel holds. It
//! is handed a panel and a display and answers with rectangles.

use alo_dock::WindowId;
use alo_dock::measures::THE_FOCUS_RING_IS_THICK;
use alo_put_aside::Panel;
use smithay::utils::{Physical, Rectangle};

use crate::desktop_look::DesktopLook;
use crate::display_scale::DisplayScale;
use crate::painted::Solid;
use crate::panel_raster::WhetherRevealed;

/// Every put-aside window's card, where the panel lays it out revealed on one
/// display, and the look of an outline travelling to one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheCards {
    /// Each window the panel holds and the card it would be drawn on.
    cards: Vec<(WindowId, Rectangle<i32, Physical>)>,
    /// The display these were laid out for, which an outline is kept inside.
    display: Rectangle<i32, Physical>,
    /// How thick an outline's line is, in display pixels.
    thickness: i32,
    /// The colour an outline is drawn in: the accent the cards are drawn in.
    /// `None` exactly when there are no cards, so nothing is drawn in it.
    colour: Option<[u8; 3]>,
}

impl TheCards {
    /// Where `panel`'s cards are when it is revealed on a display of `size`
    /// pixels at `display_scale` per cent, drawn with `look`.
    ///
    /// **A display the panel cannot be laid out on has no cards**, rather than
    /// a refusal: a window put aside there is still put aside, and simply does
    /// not travel.
    #[must_use]
    pub fn as_revealed(
        panel: &Panel,
        look: DesktopLook,
        size: (i32, i32),
        display_scale: u16,
    ) -> Self {
        let scale = DisplayScale::of_per_cent(display_scale);
        let display = Rectangle::from_size(size.into());
        let thickness = look.measure().px(scale.px(THE_FOCUS_RING_IS_THICK));
        let laid_out = crate::panel_raster::picture(
            panel,
            look,
            size,
            crate::panel_raster::the_edge_for(look.reading()),
            WhetherRevealed::Revealed,
            scale,
        );
        let colour = look.palette().map(|palette| palette.accent);
        match (laid_out, colour) {
            (Ok(picture), Ok(colour)) => Self {
                cards: panel
                    .previews()
                    .iter()
                    .map(alo_put_aside::Preview::window)
                    .zip(picture.slots)
                    .collect(),
                display,
                thickness,
                colour: Some(colour),
            },
            _ => Self {
                cards: Vec::new(),
                display,
                thickness,
                colour: None,
            },
        }
    }

    /// The card `window` would be drawn on, or `None` where it has none.
    #[must_use]
    pub fn of(&self, window: WindowId) -> Option<Rectangle<i32, Physical>> {
        self.cards
            .iter()
            .find(|(it, _)| *it == window)
            .map(|(_, card)| *card)
    }

    /// The outlines at `at`, as the flat shapes the painter draws: four sides
    /// each, inside the rectangle, kept inside the display. An outline too
    /// small to have an inside is drawn filled, so it does not vanish just
    /// before it lands.
    #[must_use]
    pub(crate) fn the_outlines(&self, at: &[Rectangle<i32, Physical>]) -> Vec<Solid> {
        let Some(colour) = self.colour else {
            return Vec::new();
        };
        let line = self.thickness.max(1);
        let mut solids = Vec::new();
        for outline in at {
            let (x, y, w, h) = (outline.loc.x, outline.loc.y, outline.size.w, outline.size.h);
            let sides: Vec<Rectangle<i32, Physical>> = if w <= line * 2 || h <= line * 2 {
                vec![*outline]
            } else {
                vec![
                    Rectangle::new((x, y).into(), (w, line).into()),
                    Rectangle::new((x, y + h - line).into(), (w, line).into()),
                    Rectangle::new((x, y + line).into(), (line, h - line * 2).into()),
                    Rectangle::new((x + w - line, y + line).into(), (line, h - line * 2).into()),
                ]
            };
            for side in sides {
                if let Some(area) = side.intersection(self.display) {
                    solids.push(Solid {
                        area: crate::painted::drawn_at(area),
                        colour,
                    });
                }
            }
        }
        solids
    }
}

#[cfg(test)]
#[path = "where_a_travelling_window_lands_tests.rs"]
mod tests;
