//! Scalable icons drawn into real pixels, read back pixel by pixel.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::{drawn, drawn_from};

/// The four bytes of the pixel at `x`, `y` of a picture `across` wide.
fn at(pixels: &[u8], across: u32, x: u32, y: u32) -> [u8; 4] {
    let from = ((y * across + x) * 4) as usize;
    <[u8; 4]>::try_from(pixels.get(from..from + 4).unwrap()).unwrap()
}

/// **A gradient**, which `alo-vector-shapes` refuses by design and an
/// application's icon routinely uses — Breeze's are full of them.
#[test]
fn a_gradient_is_drawn_from_one_colour_to_the_other() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16">
        <defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="0">
            <stop offset="0" stop-color="#ff0000"/><stop offset="1" stop-color="#0000ff"/>
        </linearGradient></defs>
        <rect width="16" height="16" fill="url(#g)"/></svg>"##;
    let picture = drawn_from(svg, 32).unwrap();
    assert_eq!((picture.across(), picture.down()), (32, 32));
    let left = at(picture.pixels(), 32, 0, 16);
    let right = at(picture.pixels(), 32, 31, 16);
    assert_eq!(left[3], 255, "the icon's edge was not covered");
    assert!(
        left[0] > 200 && left[2] < 55,
        "the left is not red: {left:?}"
    );
    assert!(
        right[2] > 200 && right[0] < 55,
        "the right is not blue: {right:?}"
    );
}

/// **Straight colour, as the Dock composites it**: half-covered red is red at
/// half opacity, not the dark red a premultiplied buffer holds.
#[test]
fn half_covered_colour_is_handed_over_straight() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 4 4">
        <rect width="4" height="4" fill="#ff0000" fill-opacity="0.5"/></svg>"##;
    let picture = drawn_from(svg, 8).unwrap();
    let [red, green, blue, alpha] = at(picture.pixels(), 8, 4, 4);
    assert!((126..=129).contains(&alpha), "not half covered: {alpha}");
    assert!(red >= 253, "the colour was left premultiplied: {red}");
    assert_eq!((green, blue), (0, 0));
}

/// **An icon that is not square keeps its shape**, centred in its square.
#[test]
fn a_wide_icon_is_centred_and_keeps_its_proportions() {
    let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 10">
        <rect width="20" height="10" fill="#000000"/></svg>"##;
    let picture = drawn_from(svg, 20).unwrap();
    assert_eq!(
        at(picture.pixels(), 20, 10, 0)[3],
        0,
        "it was stretched to fill"
    );
    assert_eq!(
        at(picture.pixels(), 20, 10, 10)[3],
        255,
        "the middle is empty"
    );
    assert_eq!(
        at(picture.pixels(), 20, 10, 19)[3],
        0,
        "it was stretched to fill"
    );
}

/// **What is not an SVG is no picture**, and says so rather than panicking.
#[test]
fn rubbish_is_no_picture() {
    assert!(drawn_from(b"not an svg", 48).is_none());
    assert!(drawn_from(b"", 48).is_none());
    assert!(
        drawn_from(b"<svg/>", 48).is_none(),
        "an SVG that draws nothing was handed over as a picture"
    );
}

/// **A file larger than any icon is not read.**
#[test]
fn a_file_larger_than_any_icon_is_not_read() {
    let at = std::env::temp_dir().join(format!("alo-too-large-{}.svg", std::process::id()));
    let file = std::fs::File::create(&at).unwrap();
    file.set_len(super::AT_MOST + 1).unwrap();
    assert!(drawn(&at, 48).is_none());
    let _ = std::fs::remove_file(&at);
}
