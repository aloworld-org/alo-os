//! What a band and some fixed controls come to, with no compositor involved.
//!
//! **The arithmetic was only reachable through a Wayland fixture, and every frame
//! in those fixtures is sixteen pixels wide.** So the interesting cases — 43
//! pixels against 45, two controls splitting a name into halves that are each
//! useless — could not be written at all, and the first version of the rule
//! failed three tests at once for a reason none of them could show: a 44-pixel
//! handle does not fit in a 16-pixel band.
//!
//! A band and a list of rectangles is not a fact about Wayland.

use super::{enough_of_it_is_reachable, the_longest_reachable_run};
use smithay::utils::{Physical, Rectangle};

/// A control crossing a band of this height, from `left` for `width`.
fn across(left: i32, width: i32) -> Rectangle<i32, Physical> {
    Rectangle::new((left, -10).into(), (width, 100).into())
}

/// A 300-pixel name band at the origin, 48 tall — a frame of ordinary size.
const BAND: (f64, f64, f64, f64) = (0.0, 0.0, 300.0, 48.0);

/// The handle at the ordinary text scale.
const HANDLE: (f64, f64) = (44.0, 24.0);

/// **43 pixels is not a handle and 45 is**, either side of the design's 44.
///
/// This is the pair that separates the new rule from the old one. *Entirely under
/// the dock* is satisfied by a single visible pixel, so a test that only checked
/// the fully-covered case would pass the rule this replaces unchanged.
#[test]
fn a_sliver_is_refused_and_a_usable_run_is_allowed() {
    for (clear, reachable) in [(43, false), (44, true), (45, true)] {
        let control = across(clear, 1000);
        assert_eq!(
            enough_of_it_is_reachable(BAND, &[control], HANDLE),
            reachable,
            "{clear} pixels clear should be reachable={reachable}"
        );
    }
}

/// **Two controls can leave no handle while leaving plenty of name.**
///
/// The case a check that summed the leftovers would pass. Forty pixels clear at
/// each end is eighty pixels of name and nothing a person can grab.
#[test]
fn the_longest_run_is_measured_rather_than_the_total() {
    // Clear 0..40, control 40..260, clear 260..300 — two runs of 40, total 80.
    let control = across(40, 220);
    assert_eq!(the_longest_reachable_run(BAND, &[control]), 40.0);
    assert!(
        !enough_of_it_is_reachable(BAND, &[control], HANDLE),
        "eighty pixels of name in two useless halves was called reachable"
    );
}

/// **Overlapping controls are one obstruction, not two.**
///
/// Two controls covering 40..200 and 100..260 leave 0..40 and 260..300. A reader
/// that subtracted each control's width in turn would take 320 pixels from a
/// 300-pixel band and conclude something absurd.
#[test]
fn overlapping_controls_do_not_double_count() {
    let run = the_longest_reachable_run(BAND, &[across(40, 160), across(100, 160)]);
    assert_eq!(run, 40.0, "the clear runs are 0..40 and 260..300");
}

/// **A control that only clips the top of a band hides nothing.**
///
/// It leaves a shorter but still grabbable strip, and calling that hidden would
/// refuse drags a person can plainly make.
#[test]
fn a_control_that_does_not_cross_the_whole_band_takes_nothing() {
    // Covers y 0..10 of a band running 0..48: the band is still 38 tall beneath it.
    let clipping = Rectangle::new((0, 0).into(), (1000, 10).into());
    assert_eq!(the_longest_reachable_run(BAND, &[clipping]), 300.0);
    assert!(enough_of_it_is_reachable(BAND, &[clipping], HANDLE));
}

/// **The handle a frame owes is capped by the band it has.**
///
/// A 16-pixel name cannot offer 44 and is not therefore unreachable — it is small.
/// Its whole name is its handle, so all 16 must stay clear and 15 must not.
#[test]
fn a_band_narrower_than_a_handle_owes_only_what_it_has() {
    let narrow = (0.0, 0.0, 16.0, 48.0);
    assert!(
        enough_of_it_is_reachable(narrow, &[], HANDLE),
        "a 16-pixel band with nothing over it was called unreachable"
    );
    // One pixel taken from sixteen is no longer the whole of it.
    assert!(!enough_of_it_is_reachable(
        narrow,
        &[across(15, 100)],
        HANDLE
    ));
    // And covered entirely, plainly not.
    assert!(!enough_of_it_is_reachable(
        narrow,
        &[across(-5, 100)],
        HANDLE
    ));
}

/// **No controls is always reachable**, and a band of no width is not this rule's
/// to refuse.
///
/// The refusal path: a frame drawn at no width has nothing to protect, and
/// stopping its drag would be refusing a gesture for a reason nobody could see.
#[test]
fn nothing_over_a_band_and_a_band_of_nothing() {
    assert!(enough_of_it_is_reachable(BAND, &[], HANDLE));
    assert_eq!(the_longest_reachable_run(BAND, &[]), 300.0);
    for degenerate in [
        (0.0, 0.0, 0.0, 48.0),
        (0.0, 0.0, -5.0, 48.0),
        (f64::NAN, 0.0, 300.0, 48.0),
        (0.0, f64::INFINITY, 300.0, 48.0),
    ] {
        assert!(
            enough_of_it_is_reachable(degenerate, &[across(-100, 10_000)], HANDLE),
            "{degenerate:?} was refused, and a band that is not a band is not hidden"
        );
    }
}

/// **A larger text scale asks for a larger run**, which is the accessibility half.
#[test]
fn a_larger_handle_refuses_what_a_smaller_one_allowed() {
    let control = across(50, 1000);
    assert!(
        enough_of_it_is_reachable(BAND, &[control], (44.0, 24.0)),
        "fifty pixels clear satisfies a forty-four pixel handle"
    );
    assert!(
        !enough_of_it_is_reachable(BAND, &[control], (88.0, 48.0)),
        "fifty pixels clear should not satisfy an eighty-eight pixel handle"
    );
}
