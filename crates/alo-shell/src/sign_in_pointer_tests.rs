//! What a pointer at the sign-in screen does, and what it refuses.
//!
//! **The refusals are the point of this file.** A press before anybody has
//! signed in can reach the power target, so a pointer that answered a
//! malformed event with a position is a machine somebody can turn off by
//! sending it nonsense.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a layout that would not lay out is the failure being reported"
)]

use super::ThePointerOnTheSignInScreen;
use crate::sign_in_layout::{SignInLayout, sign_in_laid_out};
use crate::sign_in_pointing::OnTheSignInScreen;
use crate::{DirectPointerEvent, InputError};
use smithay::backend::input::ButtonState;
use smithay::utils::{Logical, Point, Rectangle, Size};

/// The reference screen, and the extent a display of that size reports.
const SCREEN: (i32, i32) = (1440, 960);

/// Which rectangle of the layout a test is aiming at.
type Which = fn(&SignInLayout) -> Rectangle<i32, Logical>;

fn the_layout() -> SignInLayout {
    sign_in_laid_out(Size::from(SCREEN)).expect("the reference size lays out")
}

/// A relative nudge.
fn nudge(dx: f64, dy: f64) -> DirectPointerEvent {
    DirectPointerEvent::Relative { dx, dy, time: 1 }
}

/// An absolute position, as a fraction of the screen.
fn put_at(x: f64, y: f64) -> DirectPointerEvent {
    DirectPointerEvent::Absolute { x, y, time: 1 }
}

/// A pointer moved onto the middle of a rectangle of the layout.
fn onto(layout: &SignInLayout, which: Which) -> ThePointerOnTheSignInScreen {
    let target = which(layout);
    let mut pointer = ThePointerOnTheSignInScreen::default();
    let x = f64::from(target.loc.x + target.size.w / 2) / f64::from(SCREEN.0 - 1);
    let y = f64::from(target.loc.y + target.size.h / 2) / f64::from(SCREEN.1 - 1);
    pointer
        .moved(&put_at(x, y), SCREEN)
        .expect("a position inside the screen is taken");
    pointer
}

/// **A pointer no device has moved has no position**, so a press means nothing.
///
/// Without this a stray button event would land at (0, 0) — and the layout has
/// *Back to lock* at (32, 28), which is close enough to the corner that the
/// wrong answer would look plausible.
#[test]
fn a_pointer_nothing_has_moved_has_no_position_and_presses_nothing() {
    let pointer = ThePointerOnTheSignInScreen::default();
    assert_eq!(pointer.where_it_is(), None);
    assert_eq!(pointer.pressed(&the_layout()), None);
}

/// **The first delta is applied from the middle of the screen**, not from the
/// corner, so the arrow appears where a person will see it.
#[test]
fn the_first_delta_starts_from_the_middle_of_the_screen() {
    let mut pointer = ThePointerOnTheSignInScreen::default();
    assert!(
        pointer
            .moved(&nudge(0.0, 0.0), SCREEN)
            .expect("a well-formed delta is taken"),
        "a delta did not move the pointer"
    );
    let at = pointer.where_it_is().expect("a delta gives it a position");
    assert!(
        (at.x - 719.5).abs() < 0.001 && (at.y - 479.5).abs() < 0.001,
        "the first delta did not start from the middle: {at:?}"
    );
}

/// **A button, a scroll and a pinch move nothing, and none of them is an
/// error.** A trackpad at a sign-in screen produces all three.
#[test]
fn a_button_a_scroll_and_a_pinch_move_nothing_and_fail_at_nothing() {
    let mut pointer = ThePointerOnTheSignInScreen::default();
    for event in [
        DirectPointerEvent::Button {
            code: 0x110,
            state: ButtonState::Pressed,
            time: 1,
        },
        DirectPointerEvent::Axis(smithay::input::pointer::AxisFrame::new(1)),
        DirectPointerEvent::Pinch(crate::DirectPinch::Begin {
            fingers: 2,
            time: 1,
        }),
    ] {
        assert!(
            !pointer
                .moved(&event, SCREEN)
                .expect("a button, a scroll and a pinch are not refusals"),
            "something that is not motion claimed to have moved the pointer"
        );
        assert_eq!(
            pointer.where_it_is(),
            None,
            "something that is not motion gave the pointer a position"
        );
    }
}

/// **A malformed event is refused and leaves the pointer where it was.**
///
/// Three refusals, each the same one `Server::direct_pointer` makes: a screen
/// with no area, a delta that is not a number, and an absolute coordinate
/// outside the normalised range a backend promises.
#[test]
fn a_malformed_event_is_refused_and_moves_nothing() {
    let mut pointer = onto(&the_layout(), |l| l.entry);
    let was = pointer.where_it_is();

    assert!(
        matches!(
            pointer.moved(&nudge(1.0, 1.0), (0, 960)),
            Err(InputError::InvalidPointer)
        ),
        "a screen with no area was accepted"
    );
    assert!(
        matches!(
            pointer.moved(&nudge(f64::NAN, 0.0), SCREEN),
            Err(InputError::InvalidPointer)
        ),
        "a delta that is not a number was accepted"
    );
    assert!(matches!(
        pointer.moved(&nudge(0.0, f64::INFINITY), SCREEN),
        Err(InputError::InvalidPointer)
    ));
    for (x, y) in [(1.5, 0.5), (-0.1, 0.5), (0.5, 2.0)] {
        assert!(
            matches!(
                pointer.moved(&put_at(x, y), SCREEN),
                Err(InputError::InvalidPointer)
            ),
            "({x}, {y}) is outside 0..=1 and was accepted"
        );
    }

    assert_eq!(
        pointer.where_it_is(),
        was,
        "a refused event moved the pointer anyway"
    );
    // The control: the same pointer still takes a well-formed event, so the
    // refusals above are about the events and not a pointer that has stopped.
    assert!(
        pointer
            .moved(&nudge(1.0, 1.0), SCREEN)
            .expect("the same pointer still takes a well-formed event"),
        "a well-formed delta after a refusal moved nothing"
    );
}

/// **Motion is clamped to the screen**, so no press can land off it.
#[test]
fn motion_is_clamped_to_the_screen() {
    let mut pointer = ThePointerOnTheSignInScreen::default();
    pointer
        .moved(&nudge(-100_000.0, -100_000.0), SCREEN)
        .expect("a large delta is clamped, not refused");
    assert_eq!(
        pointer.where_it_is().expect("it has a position"),
        Point::from((0.0, 0.0))
    );
    pointer
        .moved(&nudge(100_000.0, 100_000.0), SCREEN)
        .expect("a large delta is clamped, not refused");
    assert_eq!(
        pointer.where_it_is().expect("it has a position"),
        Point::from((1439.0, 959.0))
    );
}

/// **Every control the design draws answers at its own middle.**
///
/// The one that matters most is Submit: the owner's direction puts the arrow
/// *inside* the entry, so a hit test that asked the entry first would never
/// report a submission at all.
#[test]
fn every_control_answers_at_its_own_middle() {
    let layout = the_layout();
    let each: [(OnTheSignInScreen, Which); 6] = [
        (OnTheSignInScreen::Submit, |l| l.submit),
        (OnTheSignInScreen::Back, |l| l.back),
        (OnTheSignInScreen::Power, |l| l.power),
        (OnTheSignInScreen::SwitchPerson, |l| l.switch_person),
        (OnTheSignInScreen::Accessibility, |l| l.accessibility),
        (OnTheSignInScreen::OtherWays, |l| l.other_ways),
    ];
    for (what, which) in each {
        assert_eq!(
            onto(&layout, which).pressed(&layout),
            Some(what),
            "{what:?} does not answer at its own middle"
        );
    }

    // The entry answers at its own left, away from the submit target inside it.
    let mut pointer = ThePointerOnTheSignInScreen::default();
    pointer
        .moved(
            &put_at(
                f64::from(layout.entry.loc.x + 4) / f64::from(SCREEN.0 - 1),
                f64::from(layout.entry.loc.y + layout.entry.size.h / 2) / f64::from(SCREEN.1 - 1),
            ),
            SCREEN,
        )
        .expect("a position inside the screen is taken");
    assert_eq!(pointer.pressed(&layout), Some(OnTheSignInScreen::TheEntry));
}

/// **A press on the ground is on nothing**, and in particular is not on the
/// power target.
#[test]
fn a_press_on_the_ground_is_on_nothing() {
    let layout = the_layout();
    let mut pointer = ThePointerOnTheSignInScreen::default();
    for (x, y) in [(0.01, 0.5), (0.5, 0.05), (0.99, 0.05), (0.5, 0.99)] {
        pointer
            .moved(&put_at(x, y), SCREEN)
            .expect("a position inside the screen is taken");
        assert_eq!(
            pointer.pressed(&layout),
            None,
            "({x}, {y}) is the screen's ground and answered a control"
        );
    }
}
