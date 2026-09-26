//! What a person is looking at, and the one place a pan reaches the drawing.
//!
//! `docs/autonomy/the-smallest-canvas-worth-showing.md` task 1: *a plane that
//! moves under a viewport that does not.*
//!
//! # Where the transform is applied, and why only there
//!
//! `crate::scene::trees` — once, for every window. Three roads read that list:
//! the drawing, the pointer's hit test and popup placement. The pointer
//! **subtracts** the origin it is given, so moving it there moves what is drawn
//! and what can be clicked by the same amount, in the same arithmetic, with
//! nothing to keep in step.
//!
//! A camera applied in the drawing alone would put every window's pixels
//! somewhere its clicks are not — and it would pass a test that only looked at
//! where the pixels went.
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
//! # The zoom is held and not yet drawn
//!
//! A [`alo_canvas::Zoom`] can be set and is carried, and `crate::drawing` has no
//! scaling path at all — so the frames do not change size yet. That is named in
//! the plan's inventory as the expensive half of this task, and it is written here
//! too, because a camera that accepted a zoom and silently ignored it is worse
//! than one that refused.

use alo_canvas::{At, Camera, Zoom};

impl crate::Server {
    /// What this session is looking at.
    #[must_use]
    pub fn the_camera(&self) -> Camera {
        self.camera
    }

    /// Look at the plane from here instead.
    ///
    /// # Errors
    /// [`None`] where that is off the plane, which `alo-canvas` refuses by name
    /// rather than clamping: a person who panned and was silently not moved has a
    /// canvas that ignores them.
    pub fn look_at_the_canvas(&mut self, at: At) -> Option<Camera> {
        let moved = self.camera.looking_at(at)?;
        self.camera = moved;
        self.surfaces.popups.camera = moved;
        Some(moved)
    }

    /// Pan by this many screen pixels, as a hand moves.
    ///
    /// # Errors
    /// [`None`] where that leaves the plane.
    pub fn pan_the_canvas(&mut self, x: i32, y: i32) -> Option<Camera> {
        let moved = self.camera.panned_by(x, y)?;
        self.camera = moved;
        self.surfaces.popups.camera = moved;
        Some(moved)
    }

    /// Zoom to this, keeping the plane point under `held` where it is.
    ///
    /// Pointer-centred, which is `alo-canvas`' own arithmetic. **What is drawn
    /// does not change size yet**: see this file's header.
    ///
    /// # Errors
    /// [`None`] where the resulting camera would leave the plane.
    pub fn zoom_the_canvas(&mut self, zoom: Zoom, held: (i32, i32)) -> Option<Camera> {
        let moved = self.camera.zoomed_to(zoom, held)?;
        self.camera = moved;
        self.surfaces.popups.camera = moved;
        Some(moved)
    }
}
