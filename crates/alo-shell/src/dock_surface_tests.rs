//! **The Dock's surface has the corners, the hairline and the shadow ADR 0101
//! ruled**, read back out of the pixels it rasterises.
//!
//! A raster can be asked what it drew, which is the whole reason this is worth
//! testing at the pixel rather than at the call: *the corner is rounded* is a
//! claim about coverage, and coverage is a number here.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None is the failure being reported, and a \
              formatted panic names the pixel it was reading"
)]

use alo_dock::Edge;
use smithay::utils::Rectangle;

use super::{ADocksSurface, surface};

/// A Dock's surface at the ordinary scale, with colours a test can tell apart.
fn a_surface(against: Edge) -> super::Washed {
    surface(ADocksSurface {
        panel: Rectangle::new((200, 400).into(), (600, 76).into()),
        fill: [255, 255, 255],
        edge: [0xE7, 0xEB, 0xEF],
        accent: [0x11, 0x22, 0x33],
        against,
        accent_is: 2,
        room: (1920, 1080),
        scale: 100,
    })
    .expect("a panel with area rasterises")
}

/// The four bytes at this point of the raster, or a panic naming where.
fn pixel(of: &super::Washed, x: i32, y: i32) -> [u8; 4] {
    let width = of.area.size.w;
    assert!(
        x >= 0 && y >= 0 && x < width && y < of.area.size.h,
        "({x}, {y}) is outside a raster of {width} by {}",
        of.area.size.h
    );
    let at = ((y as usize) * (width as usize) + (x as usize)) * 4;
    let bytes = of
        .pixels
        .get(at..at + 4)
        .unwrap_or_else(|| panic!("no pixel at ({x}, {y})"));
    // The workspace denies indexing a slice, in a test as much as anywhere: a
    // row that is not there should say which one was wanted rather than panic
    // with a bare index.
    let mut out = [0_u8; 4];
    out.copy_from_slice(bytes);
    out
}

/// Where the panel's own corner is, inside the raster.
fn panel_at(of: &super::Washed) -> (i32, i32) {
    (200 - of.area.loc.x, 400 - of.area.loc.y)
}

/// **The corners are rounded, and the middle of each edge is not.**
///
/// The claim a rectangle cannot make. At the panel's own corner pixel the
/// rounded shape has pulled away, so that pixel is nearly clear of the fill; at
/// the middle of the top edge, one pixel in, it is solid.
#[test]
fn the_corners_are_rounded_and_the_edges_are_not() {
    let drawn = a_surface(Edge::Bottom);
    let (left, top) = panel_at(&drawn);

    let corner = pixel(&drawn, left, top);
    let middle = pixel(&drawn, left + 300, top + 38);
    assert!(
        corner[3] < 64,
        "the panel's corner pixel is {} opaque, so the corner is square",
        corner[3]
    );
    assert_eq!(
        middle[3], 255,
        "the middle of the panel is not solid, so the fill is not filling"
    );

    // And the radius is the one ADR 0101 ruled: a pixel just inside the corner
    // along the diagonal is clear, and one at the radius along the top edge is
    // solid. Eighteen is the number; this checks either side of it.
    let r = i32::try_from(alo_dock::measures::THE_DOCKS_CORNER).expect("a radius fits");
    assert!(
        pixel(&drawn, left + 2, top + 2)[3] < 64,
        "two pixels along the diagonal from the corner is inside the radius"
    );
    assert_eq!(
        pixel(&drawn, left + r + 4, top + 1)[3],
        255,
        "past the radius along the top edge the panel is not solid"
    );
}

/// **There is a hairline, in the colour it was given, all the way round.**
///
/// Sampled at the middle of each of the four edges, one pixel in — which is
/// where a 1-pixel border is and nowhere else.
#[test]
fn a_hairline_runs_round_the_panel_in_its_own_colour() {
    let drawn = a_surface(Edge::Bottom);
    let (left, top) = panel_at(&drawn);
    let (right, bottom) = (left + 600 - 1, top + 76 - 1);

    for (named, x, y) in [
        ("the left edge", left, top + 38),
        ("the right edge", right, top + 38),
        ("the bottom edge", left + 300, bottom),
    ] {
        let [r, g, b, a] = pixel(&drawn, x, y);
        assert!(a > 200, "{named}: the hairline is {a} opaque");
        assert!(
            r < 250 && g < 252 && b < 252,
            "{named} is {r},{g},{b} — the panel's white rather than its hairline"
        );
    }

    // **The premise**: one pixel further in is the fill, so the three above are
    // not simply reading white and calling it a line.
    let [r, g, b, _] = pixel(&drawn, left + 2, top + 38);
    assert_eq!(
        [r, g, b],
        [255, 255, 255],
        "two pixels in is not the panel's fill, so this test cannot tell a hairline from a fill"
    );
}

/// **There is a shadow below the panel, and it fades.**
///
/// It falls downward by `THE_SHADOW_FALLS` and spreads by
/// `THE_SHADOW_SPREADS`, so just under the panel it is at its darkest and it is
/// weaker further down. Both halves are asserted: *there is one* and *it is not
/// a hard edge*.
#[test]
fn a_shadow_falls_below_the_panel_and_fades_with_distance() {
    let drawn = a_surface(Edge::Bottom);
    let (left, top) = panel_at(&drawn);
    let bottom = top + 76;

    let near = pixel(&drawn, left + 300, bottom + 4)[3];
    let far = pixel(&drawn, left + 300, bottom + 28)[3];
    assert!(near > 0, "there is no shadow under the panel at all");
    assert!(
        near > far,
        "the shadow is {near} just below the panel and {far} further down, so it does not fade"
    );
    assert!(
        far < near,
        "the shadow has a hard edge rather than a blur: {near} against {far}"
    );

    // **Above the panel it is lighter than below**, because the shadow is
    // offset downward rather than drawn evenly all round.
    let above = pixel(&drawn, left + 300, top - 12)[3];
    assert!(
        above < near,
        "the shadow is {above} above the panel and {near} below it, so it is not offset"
    );
}

/// **The accent lies on the face that looks at the canvas, on every edge.**
///
/// The same rule `dock_raster::the_accent_on` holds for the band, now that this
/// is where that face is drawn. Four edges, because the fault this forbids is
/// the one the accent already had once: a face chosen by an axis rather than by
/// which way the canvas is.
#[test]
fn the_accent_lies_on_the_face_that_looks_at_the_canvas() {
    for edge in Edge::EVERY {
        let drawn = a_surface(edge);
        let (left, top) = panel_at(&drawn);
        let (right, bottom) = (left + 600 - 1, top + 76 - 1);

        // Two pixels in from each face, which is inside the hairline and inside
        // a 2-pixel accent.
        let inward = match edge {
            Edge::Bottom => (left + 300, top + 1),
            Edge::Top => (left + 300, bottom - 1),
            Edge::Left => (right - 1, top + 38),
            Edge::Right => (left + 1, top + 38),
        };
        let opposite = match edge {
            Edge::Bottom => (left + 300, bottom - 1),
            Edge::Top => (left + 300, top + 1),
            Edge::Left => (left + 1, top + 38),
            Edge::Right => (right - 1, top + 38),
        };

        let [r, g, b, _] = pixel(&drawn, inward.0, inward.1);
        assert_eq!(
            [r, g, b],
            [0x11, 0x22, 0x33],
            "{edge:?}: the face looking at the canvas is not the accent"
        );
        let [r, g, b, _] = pixel(&drawn, opposite.0, opposite.1);
        assert_ne!(
            [r, g, b],
            [0x11, 0x22, 0x33],
            "{edge:?}: the accent is on both faces, so it is not following the edge"
        );
    }
}

/// **A panel with no area rasterises nothing**, which is a Dock that gave way
/// rather than a failure.
///
/// **Zero only, because a negative size cannot be built.** This walked `-4`
/// too, and smithay's `Size` panics on construction — *attempting to create a
/// `Size` of negative size* — so the case is the type's to refuse and not this
/// function's. The guard inside still reads `<= 0` rather than `== 0`: it costs
/// nothing and it does not depend on a invariant another crate holds.
#[test]
fn a_panel_with_no_area_draws_nothing() {
    for (w, h) in [(0, 76), (600, 0), (0, 0)] {
        let nothing = surface(ADocksSurface {
            panel: Rectangle::new((200, 400).into(), (w, h).into()),
            fill: [255, 255, 255],
            edge: [0xE7, 0xEB, 0xEF],
            accent: [0x11, 0x22, 0x33],
            against: Edge::Bottom,
            accent_is: 2,
            room: (1920, 1080),
            scale: 100,
        });
        assert!(nothing.is_none(), "a {w} by {h} panel rasterised something");
    }
}

/// **The radius and the hairline pass through the display's scale.**
///
/// `CLAUDE.md`'s standing rule: no figure reaches a display without passing
/// through that display's scale. A dense screen draws the same Dock sharper,
/// not a Dock with a corner half the size — so at 200% the rounded part of the
/// corner reaches twice as far.
#[test]
fn a_dense_display_gets_the_same_corner_drawn_larger() {
    let of = |scale| {
        surface(ADocksSurface {
            panel: Rectangle::new((200, 400).into(), (600, 152).into()),
            fill: [255, 255, 255],
            edge: [0xE7, 0xEB, 0xEF],
            accent: [0x11, 0x22, 0x33],
            against: Edge::Bottom,
            accent_is: 2,
            room: (1920, 1080),
            scale,
        })
        .expect("a panel with area rasterises")
    };
    let ordinary = of(100);
    let dense = of(200);

    // How far along the top edge the panel first becomes solid, which is the
    // corner's reach.
    let reach = |drawn: &super::Washed| {
        let (left, top) = panel_at(drawn);
        (0..120)
            .find(|step| pixel(drawn, left + step, top + 1)[3] == 255)
            .unwrap_or_else(|| panic!("the top edge never becomes solid"))
    };
    let (one, two) = (reach(&ordinary), reach(&dense));
    assert!(
        two > one,
        "at 200% the corner reaches {two} and at 100% it reaches {one}, so the radius is not \
         passing through the display's scale"
    );
}
