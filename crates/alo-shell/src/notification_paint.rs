//! Paint laid-out notifications into a frame, above every client.
//!
//! Painted after clients, popups and native controls and before the cursor, so
//! no window a client maps can cover a message that arrived.

use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Size},
};

use crate::RenderError;
use crate::notification_raster::NotificationPicture;

impl NotificationPicture {
    /// Refuse a frame the cards were not laid out for, before anything is
    /// imported or drawn.
    pub(crate) fn validate(&self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        if (size.w, size.h) == self.size {
            Ok(())
        } else {
            Err(RenderError::NotificationScene)
        }
    }

    /// Draw the cards' shapes, then their words.
    pub(crate) fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        crate::painted::paint(frame, &self.solids, &self.inked)
    }
}
