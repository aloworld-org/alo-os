//! Trusted pointer routing and drag lifetime; no agent input interface.

use crate::{InputError, Server, surfaces::Surfaces};
use smithay::{
    backend::{input::ButtonState, renderer::utils::with_renderer_surface_state},
    desktop::{WindowSurfaceType, utils::under_from_surface_tree},
    input::pointer::{AxisFrame, ButtonEvent, MotionEvent, PointerHandle},
    reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface},
    utils::{Logical, Point, SERIAL_COUNTER},
    wayland::compositor::get_parent,
};

/// Backend bookkeeping complements Smithay's protocol and implicit click grab.
pub(crate) struct Pointer {
    /// Protocol handle on the existing keyboard seat.
    handle: PointerHandle<Surfaces>,
    /// Accepted buttons only; duplicates never reach Smithay's grab state.
    buttons: Vec<u32>,
    /// Last accepted location and timestamp for cancellation.
    location: Point<f64, Logical>,
    /// Timestamp used for synthetic cancellation events.
    time: u32,
}

impl Server {
    /// Enable pointer capability on the keyboard seat, idempotently.
    ///
    /// Backends must route motion, buttons, axes and leave after enabling this.
    /// This internal API creates no agent verb; cursor presentation is separate.
    pub fn enable_pointer(&mut self) -> Result<(), InputError> {
        if self.surfaces.pointer.is_none() {
            let keyboard = self
                .surfaces
                .keyboard
                .as_mut()
                .ok_or(InputError::Unavailable)?;
            self.surfaces.pointer = Some(Pointer {
                handle: keyboard.seat.add_pointer(),
                buttons: Vec::new(),
                location: (0.0, 0.0).into(),
                time: 0,
            });
        }
        Ok(())
    }

    /// Hit-test mapped trees in renderer order at logical scale one.
    ///
    /// Roots currently share origin (0,0), matching the renderer. Input regions,
    /// buffer dimensions and subsurface stacking/offsets are respected. A held
    /// button retains the original recipient until release, including outside
    /// its bounds. Invalid coordinates leave routing state unchanged.
    pub fn pointer_motion(&mut self, x: f64, y: f64, time: u32) -> Result<(), InputError> {
        if !bounded(x) || !bounded(y) {
            return Err(InputError::InvalidPointer);
        }
        self.surfaces.prune_pointer_focus();
        let location = (x, y).into();
        let focus = self.mapped_surfaces().find_map(|root| {
            under_from_surface_tree(root, location, (0, 0), WindowSurfaceType::ALL)
                .map(|(surface, origin)| (surface, origin.to_f64()))
        });
        let pointer = self
            .surfaces
            .pointer
            .as_mut()
            .ok_or(InputError::PointerUnavailable)?;
        pointer.location = location;
        pointer.time = time;
        let handle = pointer.handle.clone();
        handle.motion(
            &mut self.surfaces,
            focus,
            &MotionEvent {
                location,
                serial: SERIAL_COUNTER.next_serial(),
                time,
            },
        );
        handle.frame(&mut self.surfaces);
        Ok(())
    }

    /// Route a Linux mouse button (BTN_LEFT through BTN_TASK).
    ///
    /// No focus, duplicate presses and unmatched releases return false. Buttons
    /// pressed outside a client never authorize a later release into a client.
    pub fn pointer_button(
        &mut self,
        button: u32,
        state: ButtonState,
        time: u32,
    ) -> Result<bool, InputError> {
        if !(0x110..=0x117).contains(&button) {
            return Err(InputError::InvalidPointer);
        }
        self.surfaces.prune_pointer_focus();
        let pointer = self
            .surfaces
            .pointer
            .as_mut()
            .ok_or(InputError::PointerUnavailable)?;
        if pointer.handle.current_focus().is_none()
            || pointer.buttons.contains(&button) == (state == ButtonState::Pressed)
        {
            return Ok(false);
        }
        if state == ButtonState::Pressed {
            pointer.buttons.push(button);
        } else {
            pointer.buttons.retain(|held| *held != button);
        }
        pointer.time = time;
        let handle = pointer.handle.clone();
        handle.button(
            &mut self.surfaces,
            &ButtonEvent {
                button,
                state,
                time,
                serial: SERIAL_COUNTER.next_serial(),
            },
        );
        handle.frame(&mut self.surfaces);
        Ok(true)
    }

    /// Forward one finite scroll frame, preserving source, v120 and stop flags.
    /// Returns false without focus. Timestamps use the backend's monotonic clock.
    pub fn pointer_axis(&mut self, frame: AxisFrame) -> Result<bool, InputError> {
        if !bounded(frame.axis.0) || !bounded(frame.axis.1) {
            return Err(InputError::InvalidPointer);
        }
        self.surfaces.prune_pointer_focus();
        let pointer = self
            .surfaces
            .pointer
            .as_mut()
            .ok_or(InputError::PointerUnavailable)?;
        if pointer.handle.current_focus().is_none() {
            return Ok(false);
        }
        pointer.time = frame.time;
        let handle = pointer.handle.clone();
        handle.axis(&mut self.surfaces, frame);
        handle.frame(&mut self.surfaces);
        Ok(true)
    }

    /// Cancel held buttons and clear focus on parent leave, deactivation or close.
    /// A subsequent motion is required before another client can receive input.
    pub fn pointer_leave(&mut self) -> Result<(), InputError> {
        self.surfaces.clear_pointer()
    }
}

/// Wayland fixed-point values have a signed 24-bit integer part.
fn bounded(value: f64) -> bool {
    value.is_finite() && (-8_388_608.0..8_388_608.0).contains(&value)
}

impl Surfaces {
    /// Clear stale focus even if its root survives a child unmap/destruction.
    pub(crate) fn prune_pointer_focus(&mut self) {
        let focus = self.pointer.as_ref().and_then(|p| p.handle.current_focus());
        if focus.is_some_and(|surface| !self.pointer_surface_mapped(surface)) {
            let _ = self.clear_pointer();
        }
    }

    /// Require buffered ancestors ending in a live mapped root.
    fn pointer_surface_mapped(&self, mut surface: WlSurface) -> bool {
        loop {
            if !surface.is_alive()
                || !with_renderer_surface_state(&surface, |s| s.buffer().is_some()).unwrap_or(false)
            {
                return false;
            }
            match get_parent(&surface) {
                Some(parent) => surface = parent,
                None => return self.mapped().any(|root| *root == surface),
            }
        }
    }

    /// Clear the pending grab target before synthesizing button releases.
    fn clear_pointer(&mut self) -> Result<(), InputError> {
        let pointer = self
            .pointer
            .as_mut()
            .ok_or(InputError::PointerUnavailable)?;
        let handle = pointer.handle.clone();
        let location = pointer.location;
        let time = pointer.time;
        let buttons = std::mem::take(&mut pointer.buttons);
        // Remove the pending recipient before releasing the implicit grab.
        handle.motion(
            self,
            None,
            &MotionEvent {
                location,
                time,
                serial: SERIAL_COUNTER.next_serial(),
            },
        );
        for button in buttons {
            handle.button(
                self,
                &ButtonEvent {
                    button,
                    state: ButtonState::Released,
                    time,
                    serial: SERIAL_COUNTER.next_serial(),
                },
            );
        }
        handle.unset_grab(self, SERIAL_COUNTER.next_serial(), time);
        handle.motion(
            self,
            None,
            &MotionEvent {
                location,
                time,
                serial: SERIAL_COUNTER.next_serial(),
            },
        );
        handle.frame(self);
        Ok(())
    }
}
