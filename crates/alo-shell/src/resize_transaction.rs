//! Pointer-owned resize negotiation and acknowledged commit anchoring.
use crate::{InputError, ResizeEdge, ResizeGeometry, surfaces::Surfaces};
use smithay::{
    backend::input::ButtonState,
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::protocol::wl_surface::WlSurface,
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
    // **`start_window_resize` was here, and it is gone with the road it served.**
    //
    // It validated a client's `xdg_toplevel.resize` — the serial, the seat, that
    // the press was inside that client's own surface — and ADR 0071 refuses that
    // request outright. `Surfaces::window_press` went with it for the same reason:
    // the move request was already refused, so once resize was too, nothing was
    // left that needed to ask *was this press the client's own*. Clippy found both
    // the moment the refusal landed, which is the dead-code lint doing exactly the
    // job it exists for.

    /// Begin a resize from the shell's own edge band, rather than from a client's
    /// request.
    ///
    /// **The same transaction, entered from the other end.** Everything after the
    /// press is identical — the geometry snapshot, the first configure, the limits
    /// refreshed on every motion, the anchored opposite edge — because a
    /// compositor with two resize implementations is one where a person can find
    /// the difference.
    ///
    /// What is skipped is `window_press`, and only that. It asks whether the press
    /// that started this was inside the client's own surface, which is the right
    /// question for `xdg_toplevel.resize` and the wrong one here: the band is the
    /// shell's, drawn **outside** the frame, so no client was under the pointer and
    /// there is no client press to validate. The checks that guard the *window*
    /// rather than the press are kept and made here instead.
    ///
    /// `starting` is the button whose press is beginning this drag, where one is —
    /// it has not reached the seat yet, and is held alongside whatever already had.
    ///
    /// Whether one began.
    pub(crate) fn begin_resize_on_the_shells_own_band(
        &mut self,
        role: ToplevelSurface,
        edge: ResizeEdge,
        starting: Option<u32>,
    ) -> bool {
        self.prune();
        let root = role.wl_surface().clone();
        if self.window_move.is_some()
            || self.window_resize.is_some()
            || self.has_window_mode(&root)
            || self.popup_grab.is_some()
            || self.mapped_toplevel(&root).is_none()
        {
            return false;
        }
        // A drag needs a button held and a place it started from. Taken from the
        // seat's own state rather than from a serial, since there is no client
        // grab to name.
        //
        // **And from `starting`, which the seat does not know about yet.** A press
        // on a band *is* where a resize begins, and `crate::pointer` consults the
        // bands before it records the button — a transaction that read only the
        // seat would refuse the very press that started it and begin on the
        // *second* click instead. `crate::window_move` has taken its button this
        // way since the name band was written; this is the same fact one gesture
        // over.
        let Some(pointer) = &self.pointer else {
            return false;
        };
        let mut buttons: Vec<u32> = pointer.buttons.clone();
        if let Some(button) = starting
            && !buttons.contains(&button)
        {
            buttons.push(button);
        }
        if buttons.is_empty() {
            return false;
        }
        let pointer = self.on_the_plane(pointer.location);
        let Ok(geometry) = self.resize_geometry(&root, edge) else {
            return false;
        };
        let Ok(size) = geometry.requested_size((0.0, 0.0)) else {
            return false;
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
        true
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

    /// Hiding a root ends only its own resize, while it can still be configured.
    pub(crate) fn cancel_resize_for(&mut self, surface: &WlSurface) {
        if self
            .window_resize
            .as_ref()
            .is_some_and(|resize| resize.role.wl_surface() == surface)
        {
            self.cancel_window_resize();
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
