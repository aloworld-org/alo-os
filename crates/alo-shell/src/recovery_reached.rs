//! Reaching the recovery screen because the desktop would not start.
//!
//! `ROADMAP.md` v0.5 says the recovery screen is *reachable when the workspace
//! is not*, and the only way that is true is if the thing that notices the
//! workspace is not there is the same thing that draws the screen. So the
//! desktop's own composition is attempted first, and **every refusal it makes
//! reaches the recovery screen instead of the person's eyes**: a frame the
//! desktop refused is, from where a person sits, a machine that will not start.
//!
//! # What is not caught here
//!
//! Only the composition of one desktop frame. A backend that cannot submit at
//! all, a parent window that closed, a graphics device that is gone: none of
//! those is a frame this could draw either, and each is handed back as it is.
//! The refusal that reached the screen is carried beside it
//! ([`Composed::Recovery`]) so that whoever runs the session can write it where
//! a maintainer will read it — never on the screen, where it would be a
//! sentence no vocabulary collected.
//!
//! # And the screen can refuse too
//!
//! An output too small to hold the whole sentence refuses
//! ([`RenderError::RecoveryScene`]), and that refusal is handed back rather
//! than drawn half. There is no third fallback: a machine that can draw
//! neither its desktop nor one panel has nothing this crate can put on a
//! screen, and pretending otherwise would be the most misleading screen of all.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::nested_desktop::DesktopPictures;
use crate::recovery_raster::{RecoveryLook, RecoveryPicture};
use crate::recovery_screen::RecoveryScreen;
use crate::{RenderError, WindowControlLabels};

/// What one frame composed.
pub(crate) enum Composed {
    /// The desktop, which started.
    Desktop(Box<DesktopPictures>),
    /// The recovery screen, and the refusal that sent the person to it.
    Recovery {
        /// The screen, laid out for this output.
        picture: Box<RecoveryPicture>,
        /// Why the desktop would not start, for a maintainer's log.
        why: RenderError,
    },
}

/// Which screen a person is actually looking at after one frame.
pub enum Reached {
    /// The desktop, and the client surfaces drawn in it.
    TheDesktop(Vec<WlSurface>),
    /// The recovery screen, and why the desktop would not start — for a
    /// maintainer's log, never for the screen.
    Recovery(RenderError),
}

/// The desktop when it composed, and the recovery screen when it did not.
///
/// # Errors
/// [`RenderError::RecoveryScene`] when the desktop refused **and** this output
/// cannot hold the recovery screen whole.
pub(crate) fn instead_of_the_desktop(
    desktop: Result<DesktopPictures, RenderError>,
    screen: &RecoveryScreen,
    labels: &mut WindowControlLabels,
    size: (i32, i32),
    look: RecoveryLook,
) -> Result<Composed, RenderError> {
    match desktop {
        Ok(pictures) => Ok(Composed::Desktop(Box::new(pictures))),
        Err(why) => {
            let picture = crate::recovery_raster::picture(screen.shows(), labels, size, look)?;
            Ok(Composed::Recovery {
                picture: Box::new(picture),
                why,
            })
        }
    }
}

#[cfg(test)]
#[path = "recovery_reached_tests.rs"]
mod tests;
