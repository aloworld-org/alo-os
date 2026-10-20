//! Where everything on one of the environment's screens sits, on whatever
//! display the firmware left.
//!
//! **Arithmetic from `docs/design/the-installer-screens-in-rules.md`**, which
//! read the numbers off the six frames the owner adopted on 2026-10-20 at the
//! design's reference size of 1440 × 960. No machine is that size, and this
//! environment cannot change the one it is given: `crate::a_surface_to_draw_on`
//! takes the firmware's own mode on purpose, because *a black screen at this
//! moment is worse than an unfashionable resolution*.
//!
//! # One layout, five screens
//!
//! Every frame is the same page with different words — they are clones of one
//! another — so there is one set of places here and not five. What differs is
//! what is said, which is `crate::words`', and how many cards there are.
//!
//! # No scale, and the reason is these screens rather than the installer
//!
//! `alo_displays::Scale` works a scale out from pixels and the glass. It
//! answers **100%** when a display reports no physical size, which is the only
//! honest answer to being told nothing — and on the desktop a person then opens
//! Settings. **Here they cannot.** There is no Settings, no session and no next
//! boot, and the screen in front of them asks whether to erase a disk. So the
//! layout is built not to need the answer, and the one thing that does need a
//! physical size degrades rather than guesses: see [`TheType::on`].

use crate::what_display_there_is::Glass;

/// How wide the block of words is, as a share of the display.
const THE_BLOCK_IS_WIDE: f32 = 600.0 / 1440.0;
/// How far in it begins.
const THE_BLOCK_BEGINS_ACROSS: f32 = 160.0 / 1440.0;
/// How far down it begins.
const THE_BLOCK_BEGINS_DOWN: f32 = 164.0 / 960.0;
/// How wide a button is, as a share of the display.
const A_BUTTON_IS_WIDE: f32 = 160.0 / 1440.0;

/// The design's own reference, which every proportion above is taken against.
///
/// **Named rather than repeated.** Two copies of 960 in this file would be two
/// chances to change one of them.
pub const THE_DESIGN_WAS_DRAWN_AT: (u32, u32) = (1440, 960);

/// A rectangle on the screen, in the framebuffer's own pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Box {
    /// How far across its left edge is.
    pub across: u32,
    /// How far down its top edge is.
    pub down: u32,
    /// How wide it is.
    pub wide: u32,
    /// How tall it is.
    pub tall: u32,
}

/// How large one kind of writing is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheType {
    /// The height of the type itself, in pixels.
    pub tall: u32,
    /// How far apart two lines of it sit.
    pub apart: u32,
}

impl TheType {
    /// This kind of writing on a display of this height, by proportion.
    const fn by_proportion(of_the_reference: u32, line: u32, height: u32) -> Self {
        Self {
            tall: (of_the_reference * height) / THE_DESIGN_WAS_DRAWN_AT.1,
            apart: (line * height) / THE_DESIGN_WAS_DRAWN_AT.1,
        }
    }

    /// The same, raised if the glass says it would come out too small to read.
    ///
    /// **The floor is the design's own physical intent and not a number from
    /// outside**: the body's cap height when 1440 × 960 is read at 96 dpi,
    /// which is about 3.05 mm. A proportion keeps the *shape* of a screen and
    /// not the *size* of its letters, and on a 15-inch 4K panel it draws body
    /// text at about three-quarters that — the smallest thing a person has to
    /// read on a screen asking whether to erase a disk.
    ///
    /// **When the display reports no glass, the proportion stands.** Nothing
    /// is guessed from the pixel count, because a pixel count cannot tell a
    /// 13-inch 2560 laptop from a 27-inch 2560 monitor and those want opposite
    /// answers — a guess between them is a table of models wearing
    /// arithmetic's clothes.
    fn on(self, glass: Option<Glass>, display_is_tall: u32, floor_mm: f32) -> Self {
        let Some(glass) = glass else {
            return self;
        };
        // **A panel reporting nought millimetres is a panel reporting nothing.**
        // Virtual outputs do it, and so do some real ones; dividing by it would
        // be an answer invented out of a zero.
        if glass.down() == 0 {
            return self;
        }
        let per_mm = display_is_tall as f32 / glass.down() as f32;
        // Cap height is about 0.72 of the type's own height, which is what a
        // person actually sees. Judging a floor on the type size overstates it
        // by about a third.
        let cap_mm = (self.tall as f32 * CAP_OF_AN_EM) / per_mm;
        if cap_mm >= floor_mm {
            return self;
        }
        let wanted = ((floor_mm * per_mm) / CAP_OF_AN_EM).ceil() as u32;
        let grown = wanted.max(self.tall);
        Self {
            tall: grown,
            apart: (self.apart * grown) / self.tall.max(1),
        }
    }
}

/// How much of a type's height its capitals take up.
const CAP_OF_AN_EM: f32 = 0.72;

/// The smallest the body may be, in millimetres of capital.
///
/// The design's own body — 16 px at 1440 × 960 read at 96 dpi.
const THE_BODY_IS_AT_LEAST_MM: f32 = 3.05;

/// Everywhere on one of the environment's screens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AScreensPlaces {
    /// The small line above the title.
    pub eyebrow: Box,
    /// The question itself.
    pub title: Box,
    /// The sentence under it.
    pub introduction: Box,
    /// Where the first card sits, and how large every card is.
    pub first_card: Box,
    /// How far down from one card to the next.
    pub card_to_card: u32,
    /// The line under the cards.
    pub consequence: Box,
    /// The button that goes on.
    pub onward: Box,
    /// The button that goes back.
    pub back: Box,
    /// The accessibility control, bottom left.
    pub accessibility: Box,
    /// How large the eyebrow and the help line are drawn.
    pub small: TheType,
    /// How large the title is drawn.
    pub title_type: TheType,
    /// How large the introduction is drawn.
    pub body: TheType,
    /// How large a card's description and the consequence line are drawn.
    pub quiet: TheType,
}

/// Where everything goes on a display this size.
///
/// `glass` is what the display says about its own physical size, where it says
/// anything. It is used **only** by the legibility floor — never to place
/// anything — so a display that reports nothing still lays out.
#[must_use]
pub fn where_a_screen_goes(across: u32, down: u32, glass: Option<Glass>) -> Option<AScreensPlaces> {
    // **Too small to lay out is said rather than squeezed.** A screen that
    // asks whether to erase a disk, drawn into less room than its own words
    // need, is worse than one that says it cannot be drawn.
    if across < 640 || down < 480 {
        return None;
    }

    let share_across = |of: f32| (across as f32 * of).round() as u32;
    let down_from = |at: u32| (at * down) / THE_DESIGN_WAS_DRAWN_AT.1;

    let small = TheType::by_proportion(11, 16, down);
    let title_type = TheType::by_proportion(36, 44, down);
    // **The floor is on the body and on the quiet type beside it**, because
    // they are the smallest things a person has to read. Raising the title
    // would leave them where they were.
    let body = TheType::by_proportion(16, 25, down).on(glass, down, THE_BODY_IS_AT_LEAST_MM);
    let quiet = TheType::by_proportion(14, 22, down).on(glass, down, THE_BODY_IS_AT_LEAST_MM);

    let left = share_across(THE_BLOCK_BEGINS_ACROSS);
    let top = (down as f32 * THE_BLOCK_BEGINS_DOWN).round() as u32;
    let wide = share_across(THE_BLOCK_IS_WIDE);
    let button = share_across(A_BUTTON_IS_WIDE);
    let a_card_is_tall = down_from(86);
    let a_button_is_tall = down_from(48);
    let at = |down_by: u32, wide: u32, tall: u32| Box {
        across: left,
        down: top + down_by,
        wide,
        tall,
    };

    Some(AScreensPlaces {
        eyebrow: at(0, wide, small.apart),
        title: at(down_from(32), wide, title_type.apart * 2),
        introduction: at(down_from(136), wide, body.apart),
        first_card: at(down_from(177), wide, a_card_is_tall),
        card_to_card: a_card_is_tall + down_from(10),
        consequence: at(down_from(375), wide, quiet.apart),
        onward: at(down_from(413), button, a_button_is_tall),
        back: Box {
            across: left + button + down_from(12),
            down: top + down_from(413),
            wide: button,
            tall: a_button_is_tall,
        },
        accessibility: Box {
            across: share_across(40.0 / 1440.0),
            down: down - down_from(84),
            wide: button,
            tall: a_button_is_tall,
        },
        small,
        title_type,
        body,
        quiet,
    })
}

#[cfg(test)]
#[path = "where_a_screen_goes_tests.rs"]
mod tests;
