//! That artwork lands where its file says, at any size, with strokes that
//! widen with it.

use crate::AShape;

/// The design file's Stop square, as exported for `427:28086` on 2026-10-30.
const STOP: &str = r#"<svg width="28" height="28" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg">
<g id="Stop / icon">
<path id="Vector" d="M18.6667 7H9.33333C8.04467 7 7 8.04467 7 9.33333V18.6667C7 19.9553 8.04467 21 9.33333 21H18.6667C19.9553 21 21 19.9553 21 18.6667V9.33333C21 8.04467 19.9553 7 18.6667 7Z" fill="white"/>
</g>
</svg>"#;

/// A vertical stroke two units wide down the middle of a 28-unit box.
const A_LINE: &str =
    r#"<svg viewBox="0 0 28 28"><path d="M14 4V24" stroke="black" stroke-width="2"/></svg>"#;

/// The artwork, read.
fn read(svg: &str) -> AShape {
    AShape::read(svg).unwrap_or_else(|why| unreachable!("the fixture reads: {why}"))
}

/// The coverage at `(x, y)` of a box `wide` across.
fn at(covered: &[u8], wide: u32, x: u32, y: u32) -> u8 {
    let index = usize::try_from(y * wide + x).unwrap_or(usize::MAX);
    covered.get(index).copied().unwrap_or_default()
}

#[test]
fn the_stop_square_is_inside_its_box_and_nowhere_else() {
    let covered = read(STOP).coverage(28, 28);
    assert_eq!(covered.len(), 28 * 28);
    assert_eq!(at(&covered, 28, 14, 14), 255, "the middle is not covered");
    assert_eq!(at(&covered, 28, 3, 3), 0, "the corner of the box is covered");
    assert_eq!(at(&covered, 28, 24, 14), 0, "right of the square is covered");
    assert_eq!(at(&covered, 28, 8, 14), 255, "the square's own edge is not covered");
}

#[test]
fn drawn_twice_the_size_it_covers_four_times_as_much() {
    let shape = read(STOP);
    let sum = |covered: Vec<u8>| covered.into_iter().map(u64::from).sum::<u64>();
    let (one, two) = (sum(shape.coverage(28, 28)), sum(shape.coverage(56, 56)));
    #[expect(
        clippy::cast_precision_loss,
        reason = "sums of a few thousand bytes are exact in f64"
    )]
    let ratio = two as f64 / one as f64;
    assert!((3.9..=4.1).contains(&ratio), "twice the size covers {ratio} times as much");
}

#[test]
fn a_stroke_widens_with_the_box() {
    let shape = read(A_LINE);
    let across = |wide: u32| {
        let covered = shape.coverage(wide, wide);
        (0..wide)
            .map(|x| u32::from(at(&covered, wide, x, wide / 2)))
            .sum::<u32>()
    };
    // Two units wide is two pixels at 28 and four at 56, in coverage.
    assert_eq!(across(28), 2 * 255);
    assert_eq!(across(56), 4 * 255);
}

#[test]
fn a_box_with_no_pixels_draws_nothing() {
    assert!(read(STOP).coverage(0, 28).is_empty());
}
