//! The room a window alo places lands in, held to the walk of 2026-11-12: a
//! 1 280 × 800 screen, the Dock along the bottom, Dolphin open at (260, 107)
//! 760 × 586, and Firefox opening 1 204 wide and taller than the screen.

use smithay::utils::{Logical, Point, Rectangle, Size};

use super::{
    is_partly_out_of, kept_inside, lies_wholly_in, the_room_the_controls_leave,
    the_room_the_dock_leaves, the_room_the_panel_leaves, the_size_that_fits,
};
use crate::a_window_and_its_strip::ItsStrip;

/// The screen the walk was on, at life size with the camera at the origin.
fn the_view() -> Rectangle<i32, Logical> {
    Rectangle::new((0, 0).into(), (1280, 800).into())
}

/// The Dock as the walk's screenshots show it: 736 wide, its top at 717.
fn the_dock() -> Rectangle<i32, Logical> {
    Rectangle::new((272, 717).into(), (736, 74).into())
}

/// Dolphin, where the walk had it.
fn dolphin() -> Rectangle<i32, Logical> {
    Rectangle::new((260, 107).into(), (760, 586).into())
}

/// Firefox as it opened: 1 204 wide and taller than the screen.
fn firefox() -> Size<i32, Logical> {
    Size::from((1204, 900))
}

fn strip() -> ItsStrip {
    ItsStrip::at(100)
}

/// **The room is the view down to the Dock's top**; with no Dock, or a Dock
/// that does not reach into the view, it is the whole view.
#[test]
fn the_room_is_the_view_above_the_dock() {
    assert_eq!(
        the_room_the_dock_leaves(the_view(), Some(the_dock())),
        Rectangle::new((0, 0).into(), (1280, 717).into())
    );
    assert_eq!(the_room_the_dock_leaves(the_view(), None), the_view());
    let elsewhere = Rectangle::new((272, 5000).into(), (736, 74).into());
    assert_eq!(
        the_room_the_dock_leaves(the_view(), Some(elsewhere)),
        the_view(),
        "a Dock off the view takes nothing"
    );
    // Panned: the view and the Dock on the plane move together.
    let panned = Rectangle::new((3000, -400).into(), (1280, 800).into());
    let its_dock = Rectangle::new((3272, 317).into(), (736, 74).into());
    assert_eq!(
        the_room_the_dock_leaves(panned, Some(its_dock)),
        Rectangle::new((3000, -400).into(), (1280, 717).into())
    );
}

/// **Firefox is asked to be a size that fits above the Dock with its strip
/// and the gap**, keeping its width, which fitted.
#[test]
fn a_window_taller_than_the_room_is_asked_to_fit() {
    let room = the_room_the_dock_leaves(the_view(), Some(the_dock()));
    let fits = the_size_that_fits(firefox(), room, strip(), Size::from((0, 0)));
    assert_eq!(fits.w, 1204, "the width fitted and is kept");
    assert!(fits.h < firefox().h, "{fits:?}");
    let placed = kept_inside(Point::from((16, 709)), fits, room, strip());
    assert!(
        lies_wholly_in(placed, fits, room, strip()),
        "at its offered size it lies wholly above the Dock: {placed:?} {fits:?}"
    );
    // The most that fits: one more row would not.
    let one_more = Size::from((fits.w, fits.h + 1));
    assert!(!lies_wholly_in(
        kept_inside(Point::from((16, 16)), one_more, room, strip()),
        one_more,
        room,
        strip()
    ));
}

/// **The refusal: never smaller than the application can be**, and a window
/// that fits is offered its own size.
#[test]
fn never_smaller_than_the_application_can_be() {
    let room = the_room_the_dock_leaves(the_view(), Some(the_dock()));
    let smallest = Size::from((300, 780));
    assert_eq!(
        the_size_that_fits(firefox(), room, strip(), smallest).h,
        780
    );
    let small = Size::from((457, 157));
    assert_eq!(
        the_size_that_fits(small, room, strip(), Size::from((0, 0))),
        small
    );
}

/// **The walk's second case: a window with its top at y ≈ 710 is moved up
/// until it is wholly above the Dock**, and one already inside is not moved.
#[test]
fn a_window_below_the_room_is_kept_inside_it() {
    let room = the_room_the_dock_leaves(the_view(), Some(the_dock()));
    let small = Size::from((457, 157));
    let moved = kept_inside(Point::from((16, 710)), small, room, strip());
    assert!(lies_wholly_in(moved, small, room, strip()), "{moved:?}");
    assert_eq!(moved.x, 16, "moved only as far as it must be");
    let inside = Point::from((400, 300));
    assert_eq!(kept_inside(inside, small, room, strip()), inside);
}

/// **A window too big for the room on a side is put at the room's start on
/// that side**, so its strip, where a person takes hold of it, is on the
/// screen.
#[test]
fn a_window_too_big_for_the_room_has_its_strip_on_the_screen() {
    let room = the_room_the_dock_leaves(the_view(), Some(the_dock()));
    let at = kept_inside(Point::from((16, 709)), firefox(), room, strip());
    let taken = strip().round(Rectangle::new(at, firefox()));
    assert_eq!(taken.loc.y, 16, "the strip's top one gap below the top");
}

/// **Grown partly out of the room is placed again; grown wholly out of the
/// view — the person panned away — is not.** The walk's second case: its top
/// at 710, 900 tall.
#[test]
fn partly_out_is_told_from_wholly_out() {
    let room = the_room_the_dock_leaves(the_view(), Some(the_dock()));
    assert!(is_partly_out_of(
        Point::from((16, 710)),
        firefox(),
        room,
        strip()
    ));
    assert!(
        !is_partly_out_of(Point::from((16, 5000)), firefox(), room, strip()),
        "a window the person panned away from is left"
    );
    assert!(!is_partly_out_of(
        Point::from((400, 300)),
        Size::from((457, 157)),
        room,
        strip()
    ));
}

/// **How the walk's first case happened, from the rules as they stood**: the
/// nearest empty space for a window taller than the view put its top in view
/// and its bottom off the screen, past the Dock — and a window offered the
/// size that fits lands wholly in the room it glides to.
#[test]
fn the_nearest_space_for_a_window_taller_than_the_view_left_it_off_the_screen() {
    let open = [strip().round(dolphin())];
    let taken = strip().round_a_size(firefox());
    let found = crate::the_nearest_empty_space::the_nearest_empty_space(
        the_view(),
        &open,
        taken,
        &[],
        None,
        alo_strings::Direction::LeftToRight,
    )
    .expect("a space");
    let seen_from = Rectangle::new(found.view, the_view().size);
    let window = Rectangle::new(found.window, taken);
    assert!(
        !seen_from.contains_rect(window),
        "the control: as the rule stood, {window:?} ran off the view {seen_from:?}"
    );
    let room = the_room_the_dock_leaves(the_view(), Some(the_dock()));
    let fits = the_size_that_fits(firefox(), room, strip(), Size::from((0, 0)));
    let found = crate::the_nearest_empty_space::the_nearest_empty_space(
        room,
        &open,
        strip().round_a_size(fits),
        &[],
        None,
        alo_strings::Direction::LeftToRight,
    )
    .expect("a space");
    let its_room = Rectangle::new(found.view, room.size);
    assert!(
        its_room.contains_rect(Rectangle::new(found.window, strip().round_a_size(fits))),
        "offered the size that fits, it lands wholly in the room the view glides to"
    );
}

/// The put-aside panel's column down the right of the walk's screen, the
/// width `crate::panel_raster` reserves at 100 per cent.
fn the_panel() -> Rectangle<i32, Logical> {
    Rectangle::new((1168, 0).into(), (112, 800).into())
}

/// **The room ends where the panel's column begins** — the owner's *"Windows
/// open clear of it"* of 2026-11-13 — and starts after it on a screen read
/// right to left. A column of no width, or one off the view, takes nothing.
#[test]
fn the_room_is_clear_of_the_panel() {
    assert_eq!(
        the_room_the_controls_leave(the_view(), Some(the_dock()), Some(the_panel())),
        Rectangle::new((0, 0).into(), (1168, 717).into())
    );
    let on_the_left = Rectangle::new((0, 0).into(), (112, 800).into());
    assert_eq!(
        the_room_the_panel_leaves(the_view(), Some(on_the_left)),
        Rectangle::new((112, 0).into(), (1168, 800).into())
    );
    assert_eq!(
        the_room_the_panel_leaves(the_view(), Some(Rectangle::default())),
        the_view(),
        "a column of no width takes nothing"
    );
    let elsewhere = Rectangle::new((9000, 0).into(), (112, 800).into());
    assert_eq!(
        the_room_the_panel_leaves(the_view(), Some(elsewhere)),
        the_view(),
        "a column off the view takes nothing"
    );
    // Panned: the view and the column on the plane move together.
    let panned = Rectangle::new((3000, -400).into(), (1280, 800).into());
    let its_panel = Rectangle::new((4168, -400).into(), (112, 800).into());
    assert_eq!(
        the_room_the_panel_leaves(panned, Some(its_panel)),
        Rectangle::new((3000, -400).into(), (1168, 800).into())
    );
}

/// **A window too wide for the room left of the panel is asked to be
/// narrower** and then lies wholly clear of it; the control is the same window
/// in the room the Dock alone leaves, which its 1 204 fitted.
#[test]
fn a_window_too_wide_for_the_room_left_of_the_panel_is_asked_to_shrink() {
    let wide = Size::from((1204, 500));
    let dock_only = the_room_the_dock_leaves(the_view(), Some(the_dock()));
    assert_eq!(
        the_size_that_fits(wide, dock_only, strip(), Size::from((0, 0))).w,
        1204,
        "the control: without the panel counted it fitted"
    );
    let room = the_room_the_controls_leave(the_view(), Some(the_dock()), Some(the_panel()));
    let fits = the_size_that_fits(wide, room, strip(), Size::from((0, 0)));
    assert_eq!(
        fits,
        Size::from((1168 - 32, 500)),
        "the width that fits, the height kept"
    );
    let placed = kept_inside(Point::from((300, 100)), fits, room, strip());
    assert!(
        lies_wholly_in(placed, fits, room, strip()),
        "at its offered size it lies wholly clear of the panel: {placed:?} {fits:?}"
    );
    assert!(placed.x + fits.w <= the_panel().loc.x);
}

/// **The refusal: never narrower than the application can be.** One that
/// cannot be narrow enough keeps its least width, and its strip is put at the
/// room's start so a person can still take hold of it — the panel covering
/// part of it is the application's limit, not alo's choice.
#[test]
fn never_narrower_than_the_application_can_be() {
    let room = the_room_the_controls_leave(the_view(), Some(the_dock()), Some(the_panel()));
    let smallest = Size::from((1180, 0));
    let fits = the_size_that_fits(Size::from((1204, 500)), room, strip(), smallest);
    assert_eq!(fits.w, 1180, "never below the application's minimum");
    let placed = kept_inside(Point::from((300, 100)), fits, room, strip());
    let taken = strip().round(Rectangle::new(placed, fits));
    assert_eq!(
        taken.loc.x, 16,
        "too wide for the room, it starts one gap in"
    );
}
