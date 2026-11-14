//! The three outcomes of a look, each from a machine that shows exactly what
//! the outcome needs.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::time::Duration;

use super::{Seen, Settling, look};
use crate::a_machine_of_pictures::PicturesAndLines;
use crate::frame::Frame;

/// A short rule: a picture a second, three matching, ten seconds at most.
const RULE: Settling = Settling {
    every: Duration::from_secs(1),
    matching: 3,
    noise: 2,
    at_most: Duration::from_secs(10),
};

/// The screen before the keys.
fn before() -> Frame {
    Frame::filled(8, 8, [0, 0, 0])
}

/// The screen after them.
fn after() -> Frame {
    Frame::filled(8, 8, [255, 255, 255])
}

/// A screen that changes, moves once more and then holds still has changed
/// and settled, and the look ends on the settled picture.
#[test]
fn a_screen_that_changes_and_holds_has_changed_and_settled() {
    let moving = after()
        .with_pixel(0, 0, [1, 1, 1])
        .with_pixel(1, 0, [1, 1, 1])
        .with_pixel(2, 0, [1, 1, 1]);
    let mut machine = PicturesAndLines::showing(vec![before(), moving, after(), after(), after()]);
    let looked = look(&mut machine, Some(&before()), RULE);
    let Ok(looked) = looked else {
        panic!("the look failed: {looked:?}");
    };
    assert_eq!(looked.seen, Seen::ChangedAndSettled);
    assert_eq!(looked.took, Duration::from_secs(5));
    assert_eq!(looked.last, after());
}

/// **The refusal to call nothing something**: a screen that never differs
/// from before is unchanged, after the whole time allowed.
#[test]
fn a_screen_that_never_differs_is_unchanged() {
    let mut machine = PicturesAndLines::showing(vec![before()]);
    let looked = look(&mut machine, Some(&before()), RULE);
    assert!(
        matches!(&looked, Ok(it) if it.seen == Seen::Unchanged && it.took == Duration::from_secs(10)),
        "{looked:?}"
    );
}

/// A caret's few pixels are noise: a screen that differs from before only by
/// them is unchanged.
#[test]
fn a_difference_within_the_noise_is_no_change() {
    let blinking = before().with_pixel(3, 3, [255, 255, 255]);
    let mut machine = PicturesAndLines::showing(vec![blinking.clone(), before(), blinking]);
    let looked = look(&mut machine, Some(&before()), RULE);
    assert!(
        matches!(&looked, Ok(it) if it.seen == Seen::Unchanged),
        "{looked:?}"
    );
}

/// A screen that keeps moving changed and never settled.
#[test]
fn a_screen_that_keeps_moving_never_settles() {
    let pictures: Vec<Frame> = (0..20_u8)
        .map(|shade| Frame::filled(8, 8, [shade.wrapping_mul(13), 200, 7]))
        .collect();
    let mut machine = PicturesAndLines::showing(pictures);
    let looked = look(&mut machine, Some(&before()), RULE);
    assert!(
        matches!(&looked, Ok(it) if it.seen == Seen::ChangedNeverSettled),
        "{looked:?}"
    );
}

/// With nothing before to compare, a look waits only for the screen to hold
/// still.
#[test]
fn with_nothing_before_a_look_waits_to_settle() {
    let mut machine = PicturesAndLines::showing(vec![before(), after(), after(), after()]);
    let looked = look(&mut machine, None, RULE);
    assert!(
        matches!(&looked, Ok(it) if it.seen == Seen::Settled && it.took == Duration::from_secs(4)),
        "{looked:?}"
    );
}
