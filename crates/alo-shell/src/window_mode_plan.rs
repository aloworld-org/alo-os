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
    /// Whether this window can be given a mode at all, and its role if it can.
    ///
    /// The two conditions that depend on nothing about *which* mode is being
    /// asked for: the window is mapped, and nothing else has hold of it. Asked
    /// on its own by `crate::window_dividing`, which has to know both windows
    /// will take their shares **before** it changes a division — a division
    /// that moved while the screen did not would be a layout right in the tree
    /// and wrong in front of the person.
    ///
    /// # Errors
    /// [`WindowModeError::Unmapped`] for a window that is not mapped;
    /// [`WindowModeError::Busy`] during a move, a resize or a popup grab.
    pub(crate) fn ready_for_a_mode(
        &self,
        surface: &WlSurface,
    ) -> Result<ToplevelSurface, WindowModeError> {
        let role = self
            .mapped_toplevel(surface)
            .ok_or(WindowModeError::Unmapped)?
            .clone();
        if self.window_move.is_some() || self.window_resize.is_some() || self.popup_grab.is_some() {
            return Err(WindowModeError::Busy);
        }
        Ok(role)
    }

    /// Check every request condition without pruning, configuring or remembering.
    /// A successful absent plan means the request is already satisfied.
    pub(crate) fn plan_window_mode(
        &self,
        surface: &WlSurface,
        mode: Mode,
    ) -> Result<Option<ModePlan>, WindowModeError> {
        let role = self.ready_for_a_mode(surface)?;
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
        let share = match mode {
            Mode::InAShare(share) => Some(share),
            _ => None,
        };
        if previous.is_some_and(|window| {
            window.mode == mode && (!matches!(mode, Mode::InAShare(_)) || window.pending.is_some())
        }) {
            return Ok(None);
        }
        let normal = match previous {
            Some(window) => window.normal,
            None => self.resize_geometry(surface, ResizeEdge::BottomRight)?,
        };
        let (size, anchor) = match (output, share) {
            (_, Some(share)) => (share.size, Anchor::Fixed(share.at)),
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
