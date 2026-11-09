//! The halves Left and Right give, and their refusals.

use alo_dividing::area::Size;
use smithay::utils::Rectangle;

use super::*;

fn any() -> Window {
    Window::any_size(WindowId::from_compositor(1))
}

fn at_least(width: u32, height: u32) -> Window {
    Window::at_least(WindowId::from_compositor(1), Size::of(width, height))
}

fn a_room(x: i32, y: i32, w: i32, h: i32) -> ADisplaysRoom {
    ADisplaysRoom {
        whole: Rectangle::new((x, y).into(), (w, h).into()),
        work: Rectangle::new((x, y).into(), (w, h).into()),
        band: 54,
    }
}

#[test]
fn left_is_the_left_half_of_the_work_area_under_the_band() {
    let half = the_half(a_room(0, 84, 1329, 788), OneSide::Left, any()).expect("a half");
    assert_eq!(
        half.outline,
        Rectangle::new((0, 84).into(), (664, 788).into())
    );
    let share = half.share.expect("room");
    assert_eq!(
        share.at,
        (0, 84 + 54),
        "the band sits at the top of the half"
    );
    assert_eq!(
        share.size,
        (664, 788 - 54),
        "the odd unit is the right half's, as the division cuts"
    );
}

#[test]
fn right_is_the_rest_and_the_two_meet() {
    let room = a_room(0, 84, 1329, 788);
    let left = the_half(room, OneSide::Left, any()).expect("a half");
    let right = the_half(room, OneSide::Right, any()).expect("a half");
    assert_eq!(
        right.outline.loc.x,
        left.outline.loc.x + left.outline.size.w
    );
    assert_eq!(
        left.outline.size.w + right.outline.size.w,
        1329,
        "the halves cover the work area and nothing outside it"
    );
}

#[test]
fn a_half_is_of_the_display_the_window_is_on() {
    let second = a_room(1440, 0, 1000, 800);
    let right = the_half(second, OneSide::Right, any()).expect("a half");
    assert_eq!(right.outline.loc.x, 1940);
    assert_eq!(
        right.share.expect("room").at,
        (1940, 54),
        "on the plane where that display is"
    );
}

#[test]
fn a_window_wider_than_the_half_is_refused_and_still_previewed() {
    let half = the_half(a_room(0, 0, 1000, 800), OneSide::Left, at_least(501, 100))
        .expect("an outline to show the refusal in");
    assert_eq!(half.share, Err(NoRoom::TooNarrow));
    assert_eq!(half.outline.size.w, 500);
}

#[test]
fn a_window_taller_than_the_half_under_its_band_is_refused() {
    let half =
        the_half(a_room(0, 0, 1000, 800), OneSide::Right, at_least(100, 747)).expect("an outline");
    assert_eq!(half.share, Err(NoRoom::TooShort));
    let fits =
        the_half(a_room(0, 0, 1000, 800), OneSide::Right, at_least(500, 746)).expect("an outline");
    assert!(fits.share.is_ok(), "exactly the half's size fits");
}

#[test]
fn a_work_area_with_no_half_to_give_has_no_outline() {
    assert_eq!(the_half(a_room(0, 0, 1, 800), OneSide::Right, any()), None);
    assert_eq!(the_half(a_room(0, 0, 1000, 54), OneSide::Left, any()), None);
    assert!(
        the_half(a_room(0, 0, 1000, 55), OneSide::Left, any()).is_some(),
        "one unit under the band is a window"
    );
}

#[test]
fn a_preview_says_which_half_or_that_the_window_cannot_fit() {
    let fits = Ok(Share {
        at: (0, 0),
        size: (1, 1),
    });
    assert_eq!(
        the_words_of(OneSide::Left, fits).says(),
        "Left half · preview"
    );
    assert_eq!(
        the_words_of(OneSide::Right, fits).says(),
        "Right half · preview"
    );
    assert_eq!(
        the_words_of(OneSide::Left, Err(NoRoom::TooNarrow)),
        alo_access::words::NO_ROOM_IN_HALF
    );
    assert_eq!(
        the_words_of(OneSide::Right, Err(NoRoom::TooShort)),
        alo_access::words::NO_ROOM_IN_HALF
    );
}

/// **The chord and the panel give the same half.** The chord divides a share
/// with the next window through `alo-dividing`; the panel's Left asks the same
/// crate's cut for one window alone, so on a display with nothing reserved the
/// two windows' left share is the panel's outline.
#[test]
fn the_panel_and_the_chord_agree_on_the_half() {
    let display = alo_dividing::Area::of(alo_dividing::Point::at(0, 0), Size::of(1329, 788))
        .expect("an area");
    let mut division = alo_dividing::Division::of(display);
    let second = Window::any_size(WindowId::from_compositor(2));
    division
        .divide_with_next(any(), Some(second), Side::Left)
        .expect("divided");
    let divided = division
        .share_of(WindowId::from_compositor(1))
        .expect("a share");
    let half = the_half(a_room(0, 0, 1329, 788), OneSide::Left, any()).expect("a half");
    assert_eq!(
        (half.outline.size.w, half.outline.size.h),
        (
            i32::try_from(divided.width()).expect("small"),
            i32::try_from(divided.height()).expect("small")
        )
    );
}
