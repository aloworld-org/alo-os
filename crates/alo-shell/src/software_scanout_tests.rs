//! What the processor draws, read back pixel by pixel — on a gate with no
//! display, no graphics card and no session.
//!
//! These are the strongest claims this backend can make without a screen in
//! front of somebody: that the bytes a display would scan out carry the picture
//! that was handed in, in the right place and the right colour, and that every
//! layer this painter cannot import is refused by its own name.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or short frame is the failure being reported"
)]

use super::*;
use crate::direct_target::ScenePainter;
use crate::painted::{Inked, Solid};
use crate::scene_native::NativeScene;
use crate::sign_in_raster::SignInPicture;

/// A size Smithay's constructor would accept, built the way this crate's other
/// tests build one.
fn extent((w, h): (i32, i32)) -> Size<i32, Physical> {
    (w, h).into()
}

/// A screen of one ground colour with one block of ink in it, at a known place.
fn a_screen(size: (i32, i32)) -> SignInPicture {
    SignInPicture {
        size,
        solids: vec![Solid {
            area: Rectangle::from_size(size.into()),
            colour: [20, 30, 40],
        }],
        inked: vec![Inked {
            area: Rectangle::new((2, 1).into(), (3, 2).into()),
            pixels: vec![[255, 255, 255]; 6],
        }],
    }
}

/// The colour of one pixel in the bytes a display would scan out.
fn pixel_at(pixels: &ScanoutPixels, (x, y): (u32, u32)) -> [u8; 3] {
    let (width, _) = pixels.size();
    let start = ((y * width + x) * 4) as usize;
    let bytes = pixels.pixels().get(start..start + 4).unwrap_or(&[0; 4]);
    // Scanout bytes are blue, green, red, unused — XRGB8888 as a little-endian
    // machine spells it.
    [bytes[2], bytes[1], bytes[0]]
}

#[test]
fn the_sign_in_screen_is_drawn_into_the_bytes_a_display_would_scan_out() {
    let size = (8, 4);
    let screen = a_screen(size);
    let mut painter = SoftwarePainter::new().expect("a machine with no software renderer");

    let (pixels, surfaces) = painter
        .paint(
            extent(size),
            &[],
            &[],
            &Cursor::Default,
            Some(NativeScene::SignIn(&screen)),
        )
        .expect("the sign-in screen refused on the processor");

    assert_eq!(pixels.size(), (8, 4));
    assert!(
        surfaces.is_empty(),
        "nothing was imported, so no surface may be told its frame was shown"
    );
    assert_eq!(
        pixel_at(&pixels, (0, 0)),
        [20, 30, 40],
        "the ground is not the colour the picture asked for"
    );
    for (x, y) in [(2, 1), (4, 1), (2, 2), (4, 2)] {
        assert_eq!(
            pixel_at(&pixels, (x, y)),
            [255, 255, 255],
            "the ink is missing at {x},{y}"
        );
    }
    assert_eq!(
        pixel_at(&pixels, (5, 1)),
        [20, 30, 40],
        "the ink ran past the area it was given"
    );
    assert_eq!(
        pixel_at(&pixels, (2, 3)),
        [20, 30, 40],
        "the ink ran below the area it was given"
    );
}

#[test]
fn the_arrow_is_drawn_above_the_screen_it_points_at() {
    let size = (16, 16);
    let screen = a_screen(size);
    let mut painter = SoftwarePainter::new().expect("a machine with no software renderer");

    let (with_arrow, _) = painter
        .paint(
            extent(size),
            &[],
            &[],
            &Cursor::Arrow {
                location: (8.0, 8.0).into(),
            },
            Some(NativeScene::SignIn(&screen)),
        )
        .expect("the arrow refused on the processor");
    let (without, _) = painter
        .paint(
            extent(size),
            &[],
            &[],
            &Cursor::Default,
            Some(NativeScene::SignIn(&screen)),
        )
        .expect("the same screen refused without an arrow");

    assert_ne!(
        with_arrow.pixels(),
        without.pixels(),
        "a positioned arrow added no pixels"
    );
    // The arrow is a white shape inside a black outline and its tip is the
    // outline's own corner, so the pixel at the point given is black.
    assert_eq!(
        pixel_at(&with_arrow, (8, 8)),
        [0, 0, 0],
        "the arrow's tip is not at the point it was given"
    );
    assert_eq!(
        pixel_at(&with_arrow, (9, 10)),
        [255, 255, 255],
        "the arrow is an outline with nothing inside it"
    );
    assert_eq!(
        pixel_at(&with_arrow, (0, 0)),
        [20, 30, 40],
        "the arrow painted where it does not point"
    );
}

#[test]
fn every_layer_this_painter_cannot_import_is_refused_by_its_own_name() {
    let screen = a_screen((8, 4));
    let lock = crate::lock_raster::LockPicture {
        size: (8, 4),
        pixels: Vec::new(),
    };
    let nothing = ToImport {
        windows: false,
        menus: false,
        pointer: false,
    };

    // A window is named before a menu and a menu before a pointer, so a frame
    // carrying all three is refused for the largest thing missing rather than
    // the last one looked at.
    let refusals = [
        (
            ToImport {
                windows: true,
                menus: true,
                pointer: true,
            },
            Some(NativeScene::SignIn(&screen)),
            Some("a window a client mapped"),
        ),
        (
            ToImport {
                menus: true,
                ..nothing
            },
            Some(NativeScene::SignIn(&screen)),
            Some("a menu a client opened"),
        ),
        (
            ToImport {
                pointer: true,
                ..nothing
            },
            Some(NativeScene::SignIn(&screen)),
            Some("the pointer a client drew itself"),
        ),
        (
            nothing,
            Some(NativeScene::Lock(&lock)),
            Some("the lock screen"),
        ),
        (nothing, Some(NativeScene::SignIn(&screen)), None),
    ];

    for (carried, scene, named) in refusals {
        assert_eq!(
            refused_layer(carried, scene),
            named,
            "the painter disagreed about what it can draw"
        );
    }
}

#[test]
fn a_frame_with_nothing_of_this_shells_own_in_it_is_refused_rather_than_drawn_black() {
    let mut painter = SoftwarePainter::new().expect("a machine with no software renderer");

    let refused = painter
        .paint(extent((8, 4)), &[], &[], &Cursor::Default, None)
        .err()
        .expect("an empty frame was drawn rather than refused");

    assert!(
        matches!(
            refused,
            RenderError::SceneNotOnThisBackend { scene }
                if scene == "a frame with nothing of this shell's own in it"
        ),
        "an empty frame was refused as {refused}"
    );
}

#[test]
fn an_extent_no_display_has_is_refused_before_anything_is_allocated() {
    let screen = a_screen((8, 4));
    let mut painter = SoftwarePainter::new().expect("a machine with no software renderer");
    let mut empty = extent((8, 4));
    empty.w = 0;

    assert!(matches!(
        painter.paint(
            empty,
            &[],
            &[],
            &Cursor::Default,
            Some(NativeScene::SignIn(&screen))
        ),
        Err(RenderError::EmptySize)
    ));
}
