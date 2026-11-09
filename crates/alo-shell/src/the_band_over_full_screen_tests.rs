//! When a full-screen window's band comes out, stays and goes.

use smithay::utils::{Point, Rectangle};

use super::*;

/// A window filling a 1440 × 960 display.
fn filling() -> Rectangle<i32, Logical> {
    Rectangle::new((0, 0).into(), (1440, 960).into())
}

fn at(x: i32, y: i32) -> Option<Point<i32, Logical>> {
    Some(Point::from((x, y)))
}

#[test]
fn the_top_edge_brings_the_band_out() {
    assert!(is_the_band_out(false, filling(), at(700, 0), false));
    assert!(is_the_band_out(false, filling(), at(700, 11), false));
    assert!(
        !is_the_band_out(false, filling(), at(700, 12), false),
        "below the zone the band stays away, so the application keeps its own top"
    );
}

#[test]
fn once_out_the_whole_band_keeps_it_and_leaving_it_puts_it_away() {
    assert!(is_the_band_out(true, filling(), at(700, 53), false));
    assert!(
        !is_the_band_out(true, filling(), at(700, 54), false),
        "the pointer left the band: no wait, it goes"
    );
}

#[test]
fn a_pointer_beside_the_window_brings_nothing_out() {
    let half = Rectangle::new((0, 0).into(), (720, 960).into());
    assert!(!is_the_band_out(false, half, at(900, 0), false));
}

#[test]
fn the_keyboard_brings_it_out_and_keeps_it() {
    assert!(is_the_band_out(false, filling(), None, true));
    assert!(is_the_band_out(true, filling(), at(700, 500), true));
}

#[test]
fn no_pointer_and_no_keyboard_is_no_band() {
    assert!(!is_the_band_out(true, filling(), None, false));
}
