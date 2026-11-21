//! **The record's window edge**: *What alo did* and Close, in the
//! focused or unfocused look, following the camera — `crate::an_alo_windows_edge`,
//! the edge every one of alo's own windows wears (ADR 0145, stage 4).
//!
//! # The same edge it drew for itself, now the shared one
//!
//! The record window drew its own title strip (`crate::record_frame_drawn`),
//! measured off `560:177`: the band 44, the strip 32 from 12, the icon 16 at
//! (14, 20) on a `bg/cool` tile at corner 4, the title at (38, 19), and Close a
//! 44 target ending 4 from the window's edge — the very figures of the
//! application edge, `554:56`. On the canvas the window leaves that band to the
//! shared edge, which draws the same parts in the same places **and** the
//! unfocused look when the keys are elsewhere, which a window always in front
//! never needed. Since 2026-11-09 no band carries an icon
//! (`docs/design/the-window-controls.md`), the record's included.
//!
//! **Close shuts the window**, as the strip's own Close always did, Put back
//! confirmation open or not; Escape is the key that closes a confirmation
//! first, and keeps doing so.

use smithay::utils::{Physical, Rectangle};

use crate::an_alo_windows_edge::{WhatItShows, its_picture};

impl crate::Server {
    /// Where the record's body is seen on the display, while it is open here
    /// and has been drawn: what its edge sits on.
    pub(crate) fn where_the_records_body_is_seen(&self) -> Option<Rectangle<i32, Physical>> {
        if !self.the_record_is_open_here() {
            return None;
        }
        self.where_the_records_body_was_drawn
    }

    /// The record's edge as a picture for this display: `output` its size in
    /// pixels and `scale` its scale in per cent.
    pub(crate) fn the_records_edge_picture(
        &self,
        fonts: &mut cosmic_text::FontSystem,
        says: &alo_strings::Strings,
        (output, scale): ((i32, i32), u32),
    ) -> Option<crate::window_edge_picture::EdgePicture> {
        let body = self.where_the_records_body_is_seen()?;
        // **Looked up where the window's words are**, `crate::the_records_words`:
        // the window itself says nothing (`tests/record_source.rs`).
        let title = crate::the_records_words::TheRecordsSentences::of(says).title;
        its_picture(
            self,
            (alo_put_aside::AlosOwnWindow::TheRecord, body),
            WhatItShows {
                title: &title,
                focused: self.the_record_is_taking_the_keys(),
            },
            fonts,
            (output, scale),
        )
    }

    /// **The person closed the record window**, by its edge's Close or by
    /// Alt+F4 (`crate::a_chord_closes_alos_own_window`): one road, so the two
    /// cannot disagree. A Put back confirmation that is open goes with it, and
    /// nothing is put back.
    pub(crate) fn the_person_closes_the_record(&mut self) {
        self.record.closed();
        self.the_records_places = None;
        self.where_the_records_body_was_drawn = None;
        self.the_record_on_the_plane = None;
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "record_edge_tests.rs"]
mod tests;
