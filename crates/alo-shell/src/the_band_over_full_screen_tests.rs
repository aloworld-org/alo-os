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

/// **A band out over full screen starts after the egress indicator**, so the
/// indicator never sits on the title — and hangs one band down over the
/// window's top, its trailing end where it was.
///
/// The indicator is where a 1280 × 800 machine drew it on 2026-11-09: a circle
/// 36 across, 20 in and 14 down, over the title of a full-screen Dolphin.
#[test]
fn the_band_starts_after_the_indicator() {
    let window = Rectangle::new((0, 0).into(), (1280, 800).into());
    let indicator = Rectangle::new((20, 14).into(), (36, 36).into());
    let hung = hung_over_its_top(window, &[indicator]);
    assert_eq!(hung.loc, Point::from((56, 54)));
    assert_eq!(
        hung.loc.x + hung.size.w,
        1280,
        "the controls stay at the trailing end"
    );
    assert_eq!(hung.size.h, 800 - 54);

    let below = Rectangle::new((20, 400).into(), (36, 36).into());
    assert_eq!(
        hung_over_its_top(window, &[below]).loc,
        Point::from((0, 54)),
        "an indicator the band does not meet moves nothing"
    );
    assert_eq!(hung_over_its_top(window, &[]).loc, Point::from((0, 54)));
}
