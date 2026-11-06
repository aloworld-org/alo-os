//! An XCursor file read back by somebody else's reader, and the files this
//! refuses to write.
//!
//! The reader is the `xcursor` crate's, which follows libXcursor's own checks
//! (`parse_img` cites `file.c`). A writer tested only against itself would agree
//! with its own mistakes.

use super::*;
use crate::{CursorIcon, Silhouette, draw};

/// A picture that must draw.
fn drawn(icon: CursorIcon, size: u32) -> Picture {
    draw(icon, Silhouette::Kite, size).unwrap_or_else(|why| unreachable!("{why}"))
}

/// **Another reader reads it**: every image, at its size, with its hotspot,
/// and its pixels in the file's own `B G R A` order are the picture's
/// premultiplied `R G B A`.
#[test]
fn another_reader_reads_every_image_back() {
    let pictures: Vec<Picture> = [48, 24, 36]
        .map(|size| drawn(CursorIcon::Text, size))
        .into();
    let file = xcursor_file(&pictures).unwrap_or_else(|why| unreachable!("{why}"));
    let Some(images) = xcursor::parser::parse_xcursor(&file) else {
        unreachable!("the xcursor crate could not read the file");
    };
    assert_eq!(
        images.iter().map(|image| image.size).collect::<Vec<_>>(),
        [24, 36, 48],
        "the images are not every size, smallest first"
    );
    for image in &images {
        let Some(picture) = pictures.iter().find(|picture| picture.size == image.size) else {
            unreachable!("an image of a size nobody drew");
        };
        assert_eq!((image.width, image.height), (picture.size, picture.size));
        assert_eq!((image.xhot, image.yhot), picture.hotspot);
        assert_eq!(image.delay, 0);
        let as_drawn: Vec<u8> = image
            .pixels_rgba
            .as_chunks::<4>()
            .0
            .iter()
            .flat_map(|&[blue, green, red, alpha]| [red, green, blue, alpha])
            .collect();
        assert_eq!(as_drawn, picture.pixels, "size {}", image.size);
    }
}

/// **A file that would mislead a toolkit is refused by name**: nothing in it,
/// two images claiming one size, or a picture not holding its size's pixels.
#[test]
fn a_file_that_would_mislead_is_refused() {
    assert_eq!(xcursor_file(&[]), Err(NotWritten::NoImages));
    let one = drawn(CursorIcon::Default, 24);
    assert_eq!(
        xcursor_file(&[one.clone(), one.clone()]),
        Err(NotWritten::TheSameSizeTwice(24))
    );
    let mut short = one;
    short.pixels.truncate(10);
    assert_eq!(xcursor_file(&[short]), Err(NotWritten::NotItsSize(24)));
}
