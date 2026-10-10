//! The camera conversion, which is the part of placement this file invented.
//!
//! [`crate::where_a_window_opens`] has its own tests for the owner's rules.
//! What those cannot see is that the rectangles reaching them are in the space
//! they expect — a Dock handed over in screen pixels is in the right place at
//! zoom 1 with the camera at the origin, which is every test's default and the
//! one case where the bug is invisible. So these move the camera.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::onto_the_plane;
use smithay::utils::{Physical, Rectangle};

/// A 1440×960 display's Dock: 64 thick, flush to the bottom edge.
fn a_dock() -> Rectangle<i32, Physical> {
    Rectangle::new((0, 896).into(), (1440, 64).into())
}

#[test]
fn at_rest_the_control_is_where_it_was_drawn() {
    // The camera at the plane's origin, life size. The one case where no
    // conversion is visible, which is why the cases below exist.
    let moved = onto_the_plane(a_dock(), alo_canvas::At { x: 0, y: 0 }, 1.0)
        .expect("a finite camera converts");
    assert_eq!((moved.loc.x, moved.loc.y), (0, 896));
    assert_eq!((moved.size.w, moved.size.h), (1440, 64));
}

#[test]
fn a_panned_camera_moves_the_control_with_the_plane() {
    // The person has panned 300 right and 200 down, so the plane point under
    // the viewport's top-left corner is (300, 200). The Dock has not moved on
    // the screen — it never does — so on the plane it is 300 and 200 further
    // along.
    let moved = onto_the_plane(a_dock(), alo_canvas::At { x: 300, y: 200 }, 1.0)
        .expect("a finite camera converts");
    assert_eq!(moved.loc.x, 300, "the Dock's left edge follows the camera");
    assert_eq!(moved.loc.y, 896 + 200, "and so does its top edge");
    assert_eq!(moved.size.w, 1440, "panning does not resize it");
    assert_eq!(moved.size.h, 64);
}

#[test]
fn zoomed_out_the_control_covers_more_of_the_plane() {
    // **The case that makes this function necessary.** Zoomed to half, the
    // whole display shows twice as much plane, so the Dock — unchanged on the
    // screen — lies across twice as much of it. A Dock handed over unconverted
    // would be 64 plane units here and windows would be placed under it.
    let moved = onto_the_plane(a_dock(), alo_canvas::At { x: 0, y: 0 }, 0.5)
        .expect("a finite camera converts");
    assert_eq!(
        moved.size.h, 128,
        "64 screen pixels is 128 plane units at half"
    );
    assert_eq!(moved.size.w, 2880);
    assert_eq!(moved.loc.y, 1792, "and it starts twice as far down");
}

#[test]
fn zoomed_in_the_control_covers_less_of_the_plane() {
    let moved = onto_the_plane(a_dock(), alo_canvas::At { x: 0, y: 0 }, 2.0)
        .expect("a finite camera converts");
    assert_eq!(moved.size.h, 32);
    assert_eq!(moved.loc.y, 448);
}

#[test]
fn the_two_conversions_compose_rather_than_one_hiding_the_other() {
    // Panned **and** zoomed, because an implementation that applied the zoom
    // to the offset as well passes both tests above and fails this one. The
    // offset is a plane quantity already and is added, never scaled.
    let moved = onto_the_plane(a_dock(), alo_canvas::At { x: 300, y: 200 }, 0.5)
        .expect("a finite camera converts");
    assert_eq!(moved.loc.x, 300, "the offset is added, not multiplied");
    assert_eq!(moved.loc.y, 896 * 2 + 200);
}

#[test]
fn a_camera_that_cannot_be_divided_by_is_refused_rather_than_panicking() {
    // Neither can arrive — `Zoom` is built from thousandths above zero — and
    // both are refused anyway. Law 3 makes no exception for a panic somebody
    // has reasoned is unreachable.
    for zoom in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            onto_the_plane(a_dock(), alo_canvas::At { x: 0, y: 0 }, zoom),
            None,
            "a zoom of {zoom} is refused"
        );
    }
}

#[test]
fn a_control_far_out_on_the_plane_does_not_wrap() {
    // A person who has panned a long way. The arithmetic is done in f64 and
    // rounded once, so a large offset is a large answer rather than a wrapped
    // one — which is what an i32 multiply-then-add would have given.
    let moved = onto_the_plane(
        a_dock(),
        alo_canvas::At {
            x: 2_000_000,
            y: 2_000_000,
        },
        1.0,
    )
    .expect("a finite camera converts");
    assert!(
        moved.loc.x > 1_900_000 && moved.loc.y > 1_900_000,
        "a distant camera gives a distant control, not a wrapped one: {moved:?}"
    );
}

#[test]
fn a_control_that_does_not_divide_evenly_is_rounded_outwards() {
    // **The rounding is a decision, so it has a case of its own.** At a third,
    // 64 screen pixels is 21.33 plane units. It is taken as 22, not 21: a
    // control is tested with *completely covered*, so a control recorded
    // smaller than it is would let a handle be called reachable underneath
    // one that in fact covers it. Every other case in this file divides
    // evenly and cannot tell the two apart.
    let moved = onto_the_plane(a_dock(), alo_canvas::At { x: 0, y: 0 }, 3.0)
        .expect("a finite camera converts");
    assert_eq!(moved.size.h, 22, "21.33 units of control is taken as 22");
    assert!(
        f64::from(moved.size.h) >= 64.0 / 3.0,
        "and never less than what it covers"
    );
}
