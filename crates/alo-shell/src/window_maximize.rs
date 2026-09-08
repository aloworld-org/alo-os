//! Trusted maximization, normal-geometry memory and committed-response placement.
use crate::{ResizeEdge, ResizeGeometry, ResizeGeometryError, Server, surfaces::Surfaces};
use smithay::{
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::protocol::wl_surface::WlSurface,
    },
    utils::{Physical, Serial, Size},
    wayland::{
        compositor::with_states,
        shell::xdg::{ToplevelSurface, XdgToplevelSurfaceData},
    },
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

/// One mapping's saved normal geometry, retained until restoration commits.
pub(crate) struct MaximizedWindow {
    /// Exact mapped protocol role, never a client-selected identifier.
    role: ToplevelSurface,
    /// Initial committed normal geometry; later maximize requests never replace it.
    normal: ResizeGeometry,
    /// Last requested mode, separate from the client's committed state.
    maximized: bool,
    /// Latest configure and origin; older responses cannot place this window.
    pending: Option<(Serial, (i32, i32))>,
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

    /// Whether this mapping still owns normal-geometry memory or a restore response.
    pub(crate) fn has_window_maximize(&self, surface: &WlSurface) -> bool {
        self.window_maximize
            .iter()
            .any(|window| window.role.wl_surface() == surface)
    }

    /// Validate before remembering geometry or sending any configure.
    pub(crate) fn set_window_maximized(
        &mut self,
        surface: &WlSurface,
        maximized: bool,
    ) -> Result<Option<Serial>, WindowMaximizeError> {
        self.prune_window_maximize();
        let role = self
            .mapped_toplevel(surface)
            .ok_or(WindowMaximizeError::Unmapped)?
            .clone();
        if self.window_move.is_some() || self.window_resize.is_some() || self.popup_grab.is_some() {
            return Err(WindowMaximizeError::Busy);
        }
        let previous = self
            .window_maximize
            .iter()
            .find(|window| window.role == role);
        if previous.is_none() && !maximized {
            return Ok(None);
        }
        let output = if maximized {
            Some(
                self.maximize_output
                    .ok_or(WindowMaximizeError::OutputUnavailable)?,
            )
        } else {
            None
        };
        if previous.is_some_and(|window| window.maximized == maximized) {
            return Ok(None);
        }
        let normal = match previous {
            Some(window) => window.normal,
            None => self.resize_geometry(surface, ResizeEdge::BottomRight)?,
        };
        let (size, origin) = match output {
            Some(size) => (size, (0, 0)),
            None => {
                let size = normal
                    .with_current_limits(surface)
                    .requested_size((0.0, 0.0))?;
                (size, normal.committed_origin(size)?)
            }
        };
        let serial = configure(&role, maximized, size);
        let window = MaximizedWindow {
            role,
            normal,
            maximized,
            pending: Some((serial, origin)),
        };
        if let Some(previous) = self
            .window_maximize
            .iter_mut()
            .find(|previous| previous.role == window.role)
        {
            *previous = window;
        } else {
            self.window_maximize.push(window);
        }
        Ok(Some(serial))
    }

    /// Publish only successful output extents; retirement invalidates old anchors.
    pub(crate) fn update_maximize_output(&mut self, size: Option<Size<i32, Physical>>) {
        let size = size
            .filter(|size| {
                [size.w, size.h]
                    .into_iter()
                    .all(|v| (1..=1_000_000).contains(&v))
            })
            .map(|size| (size.w, size.h));
        if self.maximize_output == size {
            return;
        }
        self.maximize_output = size;
        self.prune_window_maximize();
        for window in &mut self.window_maximize {
            if window.maximized {
                window.pending = size.map(|size| (configure(&window.role, true, size), (0, 0)));
            }
        }
    }

    /// Only the newest acknowledged and committed response may change placement.
    pub(crate) fn commit_window_maximize(&mut self, surface: &WlSurface) {
        self.prune_window_maximize();
        let Some(window) = self
            .window_maximize
            .iter_mut()
            .find(|window| window.role.wl_surface() == surface)
        else {
            return;
        };
        let Some((required, origin)) = window.pending else {
            return;
        };
        let serial = with_states(surface, |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .and_then(|data| {
                    data.lock()
                        .unwrap_or_else(|_| std::process::abort())
                        .current_serial
                })
        });
        if !serial.is_some_and(|serial| serial >= required) {
            return;
        }
        crate::window_placement::set(surface, Some(origin.into()));
        if window.maximized {
            window.pending = None;
        } else {
            self.window_maximize
                .retain(|window| window.role.wl_surface() != surface);
        }
    }

    /// Remove dead/unmapped roles, including an unmap followed immediately by remap.
    pub(crate) fn prune_window_maximize(&mut self) {
        let mapped: Vec<_> = self.buffered().cloned().collect();
        self.window_maximize
            .retain(|window| mapped.contains(window.role.wl_surface()));
    }
}

/// Preserve unrelated XDG flags and always obtain a fresh response boundary.
fn configure(role: &ToplevelSurface, maximized: bool, size: (i32, i32)) -> Serial {
    role.with_pending_state(|pending| {
        pending.size = Some(size.into());
        if maximized {
            pending.states.set(xdg_toplevel::State::Maximized);
        } else {
            pending.states.unset(xdg_toplevel::State::Maximized);
        }
    });
    role.send_configure()
}
