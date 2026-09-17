//! **What was blurred is not in the saved file.**
//!
//! Task 3 of `docs/autonomy/v0-5-capture-and-the-room-plan.md`. A person blurs
//! a password, a home address, somebody else's name, and then sends the picture
//! to a colleague, who opens it in a program nobody here chose. If the blur were
//! a layer over the original, the first person to move it would read what was
//! underneath — and they would not have to be an attacker, only curious.
//!
//! So this saves a picture with a blur over a known secret and **reads the
//! saved bytes back**, requiring that the secret is not in them.
//!
//! The renderer is the shell's, and this crate holds no pixels; what is tested
//! here is the contract `Marks::saving` is written to — every blurred area is
//! handed over to be destroyed, and the bytes that come back are what is
//! written. A renderer that ignored those areas would fail this test with a
//! picture that still contains the password.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_capturing::{Mark, Marks, Picture, Region};

/// A picture with something in it that must not survive: eight rows of ten
/// bytes, and a password written across the middle two.
fn a_screenshot_with_a_password() -> (Picture, Region, Vec<u8>) {
    let secret = b"hunter2".to_vec();
    let mut pixels = vec![0_u8; 80];
    pixels
        .get_mut(30..37)
        .expect("the picture is long enough to hold a password")
        .copy_from_slice(&secret);
    (
        Picture::of(pixels).expect("a picture"),
        Region::of(0, 3, 10, 1).expect("the row the password is on"),
        secret,
    )
}

/// The shell's renderer, as this test stands in for it: every area handed over
/// is overwritten, which is what *destroys* means.
fn flattening(picture: &Picture, hidden: &[Region]) -> Result<Picture, std::convert::Infallible> {
    let mut pixels = picture.bytes().to_vec();
    for area in hidden {
        let from = (area.from_the_top() * 10 + area.from_the_left()) as usize;
        let to = (from + (area.width() * area.height()) as usize).min(pixels.len());
        if let Some(hidden) = pixels.get_mut(from..to) {
            for pixel in hidden {
                *pixel = 0xCC;
            }
        }
    }
    Ok(Picture::of(pixels).expect("a picture with something hidden in it"))
}

/// **The saved bytes do not contain what was blurred.**
#[test]
fn what_was_blurred_cannot_be_read_out_of_the_saved_picture() {
    let (taken, over_the_password, secret) = a_screenshot_with_a_password();
    assert!(
        contains(taken.bytes(), &secret),
        "the test's own picture does not contain the secret it is about"
    );

    let saved = Marks::on(taken)
        .and(Mark::Blur {
            over: over_the_password,
        })
        .saving(flattening)
        .expect("the renderer answered");

    assert!(
        !contains(saved.bytes(), &secret),
        "the password is still in the saved file: a blur that can be taken off is not a blur"
    );
}

/// **And a mark that is not a blur leaves the picture alone**, because those
/// are layers a person may still move.
#[test]
fn a_mark_that_is_not_a_blur_changes_nothing_in_the_saved_picture() {
    let (taken, somewhere, secret) = a_screenshot_with_a_password();
    let before = taken.bytes().to_vec();

    let saved = Marks::on(taken)
        .and(Mark::Rectangle { over: somewhere })
        .and(Mark::Arrow {
            from: (0, 0),
            to: (9, 7),
        })
        .saving(flattening)
        .expect("the renderer answered");

    assert_eq!(
        saved.bytes(),
        before.as_slice(),
        "a rectangle destroyed part of the picture"
    );
    assert!(contains(saved.bytes(), &secret));
}

/// **A blur taken off before saving takes nothing with it.**
#[test]
fn a_blur_undone_before_saving_leaves_the_picture_whole() {
    let (taken, over_the_password, secret) = a_screenshot_with_a_password();
    let saved = Marks::on(taken)
        .and(Mark::Blur {
            over: over_the_password,
        })
        .without_the_last()
        .saving(flattening)
        .expect("the renderer answered");
    assert!(
        contains(saved.bytes(), &secret),
        "undoing a blur before saving destroyed the picture anyway"
    );
}

/// Whether one run of bytes is inside another.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|run| run == needle)
}
