//! That the Bar lands where the design draws it at the reference size, and
//! stays usable on screens the design never drew.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use smithay::utils::Size;

use super::{A_TARGET_IS_TALL, where_the_bar_opens};

/// The size every number in the design document was read at.
const THE_REFERENCE: (i32, i32) = (1440, 960);

/// **At the size the design was drawn, the arithmetic lands on the design.**
///
/// This is the whole licence for deriving the rest by proportion: a rule that
/// does not reproduce the frame it came from is not a rule, it is a different
/// layout. Tolerances are a pixel or two because a proportion rounds.
#[test]
fn at_the_reference_size_it_is_what_the_frame_draws() {
    let places = where_the_bar_opens(Size::from(THE_REFERENCE)).expect("the reference fits");

    assert_eq!(
        (places.panel.size.w, places.panel.size.h),
        (680, 390),
        "the panel is not the size the frame draws"
    );
    assert_eq!(places.panel.loc.x, 380, "the panel is not centred as drawn");
    assert_eq!(
        places.panel.loc.y, 208,
        "the panel does not begin where drawn"
    );
    assert_eq!(
        places.field.size.w, 632,
        "the field is not the panel less its air on both sides"
    );
    assert_eq!(places.field.size.h, A_TARGET_IS_TALL);
    assert_eq!(
        (places.first_row.size.w, places.first_row.size.h),
        (places.field.size.w, places.field.size.h),
        "a row is not the same box as the field, which is what the design says it is"
    );
    assert_eq!(places.row_to_row, 68, "the rows are not 68 apart as drawn");
}

/// **The list scrolls rather than being capped**: at the reference the design
/// draws a fourth row cut off, so three fit wholly and the rest scroll.
#[test]
fn only_whole_rows_are_counted_as_fitting() {
    let places = where_the_bar_opens(Size::from(THE_REFERENCE)).expect("the reference fits");
    let last = places.first_row.loc.y + places.row_to_row * (places.rows_that_fit as i32 - 1);
    assert!(
        last + A_TARGET_IS_TALL <= places.panel.loc.y + places.panel.size.h,
        "the last row counted as fitting runs past the panel, so a person is shown half a row"
    );
    let one_more = last + places.row_to_row;
    assert!(
        one_more + A_TARGET_IS_TALL > places.panel.loc.y + places.panel.size.h,
        "another whole row would have fitted and was not counted, so the panel has dead space"
    );
}

/// **A target keeps its size on every screen; the panel does not.**
///
/// This is the division the design document names: what a person points at is
/// a measure, what holds it is a proportion. On a 4K panel the Bar is wider
/// and the rows are the same height, which is why a row stays a thing you can
/// hit.
#[test]
fn the_panel_scales_and_the_targets_do_not() {
    let small = where_the_bar_opens(Size::from((1280, 800))).expect("a small laptop fits");
    let large = where_the_bar_opens(Size::from((3840, 2160))).expect("a 4K panel fits");

    assert!(
        large.panel.size.w > small.panel.size.w * 2,
        "the panel did not grow with the display: {} against {}",
        large.panel.size.w,
        small.panel.size.w
    );
    assert_eq!(
        (small.field.size.h, large.field.size.h),
        (A_TARGET_IS_TALL, A_TARGET_IS_TALL),
        "the field's height changed with the screen, so a target is not a target any more"
    );
    assert!(
        large.rows_that_fit > small.rows_that_fit,
        "a taller screen shows no more results than a short one"
    );
    for places in [small, large] {
        assert!(
            places.panel.loc.x > 0 && places.panel.loc.y > 0,
            "the panel is off the top or the left of the display"
        );
    }
}

/// **A display too small for the panel is told so**, rather than being given a
/// Bar squeezed into nothing.
#[test]
fn a_display_too_small_has_no_bar() {
    assert!(
        where_the_bar_opens(Size::from((320, 200))).is_none(),
        "a display far too small was given a panel anyway"
    );
    // **The control**: one step up from the refusal still answers, so the
    // refusal above is about the size and not about the function.
    assert!(
        where_the_bar_opens(Size::from((1024, 600))).is_some(),
        "a small but ordinary display was refused, so the rule refuses more than it should"
    );
}
