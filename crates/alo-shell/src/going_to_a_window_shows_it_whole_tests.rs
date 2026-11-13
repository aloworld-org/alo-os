//! Where going to a window leaves the view, held to the walk of 2026-11-13: a
//! 1 280 × 800 screen, the Dock's top at 717, and Dolphin 760 × 586.

use super::*;

/// The walk's screen: 1 280 × 800, the Dock's top at 717.
const SCREEN: (i32, i32) = (1280, 800);

/// The walk's Dock band.
fn the_walks_dock() -> Rectangle<i32, Physical> {
    Rectangle::new((272, 717).into(), (736, 75).into())
}

/// Dolphin as the walk had it, 760 × 586, somewhere on the plane.
fn dolphin_at(x: i32, y: i32) -> Rectangle<i32, Logical> {
    Rectangle::new((x, y).into(), (760, 586).into())
}

/// Where `taken` lands on the screen when the view's corner is `corner`.
fn on_the_screen(taken: Rectangle<i32, Logical>, corner: (i32, i32)) -> Rectangle<i32, Logical> {
    Rectangle::new(
        (taken.loc.x - corner.0, taken.loc.y - corner.1).into(),
        taken.size,
    )
}

/// **The room is the screen down to the Dock's top**, and grows on the plane
/// as the person zooms out.
#[test]
fn the_room_is_the_screen_above_the_dock_on_the_plane() {
    let screen = Size::from(SCREEN);
    assert_eq!(
        the_room_on_the_plane(screen, Some(the_walks_dock()), 1.0),
        Some(Size::from((1280, 717)))
    );
    assert_eq!(
        the_room_on_the_plane(screen, Some(the_walks_dock()), 0.5),
        Some(Size::from((2560, 1434)))
    );
    assert_eq!(
        the_room_on_the_plane(screen, None, 1.0),
        Some(Size::from(SCREEN)),
        "with no Dock the room is the whole screen"
    );
    assert_eq!(the_room_on_the_plane(screen, None, 0.0), None);
}

/// **Going to Dolphin shows its band and all of it above the Dock**, from the
/// walk's figures — and the control: the old rule, the view's corner at the
/// window's, put the band off the top of the screen, as `w26-37` shows.
#[test]
fn going_to_dolphin_shows_its_band_and_all_of_it_above_the_dock() {
    let strip = ItsStrip::at(100);
    let room = the_room_on_the_plane(Size::from(SCREEN), Some(the_walks_dock()), 1.0)
        .unwrap_or_else(|| Size::from((0, 0)));
    let dolphin = dolphin_at(5260, 3107);
    let taken = strip.round(dolphin);
    let corner = where_the_view_looks(taken, room);
    assert!(
        Rectangle::from_size(room).contains_rect(on_the_screen(taken, corner)),
        "Dolphin and its band are not wholly above the Dock: {:?} in {room:?}",
        on_the_screen(taken, corner)
    );
    let the_old_corner = (dolphin.loc.x, dolphin.loc.y);
    assert!(
        on_the_screen(taken, the_old_corner).loc.y < 0,
        "the control: the old rule put the band off the top"
    );
}

/// **A window too big for the room has its band on the screen**, the gap
/// inside the room's top and leading edge.
#[test]
fn a_window_too_big_for_the_room_has_its_band_on_the_screen() {
    let strip = ItsStrip::at(100);
    let room = Size::from((1280, 717));
    let huge = Rectangle::new((400, 900).into(), (3000, 2000).into());
    let taken = strip.round(huge);
    let corner = where_the_view_looks(taken, room);
    let seen = on_the_screen(taken, corner);
    assert_eq!(seen.loc.x, THE_GAP);
    assert_eq!(seen.loc.y, THE_GAP, "the band is the gap below the top");
}
