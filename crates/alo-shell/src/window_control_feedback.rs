//! Read-only pointer presentation derived from live transaction state.
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::{Server, WindowControlSnapshot, WindowControlSnapshotError};

impl Server {
    /// Capture live native controls with hover or armed-press feedback.
    ///
    /// Supply the explicit currently painted root and geometry, and the current
    /// output-local pointer position (None after leave/input loss). This uses the
    /// shared clipped hit test, availability planner and press identity checks.
    /// Disabled controls, cancelled/foreign/stale gestures and client grabs never
    /// show hovered or pressed feedback. A held gesture suppresses hover on every
    /// other control until its release is consumed. A live policy refusal also
    /// suppresses pressed feedback without changing the eventual release error.
    ///
    /// Refresh every frame. This read sends no client events and changes no input,
    /// focus or authority. It does not observe motion or cancel transactions:
    /// route every event first, use existing leave/reset cancellation hooks, and
    /// cancel removed presentation immediately. Retained snapshots stay frozen.
    pub fn window_control_feedback(
        &self,
        surface: &WlSurface,
        viewport: (i32, i32),
        origin: (i32, i32),
        position: Option<(f64, f64)>,
    ) -> Result<WindowControlSnapshot, WindowControlSnapshotError> {
        let mut snapshot = self.window_control_snapshot(surface, viewport, origin)?;
        let eligible = position.filter(|position| {
            !self.window_control_input_busy()
                && self
                    .control_press
                    .as_ref()
                    .is_none_or(|press| press.armed_at(self, surface, viewport, origin, *position))
        });
        snapshot.layout = snapshot
            .layout
            .with_pointer_feedback(eligible, self.control_press.is_some());
        Ok(snapshot)
    }
}
