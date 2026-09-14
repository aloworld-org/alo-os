//! Validating and painting a laid-out record window into a frame.
//!
//! Painted above clients, popups and window controls, and below the approval
//! surface, the egress indicator and the cursor: no window covers what the
//! machine did, a question waiting for an answer is never under the record, and
//! nothing covers what is leaving the machine.

use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Size},
};

use crate::RenderError;
use crate::record_raster::RecordPicture;

impl RecordPicture {
    /// Refuse a frame the window was not laid out for, before anything is
    /// imported or drawn.
    pub(crate) fn validate(&self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        if (size.w, size.h) == self.size {
            Ok(())
        } else {
            Err(RenderError::RecordScene)
        }
    }

    /// Draw the panel's shapes, then its words.
    pub(crate) fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        crate::painted::paint(frame, &self.solids, &self.inked)
    }
}
