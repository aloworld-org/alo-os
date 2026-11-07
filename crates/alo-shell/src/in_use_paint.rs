//! Paint a laid-out in-use indicator into a frame, above every client.
//!
//! Painted after clients, popups and native controls and before the cursor, so
//! no window a client maps can cover the line that says the camera is on.

use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Size},
};

use crate::RenderError;
use crate::in_use_raster::InUsePicture;

impl InUsePicture {
    /// Refuse a frame the indicator was not laid out for, before anything is
    /// imported or drawn.
    pub(crate) fn validate(&self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        if (size.w, size.h) == self.size {
            Ok(())
        } else {
            Err(RenderError::InUseScene)
        }
    }

    /// Draw the rows' shapes, then their words.
    pub(crate) fn paint<F: Frame>(
        &self,
        frame: &mut F,
        ink: Option<&crate::native_inked::Uploaded<F::TextureId>>,
    ) -> Result<(), RenderError> {
        crate::painted::paint(frame, &self.solids, &[])?;
        if let Some((texture, area)) = ink {
            crate::painted::wash(frame, texture, *area)
        } else {
            crate::painted::paint(frame, &[], &self.inked)
        }
    }
}
