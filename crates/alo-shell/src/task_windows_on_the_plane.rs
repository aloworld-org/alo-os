//! **Task windows are windows on the canvas** — stage 2 of
//! [ADR 0145](../../../docs/decisions/0145-alos-own-windows-live-on-the-canvas.md).
//!
//! Until 2026-11-01 a task window was placed against the display and drawn
//! fixed to it, so it covered whatever the canvas had under it at every pan and
//! zoom, Show all could not see it, and it was on every Place at once.
//!
//! # The model keeps plane rectangles, and this is where they meet a screen
//!
//! `crate::the_tasks_a_person_opened::TheTasks` holds each window's rectangle
//! **on the plane**, and the Place it opened on. Those are the display's pixels
//! at 100 % with the canvas unmoved, which is why nothing else in the model had
//! to change. Everything that asks about a screen — a press, a wheel, the edge
//! with Close on it — asks here, through the camera:
//!
//! - a **point** on the screen is taken back onto the plane before it is
//!   tested against a window or its one button, so everything inside the
//!   window is measured in the units it was laid out in, at any zoom;
//! - a **window** is put onto the screen for its edge, whose own furniture is
//!   not zoomed, exactly as an application's edge is
//!   (`Server::where_a_window_is_drawn`).
//!
//! The picture is laid out at the window's own size and drawn scaled by the
//! camera, as alo's terminal is (`crate::painted_flat`).

use alo_canvas::Frame;
use smithay::utils::{Logical, Physical, Point, Rectangle};

use crate::the_tasks_a_person_opened::WhichTask;

impl crate::Server {
    /// Every task window on the Place being looked at, oldest first, on the
    /// plane.
    #[must_use]
    pub(crate) fn the_task_windows_on_the_plane(
        &self,
    ) -> Vec<(WhichTask, Rectangle<i32, Physical>)> {
        self.tasks().each_open_window_on(self.the_place_now())
    }

    /// Every task window on the Place being looked at, oldest first, where
    /// the camera draws it on the screen.
    #[must_use]
    pub(crate) fn the_task_windows_on_the_screen(
        &self,
    ) -> Vec<(WhichTask, Rectangle<i32, Physical>)> {
        let camera = self.the_camera();
        self.the_task_windows_on_the_plane()
            .into_iter()
            .map(|(which, window)| {
                (
                    which,
                    crate::the_terminal_on_the_plane::on_the_screen(
                        a_plane_rectangle(window),
                        camera,
                    ),
                )
            })
            .collect()
    }

    /// The point on the plane under this point on the screen, in the units a
    /// task window is laid out in.
    ///
    /// The camera's own answer, `Camera::plane_of`: the plane unit that holds
    /// this pixel. A point the plane cannot hold is answered as it is, which
    /// is a point on no window.
    #[must_use]
    pub(crate) fn the_plane_point_under(&self, at: Point<i32, Physical>) -> Point<i32, Physical> {
        self.the_camera()
            .plane_of((at.x, at.y))
            .map_or(at, |on| Point::from((on.x, on.y)))
    }

    /// The task window under this point on the screen, if one is: **the last
    /// opened answers first**, because it is drawn last and so is in front.
    #[must_use]
    pub(crate) fn the_task_window_under(&self, at: Point<i32, Physical>) -> Option<WhichTask> {
        let on_the_plane = self.the_plane_point_under(at);
        self.the_task_windows_on_the_plane()
            .into_iter()
            .rev()
            .find(|(_, window)| window.contains(on_the_plane))
            .map(|(which, _)| which)
    }

    /// The task windows on the Place being looked at, as frames on the canvas
    /// for *Show all* and *fit the selected window* to count.
    ///
    /// **Numbered from the top of the range down**, so a task window's number
    /// can never be a client's, which are counted up from one
    /// (`crate::window_number`).
    #[must_use]
    pub(crate) fn the_task_windows_as_frames(&self) -> Vec<Frame> {
        let place = self.the_place_now();
        self.the_task_windows_on_the_plane()
            .into_iter()
            .filter_map(|(which, window)| {
                let at = alo_canvas::At::checked(window.loc.x, window.loc.y)?;
                let size = alo_canvas::Size::checked(
                    u32::try_from(window.size.w).ok()?,
                    u32::try_from(window.size.h).ok()?,
                )?;
                let number = u64::MAX - u64::try_from(which.at()).ok()?;
                Some(Frame::of(number, place, at, size))
            })
            .collect()
    }

    /// The task window in front of the others, as a frame, for *fit the
    /// selected window*.
    #[must_use]
    pub(crate) fn the_task_window_in_front_as_a_frame(&self) -> Option<Frame> {
        let front = self.tasks().the_one_in_front()?;
        let number = u64::MAX - u64::try_from(front.at()).ok()?;
        self.the_task_windows_as_frames()
            .into_iter()
            .find(|frame| frame.id() == number)
    }

    /// Bring this task window into view if it is out of it: its Place first,
    /// then the camera to where it is, at the zoom the person has — the window
    /// is not moved. Answers whether the view moved.
    ///
    /// `docs/design/the-canvas-as-a-workspace.md` §11, as for alo's terminal
    /// (`Server::the_view_goes_to_the_terminal`).
    pub(crate) fn the_view_goes_to_this_task(&mut self, which: WhichTask) -> bool {
        let Some(its_place) = self.tasks().the_place_of(which) else {
            return false;
        };
        let somewhere_else = its_place != self.the_place_now();
        if somewhere_else {
            self.surfaces.place = its_place;
        }
        let Some(window) = self
            .the_task_windows_on_the_plane()
            .into_iter()
            .find(|(it, _)| *it == which)
            .map(|(_, window)| a_plane_rectangle(window))
        else {
            return somewhere_else;
        };
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

/// A task window's plane rectangle, in the type the camera's arithmetic takes.
fn a_plane_rectangle(window: Rectangle<i32, Physical>) -> Rectangle<i32, Logical> {
    Rectangle::new(
        (window.loc.x, window.loc.y).into(),
        (window.size.w, window.size.h).into(),
    )
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "task_windows_on_the_plane_tests.rs"]
mod tests;
