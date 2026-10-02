//! Trusted visibility transitions, independent of client mapping and geometry.
use crate::{Server, surfaces::Surfaces};
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
    /// state, and remapping starts visible. XDG minimize requests share this
    /// transition; this trusted entry point is not an agent endpoint.
    pub fn set_window_minimized(
        &mut self,
        surface: &WlSurface,
        minimized: bool,
    ) -> Result<bool, WindowMinimizeError> {
        let changed = self.surfaces.set_window_minimized(surface, minimized)?;
        // **A restore travels** — `the-canvas-and-its-places.md` task 4. A window
        // put aside on one Place comes back on that Place and the person is taken
        // to it, rather than the window arriving wherever they happen to be
        // looking. Only on the way back: putting one aside moves nobody.
        //
        // After the transition rather than before, so a refused restore travels
        // nowhere, and only when it actually changed — a second restore of an
        // already-visible window answers `false` and must not move the view.
        if changed && !minimized {
            self.the_view_travels_to(surface);
        }
        Ok(changed)
    }

    /// Hidden buffered roots in stacking order, available to trusted restore UI.
    /// This offers no application metadata or agent context access.
    pub fn minimized_surfaces(&self) -> impl Iterator<Item = &WlSurface> {
        self.surfaces.minimized()
    }
}

impl Surfaces {
    /// Shared mapped-root visibility policy for trusted controls and XDG requests.
    pub(crate) fn set_window_minimized(
        &mut self,
        surface: &WlSurface,
        minimized: bool,
    ) -> Result<bool, WindowMinimizeError> {
        if !self.buffered().any(|root| root == surface) {
            return Err(WindowMinimizeError::Unmapped);
        }
        if self.minimized().any(|root| root == surface) == minimized {
            return Ok(false);
        }
        if minimized {
            // Clear activation while the role is still visible to the focus path.
            if self.keyboard_root().as_ref() == Some(surface) {
                let _ = self.set_keyboard_focus(None);
            }
            self.cancel_resize_for(surface);
        }
        self.minimize(surface, minimized);
        // Visibility drives popup dismissal and focus retirement through the same
        // lifecycle path used by dispatch; maximize memory uses buffered lifetime.
        self.prune();
        Ok(true)
    }
}
