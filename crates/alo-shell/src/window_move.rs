//! XDG pointer-initiated movement; authority lasts only for the held buttons.

use crate::{InputError, surfaces::Surfaces};
use smithay::{
    backend::input::ButtonState,
    reexports::wayland_server::protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
    utils::{Logical, Point, Serial},
    wayland::shell::xdg::ToplevelSurface,
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
        let Some((pointer, buttons)) = self.window_press(root, &seat, serial) else {
            return;
        };
        let movement = Move {
            root: root.clone(),
            pointer,
            origin: crate::window_buffer_origin(root) + crate::scene::geometry_origin(root),
            buttons,
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
