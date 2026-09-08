//! XDG pointer-initiated movement; authority lasts only for the held buttons.

use crate::{InputError, surfaces::Surfaces};
use smithay::{
    backend::input::ButtonState,
    input::Seat,
    reexports::wayland_server::protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
    utils::{Logical, Point, Serial},
    wayland::{compositor::get_parent, shell::xdg::ToplevelSurface},
};

/// A compositor-owned drag, detached from client pointer delivery.
pub(crate) struct Move {
    /// Only this mapping may move.
    root: WlSurface,
    /// Pointer location when the request was accepted.
    pointer: Point<f64, Logical>,
    /// Committed window geometry origin at acceptance.
    origin: Point<f64, Logical>,
    /// Buttons consumed until all have been released.
    buttons: Vec<u32>,
}

impl Surfaces {
    /// Ignore requests lacking an active press on this exact root's tree.
    pub(crate) fn start_window_move(
        &mut self,
        role: ToplevelSurface,
        seat: WlSeat,
        serial: Serial,
    ) {
        self.prune();
        let root = role.wl_surface();
        if self.window_move.is_some()
            || self.popup_grab.is_some()
            || self.mapped_toplevel(root).is_none()
            || !self.keyboard.as_ref().is_some_and(|keyboard| {
                Seat::<Self>::from_resource(&seat).as_ref() == Some(&keyboard.seat)
            })
        {
            return;
        }
        let Some(pointer) = &self.pointer else { return };
        if !pointer.handle.has_grab(serial) || pointer.buttons.is_empty() {
            return;
        }
        let Some((mut target, _)) = pointer.handle.grab_start_data().and_then(|data| data.focus)
        else {
            return;
        };
        while let Some(parent) = get_parent(&target) {
            target = parent;
        }
        if &target != root {
            return;
        }
        let movement = Move {
            root: root.clone(),
            pointer: pointer.location,
            origin: crate::window_buffer_origin(root) + crate::scene::geometry_origin(root),
            buttons: pointer.buttons.clone(),
        };
        // Balance the client's press and leave before taking over. No synthetic
        // release may subsequently authorize a popup or another move.
        let _ = self.clear_pointer();
        self.window_move = Some(movement);
    }

    /// Move before scene hit testing. Out-of-range motion changes no drag state.
    pub(crate) fn move_window_pointer(
        &mut self,
        location: Point<f64, Logical>,
    ) -> Result<bool, InputError> {
        self.prune_window_move();
        let Some(movement) = &self.window_move else {
            return Ok(false);
        };
        let position = movement.origin + (location - movement.pointer);
        if ![position.x, position.y]
            .into_iter()
            .all(|v| (-1_000_000.0..=1_000_000.0).contains(&v))
        {
            return Err(InputError::InvalidPointer);
        }
        crate::window_placement::set(&movement.root, Some(position.to_i32_round()));
        Ok(true)
    }

    /// Consume drag buttons; the final release ends ownership without replay.
    pub(crate) fn window_move_button(&mut self, button: u32, state: ButtonState) -> bool {
        let Some(movement) = &mut self.window_move else {
            return false;
        };
        if state == ButtonState::Pressed {
            if !movement.buttons.contains(&button) {
                movement.buttons.push(button);
            }
        } else {
            movement.buttons.retain(|held| *held != button);
        }
        if movement.buttons.is_empty() {
            self.window_move = None;
        }
        true
    }

    /// A protocol object cannot carry a drag into its next mapping lifetime.
    pub(crate) fn prune_window_move(&mut self) {
        if self
            .window_move
            .as_ref()
            .is_some_and(|movement| self.mapped_toplevel(&movement.root).is_none())
        {
            self.window_move = None;
        }
    }
}
