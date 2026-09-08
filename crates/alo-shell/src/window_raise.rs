//! Explicit stacking changes from trusted native shell controls.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::{InputError, Server};

/// A failed trusted shell stacking request.
#[derive(Debug, thiserror::Error)]
pub enum WindowRaiseError {
    /// Foreign, dead, unmapped and non-toplevel targets leave order unchanged.
    #[error("raise target is not a mapped toplevel in this display")]
    Unmapped,
    /// Stacking changed, but refreshing the existing pointer route failed.
    #[error("window raised but pointer refresh failed: {0}")]
    Pointer(#[from] InputError),
}

impl Server {
    /// Raise a mapped toplevel and its popup subtree above other windows.
    ///
    /// Trusted native shell plumbing, not an agent verb or client request.
    /// Other roots retain their relative order. Raising the front root is
    /// idempotent. Rendering and hit testing consume the same order; the next
    /// successful frame presents it. Existing pointer focus is re-hit at its
    /// last location, respecting implicit button and explicit popup grabs.
    /// This neither sets XDG activation nor changes keyboard focus; those are
    /// separate policy. The nested backend's next keyboard event still selects
    /// the front root under its existing focus policy.
    pub fn raise_window(&mut self, surface: &WlSurface) -> Result<(), WindowRaiseError> {
        if !self.surfaces.raise(surface) {
            return Err(WindowRaiseError::Unmapped);
        }
        if let Some((location, time)) = self.surfaces.pointer_position() {
            self.pointer_motion(location.x, location.y, time)?;
        }
        Ok(())
    }
}
