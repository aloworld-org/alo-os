//! Side-effect-free layout planning shared by transactions and native controls.
use crate::{
    ResizeEdge, ResizeGeometry, WindowModeError,
    surfaces::Surfaces,
    window_mode::{Anchor, Mode},
};
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::shell::xdg::ToplevelSurface,
};

/// A validated request, consumed immediately on the same display thread.
pub(crate) struct ModePlan {
    /// Exact live visible root role.
    pub(crate) role: ToplevelSurface,
    /// Original geometry retained across layout requests.
    pub(crate) normal: ResizeGeometry,
    /// Requested logical dimensions.
    pub(crate) size: (i32, i32),
    /// Placement after the matching acknowledgment and commit.
    pub(crate) anchor: Anchor,
}

impl Surfaces {
    /// Check every request condition without pruning, configuring or remembering.
    /// A successful absent plan means the request is already satisfied.
    pub(crate) fn plan_window_mode(
        &self,
        surface: &WlSurface,
        mode: Mode,
    ) -> Result<Option<ModePlan>, WindowModeError> {
        let role = self
            .mapped_toplevel(surface)
            .ok_or(WindowModeError::Unmapped)?
            .clone();
        if self.window_move.is_some() || self.window_resize.is_some() || self.popup_grab.is_some() {
            return Err(WindowModeError::Busy);
        }
        let previous = self.window_modes.iter().find(|window| window.role == role);
        if previous.is_none() && mode == Mode::Normal {
            return Ok(None);
        }
        let output = if mode == Mode::Maximized {
            Some(
                self.window_mode_output
                    .ok_or(WindowModeError::OutputUnavailable)?,
            )
        } else {
            None
        };
        let tile = match mode {
            Mode::Tiled(side) => Some(self.tile_geometry(surface, side)?),
            _ => None,
        };
        if previous.is_some_and(|window| {
            window.mode == mode && (!matches!(mode, Mode::Tiled(_)) || window.pending.is_some())
        }) {
            return Ok(None);
        }
        let normal = match previous {
            Some(window) => window.normal,
            None => self.resize_geometry(surface, ResizeEdge::BottomRight)?,
        };
        let (size, anchor) = match (output, tile) {
            (_, Some(tile)) => (tile.requested_size(), Anchor::Tile(tile)),
            (Some(size), _) => (size, Anchor::Fixed((0, 0))),
            _ => {
                let size = normal
                    .with_current_limits(surface)
                    .requested_size((0.0, 0.0))?;
                (size, Anchor::Fixed(normal.committed_origin(size)?))
            }
        };
        Ok(Some(ModePlan {
            role,
            normal,
            size,
            anchor,
        }))
    }
}
