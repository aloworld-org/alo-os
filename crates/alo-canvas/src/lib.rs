//! Where a frame sits on a plane a person can move and zoom.
//!
//! `docs/autonomy/the-smallest-canvas-worth-showing.md` task 1: *a plane that
//! moves under a viewport that does not.* The one sentence the canvas has to
//! earn is **apps float on a surface you can move and zoom, and zooming out
//! shows you everything at once**, and this crate is the arithmetic underneath
//! it.
//!
//! # Two layers, and only one of them moves
//!
//! There is a **plane**, which carries the frames a person's applications are
//! in, and a **viewport**, which carries the dock, the status area, the
//! indicators and a window's controls. A pan or a zoom moves the plane under
//! the viewport. It does not move the viewport, and the viewport does not
//! correct for it.
//!
//! That is the whole architecture, and the plan says it is the part expensive to
//! retrofit. The constraint it comes with is the test of whether it was done
//! right: **nothing in the viewport layer may read the camera to correct itself.
//! If it has to, it is in the wrong layer.** This crate is therefore the only
//! place a [`Camera`] is needed, and a viewport surface that imported one would
//! be the mistake showing itself.
//!
//! # Why the zoom is a ratio and not a float
//!
//! A click at a screen point has to arrive at the right coordinate inside an
//! application's surface, and it has to be right at 40 % and at 250 % (task 2).
//! With `f64` the same point can land on two different pixels depending on the
//! order the sums were done, and *nearly the right pixel* in a drawing program is
//! a wrong pixel.
//!
//! So a zoom is **thousandths, held as an integer**: 40 % is `400`, 250 % is
//! `2500`. Every conversion is integer multiplication and one division, which
//! gives the same answer on every machine and every run. `alo-dividing` reached
//! the same conclusion about shares for the same reason, and its note says it in
//! as many words: *there is no rounding anywhere in the division itself that
//! could open a gap*.
//!
//! # What a plane coordinate is
//!
//! A **logical unit**, signed. Signed because a person may pan the plane so that
//! a frame sits left of or above where they started, and a canvas whose origin
//! were a wall would be a canvas with a corner — which is not the thing being
//! built. Logical because a scale turns units into pixels at the drawing
//! boundary, exactly as it does for a division, so a frame is the same size on a
//! laptop at 150 % as on an external screen at 100 %.

#![cfg_attr(not(test), forbid(unsafe_code))]

pub mod camera;
pub mod plane;

pub use camera::{Camera, NotZoomed, Zoom};
pub use plane::{At, Frame, Size, Span};
