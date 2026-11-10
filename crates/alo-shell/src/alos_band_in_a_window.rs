//! **The window alo is working in wears its presence**: a thin deep teal edge
//! round the window, and across its foot a band with alo's mark, what it is
//! doing in plain words, and **Stop**.
//!
//! Two promises, drawn as one surface:
//!
//! - *The agent's presence* (`docs/features.md`; Demo 2's stage 8): **a thin
//!   deep teal edge on the window the agent is working in**, for the turn, on
//!   no other window, gone when the turn ends — and deep teal never alone, so
//!   it comes with alo's mark and its word (ADR 0067).
//! - *alo working in this window* (`docs/design/the-window-controls.md`,
//!   `11:146`): the band across the window's foot, with **Stop** always there.
//!
//! # Measured, from `11:173` and `11:175`
//!
//! | | figure |
//! |---|---|
//! | band | 20 in from each side of the window, 20 above its foot, 48 tall; corner 10; `accent/soft` |
//! | words | Medium 13, deep teal, 14 down — after alo's mark, which `11:174` does not draw |
//! | Stop | 64 × 30, 12 in from the band's end and 9 down; white, corner 7; SemiBold 12, deep teal |
//!
//! **The mark and the edge are the promise's, not the frame's.** `11:146` draws
//! the band and its words and neither of these; `docs/features.md` promises a
//! teal edge, and teal never arrives without the mark. So the mark stands 16
//! in, where `11:174`'s words began, and the words move along by its 16 and a
//! gap of 8. Recorded in the design note's *Readings taken while building*.
//!
//! # Teal only while alo is acting
//!
//! While alo waits for the person — a change waiting for their OK — it is not
//! acting, and the task window draws that in navy. The band does the same: no
//! teal edge, the mark and words in navy on the panel's quiet ground, and Stop
//! still there. **Stopping** is teal, because alo has not gone yet, with Stop
//! greyed so it cannot be pressed twice. **Colour is never the only signal**:
//! the words say which of the three it is.

use alo_appearance::Role;
use cosmic_text::{FontSystem, Weight};
use smithay::utils::{Logical, Rectangle};

use crate::alo_in_a_window::{AloHere, HowAloIsHere};
use crate::painted::{Inked, Washed};
use crate::window_menu_paint::{
    a_mark, a_ring_of, a_rounded_fill, as_f32, in_a_weight, in_pixels, metrics, rgb, scaled,
};

/// How far in from each side of the window, and up from its foot, the band
/// is. `11:173` at (20, 452) in a 1056 × 520 window: `520 − 452 − 48`.
pub const IN_FROM_THE_WINDOW: i32 = 20;

/// How tall the band is. `11:173`.
pub const THE_BAND_IS_TALL: i32 = 48;

/// The band's corner. `11:173`.
const THE_BANDS_CORNER: f32 = 10.0;

/// Where alo's mark sits in the band, and how big it is: 16 in, where
/// `11:174`'s words begin, centred down the 48.
const THE_MARK: (i32, i32, i32) = (16, 16, 16);

/// Where the words start across the band: after the mark and a gap of 8.
const THE_WORDS_START: i32 = 40;

/// Where the words sit down the band. `11:174`.
const THE_WORDS_SIT: i32 = 14;

/// Stop: how wide, how tall, how far in from the band's end, how far down.
/// `11:175` at (940, 9), 64 × 30, in a 1016-wide band.
const STOP: (i32, i32, i32, i32) = (64, 30, 12, 9);

/// Stop's corner. `11:175`.
const STOPS_CORNER: f32 = 7.0;

/// How wide the teal edge round the window is: thin, the focus ring's weight.
const THE_EDGE_IS: f32 = 2.0;

/// The window card's corner, which the edge follows. `11:30`.
const THE_CARDS_CORNER: f32 = 16.0;

/// The narrowest band worth drawing: the mark, a few words and Stop.
const THE_NARROWEST_BAND: i32 = THE_WORDS_START + 96 + STOP.0 + STOP.2;

/// Where the band and its Stop are, for a window, in the band's own units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheBand {
    /// The whole band. A press anywhere in it is the band's.
    pub band: Rectangle<i32, Logical>,
    /// Stop.
    pub stop: Rectangle<i32, Logical>,
}

/// The band across the foot of `window`, or [`None`] where the window is too
/// small to hold it — then only the edge is drawn, and Stop is in the task
/// window, as it always is.
#[must_use]
pub fn the_band_of(window: Rectangle<i32, Logical>) -> Option<TheBand> {
    let wide = window.size.w - 2 * IN_FROM_THE_WINDOW;
    let room_down = window.size.h - 2 * IN_FROM_THE_WINDOW - THE_BAND_IS_TALL;
    if wide < THE_NARROWEST_BAND || room_down < 0 {
        return None;
    }
    let band = Rectangle::new(
        (
            window.loc.x + IN_FROM_THE_WINDOW,
            window.loc.y + window.size.h - IN_FROM_THE_WINDOW - THE_BAND_IS_TALL,
        )
            .into(),
        (wide, THE_BAND_IS_TALL).into(),
    );
    let stop = Rectangle::new(
        (
            band.loc.x + band.size.w - STOP.2 - STOP.0,
            band.loc.y + STOP.3,
        )
            .into(),
        (STOP.0, STOP.1).into(),
    );
    Some(TheBand { band, stop })
}

/// What the band says: the task's status, then what the person asked, in their
/// own words — *alo is working · summarise this document*.
#[must_use]
pub fn what_the_band_says(here: &AloHere) -> String {
    let what = here.what.trim();
    if what.is_empty() {
        return here.status.says().to_owned();
    }
    format!("{} \u{b7} {what}", here.status.says())
}

/// alo in one window, as the frame hands it to the band's picture.
#[derive(Debug, Clone, Copy)]
pub struct AlosBand<'a> {
    /// How alo is in the window, and what it is doing.
    pub here: &'a AloHere,
    /// The whole window card, its title band included, which the edge goes
    /// round.
    pub card: Rectangle<i32, Logical>,
    /// The window's own content, whose foot the band crosses.
    pub window: Rectangle<i32, Logical>,
    /// Whether the pointer is on Stop.
    pub stop_pointed_at: bool,
}

/// The edge and the band, drawn: rasters over the window, and the words.
#[must_use]
pub(crate) fn drawn(
    alo: AlosBand<'_>,
    fonts: &mut FontSystem,
    scale: u32,
    output: (i32, i32),
) -> (Vec<Washed>, Vec<Inked>) {
    let fraction = f64::from(scale.max(1)) / 100.0;
    let px = |it: i32| scaled(it, scale);
    let acting = alo.here.how != HowAloIsHere::WaitingForThePerson;
    let ink = if acting {
        rgb(Role::AccentDefault)
    } else {
        rgb(Role::TextPrimary)
    };
    let mut rasters = Vec::new();
    let mut words = Vec::new();
    // **The edge, only while alo is acting**: deep teal round the whole card.
    if acting {
        rasters.extend(a_ring_of(
            in_pixels(alo.card, scale),
            as_f32(f64::from(THE_CARDS_CORNER) * fraction),
            as_f32(f64::from(THE_EDGE_IS) * fraction).max(1.0),
            ink,
        ));
    }
    let Some(the_band) = the_band_of(alo.window) else {
        return (rasters, words);
    };
    let ground = if acting {
        rgb(Role::AccentSoft)
    } else {
        rgb(Role::BgCanvas)
    };
    let band = the_band.band;
    rasters.extend(a_rounded_fill(
        in_pixels(band, scale),
        as_f32(f64::from(THE_BANDS_CORNER) * fraction),
        ground,
    ));
    // **alo's mark**, in the band's ink: the mark and the word come with the
    // colour, so who is acting is never told by colour alone.
    let mark = Rectangle::<i32, Logical>::new(
        (band.loc.x + THE_MARK.0, band.loc.y + THE_MARK.1).into(),
        (THE_MARK.2, THE_MARK.2).into(),
    );
    if let Some(shape) = crate::the_approval_cards_icons::ACardsIcon::TheMark.shape() {
        rasters.extend(a_mark(in_pixels(mark, scale), shape, ink));
    }
    // **What alo is doing**, on one line, cut with an ellipsis before Stop.
    let room = the_band.stop.loc.x - 12 - (band.loc.x + THE_WORDS_START);
    let medium = crate::painted_text::alos_own_text().weight(Weight::MEDIUM);
    let fitted = crate::window_edge_title::fitted_as(
        fonts,
        &what_the_band_says(alo.here),
        room.max(0),
        metrics(13.0, 18.0, 1.0),
        &medium,
    );
    if !fitted.shown.is_empty() {
        let shaped = in_a_weight(
            fonts,
            &fitted.shown,
            Weight::MEDIUM,
            metrics(13.0, 18.0, fraction),
            px(room.max(1)),
            ground,
            ink,
        )
        .cropped_to_its_ink();
        words.extend(shaped.placed(
            px(band.loc.x + THE_WORDS_START),
            px(band.loc.y + THE_WORDS_SIT),
            output.1,
        ));
    }
    // **Stop**, greyed while stopping so it cannot be pressed twice.
    let stopping = alo.here.how == HowAloIsHere::Stopping;
    let stop_ground = if alo.stop_pointed_at && !stopping {
        rgb(Role::BgCool)
    } else {
        rgb(Role::BgSurface)
    };
    let stop_ink = if stopping { rgb(Role::TextMuted) } else { ink };
    rasters.extend(a_rounded_fill(
        in_pixels(the_band.stop, scale),
        as_f32(f64::from(STOPS_CORNER) * fraction),
        stop_ground,
    ));
    let shaped = in_a_weight(
        fonts,
        alo_access::words::STOP.says(),
        Weight::SEMIBOLD,
        metrics(12.0, 16.0, fraction),
        px(STOP.0),
        stop_ground,
        stop_ink,
    )
    .cropped_to_its_ink();
    let stop = in_pixels(the_band.stop, scale);
    words.extend(shaped.placed(
        stop.loc.x + (stop.size.w - shaped.width).max(0) / 2,
        stop.loc.y + (stop.size.h - shaped.height).max(0) / 2,
        output.1,
    ));
    (rasters, words)
}

#[cfg(test)]
#[path = "alos_band_in_a_window_tests.rs"]
mod tests;
