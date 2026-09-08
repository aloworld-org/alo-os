//! Pointer-owned resize negotiation and acknowledged commit anchoring.
use crate::{InputError, ResizeEdge, ResizeGeometry, surfaces::Surfaces};
use smithay::{
    backend::input::ButtonState,
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
    },
    utils::{Logical, Point, Serial},
    wayland::{
        compositor::with_states,
        shell::xdg::{ToplevelSurface, XdgToplevelSurfaceData},
    },
};

/// One mapping's resize, retained through the final acknowledged commit.
pub(crate) struct Resize {
    /// Exact protocol role authorized by the pointer press.
    role: ToplevelSurface,
    /// Fixed geometry anchor; limits are refreshed for every motion.
    geometry: ResizeGeometry,
    /// Initial logical pointer location.
    pointer: Point<f64, Logical>,
    /// Held buttons consumed by this transaction.
    buttons: Vec<u32>,
    /// Earliest configure whose committed response may anchor the window.
    first: Serial,
    /// Final non-resizing configure, awaiting its committed response.
    finish: Option<Serial>,
}

impl Surfaces {
    /// Validate all authority and geometry before consuming input or configuring.
    pub(crate) fn start_window_resize(
        &mut self,
        role: ToplevelSurface,
        seat: WlSeat,
        serial: Serial,
        edge: xdg_toplevel::ResizeEdge,
    ) {
        self.prune();
        let edge = match edge {
            xdg_toplevel::ResizeEdge::Top => ResizeEdge::Top,
            xdg_toplevel::ResizeEdge::Bottom => ResizeEdge::Bottom,
            xdg_toplevel::ResizeEdge::Left => ResizeEdge::Left,
            xdg_toplevel::ResizeEdge::Right => ResizeEdge::Right,
            xdg_toplevel::ResizeEdge::TopLeft => ResizeEdge::TopLeft,
            xdg_toplevel::ResizeEdge::TopRight => ResizeEdge::TopRight,
            xdg_toplevel::ResizeEdge::BottomLeft => ResizeEdge::BottomLeft,
            xdg_toplevel::ResizeEdge::BottomRight => ResizeEdge::BottomRight,
            _ => return,
        };
        let root = role.wl_surface();
        let Some((pointer, buttons)) = self.window_press(root, &seat, serial) else {
            return;
        };
        let Ok(geometry) = self.resize_geometry(root, edge) else {
            return;
        };
        let Ok(size) = geometry.requested_size((0.0, 0.0)) else {
            return;
        };
        let _ = self.clear_pointer();
        role.with_pending_state(|pending| {
            pending.size = Some(size.into());
            pending.states.set(xdg_toplevel::State::Resizing);
        });
        let first = role.send_configure();
        self.window_resize = Some(Resize {
            role,
            geometry,
            pointer,
            buttons,
            first,
            finish: None,
        });
    }

    /// Consume active motion, clamping against the client's current constraints.
    pub(crate) fn resize_window_pointer(
        &mut self,
        location: Point<f64, Logical>,
    ) -> Result<bool, InputError> {
        self.prune_window_resize();
        let Some(resize) = &self.window_resize else {
            return Ok(false);
        };
        if resize.finish.is_some() {
            return Ok(false);
        }
        let delta = location - resize.pointer;
        let size = resize
            .geometry
            .with_current_limits(resize.role.wl_surface())
            .requested_size((delta.x, delta.y));
        match size {
            Ok(size) => {
                resize.role.with_pending_state(|pending| {
                    pending.size = Some(size.into());
                    pending.states.set(xdg_toplevel::State::Resizing);
                });
                resize.role.send_pending_configure();
                Ok(true)
            }
            Err(crate::ResizeGeometryError::ClientLimits) => {
                self.cancel_window_resize();
                Ok(true)
            }
            Err(_) => Err(InputError::InvalidPointer),
        }
    }

    /// End pointer ownership on the last release, but retain commit anchoring.
    pub(crate) fn window_resize_button(&mut self, button: u32, state: ButtonState) -> bool {
        self.prune_window_resize();
        // Constraints may have committed since the last pointer motion. Refresh
        // the final suggestion before ending ownership, without replaying input.
        if state == ButtonState::Released
            && self.resizing_pointer()
            && let Some(location) = self.pointer.as_ref().map(|pointer| pointer.location)
        {
            let _ = self.resize_window_pointer(location);
            if self.window_resize.is_none() {
                return true;
            }
        }
        let Some(resize) = &mut self.window_resize else {
            return false;
        };
        if resize.finish.is_some() {
            return false;
        }
        if state == ButtonState::Pressed {
            if !resize.buttons.contains(&button) {
                resize.buttons.push(button);
            }
        } else {
            resize.buttons.retain(|held| *held != button);
        }
        if resize.buttons.is_empty() {
            resize.role.with_pending_state(|pending| {
                pending.states.unset(xdg_toplevel::State::Resizing);
            });
            resize.finish = Some(resize.role.send_configure());
        }
        true
    }

    /// Whether the resize still consumes pointer events rather than awaiting commit.
    pub(crate) fn resizing_pointer(&self) -> bool {
        self.window_resize
            .as_ref()
            .is_some_and(|resize| resize.finish.is_none())
    }

    /// Apply actual committed dimensions only after a resize configure response.
    pub(crate) fn commit_window_resize(&mut self, surface: &WlSurface) {
        self.prune_window_resize();
        let Some(resize) = &self.window_resize else {
            return;
        };
        if resize.role.wl_surface() != surface {
            return;
        }
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
        let Some(serial) = serial.filter(|serial| *serial >= resize.first) else {
            return;
        };
        let size = crate::scene::geometry(surface).size.to_i32_round();
        let Ok(origin) = resize.geometry.committed_origin((size.w, size.h)) else {
            self.cancel_window_resize();
            return;
        };
        crate::window_placement::set(surface, Some(origin.into()));
        if resize.finish.is_some_and(|finish| serial >= finish) {
            self.window_resize = None;
        }
    }

    /// Leave/cancellation abandons anchoring and clears the live protocol state.
    pub(crate) fn cancel_window_resize(&mut self) {
        if let Some(resize) = self.window_resize.take()
            && self.mapped_toplevel(resize.role.wl_surface()).is_some()
        {
            resize.role.with_pending_state(|pending| {
                pending.states.unset(xdg_toplevel::State::Resizing);
            });
            resize.role.send_pending_configure();
        }
    }

    /// Unmap and disconnect retire authority without configuring a dead mapping.
    pub(crate) fn prune_window_resize(&mut self) {
        if self
            .window_resize
            .as_ref()
            .is_some_and(|resize| self.mapped_toplevel(resize.role.wl_surface()).is_none())
        {
            self.window_resize = None;
        }
    }
}
