//! That the room *Show all* fits into gives way to each fixed control on the
//! side it sits on, and that the margin is taken off every side or not at all.

use smithay::utils::{Physical, Rectangle};

use super::{inside_the_margin, the_room_left_clear};

/// A rectangle, written the way the draw lays one out.
fn at(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Physical> {
    Rectangle::new((x, y).into(), (w, h).into())
}

/// The screen the booted machine showed the fault on.
const SCREEN: (i32, i32) = (1280, 800);

fn the_screen() -> Rectangle<i32, Physical> {
    at(0, 0, SCREEN.0, SCREEN.1)
}

/// **A Dock on the bottom edge leaves the room above it** — the case the
/// booted machine showed, where the fit ran a window under the Dock.
#[test]
fn a_dock_on_the_bottom_leaves_the_room_above_it() {
    // The Dock's band is narrower than the screen, as `dock_raster` draws one.
    let dock = at(250, 720, 780, 80);
    assert_eq!(
        the_room_left_clear(the_screen(), &[dock]),
        at(0, 0, 1280, 720)
    );
}

/// **A Dock on a side edge leaves the room beside it**, so nothing here knows
/// which edge the person chose.
#[test]
fn a_dock_on_the_left_leaves_the_room_to_its_right() {
    let dock = at(0, 100, 80, 600);
    assert_eq!(
        the_room_left_clear(the_screen(), &[dock]),
        at(80, 0, 1200, 800)
    );
}

/// **Every control gives way, not only the Dock**: the panel's column on the
/// right and the top controls take their sides too.
#[test]
fn the_dock_the_panel_and_the_top_controls_each_take_their_side() {
    let dock = at(250, 720, 780, 80);
    let panel = at(1168, 0, 112, 800);
    let top = at(0, 0, 1280, 40);
    assert_eq!(
        the_room_left_clear(the_screen(), &[dock, panel, top]),
        at(0, 40, 1168, 680)
    );
}

/// **A control that does not reach the room changes nothing**, and a control
/// of no extent — an empty panel — is one of those.
#[test]
fn a_control_off_the_room_or_of_no_extent_changes_nothing() {
    let off = at(2000, 2000, 50, 50);
    let empty = at(600, 400, 0, 0);
    assert_eq!(
        the_room_left_clear(the_screen(), &[off, empty]),
        the_screen()
    );
}

/// **A control over the whole room leaves the room as it was** rather than
/// leaving nothing to fit into.
#[test]
fn a_control_over_everything_leaves_the_room_as_it_was() {
    assert_eq!(
        the_room_left_clear(the_screen(), &[at(-10, -10, 2000, 2000)]),
        the_screen()
    );
}

/// **The margin comes off all four sides.**
#[test]
fn the_margin_comes_off_every_side() {
    assert_eq!(
        inside_the_margin(at(0, 40, 1168, 680), 32),
        Some(at(32, 72, 1104, 616))
    );
}

/// **A room too small for its margins is refused, not turned inside out.**
#[test]
fn a_room_too_small_for_its_margin_is_refused() {
    assert_eq!(inside_the_margin(at(0, 0, 64, 300), 32), None);
    assert_eq!(inside_the_margin(at(0, 0, 300, 63), 32), None);
    assert_eq!(inside_the_margin(at(0, 0, 300, 300), i32::MAX), None);
}
