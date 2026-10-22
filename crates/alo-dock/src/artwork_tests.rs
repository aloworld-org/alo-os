//! Pictures made of counted bytes, with no file and no decoder in sight.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::{APicture, NotAPicture, TheArtwork};
use crate::AppId;

/// One opaque red pixel, which is four bytes and the smallest real picture.
fn one_pixel() -> Vec<u8> {
    vec![255, 0, 0, 255]
}

#[test]
fn a_picture_is_its_size_and_its_bytes() {
    let picture = APicture::of(1, 1, one_pixel()).unwrap();
    assert_eq!(picture.across(), 1);
    assert_eq!(picture.down(), 1);
    assert_eq!(picture.pixels(), &[255, 0, 0, 255]);
}

#[test]
fn a_picture_that_is_not_square_is_still_a_picture() {
    // Plenty of application artwork is not square, and refusing it here would
    // lose those applications for a reason that belongs to the drawing.
    let picture = APicture::of(2, 1, vec![0; 8]).unwrap();
    assert_eq!((picture.across(), picture.down()), (2, 1));
}

#[test]
fn a_size_with_no_area_is_not_a_picture() {
    assert_eq!(APicture::of(0, 4, Vec::new()), Err(NotAPicture::Empty));
    assert_eq!(APicture::of(4, 0, Vec::new()), Err(NotAPicture::Empty));
}

#[test]
fn too_few_bytes_are_refused_rather_than_drawn_as_far_as_they_go() {
    // A truncated decode is one of the two ways this goes wrong, and drawing
    // what arrived would put something that looks like artwork on the Dock.
    assert_eq!(
        APicture::of(2, 2, vec![0; 8]),
        Err(NotAPicture::TheWrongNumberOfBytes {
            wanted: 16,
            given: 8
        })
    );
}

#[test]
fn too_many_bytes_are_refused_rather_than_cropped() {
    // The other way: a size read off the wrong field. Cropping would draw
    // somebody else's bytes at a plausible size.
    assert_eq!(
        APicture::of(1, 1, vec![0; 8]),
        Err(NotAPicture::TheWrongNumberOfBytes {
            wanted: 4,
            given: 8
        })
    );
    // The control: the same size with the right count is accepted, so the two
    // refusals above are the count and not the size.
    assert!(APicture::of(1, 1, one_pixel()).is_ok());
}

#[test]
fn a_machine_with_no_artwork_draws_letters_everywhere() {
    let artwork = TheArtwork::none();
    assert!(artwork.is_empty());
    assert_eq!(artwork.how_many(), 0);
    assert_eq!(artwork.of(&AppId::named("Files").unwrap()), None);
}

#[test]
fn an_applications_picture_is_read_back_by_the_name_the_dock_knows() {
    let files = AppId::named("Files").unwrap();
    let mut artwork = TheArtwork::none();
    artwork.draws(files.clone(), APicture::of(1, 1, one_pixel()).unwrap());
    assert_eq!(artwork.of(&files).unwrap().pixels(), &[255, 0, 0, 255]);
    // The control at the other end: another application in the same machine
    // has none, so the answer above is this application's and not whatever
    // picture happened to be stored.
    assert_eq!(artwork.of(&AppId::named("Browser").unwrap()), None);
    assert_eq!(artwork.how_many(), 1);
}

#[test]
fn the_first_picture_for_an_application_is_the_one_it_keeps() {
    // The icon directories are searched most-specific first, so the first
    // answer for an application is the one from the person's own directory.
    let files = AppId::named("Files").unwrap();
    let mut artwork = TheArtwork::none();
    artwork.draws(files.clone(), APicture::of(1, 1, vec![1, 1, 1, 1]).unwrap());
    artwork.draws(files.clone(), APicture::of(1, 1, vec![2, 2, 2, 2]).unwrap());
    assert_eq!(
        artwork.of(&files).unwrap().pixels(),
        &[1, 1, 1, 1],
        "the one they chose is not overwritten by the one the image shipped"
    );
    assert_eq!(artwork.how_many(), 1);
}
