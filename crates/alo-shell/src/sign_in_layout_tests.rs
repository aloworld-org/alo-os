//! The sign-in screen's geometry, held against `176-10686.xml`.
//!
//! **Every figure here is the capture's**, and the node id is in the assertion
//! that uses it so the next person checks the file rather than re-deriving the
//! number from a picture.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a layout that would not lay out is the failure being reported"
)]

use super::{
    THE_ARROW_IS, THE_ENTRY_IS, THE_GROUP_IS_TALL, THE_GROUP_IS_WIDE, THE_MONOGRAM_IS,
    THE_SMALLEST_SCREEN_IS, THE_SUBMIT_TARGET_IS, sign_in_laid_out,
};
use smithay::utils::{Logical, Size};

/// The size the design is drawn at.
fn the_reference() -> Size<i32, Logical> {
    Size::from((1440, 960))
}

fn laid_out(w: i32, h: i32) -> super::SignInLayout {
    sign_in_laid_out(Size::from((w, h))).expect("this screen is big enough to lay out")
}

/// **At the reference size, the group is where `176:10686` puts it.**
///
/// `364:25568` is the instance: 400 × 268 at (520, 266).
#[test]
fn the_group_is_where_the_file_puts_it_at_the_reference_size() {
    let at = laid_out(the_reference().w, the_reference().h);
    assert_eq!(at.group.size.w, THE_GROUP_IS_WIDE);
    assert_eq!(at.group.size.h, THE_GROUP_IS_TALL);
    assert_eq!(at.group.loc.x, 520, "centred on 1440 is 520");
    assert_eq!(at.group.loc.y, 266, "266 of 960, which is the capture's own");
}

/// **The monogram is 64 × 64 and centred over the greeting.**
///
/// `364:25558` at (168, 0) in a 400 group: `(400 - 64) / 2 = 168`.
#[test]
fn the_monogram_is_sixty_four_and_centred() {
    let at = laid_out(the_reference().w, the_reference().h);
    assert_eq!(at.monogram.size, Size::from((THE_MONOGRAM_IS, THE_MONOGRAM_IS)));
    assert_eq!(
        at.monogram.loc.x - at.group.loc.x,
        168,
        "the capture's own offset"
    );
    assert_eq!(at.monogram.loc.y, at.group.loc.y);
}

/// **The entry is 304 × 56 and the submit target is inside it.**
///
/// This is the one the owner's direction is most explicit about: *the submit
/// arrow belongs inside the entry field*. A layout that put it beside the
/// field would pass a size assertion and still be the old design.
#[test]
fn the_submit_target_is_inside_the_entry() {
    let at = laid_out(the_reference().w, the_reference().h);
    assert_eq!(at.entry.size, Size::from(THE_ENTRY_IS));
    assert_eq!(
        at.entry.loc.x - at.group.loc.x,
        48,
        "`360:25699` is at x=48 in a 400 group"
    );
    assert_eq!(at.entry.loc.y - at.group.loc.y, 144);

    assert_eq!(
        at.entry.intersection(at.submit),
        Some(at.submit),
        "the submit target is not wholly inside the entry: {:?} in {:?}",
        at.submit,
        at.entry
    );
    assert_eq!(
        at.submit.size,
        Size::from((THE_SUBMIT_TARGET_IS, THE_SUBMIT_TARGET_IS))
    );
    assert_eq!(
        at.submit.loc.x - at.entry.loc.x,
        253,
        "`364:25560` is at x=253 inside the entry"
    );
    assert_eq!(at.submit.loc.y - at.entry.loc.y, 6);

    // And the arrow inside the target.
    assert_eq!(at.arrow.size, Size::from((THE_ARROW_IS, THE_ARROW_IS)));
    assert_eq!(
        at.submit.intersection(at.arrow),
        Some(at.arrow),
        "the arrow is not inside its own target"
    );
    assert_eq!(at.arrow.loc.x - at.submit.loc.x, 12);
}

/// **The feedback sits between the entry and the row below it**, which is
/// where `364:25566` is drawn in the error state.
#[test]
fn a_failure_is_said_between_the_entry_and_the_row_below_it() {
    let at = laid_out(the_reference().w, the_reference().h);
    assert!(
        at.feedback.loc.y >= at.entry.loc.y + at.entry.size.h,
        "the feedback overlaps the entry"
    );
    assert!(
        at.feedback.loc.y + at.feedback.size.h <= at.other_ways.loc.y + 22,
        "the feedback is below the row it should sit above"
    );
    assert_eq!(at.feedback.loc.y - at.group.loc.y, 208);
}

/// **The essential actions are pinned to the bottom edge**, not placed at 884.
///
/// 884 is 76 above the bottom of a 960 screen, and the row must be 76 above
/// the bottom of any screen — a fixed 884 would be off the bottom of a shorter
/// one and halfway up a taller one.
#[test]
fn the_essential_actions_are_pinned_to_the_bottom_of_whatever_screen_this_is() {
    for (w, h) in [(1440, 960), (1366, 768), (1920, 1080), (800, 600)] {
        let at = laid_out(w, h);
        assert_eq!(
            h - at.actions.loc.y,
            76,
            "the actions row is not 76 above the bottom of {w}x{h}"
        );
        assert!(
            at.actions.loc.y + at.actions.size.h <= h,
            "the actions row falls off the bottom of {w}x{h}"
        );
        assert!(
            at.power.loc.x + at.power.size.w <= w,
            "the power target falls off the right of {w}x{h}"
        );
    }
    // At the reference size it lands on the capture's own number.
    let at = laid_out(1440, 960);
    assert_eq!(at.actions.loc.y, 884, "`176:10995` is at y=884");
    assert_eq!(at.actions.loc.x, 40, "and 40 in from the left");
}

/// **The submit target never shrinks**, however narrow the screen.
///
/// *Usable targets* is the owner's word and 44 is the floor every other
/// surface in this repository holds to. A narrow screen takes its width out of
/// the entry's text, which can scroll, and not out of the thing a finger has
/// to hit.
#[test]
fn the_submit_target_is_forty_four_on_every_screen_that_lays_out() {
    for (w, h) in [
        THE_SMALLEST_SCREEN_IS,
        (480, 400),
        (640, 480),
        (800, 600),
        (1024, 768),
        (1440, 960),
        (3840, 2160),
    ] {
        let at = laid_out(w, h);
        assert_eq!(
            at.submit.size,
            Size::from((THE_SUBMIT_TARGET_IS, THE_SUBMIT_TARGET_IS)),
            "the submit target shrank on {w}x{h}"
        );
        assert_eq!(
            at.entry.intersection(at.submit),
            Some(at.submit),
            "the submit target left the entry on {w}x{h}"
        );
    }
}

/// **Every action stays on the screen**, which is *access to every action*.
#[test]
fn nothing_falls_off_any_screen_that_lays_out() {
    for (w, h) in [
        THE_SMALLEST_SCREEN_IS,
        (480, 400),
        (640, 480),
        (1366, 768),
        (1440, 960),
        (2560, 1440),
    ] {
        let at = laid_out(w, h);
        let screen = at.screen;
        for (what, rectangle) in [
            ("the group", at.group),
            ("the monogram", at.monogram),
            ("the entry", at.entry),
            ("the submit target", at.submit),
            ("the actions row", at.actions),
            ("accessibility", at.accessibility),
            ("switch person", at.switch_person),
            ("the power target", at.power),
            ("back to lock", at.back),
        ] {
            assert_eq!(
                screen.intersection(rectangle),
                Some(rectangle),
                "{what} is not wholly on a {w}x{h} screen: {rectangle:?}"
            );
        }
    }
}

/// **The group never overlaps the essential actions.**
#[test]
fn the_group_never_sits_on_the_actions_row() {
    for (w, h) in [
        THE_SMALLEST_SCREEN_IS,
        (480, 400),
        (640, 480),
        (800, 600),
        (1440, 960),
    ] {
        let at = laid_out(w, h);
        assert_eq!(
            at.group.intersection(at.actions),
            None,
            "the group overlaps the actions row on {w}x{h}"
        );
    }
}

/// **A screen too small to hold it honestly is refused**, rather than laid out
/// with its parts on top of each other.
#[test]
fn a_screen_too_small_is_refused_rather_than_overlapped() {
    for (w, h) in [(1, 1), (319, 240), (320, 239), (100, 800)] {
        assert!(
            sign_in_laid_out(Size::from((w, h))).is_none(),
            "{w}x{h} was laid out and should have been refused"
        );
    }
    // The control: the smallest one that is allowed does lay out, so the
    // refusal above is a boundary and not a function that always refuses.
    assert!(sign_in_laid_out(Size::from(THE_SMALLEST_SCREEN_IS)).is_some());
}
