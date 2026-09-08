//! Trusted visibility transitions, independent of client mapping and geometry.
use crate::Server;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

/// A refused visibility request; no scene or input state changed.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WindowMinimizeError {
    /// The target must be this display's live buffered toplevel, even to restore.
    #[error("minimize target is not a buffered toplevel in this display")]
    Unmapped,
}

impl Server {
    /// Hide or reveal a buffered root from trusted native shell controls.
    ///
    /// Returns whether visibility changed. Hidden roots retain buffers, placement,
    /// maximize memory and stable cycling positions, but receive no scene hits or
    /// submitted frame callbacks. Hiding dismisses their popups, cancels interactive
    /// movement/resize and retires held input before another window can receive it.
    /// Revealing neither activates nor raises; use `activate_window` explicitly.
    /// Other windows retain focus. No keyboard or output is required.
    ///
    /// Buffer commits cannot reveal a hidden window. Unmap/disconnect forgets the
    /// state, and remapping starts visible. This is not an agent endpoint or XDG
    /// minimize-request policy; no client Minimize capability is advertised yet.
    pub fn set_window_minimized(
        &mut self,
        surface: &WlSurface,
        minimized: bool,
    ) -> Result<bool, WindowMinimizeError> {
        if !self.surfaces.buffered().any(|root| root == surface) {
            return Err(WindowMinimizeError::Unmapped);
        }
        if self.surfaces.minimized().any(|root| root == surface) == minimized {
            return Ok(false);
        }
        if minimized {
            // Clear activation while the role is still visible to the focus path.
            if self.surfaces.keyboard_root().as_ref() == Some(surface) {
                let _ = self.surfaces.set_keyboard_focus(None);
            }
            self.surfaces.cancel_resize_for(surface);
        }
        self.surfaces.minimize(surface, minimized);
        // Visibility drives popup dismissal and focus retirement through the same
        // lifecycle path used by dispatch; maximize memory uses buffered lifetime.
        self.surfaces.prune();
        Ok(true)
    }

    /// Hidden buffered roots in stacking order, available to trusted restore UI.
    /// This offers no application metadata or agent context access.
    pub fn minimized_surfaces(&self) -> impl Iterator<Item = &WlSurface> {
        self.surfaces.minimized()
    }
}
