//! Cooperative logical window sizing from trusted native shell controls.

use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::Serial,
    wayland::{compositor::with_states, shell::xdg::SurfaceCachedState},
};

use crate::Server;

/// A refused native size request; no configure is sent on refusal.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WindowSizeError {
    /// Only live mapped toplevel roots owned by this display can be sized.
    #[error("size target is not a mapped toplevel in this display")]
    Unmapped,
    /// Both logical dimensions must be positive; zero is not an unspecified axis.
    #[error("window dimensions must be positive")]
    InvalidSize,
    /// The requested size violates the client's currently committed size limits.
    #[error("window dimensions violate committed client size limits")]
    ClientLimits,
}

impl Server {
    /// Suggest a logical window-geometry size to a live mapped native window.
    ///
    /// Returns the queued XDG configure serial, or `None` when the latest server
    /// configuration already specifies this size. Positive dimensions outside
    /// committed client min/max limits refuse before changing pending state;
    /// zero client limits mean unconstrained axes. Limits are never silently
    /// clamped, so the caller can explain why its exact request was refused.
    ///
    /// This trusted shell API is not an agent endpoint or an interactive drag.
    /// It preserves other XDG state, focus, stacking and the existing buffer.
    /// Success is a suggestion, not proof of acknowledgement or resized pixels:
    /// normal clients may choose another size, and ordinary commits remain the
    /// authority for rendering, input geometry and popup placement. No buffer
    /// allocation, scaling, timeout or retry is performed here.
    pub fn request_window_size(
        &mut self,
        surface: &WlSurface,
        size: (i32, i32),
    ) -> Result<Option<Serial>, WindowSizeError> {
        let toplevel = self
            .surfaces
            .mapped_toplevel(surface)
            .ok_or(WindowSizeError::Unmapped)?;
        if size.0 <= 0 || size.1 <= 0 {
            return Err(WindowSizeError::InvalidSize);
        }
        let permitted = with_states(surface, |states| {
            let mut cached = states.cached_state.get::<SurfaceCachedState>();
            let current = cached.current();
            [
                (size.0, current.min_size.w, current.max_size.w),
                (size.1, current.min_size.h, current.max_size.h),
            ]
            .into_iter()
            .all(|(value, min, max)| value >= min && (max == 0 || value <= max))
        });
        if !permitted {
            return Err(WindowSizeError::ClientLimits);
        }
        toplevel.with_pending_state(|pending| pending.size = Some(size.into()));
        Ok(toplevel.send_pending_configure())
    }
}
