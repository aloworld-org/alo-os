//! Putting the camera somewhere exact, and working out where a frame appears.
//!
//! Shared by the canvas tasks' own tests — where a press lands, and how far a
//! drag moves a frame — because both ask the same two questions of the same
//! camera and a second copy of this arithmetic is a second answer to it.
#![expect(
    clippy::expect_used,
    reason = "an unexpected None or Err here is the failure the test reports"
)]

use super::Fixture;
use alo_canvas::{At, Zoom};

/// The zooms the canvas plan names, and life size between them.
pub const THREE_ZOOMS: [u32; 3] = [400, 1000, 2500];

/// Two pans, one of them off the plane's origin in both directions.
pub const TWO_PANS: [(i32, i32); 2] = [(0, 0), (-37, 64)];

/// Look at the plane from exactly here, at exactly this zoom.
///
/// The zoom is set while the camera is at the origin and held at the viewport's
/// own corner, so the pointer-centred arithmetic leaves the camera where it is;
/// the pan is then set outright. Both are read back, because a camera that
/// refused one of the two would quietly make every case a case about life size.
pub fn looking(f: &Fixture, at: (i32, i32), zoom: u32) {
    f.backend(move |s| {
        assert!(s.look_at_the_canvas(At::origin()).is_some());
        let zoom = Zoom::of(zoom).expect("a zoom inside the canvas's own bounds");
        assert!(s.zoom_the_canvas(zoom, (0, 0)).is_some());
        let at = At::checked(at.0, at.1).expect("a place on the plane");
        assert!(s.look_at_the_canvas(at).is_some());
        assert_eq!(s.the_camera().zoom(), zoom, "the camera refused the zoom");
        assert_eq!(s.the_camera().at(), at, "the camera refused the pan");
    });
}

/// Where a point of a frame at `origin` appears on the screen, under this camera.
///
/// Written out here rather than asked of the compositor: this is the sentence a
/// person experiences — *the middle of that window is there on the glass* — and a
/// test is worth nothing if it borrows the answer from the code it checks.
pub fn on_the_screen(
    point: (f64, f64),
    origin: (i32, i32),
    at: (i32, i32),
    zoom: u32,
) -> (f64, f64) {
    let scale = f64::from(zoom) / 1000.0;
    (
        (point.0 + f64::from(origin.0) - f64::from(at.0)) * scale,
        (point.1 + f64::from(origin.1) - f64::from(at.1)) * scale,
    )
}

/// Move the pointer there, in the screen's own pixels.
pub fn motion(f: &Fixture, (x, y): (f64, f64)) {
    assert!(f.backend(move |s| s.pointer_motion(x, y, 7)).is_ok());
}

/// A pointer coordinate a client was told, to within Wayland's own fixed point.
///
/// `wl_fixed` is 1/256 of a unit, so an exact comparison would be a test of the
/// order this crate's divisions happen in rather than of where the arrow was.
pub fn close_enough(got: (f64, f64), want: (f64, f64)) -> bool {
    const A_FRACTION_OF_A_UNIT: f64 = 2.0 / 256.0;
    (got.0 - want.0).abs() <= A_FRACTION_OF_A_UNIT && (got.1 - want.1).abs() <= A_FRACTION_OF_A_UNIT
}
