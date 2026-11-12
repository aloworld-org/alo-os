//! A list opened from the Dock stays on the display and out of the panel's
//! column, on either side of the display.

use super::*;

/// A column 100 wide at the right of a 1440-wide display.
fn on_the_right() -> Option<Rectangle<i32, Physical>> {
    Some(Rectangle::new((1340, 0).into(), (100, 960).into()))
}

#[test]
fn a_list_that_fits_is_left_where_it_asked_to_be() {
    let at = kept_on_the_display((200, 300), (448, 200), (1440, 960), on_the_right(), 12);
    assert_eq!((at.loc.x, at.loc.y), (200, 300));
}

#[test]
fn a_list_that_would_cover_the_panel_is_moved_clear_of_it() {
    let at = kept_on_the_display((1200, 300), (448, 200), (1440, 960), on_the_right(), 12);
    assert_eq!(at.loc.x + at.size.w, 1340 - 12);
}

#[test]
fn a_panel_on_the_left_pushes_the_list_the_other_way() {
    let column = Some(Rectangle::new((0, 0).into(), (100, 960).into()));
    let at = kept_on_the_display((-40, 300), (448, 200), (1440, 960), column, 12);
    assert_eq!(at.loc.x, 112);
}

#[test]
fn a_list_is_never_above_or_below_the_display() {
    let up = kept_on_the_display((10, -50), (448, 200), (1440, 960), None, 12);
    assert_eq!(up.loc.y, 0);
    let down = kept_on_the_display((10, 900), (448, 200), (1440, 960), None, 12);
    assert_eq!(down.loc.y + down.size.h, 960);
}
