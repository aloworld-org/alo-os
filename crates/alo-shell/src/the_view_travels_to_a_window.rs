//! **The view travels to a window of alo's**, rather than the window being
//! pulled to where the person is looking.
//!
//! `docs/design/the-canvas-as-a-workspace.md` §11: going to a window *focuses
//! it without silently moving it to the current camera position*. So the
//! Place the person is on becomes the window's, and where the window is out of
//! sight the camera centres on it at the zoom the person has; the window does
//! not move. Settings and the record go through this; the terminal and the
//! task windows keep the same rule in their own files.

use alo_canvas::Place;
use smithay::utils::{Logical, Physical, Rectangle};

impl crate::Server {
    /// Take the view to `window` on the plane, on `its_place`. Answers whether
    /// the view moved — its Place, its camera, or both.
    pub(crate) fn the_view_travels_to(
        &mut self,
        window: Rectangle<i32, Logical>,
        its_place: Place,
    ) -> bool {
        let somewhere_else = its_place != self.the_place_now();
        if somewhere_else {
            self.surfaces.place = its_place;
        }
        let Some(room) = self.surfaces.popups.output_size else {
            return somewhere_else;
        };
        let camera = self.the_camera();
        let seen = crate::the_terminal_on_the_plane::on_the_screen(window, camera);
        if Rectangle::<i32, Physical>::from_size(room).contains_rect(seen) {
            return somewhere_else;
        }
        let zoom = crate::scene::drawn_at(camera);
        if zoom <= 0.0 {
            return somewhere_else;
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "half a display divided by a zoom of at least a twentieth fits an i32"
        )]
        let at = alo_canvas::At::checked(
            window.loc.x + window.size.w / 2 - (f64::from(room.w) / 2.0 / zoom) as i32,
            window.loc.y + window.size.h / 2 - (f64::from(room.h) / 2.0 / zoom) as i32,
        );
        at.and_then(|at| self.look_at_the_canvas(at)).is_some() || somewhere_else
    }
}
