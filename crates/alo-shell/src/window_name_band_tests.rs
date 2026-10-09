//! Where the band is, and the two numbers that must not drift apart.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;
use crate::canvas_never_lost::A_USABLE_HANDLE;

/// A frame of this size at the origin.
fn frame(w: i32, h: i32) -> Rectangle<i32, Logical> {
    Rectangle::new((0, 0).into(), Size::from((w, h)))
}

/// **The band is 48 tall and as wide as the frame.**
///
/// Width matters as much as height: *the name is also what it is dragged by*,
/// so a band narrower than the frame would leave part of the top edge looking
/// like a handle and not being one.
#[test]
fn the_band_is_the_width_of_the_frame_and_forty_eight_tall() {
    let band = band_of(frame(936, 572)).expect("an ordinary frame has a band");
    assert_eq!(band.size.w, 936);
    assert_eq!(band.size.h, 48);
    assert_eq!(
        band.loc,
        frame(936, 572).loc,
        "the band is at the frame's top"
    );
}

/// **48 and 24 are different numbers, and this is what keeps them apart.**
///
/// The comment on `A_USABLE_HANDLE` called 24 *a frame's name band at the
/// height it is drawn* until 2026-10-08. If somebody ever makes that true by
/// changing the band instead of the comment, this fails.
#[test]
fn the_band_is_not_the_minimum_that_must_stay_reachable() {
    assert_eq!(BAND_IS_TALL, 48);
    assert_eq!(A_USABLE_HANDLE.1, 24.0);
    assert!(
        f64::from(BAND_IS_TALL) > A_USABLE_HANDLE.1,
        "the band must be taller than the piece of it that has to stay uncovered"
    );
}

/// **The two files hold one fact.** `MUST_STAY_REACHABLE` is written here as
/// whole pixels and in `canvas_never_lost` as floats; this is what makes a
/// change to either show up as a failure rather than as a disagreement nobody
/// reads.
#[test]
fn the_reachable_minimum_is_the_same_in_both_files() {
    assert_eq!(f64::from(MUST_STAY_REACHABLE.0), A_USABLE_HANDLE.0);
    assert_eq!(f64::from(MUST_STAY_REACHABLE.1), A_USABLE_HANDLE.1);
}

/// **A frame too short for a grabbable band gets none.**
///
/// Drawing one would be drawing a handle a person cannot use. The band follows
/// the frame's height while there is room, so a 30-tall frame has a 30-tall
/// band — which is still over the 24 that must stay reachable.
#[test]
fn a_frame_too_small_to_hold_a_handle_has_no_band() {
    assert!(
        band_of(frame(936, 30)).is_some(),
        "30 is over the 24 minimum"
    );
    let short = band_of(frame(936, 20));
    assert!(short.is_none(), "{short:?} is a handle nobody can grab");
    let narrow = band_of(frame(40, 572));
    assert!(
        narrow.is_none(),
        "{narrow:?} is narrower than a pointer can hit"
    );
}

/// **A band never grows past its frame.** A 30-tall frame has a 30-tall band
/// and not a 48-tall one hanging below it.
#[test]
fn the_band_never_overhangs_the_frame() {
    for height in [25, 30, 47, 48, 49, 572] {
        let band = band_of(frame(936, height)).expect("tall enough for a handle");
        assert!(
            band.size.h <= height,
            "a {height}-tall frame got a {}-tall band",
            band.size.h
        );
        assert_eq!(band.size.h, BAND_IS_TALL.min(height));
    }
}
