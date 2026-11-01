//! A sheet's layout in pixels, at one and at two.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::*;

/// What an ordinary sheet's words measure at one: a title on two lines and an
/// introduction on one.
const TALL: TheTextIsTall = TheTextIsTall {
    title: 88,
    introduction: 25,
    consequence: 22,
    choices: 2,
    actions: 2,
    rows: crate::setup_sheet_layout::TheRowsAre::Apart,
};

/// The four numbers of a rectangle, whatever it is measured in.
fn numbers<K>(it: Rectangle<i32, K>) -> (i32, i32, i32, i32) {
    (it.loc.x, it.loc.y, it.size.w, it.size.h)
}

/// **At one, every rectangle is the layout's own**, which is what keeps the
/// sheet most people meet the picture it was.
#[test]
fn at_one_nothing_moves() {
    let laid = laid_out((1440, 960), TALL, DisplayScale::ONE).expect("the reference size");
    let at = &laid.logical;
    let drawn = &laid.pixels;
    for (logical, pixels) in [
        (at.stage, drawn.stage),
        (at.eyebrow, drawn.eyebrow),
        (at.title, drawn.title),
        (at.introduction, drawn.introduction),
        (at.consequence, drawn.consequence),
        (at.accessibility, drawn.accessibility),
        (at.footer, drawn.footer),
    ] {
        assert_eq!(numbers(logical), numbers(pixels));
    }
    assert_eq!(
        at.choices.iter().map(|it| numbers(*it)).collect::<Vec<_>>(),
        drawn
            .choices
            .iter()
            .map(|it| numbers(*it))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        laid.logical,
        crate::setup_sheet_layout::setup_sheet_laid_out(Size::from((1440, 960)), TALL)
            .expect("the same layout, asked directly")
    );
    assert_eq!(the_column_in_pixels(1440, DisplayScale::ONE), 600);
}

/// **At two, the sheet is laid out at half its pixels and every rectangle is
/// exactly twice itself**, with text measured at twice its height.
#[test]
fn at_two_everything_doubles() {
    let doubled = TheTextIsTall {
        title: 2 * TALL.title,
        introduction: 2 * TALL.introduction,
        consequence: 2 * TALL.consequence,
        ..TALL
    };
    let two = DisplayScale::of_per_cent(200);
    let laid = laid_out((2880, 1920), doubled, two).expect("twice the reference size");
    let one = laid_out((1440, 960), TALL, DisplayScale::ONE).expect("the reference size");
    assert_eq!(laid.logical, one.logical, "laid out at half its pixels");
    for (at_one, at_two) in [
        (one.pixels.title, laid.pixels.title),
        (one.pixels.introduction, laid.pixels.introduction),
        (one.pixels.footer, laid.pixels.footer),
        (one.pixels.accessibility, laid.pixels.accessibility),
    ]
    .into_iter()
    .chain(
        one.pixels
            .choices
            .iter()
            .copied()
            .zip(laid.pixels.choices.iter().copied()),
    )
    .chain(
        one.pixels
            .actions
            .iter()
            .copied()
            .zip(laid.pixels.actions.iter().copied()),
    ) {
        let (x, y, w, h) = numbers(at_one);
        assert_eq!((2 * x, 2 * y, 2 * w, 2 * h), numbers(at_two));
    }
    assert_eq!(the_column_in_pixels(2880, two), 1200);
}

/// **A screen too small at its scale is refused**, which is what lets the
/// caller try a smaller one: 1920 × 1080 at 200 is 960 × 540, under the
/// smallest sheet.
#[test]
fn a_screen_too_small_at_its_scale_is_refused() {
    assert!(laid_out((1920, 1080), TALL, DisplayScale::of_per_cent(200)).is_none());
    assert!(laid_out((1920, 1080), TALL, DisplayScale::of_per_cent(150)).is_some());
}

/// **A height between two logical pixels is rounded up**, so a block never
/// overlaps what follows it by a fraction.
#[test]
fn a_text_height_is_rounded_up() {
    let tall = TheTextIsTall {
        title: 2 * TALL.title + 1,
        ..TALL
    };
    let laid = laid_out((2880, 1920), tall, DisplayScale::of_per_cent(200)).expect("fits");
    assert_eq!(laid.logical.title.size.h, TALL.title + 1);
}
