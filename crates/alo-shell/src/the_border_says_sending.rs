//! **A window alo operates says, on its border, that it is sending.**
//!
//! The owner, 2026-11-15, asked where the notice that alo's copy of an
//! application is sending should live: *"It's should be at the edges of the
//! window."* So while alo's work in a window's application has a line up on the
//! egress indicator (`crate::alo_at_work::WorkingInApplications::what_is_sending`),
//! that window's border carries it too:
//!
//! - **the border changes in weight**, 2 to 4, so the change is seen without
//!   reading a colour (*who-is-acting.md*);
//! - **the words are written on a tab of the border**, outside the window on its
//!   foot, 16 in from the left-hand corner — *sending* once alo's copy is seen
//!   sending, *may be sending* where nothing can count it — in `bg/surface` on
//!   the border's own ink. Outside, so none of the application's own controls
//!   is covered; the mark's tab keeps the top edge's right-hand corner.
//!
//! Where the screen has no room under the window the words stand on its top
//! edge, and where it has none there either, inside the foot's left-hand
//! corner — the one place they cover the window, because a window that fills
//! the screen has no edge outside it.
//!
//! # What this is not
//!
//! **It is not the record of what left, and it does not replace the
//! indicator.** The indicator still says it, in its corner, and the record
//! still keeps it (law 1); whether the corner keeps saying it for alo's copy is
//! the owner's question, asked and not decided here. This only says, where the
//! person is looking, the same thing at the same moment — read from the same
//! line, so the two can never disagree.

use alo_appearance::Role;
use alo_egress::HowItIsShown;
use cosmic_text::{FontSystem, Weight};
use smithay::utils::{Logical, Rectangle};

use crate::painted::{Inked, Washed};
use crate::window_menu_paint::{
    a_rounded_fill, as_f32, in_a_weight, in_pixels, metrics, rgb, scaled,
};

/// How tall the tab the words stand on is: the mark's tab's height.
const TALL: i32 = 20;

/// How far in from the card's left-hand corner it stands.
const IN: i32 = 16;

/// The room either side of the words.
const BESIDE: i32 = 8;

/// Its corner: the mark's tab's.
const CORNER: f32 = 6.0;

/// How thick the edge is while the window's application is sending: twice its
/// resting weight, so the change is seen without reading a colour.
pub const THE_EDGE_WHILE_SENDING: f32 = 4.0;

/// **Where the words stand**, `wide` logical units across: on the card's foot,
/// outside it, 16 in from its left-hand corner; on its top edge where the
/// screen has no room under it; inside the foot's corner where it has none
/// there either. `output` is the display in pixels at `scale`, so the room is
/// read in the same logical units at any density.
#[must_use]
pub fn where_sending_is_said(
    card: Rectangle<i32, Logical>,
    wide: i32,
    output: (i32, i32),
    scale: u32,
) -> Rectangle<i32, Logical> {
    let tall =
        i32::try_from(i64::from(output.1) * 100 / i64::from(scale.max(1))).unwrap_or(i32::MAX);
    let x = card.loc.x + IN;
    let below = card.loc.y + card.size.h;
    let above = card.loc.y - TALL;
    let y = if below + TALL <= tall {
        below
    } else if above >= 0 {
        above
    } else {
        below - IN - TALL
    };
    Rectangle::new((x, y).into(), (wide, TALL).into())
}

/// **What the border says**: *sending* where alo's copy was seen sending, and
/// *may be sending* where nothing can count what the window sends — the
/// indicator's two truths, in fewer words.
#[must_use]
pub const fn what_the_border_says(how: HowItIsShown) -> alo_strings::Word {
    match how {
        HowItIsShown::Counted => alo_egress::words::ON_THE_BORDER_SENDING,
        HowItIsShown::Uncounted | HowItIsShown::TheirWindow => {
            alo_egress::words::ON_THE_BORDER_MAY_BE_SENDING
        }
    }
}

/// The words on their tab, drawn round `card` in `ink`.
pub(crate) fn drawn(
    how: HowItIsShown,
    card: Rectangle<i32, Logical>,
    ink: [u8; 3],
    fonts: &mut FontSystem,
    scale: u32,
    output: (i32, i32),
) -> (Vec<Washed>, Vec<Inked>) {
    let fraction = f64::from(scale.max(1)) / 100.0;
    let shaped = in_a_weight(
        fonts,
        what_the_border_says(how).says(),
        Weight::SEMIBOLD,
        metrics(11.0, 14.0, fraction),
        scaled(400, scale),
        ink,
        rgb(Role::BgSurface),
    )
    .cropped_to_its_ink();
    let words_wide =
        i32::try_from(i64::from(shaped.width) * 100 / i64::from(scale.max(1))).unwrap_or(0) + 1;
    let place = where_sending_is_said(card, words_wide + 2 * BESIDE, output, scale);
    let tab = in_pixels(place, scale);
    let rasters: Vec<Washed> = a_rounded_fill(tab, as_f32(f64::from(CORNER) * fraction), ink)
        .into_iter()
        .collect();
    let (wide, high) = (shaped.width, shaped.height);
    let words: Vec<Inked> = shaped
        .placed(
            tab.loc.x + (tab.size.w - wide).max(0) / 2,
            tab.loc.y + (tab.size.h - high).max(0) / 2,
            output.1,
        )
        .into_iter()
        .collect();
    (rasters, words)
}

#[cfg(test)]
#[path = "the_border_says_sending_tests.rs"]
mod tests;
