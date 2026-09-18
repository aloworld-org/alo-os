//! Bounded decoding, selection, alpha and fitting refusal paths.
#![allow(clippy::panic, clippy::indexing_slicing)]
#![expect(clippy::unwrap_used, reason = "a failed fixture is a failed test")]
use crate::LockBackground;
use alo_appearance::{Background, Fitting, Picture};
use std::time::Duration;

#[test]
/// Selected images are decoded and missing malformed or oversized inputs refuse.
fn selected_images_are_decoded_and_missing_malformed_or_oversized_inputs_refuse() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("chosen.png");
    let chosen = Background::Picture(Picture::file(file.clone()).unwrap());
    assert!(LockBackground::prepare(&chosen, (640, 480), Duration::ZERO).is_err());
    std::fs::write(&file, b"not an image").unwrap();
    assert!(LockBackground::prepare(&chosen, (640, 480), Duration::ZERO).is_err());
    image::RgbaImage::from_pixel(2, 2, image::Rgba([255, 0, 0, 0]))
        .save(&file)
        .unwrap();
    let prepared = LockBackground::prepare(&chosen, (640, 480), Duration::ZERO).unwrap();
    assert!(
        prepared
            .pixels
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [0, 0, 0, 255])
    );
    for size in [(0, 480), (640, -1), (8193, 480), (8192, 8192)] {
        assert!(LockBackground::prepare(&chosen, size, Duration::ZERO).is_err());
    }
    // Huge dimensions are refused before allocation even if compressed data is tiny.
    image::RgbImage::new(8193, 1).save(&file).unwrap();
    assert!(LockBackground::prepare(&chosen, (640, 480), Duration::ZERO).is_err());
}
#[test]
/// The five fittings crop margin distort centre and repeat as selected.
fn the_five_fittings_crop_margin_distort_centre_and_repeat_as_selected() {
    let image = image::RgbImage::from_fn(2, 1, |x, _| {
        image::Rgb(if x == 0 { [255, 0, 0] } else { [0, 0, 255] })
    });
    let sample = |fit| {
        let mut pixels = vec![0; 4 * 4 * 4];
        super::lock_image_fit::paint(&image, fit, (4, 4), &mut pixels);
        pixels
    };
    let fit = sample(Fitting::Fit);
    assert_eq!(&fit[..4], &[0, 0, 0, 255]);
    assert_eq!(&fit[16..20], &[255, 0, 0, 255]);
    let stretch = sample(Fitting::Stretch);
    assert_eq!(&stretch[..4], &[255, 0, 0, 255]);
    assert_eq!(&stretch[12..16], &[0, 0, 255, 255]);
    let centre = sample(Fitting::Centre);
    assert_eq!(&centre[..4], &[0, 0, 0, 255]);
    let tile = sample(Fitting::Tile);
    assert_eq!(&tile[..8], &tile[8..16]);
    assert_eq!(&tile[..16], &tile[16..32]);
    let fill = sample(Fitting::Fill);
    assert!(
        fill.as_chunks::<4>()
            .0
            .iter()
            .all(|p| p[0] == 255 || p[2] == 255)
    );
}

/// Rotation follows the appearance model over a bounded, sorted image inventory.
#[test]
fn an_explicit_rotation_follows_sorted_images_and_refuses_an_empty_folder() {
    let temp = tempfile::tempdir().unwrap();
    let rotating = alo_appearance::Rotating::folder(
        temp.path().to_owned(),
        alo_appearance::Every::minutes(1).unwrap(),
    )
    .unwrap();
    let chosen = Background::Rotating(rotating);
    assert!(LockBackground::prepare(&chosen, (320, 480), Duration::ZERO).is_err());
    image::RgbImage::from_pixel(1, 1, image::Rgb([0, 0, 255]))
        .save(temp.path().join("b.png"))
        .unwrap();
    image::RgbImage::from_pixel(1, 1, image::Rgb([255, 0, 0]))
        .save(temp.path().join("a.png"))
        .unwrap();
    std::fs::write(temp.path().join("private.txt"), "not an image").unwrap();
    let first = LockBackground::prepare(&chosen, (320, 480), Duration::ZERO).unwrap();
    let next = LockBackground::prepare(&chosen, (320, 480), Duration::from_secs(60)).unwrap();
    assert_eq!(&first.pixels[..4], &[255, 0, 0, 255]);
    assert_eq!(&next.pixels[..4], &[0, 0, 255, 255]);
}

/// Ordinary files are read unchanged; directories, FIFOs and excessive bytes refuse.
#[test]
fn image_reads_preserve_the_file_and_refuse_nonordinary_or_excessive_input() {
    use rustix::fs::{Mode, OFlags};
    let temp = tempfile::tempdir().unwrap();
    let at = temp.path().join("image.png");
    image::RgbImage::from_pixel(1, 1, image::Rgb([10, 20, 30]))
        .save(&at)
        .unwrap();
    let before = std::fs::read(&at).unwrap();
    let decoded = crate::lock_image_decode::decode(&at).unwrap();
    assert_eq!(decoded.get_pixel(0, 0).0, [10, 20, 30]);
    assert_eq!(std::fs::read(&at).unwrap(), before);
    assert!(crate::lock_image_decode::decode(temp.path()).is_err());
    let large = temp.path().join("oversized.png");
    std::fs::File::create(&large)
        .unwrap()
        .set_len(32 * 1024 * 1024 + 1)
        .unwrap();
    assert!(crate::lock_image_decode::decode(&large).is_err());
    let fifo = temp.path().join("pipe.png");
    rustix::fs::mkfifoat(rustix::fs::CWD, &fifo, Mode::RUSR | Mode::WUSR).unwrap();
    // Keep a peer open so a regression cannot strand the test in FIFO open.
    // The decoder must still reject this nonordinary file before reading it.
    let _peer = rustix::fs::open(
        &fifo,
        OFlags::RDWR | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .unwrap();
    assert!(crate::lock_image_decode::decode(&fifo).is_err());
}
