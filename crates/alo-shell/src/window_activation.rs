//! Keyboard-owned XDG activation and explicit trusted shell selection.

use smithay::reexports::{
    wayland_protocols::xdg::shell::server::xdg_toplevel::State,
    wayland_server::protocol::wl_surface::WlSurface,
};

use crate::{InputError, Server, WindowRaiseError, surfaces::Surfaces};

/// Failure to select a window from trusted native shell controls.
#[derive(Debug, thiserror::Error)]
pub enum WindowActivationError {
    /// Focus validation failed; selection and stacking are unchanged.
    #[error(transparent)]
    Input(#[from] InputError),
    /// Focus changed, but raising or refreshing pointer routing failed.
    #[error("window focused but raising failed: {0}")]
    Raise(#[from] WindowRaiseError),
}

impl Server {
    /// Activate and raise a live mapped toplevel belonging to this display.
    ///
    /// This is trusted shell plumbing, not an agent endpoint or a client token.
    /// Invalid roots and absent keyboards refuse before any mutation. Selection
    /// releases held keys before changing recipients. Selecting a grabbed root
    /// preserves its popup focus; selecting another dismisses the popup chain.
    /// XDG activation follows keyboard ownership, including popup descendants.
    /// Configures are queued, not proof of client acknowledgement or rendering.
    pub fn activate_window(&mut self, surface: &WlSurface) -> Result<(), WindowActivationError> {
        self.keyboard_focus(Some(surface))?;
        self.raise_window(surface)?;
        Ok(())
    }
}

impl Surfaces {
    /// Reflect actual seat focus, without treating pointer focus as activation.
    pub(crate) fn configure_activation(&self, focus: Option<&WlSurface>) {
        let root = focus.and_then(|surface| {
            if self.mapped_toplevel(surface).is_some() {
                Some(surface)
            } else {
                self.popup_grab.as_ref().and_then(|grab| {
                    (self.popup_keyboard_focus().as_ref() == Some(surface)).then_some(&grab.root)
                })
            }
        });
        for surface in self.mapped() {
            if let Some(toplevel) = self.mapped_toplevel(surface) {
                toplevel.with_pending_state(|pending| {
                    if Some(surface) == root {
                        pending.states.set(State::Activated);
                    } else {
                        pending.states.unset(State::Activated);
                    }
                });
                toplevel.send_pending_configure();
            }
        }
    }
}
