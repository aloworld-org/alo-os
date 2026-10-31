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

    /// Every raster this indicator carries — the quiet circle, or each pill
    /// with its lamp — in painting order, for the painter to upload before its
    /// frame opens, as it uploads the Dock's.
    pub(crate) fn washed(&self) -> impl Iterator<Item = &crate::painted::Washed> {
        self.washed.iter()
    }

    /// Draw the circle or the pills, then any flat shape, then the words.
    ///
    /// `washed` is what [`Self::washed`] yielded, uploaded, in the same order.
    pub(crate) fn paint<F: Frame>(
        &self,
        frame: &mut F,
        washed: &[(F::TextureId, smithay::utils::Rectangle<i32, Physical>)],
    ) -> Result<(), RenderError> {
        for (texture, area) in washed {
            crate::painted::wash(frame, texture, *area)?;
        }
        crate::painted::paint(frame, &self.solids, &self.inked)
    }
}
