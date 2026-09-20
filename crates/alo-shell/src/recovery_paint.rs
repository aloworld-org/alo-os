//! Validating and painting a laid-out recovery screen into a frame.
//!
//! Painted as the whole output, like the sign-in screen and unlike every other
//! native surface: a machine whose desktop will not start has no client to draw
//! under it, and a recovery screen with somebody's half-drawn session showing
//! through it would be a screen nobody could read.

use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Size},
};

use crate::RenderError;
use crate::recovery_raster::RecoveryPicture;

impl RecoveryPicture {
    /// Refuse a frame the screen was not laid out for, before anything is
    /// imported or drawn.
    pub(crate) fn validate(&self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        if (size.w, size.h) == self.size {
            Ok(())
        } else {
            Err(RenderError::RecoveryScene)
        }
    }

    /// Draw the panel's shapes, then its words.
    pub(crate) fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        crate::painted::paint(frame, &self.solids, &self.inked)
    }
}
