//! Paint a laid-out egress indicator into a frame, above every client.
//!
//! Painted after clients, popups and native controls and before the cursor,
//! so no window a client maps can cover a line of what is leaving.

use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Size},
};

use crate::RenderError;
use crate::egress_status_raster::EgressStatusPicture;

impl EgressStatusPicture {
    /// Refuse a frame the indicator was not laid out for, before anything is
    /// imported or drawn.
    pub(crate) fn validate(&self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        if (size.w, size.h) == self.size {
            Ok(())
        } else {
            Err(RenderError::EgressStatusScene)
        }
    }

    /// Draw the rows' shapes, then their words.
    pub(crate) fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        crate::painted::paint(frame, &self.solids, &self.inked)
    }
}
