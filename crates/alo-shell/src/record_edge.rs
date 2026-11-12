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

use smithay::utils::{Logical, Physical, Point, Rectangle};

use crate::an_alo_windows_edge::{OnItsEdge, WhatItShows, its_picture, what_a_press_is_on};
use crate::which_window_is_in_front::AKindOfWindow;

impl crate::Server {
    /// Where the record's body is seen on the display, while it is open here
    /// and has been drawn: what its edge sits on.
    fn where_the_records_body_is_seen(&self) -> Option<Rectangle<i32, Physical>> {
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
            body,
            WhatItShows {
                title: &title,
                focused: self.the_record_is_taking_the_keys(),
            },
            self.where_the_pointer_is_in_logical_pixels(),
            fonts,
            (output, scale),
        )
    }

    /// Whether the record's edge is under `at`, in the display's pixels.
    pub(crate) fn the_records_edge_is_under(&self, at: Point<i32, Logical>) -> bool {
        self.where_the_records_body_is_seen().is_some_and(|body| {
            what_a_press_is_on(
                body,
                at,
                (
                    self.the_room_an_edge_is_clamped_in(),
                    self.the_scale_an_edge_is_measured_at(),
                ),
            )
            .is_some()
        })
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

    /// A press at `at`, in the display's pixels, offered to the record's edge.
    /// Close shuts the window; anywhere else on the edge brings it in front.
    pub(crate) fn a_press_on_the_records_edge(&mut self, at: Point<i32, Logical>) -> bool {
        let Some(body) = self.where_the_records_body_is_seen() else {
            return false;
        };
        match what_a_press_is_on(
            body,
            at,
            (
                self.the_room_an_edge_is_clamped_in(),
                self.the_scale_an_edge_is_measured_at(),
            ),
        ) {
            None => false,
            Some(OnItsEdge::Close) => {
                self.the_person_closes_the_record();
                true
            }
            Some(OnItsEdge::Minimise) => {
                self.this_alo_window_is_asked_to_go_aside(alo_put_aside::AlosOwnWindow::TheRecord);
                true
            }
            Some(OnItsEdge::TheRest) => {
                self.the_composer.it_gave_the_keyboard_up();
                self.the_person_went_to(AKindOfWindow::TheRecord);
                true
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "record_edge_tests.rs"]
mod tests;
