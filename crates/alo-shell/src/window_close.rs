//! Cooperative toplevel close requests from trusted native shell controls.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::Server;

/// A refused close request at the trusted compositor boundary.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WindowCloseError {
    /// The target is foreign, dead, unmapped or not a toplevel root.
    #[error("close target is not a mapped toplevel in this display")]
    Unmapped,
}

impl Server {
    /// Ask this display's live mapped toplevel to close, without forcing exit.
    ///
    /// This is trusted native shell plumbing, not an agent verb or IPC endpoint.
    /// The application may ignore the request or ask the person to save work.
    /// Success means one XDG close event was queued; normal dispatch flushes it.
    /// It does not mean the application received it or closed. Rendering, input,
    /// focus and resource lifetimes remain governed by ordinary client commits.
    /// Each explicit call queues one request; there is no retry or timeout kill.
    /// Children, popups, stale handles and resources from other displays refuse.
    pub fn request_window_close(&mut self, surface: &WlSurface) -> Result<(), WindowCloseError> {
        let toplevel = self
            .surfaces
            .mapped_toplevel(surface)
            .ok_or(WindowCloseError::Unmapped)?;
        toplevel.send_close();
        Ok(())
    }
}
