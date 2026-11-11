//! **Which face the alo Bar shows**, decided once for every road into it.
//!
//! *The owner's decision of 2026-11-11*: "Clicking 'Find, open, or ask…' on the
//! Dock should not open the same panel as Super+Space. It should allow user to
//! text inside it" — and Super+Space is *the same as clicking the field*: one
//! place to type, by mouse or keyboard. So a click on the field, `Super`+`Space`
//! and `Super`+`A`, which all do nothing but give the composer the keyboard
//! (`crate::a_press_on_the_dock`, `crate::a_chord_opens_the_bar`), are answered
//! here with the Dock's field and the matches rising from it.
//!
//! **The centred panel is left for *Help with this window***, the one road that
//! opens the Bar about a window. Another change is moving that road into the
//! window's own panel; until it lands, the centred panel is its surface, and
//! taking it away here would leave the road with nowhere to go.

use crate::bar_raster::TheBarOpens;

impl crate::Server {
    /// Where the Bar is shown now, or [`None`] while it does not have the
    /// keyboard.
    pub(crate) fn which_face_the_bar_shows(&self) -> Option<TheBarOpens> {
        if !self.the_composer.has_the_keyboard() {
            return None;
        }
        Some(TheBarOpens::AboveTheDocksField)
    }
}

#[cfg(test)]
#[path = "which_face_the_bar_shows_tests.rs"]
mod tests;
