//! The screen a popup is constrained to, in its parent's own units.

use super::*;
use smithay::utils::Point;

/// A parent at this point, in its own units.
fn parent_at(x: f64, y: f64) -> Point<f64, Logical> {
    Point::from((x, y))
}

/// A display of this size with its corner here, on the desk.
fn screen(at: (i32, i32), size: (i32, i32)) -> Rectangle<i32, Physical> {
    Rectangle::new(at.into(), size.into())
}

/// **A display at the desk's origin is bound exactly as it always was.**
///
/// The safeguard: before task 6 the bound was `(-origin.x, -origin.y)` with
/// the screen's size, and every single-display session is this case. If this
/// moves, every machine this lane can actually test has changed.
#[test]
fn the_main_screen_is_bound_exactly_as_before() {
    let (left, top, width, height) =
        the_screen_in_the_parents_units(screen((0, 0), (1920, 1080)), parent_at(300.0, 40.0), 1.0);
    assert!((left - -300.0).abs() < f64::EPSILON, "left moved: {left}");
    assert!((top - -40.0).abs() < f64::EPSILON, "top moved: {top}");
    assert!((width - 1920.0).abs() < f64::EPSILON);
    assert!((height - 1080.0).abs() < f64::EPSILON);
}

/// **A display away from the origin is bound to its own corner**, which is
/// the whole of task 6.
///
/// A parent at 2000 across sits 80 into a display that begins at 1920. The
/// screen's left edge is therefore 80 units to its *left* — a negative number
/// — and its right edge 1840 to its right. Bound against the old
/// `(-origin.x, …)` the menu would have been told its screen began 2000 units
/// to the left, which is the far edge of the *other* display.
#[test]
fn a_second_display_is_bound_to_its_own_corner() {
    let (left, top, width, height) = the_screen_in_the_parents_units(
        screen((1920, 0), (1920, 1080)),
        parent_at(2000.0, 10.0),
        1.0,
    );
    assert!((left - -80.0).abs() < f64::EPSILON, "left was {left}");
    assert!((top - -10.0).abs() < f64::EPSILON, "top was {top}");
    assert!((width - 1920.0).abs() < f64::EPSILON);
    assert!((height - 1080.0).abs() < f64::EPSILON);
    // And the right edge is where that display ends, not where the desk does.
    assert!(
        ((left + width) - 1840.0).abs() < f64::EPSILON,
        "the right edge was {}",
        left + width
    );
}

/// **A zoom divides the screen and its corner alike.**
///
/// The existing reason the size is divided — *a menu constrained against a
/// 1366-pixel output while zoomed out to 40 % has nearly 3,400 of its own
/// units of room* — applies to the corner for the same reason. Dividing one
/// and not the other would put the second display's edge in the wrong place
/// by exactly the zoom.
#[test]
fn a_zoom_divides_the_corner_as_well_as_the_size() {
    let zoom = 0.5;
    let (left, _, width, _) =
        the_screen_in_the_parents_units(screen((1920, 0), (1920, 1080)), parent_at(0.0, 0.0), zoom);
    assert!((width - 3840.0).abs() < f64::EPSILON, "width was {width}");
    assert!(
        (left - 3840.0).abs() < f64::EPSILON,
        "the corner was {left}"
    );
}

/// **A display above or left of the main screen has a negative corner**, and
/// nothing here assumes otherwise.
///
/// `Position` says outright that across is *negative to the left of the main
/// screen*. An arrangement with the second monitor on the left is ordinary
/// and the arithmetic must not have been written for one direction.
#[test]
fn a_display_left_of_the_main_screen_is_handled() {
    let (left, top, _, _) = the_screen_in_the_parents_units(
        screen((-1920, -200), (1920, 1080)),
        parent_at(-1000.0, -100.0),
        1.0,
    );
    assert!((left - -920.0).abs() < f64::EPSILON, "left was {left}");
    assert!((top - -100.0).abs() < f64::EPSILON, "top was {top}");
}

/// **A parent on the second display is given the second display.**
#[test]
fn the_screen_a_parent_is_on_is_the_one_chosen() {
    let desk = [
        screen((0, 0), (1920, 1080)),
        screen((1920, 0), (1920, 1080)),
    ];
    let chosen = the_screen_showing(&desk, Point::from((2500, 100))).expect("a screen");
    assert_eq!(
        chosen.loc.x, 1920,
        "the parent's own display was not chosen"
    );
}

/// **A parent off the desk still gets a screen**, because a menu has to open
/// somewhere and refusing would leave a client unable to show one.
#[test]
fn a_parent_off_the_desk_falls_back_to_the_first_screen() {
    let desk = [
        screen((0, 0), (1920, 1080)),
        screen((1920, 0), (1920, 1080)),
    ];
    let chosen = the_screen_showing(&desk, Point::from((-9000, -9000))).expect("a screen");
    assert_eq!(chosen.loc.x, 0, "the fallback was not the first screen");
}

/// **No screens at all is no answer**, which is the state before the first
/// frame and the one case where a popup is left unconstrained.
#[test]
fn no_screens_is_no_answer() {
    assert!(the_screen_showing(&[], Point::from((0, 0))).is_none());
}
