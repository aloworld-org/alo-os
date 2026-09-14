//! Validating and painting a laid-out approval surface into a frame.
//!
//! Painted above clients, popups and window controls, and below the egress
//! indicator and the cursor: no window covers the sentence a person is asked
//! to approve, and the question never covers what is leaving the machine.

use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Size},
};

use crate::RenderError;
use crate::approval_raster::ApprovalPicture;

impl ApprovalPicture {
    /// Refuse a frame the surface was not laid out for, before anything is
    /// imported or drawn.
    pub(crate) fn validate(&self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        if (size.w, size.h) == self.size {
            Ok(())
        } else {
            Err(RenderError::ApprovalScene)
        }
    }

    /// Draw the panel's shapes, then its words.
    pub(crate) fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        crate::painted::paint(frame, &self.solids, &self.inked)
    }
}
