//! The egress indicator on the nested compositor: every frame of a session
//! carries it.
//!
//! [`Nested::submit_with_egress_status`] is `Nested::submit_control_scene`
//! with the status area's indicator drawn above everything a client drew. A
//! frame whose indicator cannot be drawn — never told what is leaving, or laid
//! out for an output it does not fit — is refused whole, so there is no frame
//! in which a client's pixels reach the screen and the indicator does not.

use alo_dock::Dock;
use alo_strings::Strings;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::egress_status_raster::{EgressStatusLook, EgressStatusPicture, picture};
use crate::{EgressStatus, FrameTarget, Nested, RenderError, WindowControlLabels};

/// What the status area needs to draw the indicator in one frame.
#[derive(Clone, Copy)]
pub struct EgressStatusFrame<'a> {
    /// What the indicator was last told is leaving.
    pub status: &'a EgressStatus,
    /// The vocabulary the person reads, which words every line.
    pub strings: &'a Strings,
    /// The person's dock, which decides where the status area is.
    pub dock: &'a Dock,
    /// Their scheme, text scale and reading direction.
    pub look: EgressStatusLook,
}

impl Nested {
    /// Submit clients, popups and native controls with the egress indicator
    /// drawn above them and below the cursor.
    ///
    /// While nothing is leaving the indicator draws nothing and the frame is
    /// exactly the one `Nested::submit_control_scene` would submit.
    ///
    /// # Errors
    /// [`RenderError::EgressStatusUnknown`] when `status` has never been told
    /// what is leaving; [`RenderError::EgressStatusScene`] when something is
    /// leaving and the window cannot hold a dock to put it at the end of; and
    /// every error `Nested::submit_control_scene` returns.
    pub fn submit_with_egress_status(
        &mut self,
        roots: &[WlSurface],
        popups: &[crate::Popup],
        cursor: &crate::Cursor,
        controls: Option<crate::WindowControlScene<'_>>,
        labels: &mut WindowControlLabels,
        egress: EgressStatusFrame<'_>,
    ) -> Result<Vec<WlSurface>, RenderError> {
        let size = self.size();
        let status = status_picture(egress, labels, (size.w, size.h), 0)?;
        self.submit_native_scene(
            roots,
            popups,
            cursor,
            controls.map(crate::scene_native::NativeScene::Controls),
            Some(&status),
        )
    }
}

/// The indicator for this frame, or the refusal that stops the whole frame.
///
/// `beyond` is how much of the status area's corner is already taken by the
/// in-use indicator, which has the corner (`crate::in_use_raster`). Frames that
/// do not carry that indicator pass zero.
pub(crate) fn status_picture(
    egress: EgressStatusFrame<'_>,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    beyond: i32,
) -> Result<EgressStatusPicture, RenderError> {
    let drawn = egress
        .status
        .showing()
        .ok_or(RenderError::EgressStatusUnknown)?;
    picture(
        drawn,
        egress.strings,
        egress.dock,
        labels,
        size,
        egress.look,
        beyond,
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::Contrast;
    use crate::egress_status_testing::{asking_a_provider, noon, words};
    use alo_appearance::{Scheme, TextScale};
    use alo_egress::{EgressPolicy, Indicator};
    use alo_indicator::{Drew, Indicating};
    use alo_strings::Direction;

    /// The ordinary light look, read left to right.
    fn look() -> EgressStatusLook {
        EgressStatusLook {
            contrast: Contrast::AsDesigned,
            scheme: Scheme::Light,
            scale: TextScale::ordinary(),
            reading: Direction::LeftToRight,
        }
    }

    /// **A status area nobody told what is leaving refuses the frame**, and so
    /// does one that refused what it was told, and one told of a departure on
    /// a window too small to hold a dock: there is no frame in which clients
    /// are drawn and the indicator could not be.
    #[test]
    fn a_frame_whose_indicator_was_never_told_is_refused_whole() {
        let strings = words();
        let dock = Dock::shipped();
        let mut labels = WindowControlLabels::new().unwrap();

        let never_told = EgressStatus::on_an_output();
        let frame = EgressStatusFrame {
            status: &never_told,
            strings: &strings,
            dock: &dock,
            look: look(),
        };
        assert!(matches!(
            status_picture(frame, &mut labels, (1920, 1080), 0),
            Err(RenderError::EgressStatusUnknown)
        ));

        let mut indicator = Indicator::default();
        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon())
            .unwrap();
        let mut nowhere = EgressStatus::with_no_output();
        assert!(matches!(
            Indicating::nowhere().show(Some(&mut nowhere), &indicator),
            Drew::Refused(_)
        ));
        let frame = EgressStatusFrame {
            status: &nowhere,
            strings: &strings,
            dock: &dock,
            look: look(),
        };
        assert!(matches!(
            status_picture(frame, &mut labels, (1920, 1080), 0),
            Err(RenderError::EgressStatusUnknown)
        ));

        let mut status = EgressStatus::on_an_output();
        assert_eq!(
            Indicating::nowhere().show(Some(&mut status), &indicator),
            Drew::Shown
        );
        let frame = EgressStatusFrame {
            status: &status,
            strings: &strings,
            dock: &dock,
            look: look(),
        };
        assert!(matches!(
            status_picture(frame, &mut labels, (200, 150), 0),
            Err(RenderError::EgressStatusScene)
        ));
        assert!(indicator.ended(departing));
    }

    /// **Told, it draws**: a lit indicator is a picture with a row in it, and
    /// a quiet one is a frame with nothing added.
    #[test]
    fn a_told_status_area_draws_what_it_was_told() {
        let strings = words();
        let dock = Dock::shipped();
        let mut labels = WindowControlLabels::new().unwrap();
        let mut indicator = Indicator::default();
        let mut status = EgressStatus::on_an_output();
        let mut indicating = Indicating::nowhere();

        assert_eq!(indicating.show(Some(&mut status), &indicator), Drew::Shown);
        let frame = EgressStatusFrame {
            status: &status,
            strings: &strings,
            dock: &dock,
            look: look(),
        };
        assert!(
            status_picture(frame, &mut labels, (1920, 1080), 0)
                .unwrap()
                .is_empty()
        );

        let departing = indicator
            .beginning(&EgressPolicy::Anywhere, asking_a_provider(), noon())
            .unwrap();
        assert_eq!(indicating.show(Some(&mut status), &indicator), Drew::Shown);
        let frame = EgressStatusFrame {
            status: &status,
            strings: &strings,
            dock: &dock,
            look: look(),
        };
        assert_eq!(
            status_picture(frame, &mut labels, (1920, 1080), 0)
                .unwrap()
                .rows
                .len(),
            1
        );
        assert!(indicator.ended(departing));
    }
}
