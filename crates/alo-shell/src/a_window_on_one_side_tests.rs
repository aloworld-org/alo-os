//! The halves Left and Right give, and their refusals.

use smithay::utils::Rectangle;

use super::*;

fn a_room(x: i32, y: i32, w: i32, h: i32) -> ADisplaysRoom {
    ADisplaysRoom {
        whole: Rectangle::new((x, y).into(), (w, h).into()),
        work: Rectangle::new((x, y).into(), (w, h).into()),
        band: 54,
    }
}

#[test]
fn left_is_the_left_half_of_the_work_area_under_the_band() {
    let half = the_half(a_room(0, 84, 1329, 788), OneSide::Left, None).expect("a half");
    assert_eq!(
        half.outline,
        Rectangle::new((0, 84).into(), (665, 788).into())
    );
    let share = half.share.expect("room");
    assert_eq!(
        share.at,
        (0, 84 + 54),
        "the band sits at the top of the half"
    );
    assert_eq!(
        share.size,
        (665, 788 - 54),
        "the left half takes the odd unit"
    );
}

#[test]
fn right_is_the_rest_and_the_two_meet() {
    let room = a_room(0, 84, 1329, 788);
    let left = the_half(room, OneSide::Left, None).expect("a half");
    let right = the_half(room, OneSide::Right, None).expect("a half");
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
    let right = the_half(second, OneSide::Right, None).expect("a half");
    assert_eq!(right.outline.loc.x, 1940);
}

#[test]
fn a_window_wider_than_the_half_is_refused_and_still_previewed() {
    let half = the_half(a_room(0, 0, 1000, 800), OneSide::Left, Some((501, 100)))
        .expect("an outline to show the refusal in");
    assert_eq!(half.share, Err(NoRoom::TooNarrow));
    assert_eq!(half.outline.size.w, 500);
}

#[test]
fn a_window_taller_than_the_half_under_its_band_is_refused() {
    let half =
        the_half(a_room(0, 0, 1000, 800), OneSide::Right, Some((100, 747))).expect("an outline");
    assert_eq!(half.share, Err(NoRoom::TooShort));
    let fits =
        the_half(a_room(0, 0, 1000, 800), OneSide::Right, Some((500, 746))).expect("an outline");
    assert!(fits.share.is_ok(), "exactly the half's size fits");
}

#[test]
fn a_work_area_with_no_half_to_give_has_no_outline() {
    assert_eq!(the_half(a_room(0, 0, 1, 800), OneSide::Right, None), None);
    assert_eq!(the_half(a_room(0, 0, 1000, 54), OneSide::Left, None), None);
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
