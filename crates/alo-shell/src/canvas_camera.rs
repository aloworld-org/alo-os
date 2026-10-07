//! What a person is looking at, and the one place a pan reaches the drawing.
//!
//! `docs/autonomy/the-smallest-canvas-worth-showing.md` task 1: *a plane that
//! moves under a viewport that does not.*
//!
//! # Where the transform is applied, and why only there
//!
//! `crate::scene::trees` — once, for every window. Three roads read that list:
//! the drawing, the pointer's hit test and popup placement. It answers in **screen
//! pixels**, with the pan and the zoom both in it, so what is drawn and what can
//! be clicked move by the same amount in the same arithmetic, with nothing to keep
//! in step.
//!
//! A camera applied in the drawing alone would put every window's pixels somewhere
//! its clicks are not — and it would pass a test that only looked at where the
//! pixels went. That is not hypothetical: the first landing of this seam applied
//! the zoom to each frame's *size* and not to where the frame was, and drew one of
//! two windows because the second was scaled correctly onto a screen position it
//! never had.
//!
//! The pointer converts back, once, in `Surfaces::on_the_plane`: a client is told
//! its size and given its events in its own units, and is never told about the
//! canvas. The pan needs no term in that conversion, because dividing the pointer
//! and the origins by the same zoom cancels it exactly.
//!
//! # A fourth road reads the camera, and it is a display
//!
//! [`crate::FrameTarget::look_at`], called once a frame, because the backend that
//! paints the plane has no `Server` to ask. Its default **refuses** a camera it
//! cannot draw rather than drawing the plane unmoved. The first version of this
//! seam had no such method: `Nested` held a copy whose rustdoc said it was kept in
//! step by whoever submitted, and nothing did.
//!
//! # What does not read it, which is the whole point
//!
//! The dock, the status area, the egress and in-use indicators, the approval
//! surface, the notifications, the capture tools and a window's controls. None of
//! them is in `scene::trees`; all of them are painted above it from the output's
//! own size. So a pan cannot move them, and **none of them needs to know a camera
//! exists** — which is task 1's constraint, and
//! `crates/alo-shell/tests/one_plane_under_one_viewport.rs` holds it by reading
//! this crate rather than by looking at where anything ended up.
//!
//! # What is still owed
//!
//! Dragging and resizing a frame — the plan's tasks 3 and 4. Both already work in
//! plane units, because `Surfaces::window_press` records a press there and
//! `pointer_motion` hands the gestures the same converted point, so a drag of a
//! hundred screen pixels at half zoom already moves a frame two hundred plane
//! units. What those tasks owe is the rest of their acceptance — a title area that
//! drags and a content area that does not, and an application told its new size as
//! it happens — not the arithmetic.

use alo_canvas::{At, Camera, Zoom};

impl crate::Server {
    /// What the display the person is working on is looking at.
    ///
    /// **One camera per display since task 7**, so *the* camera is now a
    /// question about which display — and this answers it the way a person
    /// would: the one their pointer is on, or the main screen when it is
    /// nowhere known. A session with one display has one camera and this is
    /// it, which is every machine this lane can test.
    #[must_use]
    pub fn the_camera(&self) -> Camera {
        self.surfaces.camera_of(&self.the_display_being_worked_on())
    }

    /// Look at the plane from here, on the display being worked on.
    ///
    /// The one writer, so that *which display did this change* is answered in
    /// a single place rather than at each of the six sites that used to
    /// assign the field directly.
    pub(crate) fn set_the_camera(&mut self, camera: Camera) {
        let named = self.the_display_being_worked_on();
        self.surfaces.cameras.insert(named, camera);
    }

    /// The display the person is working on, by name.
    ///
    /// Their pointer's display where it is on one, and the main screen
    /// otherwise — a keyboard command has no position, and the main screen is
    /// where the arrangement says a new window opens, so it is where an
    /// unplaced action belongs.
    ///
    /// The empty name when the session has no arrangement, which is the one
    /// display every unarranged machine has: one key, one camera, and the
    /// behaviour this had before task 7.
    pub(crate) fn the_display_being_worked_on(&self) -> String {
        let Some(screens) = self.the_screens() else {
            return String::new();
        };
        let under_the_pointer = self.where_the_pointer_is_in_pixels().and_then(|at| {
            self.the_display_a_window_is_on(smithay::utils::Rectangle::new(at, (1, 1).into()))
        });
        under_the_pointer
            .or_else(|| screens.on(screens.main_screen()))
            .map_or_else(String::new, |place| place.name().name().to_owned())
    }

    /// Look at the plane from here instead.
    ///
    /// # Errors
    /// [`None`] where that is off the plane, which `alo-canvas` refuses by name
    /// rather than clamping: a person who panned and was silently not moved has a
    /// canvas that ignores them.
    pub fn look_at_the_canvas(&mut self, at: At) -> Option<Camera> {
        let moved = self.the_camera().looking_at(at)?;
        self.set_the_camera(moved);
        Some(moved)
    }

    /// Pan by this many screen pixels, as a hand moves.
    ///
    /// # Errors
    /// [`None`] where that leaves the plane.
    pub fn pan_the_canvas(&mut self, x: i32, y: i32) -> Option<Camera> {
        let moved = self.the_camera().panned_by(x, y)?;
        self.set_the_camera(moved);
        Some(moved)
    }

    /// Zoom to this, keeping the plane point under `held` where it is.
    ///
    /// Pointer-centred, which is `alo-canvas`' own arithmetic.
    ///
    /// # Errors
    /// [`None`] where the resulting camera would leave the plane.
    pub fn zoom_the_canvas(&mut self, zoom: Zoom, held: (i32, i32)) -> Option<Camera> {
        let moved = self.the_camera().zoomed_to(zoom, held)?;
        self.set_the_camera(moved);
        Some(moved)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_camera_per_viewport_tests.rs"]
mod a_camera_per_viewport_tests;
