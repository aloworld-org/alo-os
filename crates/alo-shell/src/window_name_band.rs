//! Where a frame's name band is, and how much of it must stay reachable.
//!
//! # What this is for
//!
//! `docs/features.md`, `[v0.01]`: *a frame shows nothing but its content while
//! the person works, and **its name and few controls appear when they point at
//! it, select it or zoom out** — the name is also what it is dragged by, so a
//! click inside always belongs to the application.*
//!
//! Two things in that sentence decide this file.
//!
//! **The band is revealed, not permanent.** A band drawn always would look more
//! finished and would break the promise's first clause. So nothing here decides
//! *when* — the host does, the same way it already decides when to pass
//! controls to `Server::render_window_controls`. This answers only *where* and
//! *how big*, for a host that has decided to show one.
//!
//! **The band is the drag handle.** That is why its reachable height is a rule
//! rather than a nicety: a band a person cannot grab is a frame they cannot
//! move, and `crate::canvas_never_lost` is the file that refuses that.
//!
//! # 48, and the 24 that is not it
//!
//! The band is **48** logical pixels tall. Three sources, none of which says
//! anything else:
//!
//! - `canvas_never_lost_handle_tests.rs:20` — *a 300-pixel name band at the
//!   origin, 48 tall*;
//! - the design file's `Canvas / Window title`, 48 in 1,574 of its 1,814
//!   instances;
//! - a reading of that node from the live file on 2026-10-08.
//!
//! **24 is a different number and the two were confused in this crate until
//! 2026-10-08.** `canvas_never_lost::A_USABLE_HANDLE` is `(44.0, 24.0)` and its
//! comment called 24 *a frame's name band at the height it is drawn*. It is
//! not: 44 × 24 is the smallest piece of the band that must remain uncovered,
//! from the owner's ruling that *one exposed pixel is technically reachable and
//! practically lost*. A band drawn 24 tall would satisfy every exposure test
//! while being half the height it should be, which is why the mistake was worth
//! a correction rather than a shrug.
//!
//! # What this file does not do
//!
//! It draws nothing, holds no colour and words nothing. The name itself is
//! `Server::the_name_of`'s answer and is the application's own string; the
//! colours are `alo-appearance`'s; the drawing is the painter's.

use smithay::utils::{Logical, Rectangle, Size};

/// How tall a frame's name band is, in logical pixels.
///
/// See this file's header for the three sources and for the 24 it is not.
pub const BAND_IS_TALL: i32 = 48;

/// The smallest piece of the band that must stay uncovered for a frame to be
/// movable: `alo_shell::canvas_never_lost::A_USABLE_HANDLE`, as whole pixels.
///
/// Repeated here as a width and a height rather than imported as a tuple of
/// floats, because this file reasons in the logical rectangle a band occupies
/// and that one reasons in a target a pointer must hit. **If they disagree the
/// test below fails**, which is the only way two numbers in two files stay one
/// fact.
const MUST_STAY_REACHABLE: (i32, i32) = (44, 24);

/// Where a frame's name band sits, given where the frame is.
///
/// Full width at the top of the frame, which is what the design draws and what
/// *the name is also what it is dragged by* requires: a handle narrower than
/// the frame would leave part of the top edge looking like a handle and not
/// being one.
///
/// [`None`] for a frame too small to hold a band that could be grabbed — see
/// [`is_reachable`]. A band drawn on such a frame would be a handle a person
/// cannot use, and drawing nothing is the honest answer until the frame is
/// resized.
#[must_use]
pub fn band_of(frame: Rectangle<i32, Logical>) -> Option<Rectangle<i32, Logical>> {
    let band = Rectangle::new(
        frame.loc,
        Size::from((frame.size.w, BAND_IS_TALL.min(frame.size.h))),
    );
    is_reachable(band).then_some(band)
}

/// Whether this much band is enough of one to grab.
///
/// The rule `crate::canvas_never_lost` holds for a band partly under the dock,
/// applied to a band that is simply small. One number, two situations: a band
/// 20 pixels tall on a tiny frame and a band 20 pixels of which peep out from
/// under the dock are the same problem for the person trying to move it.
#[must_use]
pub fn is_reachable(band: Rectangle<i32, Logical>) -> bool {
    band.size.w >= MUST_STAY_REACHABLE.0 && band.size.h >= MUST_STAY_REACHABLE.1
}

#[cfg(test)]
#[path = "window_name_band_tests.rs"]
mod tests;
