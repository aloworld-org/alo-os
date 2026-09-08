//! Shared held-press authority for native XDG window operations.
use crate::surfaces::Surfaces;
use smithay::{
    input::Seat,
    reexports::wayland_server::protocol::{wl_seat::WlSeat, wl_surface::WlSurface},
    utils::{Logical, Point, Serial},
    wayland::compositor::get_parent,
};

impl Surfaces {
    /// Resolve this seat's active press only on the requested mapped root tree.
    pub(crate) fn window_press(
        &self,
        root: &WlSurface,
        seat: &WlSeat,
        serial: Serial,
    ) -> Option<(Point<f64, Logical>, Vec<u32>)> {
        if self.window_move.is_some()
            || self.window_resize.is_some()
            || self.has_window_maximize(root)
            || self.popup_grab.is_some()
            || self.mapped_toplevel(root).is_none()
            || !self.keyboard.as_ref().is_some_and(|keyboard| {
                Seat::<Self>::from_resource(seat).as_ref() == Some(&keyboard.seat)
            })
        {
            return None;
        }
        let pointer = self.pointer.as_ref()?;
        if !pointer.handle.has_grab(serial) || pointer.buttons.is_empty() {
            return None;
        }
        let (mut target, _) = pointer.handle.grab_start_data()?.focus?;
        while let Some(parent) = get_parent(&target) {
            target = parent;
        }
        (&target == root).then(|| (pointer.location, pointer.buttons.clone()))
    }
}
