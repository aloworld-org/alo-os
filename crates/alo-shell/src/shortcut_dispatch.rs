//! Trusted configured chords reaching the implemented native window operations.

use alo_shortcuts::{Action, Chord, Shortcuts};

use crate::{InputError, Server, WindowCloseError, WindowSwitchDirection, WindowSwitchError};

/// A resolved shortcut which could not perform its native window operation.
/// Diagnostic errors for the shell caller, not localized settings-panel text.
#[derive(Debug, thiserror::Error)]
pub enum ShortcutDispatchError {
    /// The binding exists, but this dispatcher has no implementation for it.
    #[error("shortcut action is unavailable: {0:?}")]
    Unsupported(Action),
    /// Close requires a keyboard seat before resolving its focused root.
    #[error(transparent)]
    Input(#[from] InputError),
    /// No mapped toplevel owns the keyboard focus; stacking is not a fallback.
    #[error("no focused window to close")]
    NoFocusedWindow,
    /// Cycling failed; includes any partial activation failure detail.
    #[error(transparent)]
    Switch(#[from] WindowSwitchError),
    /// The focused root is no longer a valid close target.
    #[error(transparent)]
    Close(#[from] WindowCloseError),
}

impl Server {
    /// Resolve one trusted chord against the person's current bindings and act.
    ///
    /// Next/previous window share the stable cycling path. Close targets actual
    /// keyboard ownership, including a grabbed popup's root, and queues exactly
    /// one cooperative XDG close request. It never kills or retries a client.
    /// No binding (including conflicting personal bindings) returns `Ok(None)`
    /// without touching focus. Other resolved actions explicitly refuse.
    ///
    /// This is an action bridge, not a raw input handler: callers must resolve
    /// the keyboard layout and isolate consumed press/release pairs before using
    /// it from input dispatch. Each call is one request, with no repeat filtering.
    /// The caller supplies its current settings snapshot; nothing is cached or
    /// persisted here. Success queues protocol effects, not client completion.
    /// This trusted API adds no agent endpoint, grant bypass or context capture.
    pub fn dispatch_window_shortcut(
        &mut self,
        shortcuts: &Shortcuts,
        chord: Chord,
    ) -> Result<Option<Action>, ShortcutDispatchError> {
        let Some(action) = shortcuts.action_for(chord) else {
            return Ok(None);
        };
        match action {
            Action::NextWindow => {
                self.switch_window(WindowSwitchDirection::Forward)?;
            }
            Action::PreviousWindow => {
                self.switch_window(WindowSwitchDirection::Backward)?;
            }
            Action::CloseWindow => {
                if self.surfaces.keyboard.is_none() {
                    return Err(InputError::Unavailable.into());
                }
                let root = self
                    .surfaces
                    .keyboard_root()
                    .ok_or(ShortcutDispatchError::NoFocusedWindow)?;
                self.request_window_close(&root)?;
            }
            other => return Err(ShortcutDispatchError::Unsupported(other)),
        }
        Ok(Some(action))
    }
}
