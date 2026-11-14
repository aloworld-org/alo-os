//! Reading qemu's pictures, and counting what changed between two.

use super::{Frame, NotAPicture};

/// A 2x1 PPM: one red pixel and one blue.
fn two_pixels() -> Vec<u8> {
    let mut bytes = b"P6\n2 1\n255\n".to_vec();
    bytes.extend_from_slice(&[255, 0, 0, 0, 0, 255]);
    bytes
}

/// A picture qemu writes reads back as its pixels.
#[test]
fn a_ppm_reads_as_its_pixels() {
    let read = Frame::from_ppm(&two_pixels());
    let expected = Frame::filled(2, 1, [0, 0, 255]).with_pixel(0, 0, [255, 0, 0]);
    assert_eq!(read, Ok(expected));
}

/// **The refusals**: a truncated picture, a picture of another kind and a
/// header with no size are each refused, never read as a blank screen.
#[test]
fn what_is_not_a_whole_picture_is_refused() {
    let mut short = two_pixels();
    short.pop();
    assert_eq!(
        Frame::from_ppm(&short),
        Err(NotAPicture("fewer pixels than the header says"))
    );
    assert_eq!(
        Frame::from_ppm(b"P5\n2 1\n255\n\0\0"),
        Err(NotAPicture("not P6"))
    );
    assert_eq!(
        Frame::from_ppm(b"P6\nx 1\n255\n"),
        Err(NotAPicture("a size that is not a number"))
    );
    assert_eq!(Frame::from_ppm(b""), Err(NotAPicture("a short header")));
}

/// Two pictures differ by the pixels that differ, and by every pixel when
/// the screen changed size.
#[test]
fn a_difference_is_counted_in_pixels() {
    let before = Frame::filled(4, 4, [1, 2, 3]);
    let after = before
        .clone()
        .with_pixel(0, 0, [9, 9, 9])
        .with_pixel(3, 3, [9, 9, 9]);
    assert_eq!(before.differs_from(&before), 0);
    assert_eq!(before.differs_from(&after), 2);
    assert_eq!(before.differs_from(&Frame::filled(8, 2, [1, 2, 3])), 32);
}
