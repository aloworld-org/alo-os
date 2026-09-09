//! Shared normal/maximized/tiled transactions and acknowledged placement.
use crate::{ResizeGeometry, ResizeGeometryError, surfaces::Surfaces};
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

/// A refused trusted normal/maximized/tiled request; refusal changes no protocol state.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum WindowModeError {
    /// Only this display's live mapped toplevel roots are eligible.
    #[error("window mode target is not a mapped toplevel in this display")]
    Unmapped,
    /// No successfully submitted output with supported dimensions is available.
    #[error("window mode requires a submitted output within supported dimensions")]
    OutputUnavailable,
    /// Pointer movement, resize or a popup grab currently owns an operation.
    #[error("another interactive window operation is outstanding")]
    Busy,
    /// Normal geometry or current restore limits cannot be represented safely.
    #[error(transparent)]
    Geometry(#[from] ResizeGeometryError),
    /// Exact tiling must satisfy committed hints and output bounds.
    #[error(transparent)]
    Tile(#[from] crate::TileGeometryError),
}

/// One mapping's saved normal geometry, retained until restoration commits.
pub(crate) struct ModeWindow {
    /// Exact mapped protocol role, never a client-selected identifier.
    pub(crate) role: ToplevelSurface,
    /// Initial committed normal geometry; later maximize requests never replace it.
    pub(crate) normal: ResizeGeometry,
    /// Last requested mode, separate from the client's committed state.
    pub(crate) mode: Mode,
    /// Latest configure and origin; older responses cannot place this window.
    pub(crate) pending: Option<(Serial, Anchor)>,
}

/// Requested mode, independent of the client's committed XDG state.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Return to saved normal geometry and retire layout ownership on commit.
    Normal,
    /// Use the whole submitted output without normal-size hints.
    Maximized,
    /// Use an exact half-output size constrained by current hints.
    Tiled(crate::TileSide),
}

/// Fixed output anchor, or a tile anchored using actual committed dimensions.
#[derive(Clone, Copy)]
pub(crate) enum Anchor {
    /// Saved normal origin or the maximized output origin.
    Fixed((i32, i32)),
    /// Outside edge calculated from actual committed dimensions.
    Tile(crate::TileGeometry),
}

impl Surfaces {
    /// Latest requested mode, so toggles can supersede an uncommitted response.
    pub(crate) fn requested_window_mode(&self, surface: &WlSurface) -> Mode {
        self.window_modes
            .iter()
            .find(|window| window.role.wl_surface() == surface)
            .map_or(Mode::Normal, |window| window.mode)
    }

    /// Whether this mapping still owns normal-geometry memory or a restore response.
    pub(crate) fn has_window_mode(&self, surface: &WlSurface) -> bool {
        self.window_modes
            .iter()
            .any(|window| window.role.wl_surface() == surface)
    }

    /// Validate before remembering geometry or sending any configure.
    pub(crate) fn set_window_mode(
        &mut self,
        surface: &WlSurface,
        mode: Mode,
    ) -> Result<Option<Serial>, WindowModeError> {
        self.prune_window_modes();
        let Some(plan) = self.plan_window_mode(surface, mode)? else {
            return Ok(None);
        };
        let crate::window_mode_plan::ModePlan {
            role,
            normal,
            size,
            anchor,
        } = plan;
        let serial = configure(&role, mode, size);
        let window = ModeWindow {
            role,
            normal,
            mode,
            pending: Some((serial, anchor)),
        };
        if let Some(previous) = self
            .window_modes
            .iter_mut()
            .find(|previous| previous.role == window.role)
        {
            *previous = window;
        } else {
            self.window_modes.push(window);
        }
        Ok(Some(serial))
    }

    /// Publish only successful output extents; retirement invalidates old anchors.
    pub(crate) fn update_window_mode_output(&mut self, size: Option<Size<i32, Physical>>) {
        let size = size
            .filter(|size| {
                [size.w, size.h]
                    .into_iter()
                    .all(|v| (1..=1_000_000).contains(&v))
            })
            .map(|size| (size.w, size.h));
        if self.window_mode_output == size {
            return;
        }
        self.window_mode_output = size;
        self.prune_window_modes();
        for window in &mut self.window_modes {
            window.pending = match window.mode {
                Mode::Normal => window.pending,
                Mode::Maximized => size.map(|size| {
                    (
                        configure(&window.role, window.mode, size),
                        Anchor::Fixed((0, 0)),
                    )
                }),
                Mode::Tiled(side) => size
                    .and_then(|output| {
                        crate::TileGeometry::for_surface(window.role.wl_surface(), output, side)
                            .ok()
                    })
                    .map(|tile| {
                        (
                            configure(&window.role, window.mode, tile.requested_size()),
                            Anchor::Tile(tile),
                        )
                    }),
            };
        }
    }

    /// Only the newest acknowledged and committed response may change placement.
    pub(crate) fn commit_window_mode(&mut self, surface: &WlSurface) {
        self.prune_window_modes();
        let Some(window) = self
            .window_modes
            .iter_mut()
            .find(|window| window.role.wl_surface() == surface)
        else {
            return;
        };
        let Some((required, anchor)) = window.pending else {
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
        let origin = match anchor {
            Anchor::Fixed(origin) => origin,
            Anchor::Tile(tile) => {
                // Limits committed with this response can invalidate its old plan.
                if tile.revalidate(surface).is_err() {
                    window.pending = None;
                    return;
                }
                let size = crate::scene::geometry(surface).size.to_i32_round();
                let Ok(origin) = tile.committed_origin((size.w, size.h)) else {
                    window.pending = None;
                    return;
                };
                origin
            }
        };
        crate::window_placement::set(surface, Some(origin.into()));
        if window.mode == Mode::Maximized {
            window.pending = None;
        } else if window.mode == Mode::Normal {
            self.window_modes
                .retain(|window| window.role.wl_surface() != surface);
        }
    }

    /// Remove dead/unmapped roles, including an unmap followed immediately by remap.
    pub(crate) fn prune_window_modes(&mut self) {
        let mapped: Vec<_> = self.buffered().cloned().collect();
        self.window_modes
            .retain(|window| mapped.contains(window.role.wl_surface()));
    }
}

/// Replace only layout flags; activation and unrelated XDG states survive.
fn configure(role: &ToplevelSurface, mode: Mode, size: (i32, i32)) -> Serial {
    role.with_pending_state(|pending| {
        pending.size = Some(size.into());
        for flag in [
            xdg_toplevel::State::Maximized,
            xdg_toplevel::State::TiledLeft,
            xdg_toplevel::State::TiledRight,
            xdg_toplevel::State::TiledTop,
            xdg_toplevel::State::TiledBottom,
        ] {
            pending.states.unset(flag);
        }
        match mode {
            Mode::Maximized => {
                pending.states.set(xdg_toplevel::State::Maximized);
            }
            Mode::Tiled(_) => {
                // Both halves abut output edges or the split on every edge.
                for flag in [
                    xdg_toplevel::State::TiledLeft,
                    xdg_toplevel::State::TiledRight,
                    xdg_toplevel::State::TiledTop,
                    xdg_toplevel::State::TiledBottom,
                ] {
                    pending.states.set(flag);
                }
            }
            Mode::Normal => {}
        }
    });
    role.send_configure()
}
