//! What a press on the sign-in screen is on, and what it is not.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a layout that would not lay out is the failure being reported"
)]

use super::{OnTheSignInScreen, what_is_under_the_pointer};
use crate::sign_in_layout::{SignInLayout, THE_SMALLEST_SCREEN_IS, sign_in_laid_out};
use smithay::utils::{Logical, Point, Size};

fn laid_out(w: i32, h: i32) -> SignInLayout {
    sign_in_laid_out(Size::from((w, h))).expect("this screen lays out")
}

fn middle_of(r: smithay::utils::Rectangle<i32, Logical>) -> Point<i32, Logical> {
    Point::from((r.loc.x + r.size.w / 2, r.loc.y + r.size.h / 2))
}

/// **Every control answers at its own middle**, which is the least a hit test
/// must do and the thing a wrong rectangle breaks first.
#[test]
fn every_control_answers_at_its_middle() {
    let at = laid_out(1440, 960);
    for (what, rectangle) in [
        (OnTheSignInScreen::Submit, at.submit),
        (OnTheSignInScreen::Back, at.back),
        (OnTheSignInScreen::Power, at.power),
        (OnTheSignInScreen::SwitchPerson, at.switch_person),
        (OnTheSignInScreen::Accessibility, at.accessibility),
    ] {
        assert_eq!(
            what_is_under_the_pointer(&at, middle_of(rectangle)),
            Some(what),
            "{what:?} does not answer at its own middle"
        );
    }
}

/// **The submit target wins over the entry it sits inside.**
///
/// A press at the arrow is a submission and not a request to type. The target
/// is wholly inside the entry, so an order that asked the entry first would
/// never reach the arrow at all — and the screen would look right and submit
/// nothing.
#[test]
fn the_submit_target_wins_over_the_entry_it_sits_inside() {
    let at = laid_out(1440, 960);
    assert_eq!(
        at.entry.intersection(at.submit),
        Some(at.submit),
        "the fixture's submit target is not inside the entry, so this proves nothing"
    );
    assert_eq!(
        what_is_under_the_pointer(&at, middle_of(at.submit)),
        Some(OnTheSignInScreen::Submit)
    );
    // And the entry still answers where the target is not.
    let left_of_it = Point::from((at.entry.loc.x + 4, at.entry.loc.y + at.entry.size.h / 2));
    assert_eq!(
        what_is_under_the_pointer(&at, left_of_it),
        Some(OnTheSignInScreen::TheEntry)
    );
}

/// **The screen itself is not a control.**
///
/// A press on the wallpaper must answer nothing, or every press anywhere would
/// do something — which is how a person dismisses a screen by accident.
#[test]
fn a_press_on_the_wallpaper_is_on_nothing() {
    let at = laid_out(1440, 960);
    for point in [(4, 4), (1430, 500), (720, 60), (200, 700)] {
        assert_eq!(
            what_is_under_the_pointer(&at, Point::from(point)),
            None,
            "{point:?} answered a control and should be the screen"
        );
    }
}

/// **The arrow is pressable at every size that lays out**, because the target
/// is 44 × 44 on all of them.
#[test]
fn the_arrow_can_be_pressed_on_every_screen() {
    for (w, h) in [
        THE_SMALLEST_SCREEN_IS,
        (480, 400),
        (640, 480),
        (1366, 768),
        (1440, 960),
        (2560, 1440),
    ] {
        let at = laid_out(w, h);
        assert_eq!(
            what_is_under_the_pointer(&at, middle_of(at.submit)),
            Some(OnTheSignInScreen::Submit),
            "the arrow cannot be pressed on {w}x{h}"
        );
    }
}

/// **Accessibility is reachable before anybody has signed in**, which is the
/// owner's direction and the reason it is on this screen at all.
#[test]
fn accessibility_is_reachable_on_every_screen() {
    for (w, h) in [THE_SMALLEST_SCREEN_IS, (640, 480), (1440, 960)] {
        let at = laid_out(w, h);
        assert_eq!(
            what_is_under_the_pointer(&at, middle_of(at.accessibility)),
            Some(OnTheSignInScreen::Accessibility),
            "accessibility cannot be reached on {w}x{h}"
        );
    }
}

/// **No two controls claim one point**, which is what a half-open rule is for.
#[test]
fn no_point_belongs_to_two_controls() {
    let at = laid_out(1440, 960);
    let all = [
        at.submit,
        at.back,
        at.power,
        at.switch_person,
        at.accessibility,
        at.other_ways,
    ];
    for (i, one) in all.iter().enumerate() {
        for other in all.iter().skip(i + 1) {
            assert_eq!(
                one.intersection(*other),
                None,
                "two controls overlap: {one:?} and {other:?}"
            );
        }
    }
}

/// **A press just outside a control is not on it.**
///
/// The half-open rule: the far edge belongs to whatever is beyond it, so two
/// rectangles that touch do not both answer.
#[test]
fn the_far_edge_of_a_control_is_not_the_control() {
    let at = laid_out(1440, 960);
    let past = Point::from((
        at.submit.loc.x + at.submit.size.w,
        at.submit.loc.y + at.submit.size.h / 2,
    ));
    assert_ne!(
        what_is_under_the_pointer(&at, past),
        Some(OnTheSignInScreen::Submit),
        "the pixel past the submit target still answered as the target"
    );
}
