//! Ordered backend retirement and withdrawal of the single advertised output.

use crate::{FrameTarget, RenderError, Server, presentation::Presentation, surfaces::Surfaces};
use smithay::reexports::wayland_server::DisplayHandle;

impl Server {
    /// Disable the current trusted target, then withdraw its output and membership.
    ///
    /// Call before releasing an active session device. The caller must pass the
    /// same target used for rendering; metadata identity is checked before I/O.
    /// Failure preserves advertised state and all pending callbacks; a failed
    /// direct disable requires descriptor retirement, not retrying submission.
    /// Success clears popup output constraints and allows a fresh target lifetime.
    /// This does not poll the seat or acknowledge session pause notifications.
    pub fn retire_output(&mut self, target: &mut impl FrameTarget) -> Result<(), RenderError> {
        self.presentation.retire(&self.display_handle(), target)?;
        self.surfaces.popups.output_size = None;
        Ok(())
    }
}

impl Presentation {
    /// Preserve protocol state unless the backend reports complete retirement.
    pub(crate) fn retire(
        &mut self,
        display: &DisplayHandle,
        target: &mut impl FrameTarget,
    ) -> Result<(), RenderError> {
        self.validate_target(target)?;
        target.retire()?;
        if let Some(output) = self.output.take() {
            for surface in self.entered.drain(..) {
                output.leave(&surface);
            }
        }
        if let Some(global) = self.global.take() {
            // Keep inert binding data until display teardown: a client may have
            // queued bind before receiving global_remove. Immediate destruction
            // would disconnect that otherwise valid client.
            display.disable_global::<Surfaces>(global);
        }
        self.metadata = None;
        Ok(())
    }
}
