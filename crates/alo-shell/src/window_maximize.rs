//! Trusted maximize entry and client intent policy using shared layout transactions.
use crate::{ResizeGeometryError, Server, WindowModeError, surfaces::Surfaces, window_mode::Mode};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface, utils::Serial,
    wayland::shell::xdg::ToplevelSurface,
};

/// A refused trusted maximize/restore request; refusal changes no protocol state.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WindowMaximizeError {
    /// Only this display's live mapped toplevel roots are eligible.
    #[error("maximize target is not a mapped toplevel in this display")]
    Unmapped,
    /// No successfully submitted output with supported dimensions is available.
    #[error("maximize requires a submitted output within supported dimensions")]
    OutputUnavailable,
    /// Pointer movement, resize or a popup grab currently owns an operation.
    #[error("another interactive window operation is outstanding")]
    Busy,
    /// Normal geometry or current restore limits cannot be represented safely.
    #[error(transparent)]
    Geometry(#[from] ResizeGeometryError),
}

/// Preserve the existing exhaustive public maximize error contract.
///
/// **The tile translations went with the half on 2026-09-26.** They were
/// defensive: tile planning was never reached by maximize or restore, and they
/// existed so that a change to that invariant refused rather than passed. A
/// share carries its own rectangle and has nothing left to refuse about, so
/// there is nothing here to translate.
pub(crate) fn maximize_refusal(error: WindowModeError) -> WindowMaximizeError {
    match error {
        WindowModeError::Unmapped => WindowMaximizeError::Unmapped,
        WindowModeError::OutputUnavailable => WindowMaximizeError::OutputUnavailable,
        WindowModeError::Busy => WindowMaximizeError::Busy,
        WindowModeError::Geometry(error) => WindowMaximizeError::Geometry(error),
    }
}

impl Server {
    /// Request maximization or restoration from trusted native shell controls.
    ///
    /// Maximization uses the last successfully submitted single output at scale
    /// one, ignores normal client size limits as permitted by XDG, and remembers
    /// the original committed geometry. Restore clamps that saved size to current
    /// client limits. Returns `None` for an unchanged request. Neither request
    /// changes focus, stacking, buffers or placement before an acknowledged root
    /// commit. Actual client buffers remain authoritative; they are never scaled.
    ///
    /// Output changes supersede pending maximize responses after successful frame
    /// submission. Retirement suspends maximization until a new output submits.
    /// Unmap/disconnect forget normal geometry. Movement and exact sizing refuse
    /// until restore commits. Client XDG requests share these transactions;
    /// this trusted entry point is not an agent API.
    pub fn set_window_maximized(
        &mut self,
        surface: &WlSurface,
        maximized: bool,
    ) -> Result<Option<Serial>, WindowMaximizeError> {
        self.surfaces.set_window_maximized(surface, maximized)
    }
}

impl Surfaces {
    /// Answer client intent without bypassing mapping, output or operation policy.
    pub(crate) fn client_window_maximize(&mut self, role: ToplevelSurface, value: bool) {
        // Before the first empty commit there is no initial configure boundary.
        // Decline pre-map intent there, rather than sending a premature configure
        // or inventing normal geometry to restore later.
        if !role.is_initial_configure_sent() {
            return;
        }
        if !matches!(
            self.set_window_maximized(role.wl_surface(), value),
            Ok(Some(_))
        ) {
            // XDG requires a configure response even when policy declines or the
            // mode is unchanged. Preserve the latest pending state, including an
            // in-flight restore/resize; do not erase another transaction's flags.
            role.send_configure();
        }
    }

    /// Translate existing maximize intent into the shared layout transaction.
    pub(crate) fn set_window_maximized(
        &mut self,
        surface: &WlSurface,
        maximized: bool,
    ) -> Result<Option<Serial>, WindowMaximizeError> {
        self.set_window_mode(
            surface,
            if maximized {
                Mode::Maximized
            } else {
                Mode::Normal
            },
        )
        .map_err(maximize_refusal)
    }
}
