//! XDG pointer-initiated movement; authority lasts only for the held buttons.

use crate::{InputError, surfaces::Surfaces};
use smithay::{
    backend::input::ButtonState,
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
};

/// Where a drag would put a frame, and which frame it holds.
///
/// **Named rather than returned as a pair**, because a pair of a surface and a
/// point is two facts a caller has to keep together by hand — and clippy's
/// `type_complexity` refused the tuple, which was the right refusal for the
/// wrong reason. The point means nothing without the surface it is about: a
/// caller holding one and not the other has a position and no frame.
#[derive(Debug)]
pub(crate) struct Proposed {
    /// The frame this drag holds.
    pub(crate) root: WlSurface,
    /// Where the pointer would put it, before any rule has looked.
    pub(crate) at: Point<i32, Logical>,
}

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
    /// Take hold of this frame by the name above it.
    ///
    /// **The only road into this gesture**, and the shell's own — ADR 0071: the
    /// edge and the corners resize, and the name above the frame moves it. There
    /// is no serial and no seat here because there is no client request to
    /// authorise: the press landed on the band above the frame, which belongs to
    /// the shell and which no client was ever told about. See
    /// `crate::frame_handle` for where that band is.
    ///
    /// Answers whether the frame was taken hold of. `false` where another gesture
    /// or a menu already owns the pointer, where this root is not a live mapped
    /// toplevel, or where there is no pointer to have pressed with.
    pub(crate) fn start_move_from_the_name(&mut self, root: &WlSurface, button: u32) -> bool {
        self.prune();
        if self.window_move.is_some()
            || self.window_resize.is_some()
            || self.has_window_mode(root)
            || self.popup_grab.is_some()
            || self.mapped_toplevel(root).is_none()
        {
            return false;
        }
        let Some(location) = self.pointer.as_ref().map(|pointer| pointer.location) else {
            return false;
        };
        // Plane units, like the origin it will be compared against. The band was
        // hit in screen pixels and a drag is measured on the plane; this is the
        // same one conversion every other frame-side reader makes.
        let press = self.on_the_plane(location);
        self.window_move = Some(Move {
            root: root.clone(),
            pointer: press,
            origin: crate::window_buffer_origin(root) + crate::scene::geometry_origin(root),
            buttons: vec![button],
        });
        true
    }

    /// Where this drag would put the frame, without putting it there.
    ///
    /// **Proposes rather than commits, and that split is the whole of why this
    /// function changed shape.** The owner ruled on 2026-09-30: *keep the last
    /// valid position while the pointer continues moving; never accept an invalid
    /// placement and then pull the frame back.* This used to compute the position
    /// and call `crate::window_placement::set` in the same breath, so there was no
    /// moment between proposing and committing for a rule to run in — and a
    /// correction afterwards is exactly the snap-back the owner forbade.
    ///
    /// So the answer comes back here and `crate::Server` decides. The rule lives
    /// there because it needs the frame's name band and every frame on the plane,
    /// neither of which this type can see.
    ///
    /// [`None`] where no drag is held. The surface is returned with the point
    /// because the caller has no other way to know which frame this drag owns.
    pub(crate) fn where_this_drag_would_put_it(
        &mut self,
        location: Point<f64, Logical>,
    ) -> Result<Option<Proposed>, InputError> {
        self.prune_window_move();
        let Some(movement) = &self.window_move else {
            return Ok(None);
        };
        let position = movement.origin + (location - movement.pointer);
        if ![position.x, position.y]
            .into_iter()
            .all(|v| (-1_000_000.0..=1_000_000.0).contains(&v))
        {
            return Err(InputError::InvalidPointer);
        }
        Ok(Some(Proposed {
            root: movement.root.clone(),
            at: position.to_i32_round(),
        }))
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
