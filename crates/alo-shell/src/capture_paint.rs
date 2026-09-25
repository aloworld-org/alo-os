//! Paint the laid-out capture tools into a frame, above everything else.
//!
//! Above the notifications and both indicators, because a message arriving
//! while somebody is choosing what to capture must not land on top of the thing
//! they are drawing a box around. Below the cursor, which is how they are
//! drawing it.

use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Size},
};

use crate::RenderError;
use crate::capture_raster::CapturePicture;

impl CapturePicture {
    /// Refuse a frame the tools were not laid out for, before anything is
    /// imported or drawn.
    pub(crate) fn validate(&self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        if (size.w, size.h) == self.size {
            Ok(())
        } else {
            Err(RenderError::CaptureScene)
        }
    }

    /// Draw the tools' shapes, then any words on them.
    pub(crate) fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        crate::painted::paint(frame, &self.solids, &self.inked)
    }
}
