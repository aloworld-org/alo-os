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
    /// Where the frame sat on the **plane** as its ordinary self, or [`None`]
    /// where it has no rectangle the plane can hold.
    ///
    /// **Beside [`Self::normal`] rather than derived from it, because they are in
    /// different spaces.** `ResizeGeometry`'s own field says *in output
    /// coordinates*, and an arrangement is in plane units — converting one to the
    /// other needs the camera as it was at the moment the mode was entered, and
    /// nothing keeps that. So this is captured directly, where the frame still is
    /// ordinary, from the buffer origin that is already in plane units.
    ///
    /// Task 9 of `docs/autonomy/the-smallest-canvas-worth-showing.md` names this
    /// as the half that was missing: *the state survives and the size it would go
    /// back to does not*. `alo_arranging::AWindowWas`'s `normal` field has
    /// promised to be *always the ordinary geometry, never the geometry it was
    /// last drawn at* since it was written, and its only producer was handing it
    /// the rectangle the window was last drawn at.
    pub(crate) ordinary_on_the_plane: Option<(alo_canvas::At, alo_canvas::Size)>,
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
        // **Captured every time the frame leaves `Normal`, not once ever.**
        //
        // `normal` above is kept from the first departure and never replaced,
        // which its own comment says and which is right for a *size*. It is wrong
        // for a *position*: un-maximise a window, drag it across the canvas,
        // maximise it again, and the rectangle it should come back to is where the
        // person just put it. A capture-once rule would restore it to where it sat
        // before the drag, and the arrangement would be confidently wrong about a
        // move the person made deliberately.
        //
        // So the test is *was it ordinary a moment ago* — no mode record at all,
        // or a record saying `Normal` — and not *has it ever been given a mode*.
        // Asking for `Normal` itself captures nothing: the frame is about to
        // become its ordinary self, and from then on its current rectangle **is**
        // the ordinary one, which is what `crate::canvas_remembered` reads.
        let ordinary_on_the_plane = match previous {
            _ if mode == Mode::Normal => None,
            Some(window) if window.mode != Mode::Normal => window.ordinary_on_the_plane,
            _ => crate::canvas_show_all::a_rectangle(
                crate::window_buffer_origin(surface),
                crate::scene::geometry(surface),
            ),
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
            ordinary_on_the_plane,
            size,
            anchor,
        }))
    }
}
