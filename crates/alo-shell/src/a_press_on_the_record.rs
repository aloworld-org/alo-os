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

/// Where the record window and its Close were drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheRecordsPlaces {
    /// The whole window.
    pub(crate) window: Area,
    /// Close's target.
    pub(crate) close: Area,
}

impl crate::Server {
    /// Where the record window was drawn on the last frame, or that it was not.
    pub(crate) fn the_record_was_drawn(&mut self, places: Option<TheRecordsPlaces>) {
        self.the_records_places = places;
    }

    /// A press at `at`, in the display's pixels. Answers whether the record
    /// window took it.
    pub(crate) fn a_press_on_the_record(&mut self, at: Point<i32, Physical>) -> bool {
        if !self.record.is_open() {
            return false;
        }
        let Some(places) = self.the_records_places else {
            return false;
        };
        if places.close.contains(at) {
            self.record.closed();
            self.the_records_places = None;
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
