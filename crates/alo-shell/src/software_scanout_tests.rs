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

/// **Every window's edge is painted, not the first one.**
///
/// Until 2026-10-10 the edge travelled in [`crate::scene_native::NativeLayers`]'s
/// `scene`, a field that holds **one of** the lock screen, the window controls,
/// the reader, the sign-in screen or the recovery screen. So a machine with
/// three windows open drew exactly one edge, and the only caller that built them
/// — `examples/a_real_application.rs` — built every window's and then passed
/// `edges.first()`. Every line of that was correct for the signature it had.
///
/// **This is the assertion that shape could not satisfy.** Two windows at
/// different heights, each with its own edge, painted together; the second
/// window's strip must differ from the ground it would be if nothing were drawn
/// there. A painter that took `edges.first()`, or that broke out of the loop,
/// leaves that region untouched and fails here.
///
/// It reads the scanned-out bytes rather than counting calls, because *painted*
/// is a claim about pixels a display would receive.
#[test]
fn both_windows_edges_reach_the_bytes_a_display_would_scan_out() {
    use crate::window_edge::{Decorations, edge_of};
    use crate::window_edge_paint::Pointing;
    use crate::window_edge_picture::EdgePicture;
    use smithay::utils::Logical;

    /// Both bundled faces, as the shell loads them and as
    /// `crate::window_edge_picture`'s own tests do.
    fn fonts() -> cosmic_text::FontSystem {
        let mut fonts = cosmic_text::FontSystem::new();
        fonts
            .db_mut()
            .load_font_data(include_bytes!("../fonts/Manrope.ttf").to_vec());
        fonts
            .db_mut()
            .load_font_data(include_bytes!("../fonts/Inter.ttf").to_vec());
        fonts
    }

    let size = (400, 400);
    let mut faces = fonts();
    // Two windows, far enough apart that their edges share no row.
    let upper = Rectangle::new((20, 120).into(), (200, 60).into());
    let lower = Rectangle::new((20, 300).into(), (200, 60).into());
    let first = EdgePicture::of(
        &edge_of(upper, Decorations::TheShellDraws, true),
        Pointing::default(),
        None,
        &mut faces,
        size,
        100,
    )
    .expect("the upper window's edge fits on this output");
    let second = EdgePicture::of(
        &edge_of(lower, Decorations::TheShellDraws, true),
        Pointing::default(),
        None,
        &mut faces,
        size,
        100,
    )
    .expect("the lower window's edge fits on this output");

    /// Which frame row an edge drew into, so the two cannot be confused.
    ///
    /// A `u32` because that is what `pixel_at` reads, and the conversion is
    /// justified here once instead of at four call sites: both windows below sit
    /// well below their own edge's height, so the row is positive by
    /// construction and a negative one would mean this test's fixture had
    /// changed rather than the painter.
    fn a_row_inside(window: Rectangle<i32, Logical>) -> u32 {
        let row = window.loc.y - crate::window_edge::THE_REGION_IS_TALL
            + crate::window_edge::THE_STRIP_STARTS_AT
            + 4;
        u32::try_from(row).expect("both windows in this test sit below their own edge")
    }

    let mut painter = SoftwarePainter::new().expect("a machine with no software renderer");
    let paint = |painter: &mut SoftwarePainter, edges: &[EdgePicture]| {
        painter
            .paint(
                extent(size),
                &[],
                &[],
                &Cursor::Default,
                crate::scene_native::NativeLayers {
                    edges,
                    ..crate::scene_native::NativeLayers::nothing()
                },
                alo_canvas::Camera::new(),
            )
            .expect("a frame carrying window edges was refused")
            .0
    };

    let only_the_first = paint(&mut painter, std::slice::from_ref(&first));
    let both = paint(&mut painter, &[first, second]);

    let upper_row = a_row_inside(upper);
    let lower_row = a_row_inside(lower);
    let column = 60_u32;

    // **The premise, asserted rather than assumed.** The first edge really did
    // draw, so this test cannot pass by nothing ever being painted — which is
    // how the four-control test and the maximised-geometry test in this crate
    // both passed vacuously before somebody looked.
    assert_ne!(
        pixel_at(&only_the_first, (column, upper_row)),
        pixel_at(&only_the_first, (column, lower_row)),
        "painting one edge left the upper and lower rows identical, so nothing was drawn at \
         all and every comparison below would be between two untouched grounds"
    );

    // **The claim.** Adding the second window's edge changes its own rows.
    assert_ne!(
        pixel_at(&only_the_first, (column, lower_row)),
        pixel_at(&both, (column, lower_row)),
        "the second window's edge did not reach the frame. A painter that draws `edges.first()`, \
         or that returns from the loop after one, leaves this row exactly as it was"
    );

    // **And it changes nothing of the first's.** Each edge sits on its own
    // window; a second one that moved the first would be a layer drawn at the
    // wrong origin.
    assert_eq!(
        pixel_at(&only_the_first, (column, upper_row)),
        pixel_at(&both, (column, upper_row)),
        "adding a second window's edge moved the first window's"
    );
}
