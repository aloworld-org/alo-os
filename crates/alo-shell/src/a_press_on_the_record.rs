//! A press while the record window is open: on Close, or anywhere on the
//! window.
//!
//! **Where it is comes from the draw**, recorded by
//! [`crate::Server::the_record_was_drawn`], because the window is laid out
//! where the display's scale is known and nowhere else — the rule the Dock's
//! slots already keep.
//!
//! Close does what Escape does. **A press anywhere else on the window is the
//! window's** and reaches nothing behind it: the record is drawn over the
//! applications, so a press on it was never a press on whatever is under it.
//! A press off the window is not taken, and the window stays open, as it does
//! for a press on an application beside it today.

use smithay::utils::{Physical, Point};

use crate::settings_canvas::Area;

/// Where the record window and its Close were drawn — and *Put back…* on each
/// change that offers it, and the confirmation's two buttons while it is open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheRecordsPlaces {
    /// The whole window.
    pub(crate) window: Area,
    /// Close's target.
    pub(crate) close: Area,
    /// *Put back…*, by the change's place in the account.
    pub(crate) buttons: Vec<(usize, Area)>,
    /// The confirmation's *Put back*, where it is drawn.
    pub(crate) put_back: Option<Area>,
    /// The confirmation's *Cancel*, where it is drawn.
    pub(crate) cancel: Option<Area>,
}

impl TheRecordsPlaces {
    /// Where everything was drawn on one picture of the window.
    pub(crate) fn of(picture: &crate::record_raster::RecordPicture) -> Option<Self> {
        let (window, close) = picture.panel.zip(picture.close)?;
        let confirmation = picture
            .entries
            .iter()
            .find_map(|entry| entry.confirmation.as_ref());
        Some(Self {
            window,
            close,
            buttons: picture
                .entries
                .iter()
                .filter_map(|entry| entry.put_back_button.map(|at| (entry.words.index, at)))
                .collect(),
            put_back: confirmation.and_then(|it| it.put_back),
            cancel: confirmation.and_then(|it| it.cancel),
        })
    }
}

impl TheRecordsPlaces {
    /// **The places as the camera shows them** (ADR 0145, stage 4): laid out in
    /// a room `room` big that landed at `room_on_screen`, each carried onto the
    /// display, so a press is tested where the button is seen at any zoom.
    ///
    /// **Close is the canvas edge's** on the canvas (`crate::record_edge`),
    /// which keeps its size at any zoom, so the place the window's own strip
    /// would have had is left empty here rather than found twice.
    #[must_use]
    pub(crate) fn as_seen(
        &self,
        room: (i32, i32),
        room_on_screen: smithay::utils::Rectangle<i32, Physical>,
    ) -> Self {
        let seen = |area: Area| {
            crate::a_room_through_the_camera::on_the_screen(area, room, room_on_screen)
        };
        Self {
            window: seen(self.window),
            close: Area::default(),
            buttons: self
                .buttons
                .iter()
                .map(|(which, area)| (*which, seen(*area)))
                .collect(),
            put_back: self.put_back.map(seen),
            cancel: self.cancel.map(seen),
        }
    }
}

impl crate::Server {
    /// Where the record window was drawn on the last frame, or that it was not.
    pub(crate) fn the_record_was_drawn(&mut self, places: Option<TheRecordsPlaces>) {
        self.the_records_places = places;
    }

    /// Whether the record window is drawn under `at`, in the display's pixels.
    pub(crate) fn the_record_is_under(&self, at: Point<i32, Physical>) -> bool {
        self.the_record_is_open_here()
            && self
                .the_records_places
                .as_ref()
                .is_some_and(|places| places.window.contains(at))
    }

    /// A press at `at`, in the display's pixels. Answers whether the record
    /// window took it.
    pub(crate) fn a_press_on_the_record(&mut self, at: Point<i32, Physical>) -> bool {
        if !self.the_record_is_open_here() {
            return false;
        }
        let Some(places) = self.the_records_places.clone() else {
            return false;
        };
        if places.close.contains(at) {
            self.record.closed();
            self.the_records_places = None;
            return true;
        }
        // **A press on the window brings it in front**, as on any window
        // (#1044), before what it is on is done.
        if places.window.contains(at) {
            self.the_composer.it_gave_the_keyboard_up();
            self.the_person_went_to(crate::which_window_is_in_front::AKindOfWindow::TheRecord);
        }
        // **The confirmation's buttons**, as Enter on them: the keyboard goes
        // to the one pressed, and it is pressed.
        for (button, on) in [
            (places.put_back, crate::OnTheConfirmation::PutBack),
            (places.cancel, crate::OnTheConfirmation::Cancel),
        ] {
            if button.is_some_and(|it| it.contains(at)) {
                self.record.the_keyboard_is_on(on);
                let opened = self.record.the_confirmation_is_pressed();
                self.the_record_answered(opened);
                return true;
            }
        }
        // **Put back…**, as Enter on it: the confirmation opens under it.
        if let Some((which, _)) = places.buttons.iter().find(|(_, it)| it.contains(at)) {
            self.the_record_answered(Some(crate::RecordOpened::AskedToPutBack(*which)));
            return true;
        }
        places.window.contains(at)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::record_testing::an_afternoon_kept;
    use smithay::utils::Rectangle;
    use std::os::unix::fs::PermissionsExt;

    /// A display with the record window open on the afternoon, drawn where
    /// the 1440 × 960 frame draws it.
    fn open() -> (
        tempfile::TempDir,
        crate::record_testing::Kept,
        crate::Server,
    ) {
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut server = crate::Server::bind(runtime.path(), "record-press").unwrap();
        let kept = an_afternoon_kept();
        server.record.opened_by_hand(&kept.recounting());
        server.the_record_was_drawn(Some(TheRecordsPlaces {
            window: Rectangle::new((280, 56).into(), (880, 796).into()),
            close: Rectangle::new((1112, 56).into(), (44, 44).into()),
            buttons: vec![(3, Rectangle::new((900, 300).into(), (128, 40).into()))],
            put_back: None,
            cancel: None,
        }));
        (runtime, kept, server)
    }

    /// **Close closes**, as Escape does.
    #[test]
    fn close_closes() {
        let (_runtime, _kept, mut server) = open();
        assert!(server.a_press_on_the_record((1130, 70).into()));
        assert!(!server.record.is_open());
        assert!(
            !server.a_press_on_the_record((1130, 70).into()),
            "a shut window takes nothing"
        );
    }

    /// **A press on a change's *Put back…* asks for its plan**, as Enter on it
    /// does, and is the window's.
    #[test]
    fn put_back_on_a_change_asks_for_its_plan() {
        let (_runtime, _kept, mut server) = open();
        assert!(server.a_press_on_the_record((950, 320).into()));
        assert_eq!(server.take_the_plan_asked(), Some(3));
        assert!(server.record.is_open());
    }

    /// **A press on the window is the window's and closes nothing**; a press
    /// off it is not taken.
    #[test]
    fn a_press_on_the_window_is_its_own() {
        let (_runtime, _kept, mut server) = open();
        assert!(server.a_press_on_the_record((600, 400).into()));
        assert!(server.record.is_open());
        assert!(!server.a_press_on_the_record((100, 400).into()));
        assert!(server.record.is_open());
    }
}
