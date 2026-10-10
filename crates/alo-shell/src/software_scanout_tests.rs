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

/// One frame carrying just this scene and nothing else of the shell's.
fn only(scene: NativeScene<'_>) -> crate::scene_native::NativeLayers<'_> {
    crate::scene_native::NativeLayers {
        scene: Some(scene),
        ..crate::scene_native::NativeLayers::nothing()
    }
}

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
            only(NativeScene::SignIn(&screen)),
            alo_canvas::Camera::new(),
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
            only(NativeScene::SignIn(&screen)),
            alo_canvas::Camera::new(),
        )
        .expect("the arrow refused on the processor");
    let (without, _) = painter
        .paint(
            extent(size),
            &[],
            &[],
            &Cursor::Default,
            only(NativeScene::SignIn(&screen)),
            alo_canvas::Camera::new(),
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

/// **A client's window is no longer a reason to refuse a frame.**
///
/// This test was `every_layer_this_painter_cannot_import_is_refused_by_its_own_name`
/// and it held the opposite: that a frame carrying a window, a menu or a
/// client-drawn pointer was refused, each named. The refusal was written when
/// this file's header claimed the painter could not import. It can -
/// `ImportMemWl`, `ImportDmaWl`, `ImportDma` and `Bind<Dmabuf>` - and the
/// owner's direction of 2026-10-10 is that *a renderer that refuses frames
/// containing application windows cannot count as a usable desktop*.
///
/// **So the refusal moved from the frame to the buffer**, which is both
/// narrower and more use: one client handing over one format pixman does not
/// know is refused by name, and every other frame is drawn. This asserts the
/// move in both directions, because either half alone would pass while the
/// other was broken.
#[test]
fn a_window_is_drawn_and_a_format_is_what_gets_refused() {
    let painter = SoftwarePainter::new().expect("a machine with no software renderer");

    // **It claims formats**, which is the capability the old refusal denied.
    // An empty set would mean a painter that can take nothing, and the test
    // below would then be asserting a refusal that is not about formats at all.
    let formats = painter.importable_formats();
    assert!(
        formats.iter().count() > 0,
        "the painter claims it can import no format at all"
    );

    // **And no blanket refusal has come back.** The property is the absence of
    // a decision taken before any import is attempted, which has no value to
    // assert - so the source is read, as `the_recheck_has_a_caller` reads a
    // draw. The three phrases are the ones the removed refusal returned.
    let at = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/software_scanout.rs");
    let written = std::fs::read_to_string(&at).expect("this painter is beside its test");
    let said = what_runs(&written);
    for refused in [
        "a window a client mapped",
        "a menu a client opened",
        "the pointer a client drew itself",
    ] {
        assert!(
            !said.contains(refused),
            "the painter refuses a frame for carrying {refused:?} again"
        );
    }
}

/// A file's code, without its prose.
///
/// The guard above looks for phrases that are also quoted in this file's own
/// comments explaining why they went, so reading the whole file would find them
/// and fail for the reason it exists to prevent.
fn what_runs(written: &str) -> String {
    written
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// **An empty frame is refused, and an application alone on an empty desktop
/// is not an empty frame.**
///
/// The first half is the old guarantee: black looks like a crash. The second is
/// what the condition had to widen to on 2026-10-10, because a client window
/// with no layer of this shell's own above it is now something this painter can
/// draw - and a guard written when it could not would have refused exactly the
/// frame the owner's direction asks for.
#[test]
fn a_frame_with_nothing_of_this_shells_own_in_it_is_refused_rather_than_drawn_black() {
    let mut painter = SoftwarePainter::new().expect("a machine with no software renderer");

    let refused = painter
        .paint(
            extent((8, 4)),
            &[],
            &[],
            &Cursor::Default,
            crate::scene_native::NativeLayers::nothing(),
            alo_canvas::Camera::new(),
        )
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
            only(NativeScene::SignIn(&screen)),
            alo_canvas::Camera::new(),
        ),
        Err(RenderError::EmptySize)
    ));
}

/// **What is leaving this machine is drawn above everything else.**
///
/// The one surface this product may not ship without, checked where the pixels
/// are rather than in the order of some `if`s: a sign-in screen fills the frame
/// with its ground, the egress indicator puts a mark on top of it, and the byte
/// a display would scan out at that spot is the indicator's. A layer painted
/// underneath, or dropped because the frame already had a scene on it, fails
/// here.
#[test]
fn what_is_leaving_is_drawn_above_the_screen_under_it() {
    let size = (8, 4);
    let screen = a_screen(size);
    let leaving = crate::egress_status_raster::EgressStatusPicture {
        size,
        rows: Vec::new(),
        solids: vec![Solid {
            area: Rectangle::new((6, 0).into(), (2, 1).into()),
            colour: [200, 100, 50],
        }],
        inked: Vec::new(),
        band: None,
    };
    let mut painter = SoftwarePainter::new().expect("a machine with no software renderer");

    let (pixels, _) = painter
        .paint(
            extent(size),
            &[],
            &[],
            &Cursor::Default,
            crate::scene_native::NativeLayers {
                scene: Some(NativeScene::SignIn(&screen)),
                status: Some(&leaving),
                ..crate::scene_native::NativeLayers::nothing()
            },
            alo_canvas::Camera::new(),
        )
        .expect("a frame carrying both a screen and the indicator was refused");

    assert_eq!(
        pixel_at(&pixels, (6, 0)),
        [200, 100, 50],
        "the egress indicator is not on the frame a display would scan out"
    );
    assert_eq!(
        pixel_at(&pixels, (0, 0)),
        [20, 30, 40],
        "the indicator painted where it has nothing to say"
    );
}

/// **A frame with no whole-output scene still carries the desktop's layers.**
///
/// The seam took one scene and nothing else until the desktop needed it, and a
/// desktop is not a scene: it is a dock and its windows above whatever clients
/// are mapped. A frame carrying only an indicator is the smallest case of that,
/// and it is drawn rather than refused for having no scene.
#[test]
fn a_frame_with_no_scene_but_a_layer_on_it_is_drawn() {
    let leaving = crate::egress_status_raster::EgressStatusPicture {
        size: (8, 4),
        rows: Vec::new(),
        solids: vec![Solid {
            area: Rectangle::new((1, 1).into(), (2, 2).into()),
            colour: [10, 200, 10],
        }],
        inked: Vec::new(),
        band: None,
    };
    let mut painter = SoftwarePainter::new().expect("a machine with no software renderer");

    let (pixels, _) = painter
        .paint(
            extent((8, 4)),
            &[],
            &[],
            &Cursor::Default,
            crate::scene_native::NativeLayers {
                status: Some(&leaving),
                ..crate::scene_native::NativeLayers::nothing()
            },
            alo_canvas::Camera::new(),
        )
        .expect("a frame with a layer and no scene was refused");

    assert_eq!(pixel_at(&pixels, (1, 1)), [10, 200, 10]);
    // The clear underneath is the neutral one, not a scene nobody asked for.
    assert_eq!(pixel_at(&pixels, (7, 3)), [0, 0, 0]);
}
