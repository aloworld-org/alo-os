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

/// Scaling a picture into the box the Dock draws it in.
mod scaled_into_the_box_it_is_drawn_in {
    use super::super::APicture;

    /// A picture of one solid opaque colour, which must survive any scaling.
    fn solid(across: u32, down: u32, colour: [u8; 4]) -> APicture {
        let mut pixels = Vec::new();
        for _ in 0..across * down {
            pixels.extend_from_slice(&colour);
        }
        APicture::of(across, down, pixels).unwrap()
    }

    #[test]
    fn the_size_asked_for_is_the_size_returned() {
        let scaled = solid(64, 64, [1, 2, 3, 255]).scaled_to(32, 32);
        assert_eq!((scaled.across(), scaled.down()), (32, 32));
        assert_eq!(scaled.pixels().len(), 32 * 32 * 4);
    }

    #[test]
    fn a_picture_already_the_right_size_comes_back_unchanged() {
        let before = solid(48, 48, [9, 8, 7, 255]);
        let after = before.scaled_to(48, 48);
        assert_eq!(after, before);
    }

    #[test]
    fn one_solid_colour_is_the_same_colour_after_scaling_down() {
        let scaled = solid(64, 64, [200, 100, 50, 255]).scaled_to(16, 16);
        for pixel in scaled.pixels().as_chunks::<4>().0 {
            let pixel = *pixel;
            assert_eq!(
                pixel,
                [200, 100, 50, 255],
                "averaging must not shift a flat colour"
            );
        }
    }

    #[test]
    fn one_solid_colour_is_the_same_colour_after_scaling_up() {
        let scaled = solid(8, 8, [200, 100, 50, 255]).scaled_to(32, 32);
        for pixel in scaled.pixels().as_chunks::<4>().0 {
            let pixel = *pixel;
            assert_eq!(pixel, [200, 100, 50, 255]);
        }
    }

    #[test]
    fn a_transparent_edge_does_not_darken_the_colour_beside_it() {
        // **The fault this is here for.** Half the source is opaque white and
        // half is fully transparent black — which is what the pixels outside a
        // rounded icon actually are. Averaging the colours on their own gives
        // 127 grey; weighting them by alpha keeps the white.
        let mut pixels = Vec::new();
        for _ in 0..2 {
            pixels.extend_from_slice(&[255, 255, 255, 255]);
            pixels.extend_from_slice(&[0, 0, 0, 0]);
        }
        let halved = APicture::of(2, 2, pixels).unwrap().scaled_to(1, 1);
        let pixel: [u8; 4] = halved.pixels().try_into().unwrap();
        assert_eq!(
            pixel.first_chunk::<3>(),
            Some(&[255, 255, 255]),
            "the colour under the opacity is white, and the transparent black beside it is \
             not a colour at all -- averaging it in is what puts a dark halo round every icon"
        );
        assert_eq!(
            pixel.last().copied(),
            Some(127),
            "and the opacity itself is halved, which is the part that is a real average"
        );
    }

    #[test]
    fn a_fully_transparent_picture_scales_to_a_fully_transparent_one() {
        let scaled = solid(4, 4, [0, 0, 0, 0]).scaled_to(2, 2);
        assert!(
            scaled
                .pixels()
                .as_chunks::<4>()
                .0
                .iter()
                .all(|pixel| pixel[3] == 0),
            "nothing opaque went in, so nothing opaque comes out"
        );
    }

    #[test]
    fn a_size_with_no_area_is_refused_by_returning_the_picture_unscaled() {
        // Nothing can be drawn into a box with no area, and inventing an empty
        // picture would hand the raster a buffer its own size check refuses.
        let before = solid(4, 4, [1, 1, 1, 255]);
        assert_eq!(before.scaled_to(0, 4), before);
        assert_eq!(before.scaled_to(4, 0), before);
    }

    #[test]
    fn a_picture_that_is_not_square_scales_into_a_square_box() {
        let scaled = solid(64, 16, [5, 5, 5, 255]).scaled_to(32, 32);
        assert_eq!((scaled.across(), scaled.down()), (32, 32));
        assert_eq!(scaled.pixels().len(), 32 * 32 * 4);
    }
}
