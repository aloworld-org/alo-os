use super::*;

/// **Blue, green, red and a byte nobody reads become red, green and blue**,
/// row by row, at the size given.
#[test]
fn a_frame_becomes_a_png_of_the_same_pixels() {
    // Two pixels wide, one tall: pure red, then pure blue, as B,G,R,0.
    let frame = [0, 0, 255, 0, 255, 0, 0, 0];
    let png = a_png_of(2, 1, &frame).expect("a PNG");
    let decoded = image::load_from_memory_with_format(&png, image::ImageFormat::Png)
        .expect("it decodes")
        .to_rgb8();
    assert_eq!((decoded.width(), decoded.height()), (2, 1));
    assert_eq!(decoded.get_pixel(0, 0).0, [255, 0, 0]);
    assert_eq!(decoded.get_pixel(1, 0).0, [0, 0, 255]);
}

/// **A frame whose bytes are not its size is no picture**, rather than a
/// picture of something else.
#[test]
fn a_frame_of_the_wrong_length_is_no_picture() {
    assert_eq!(a_png_of(2, 2, &[0; 8]), None);
    assert_eq!(a_png_of(0, 1, &[]), None);
}
