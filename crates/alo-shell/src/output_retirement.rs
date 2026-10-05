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
        // **The target retires once; every display withdraws its own global.**
        //
        // The two were one call until 2026-10-05, which is correct for one
        // display and wrong for two in a way worth naming: `target.retire()`
        // is the *backend* going away, so calling it once per presentation
        // would retire the same backend twice and refuse the second time. The
        // globals are per display and all of them go.
        //
        // The identity check still happens, against the presentation this
        // target's own name matches — and against nothing where it matches
        // none, which is a backend retiring before it ever presented.
        let handle = self.display_handle();
        let named = target.metadata()?;
        named.validate()?;
        match self.presentations.get(&named.name) {
            Some(presented) => {
                presented.validate_target(target)?;
            }
            // **A target naming no display this session presented is not this
            // session's**, and that is the same refusal as before by a
            // different route. With one presentation it was *the identity
            // changed*; with several it is *that is not one of mine*, and the
            // two are the same mistake — retiring through a target the
            // compositor never drew to.
            //
            // Nothing presented at all is **not** that mistake: a backend
            // retiring before its first frame is an ordinary shutdown, and
            // `target.retire()` below is what answers for whether it can.
            None if !self.presentations.is_empty() => {
                return Err(RenderError::OutputIdentityChanged);
            }
            None => {}
        }
        target.retire()?;
        for presentation in self.presentations.values_mut() {
            presentation.withdraw(&handle);
        }
        self.surfaces.popups.output_size = None;
        self.surfaces.update_window_mode_output(None);
        self.the_display_retired();
        Ok(())
    }
}

impl Presentation {
    /// Preserve protocol state unless the backend reports complete retirement.
    pub(crate) fn withdraw(&mut self, display: &DisplayHandle) {
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
    }
}
