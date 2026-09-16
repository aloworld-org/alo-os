//! Validating and painting laid-out Settings into a frame.
//!
//! Painted above clients, popups, window controls, the desktop and the record
//! window, and below the approval surface, the egress indicator and the
//! cursor: changing a setting is never a reason to hide a question waiting for
//! an answer or what is leaving the machine.

use smithay::{
    backend::renderer::Frame,
    utils::{Physical, Size},
};

use crate::RenderError;
use crate::settings_raster::SettingsPicture;

impl SettingsPicture {
    /// Refuse a frame Settings was not laid out for, before anything is
    /// imported or drawn.
    pub(crate) fn validate(&self, size: Size<i32, Physical>) -> Result<(), RenderError> {
        if (size.w, size.h) == self.size {
            Ok(())
        } else {
            Err(RenderError::SettingsScene)
        }
    }

    /// Draw the panel's shapes, then its words.
    pub(crate) fn paint(&self, frame: &mut impl Frame) -> Result<(), RenderError> {
        crate::painted::paint(frame, &self.solids, &self.inked)
    }
}
