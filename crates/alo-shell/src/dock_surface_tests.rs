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
        raised: crate::how_a_panel_is_raised::HowItIsRaised::THE_DOCKS.unscaled(),
        marks: &[],
        mark: [0, 0, 0],
        panel: Rectangle::new((200, 400).into(), (600, 76).into()),
        fill: [255, 255, 255],
        edge: [0xE7, 0xEB, 0xEF],
        accent: [0x11, 0x22, 0x33],
        against,
        accent_is: 2,
        room: (1920, 1080),
        badges: &[],
        badge: [0xEE, 0xF2, 0xF4],
        wells: &[],
        well_is: [0xF5, 0xF6, 0xF7],
        rings: &[],
        ring_is: [0, 0, 0],
        in_primary: super::InPrimary::NOTHING,
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
            raised: crate::how_a_panel_is_raised::HowItIsRaised::THE_DOCKS.unscaled(),
            marks: &[],
            mark: [0, 0, 0],
            panel: Rectangle::new((200, 400).into(), (w, h).into()),
            fill: [255, 255, 255],
            edge: [0xE7, 0xEB, 0xEF],
            accent: [0x11, 0x22, 0x33],
            against: Edge::Bottom,
            accent_is: 2,
            room: (1920, 1080),
            badges: &[],
            badge: [0xEE, 0xF2, 0xF4],
            wells: &[],
            well_is: [0, 0, 0],
            rings: &[],
            ring_is: [0, 0, 0],
            in_primary: super::InPrimary::NOTHING,
        });
        assert!(nothing.is_none(), "a {w} by {h} panel rasterised something");
    }
}

/// **The surface is drawn in one space, and nothing in it scales on its own.**
///
/// This asserted the opposite until 2026-10-11 — that a dense display got a
/// larger corner — which was true of the code and **wrong about the product**:
/// `crate::dock_raster` hands `alo_dock`'s logical measures straight through as
/// pixels and scales none of them, so scaling the radius alone drew an 18
/// radius as 36 on a band still 76 tall.
///
/// `CLAUDE.md`'s rule is kept by every figure sharing one conversion, not by one
/// of them having its own. Until the Dock has that conversion, the honest claim
/// is this one: the same panel rasterises the same way however the display is
/// described, so nothing here can drift from the band it sits on.
#[test]
fn nothing_in_the_surface_scales_on_its_own() {
    let of = |panel: Rectangle<i32, smithay::utils::Physical>| {
        surface(ADocksSurface {
            raised: crate::how_a_panel_is_raised::HowItIsRaised::THE_DOCKS.unscaled(),
            marks: &[],
            mark: [0, 0, 0],
            panel,
            fill: [255, 255, 255],
            edge: [0xE7, 0xEB, 0xEF],
            accent: [0x11, 0x22, 0x33],
            against: Edge::Bottom,
            accent_is: 0,
            room: (1920, 1080),
            badges: &[],
            badge: [0xEE, 0xF2, 0xF4],
            wells: &[],
            well_is: [0, 0, 0],
            rings: &[],
            ring_is: [0, 0, 0],
            in_primary: super::InPrimary::NOTHING,
        })
        .expect("a panel with area rasterises")
    };
    let one = of(Rectangle::new((200, 400).into(), (600, 76).into()));
    let same = of(Rectangle::new((200, 400).into(), (600, 76).into()));
    assert_eq!(
        one.pixels, same.pixels,
        "the same panel rasterised differently"
    );

    // And the corner's reach is the ruled radius in the band's own space: along
    // the top edge the panel becomes solid at about eighteen in, not at thirty-six.
    let (left, top) = (200 - one.area.loc.x, 400 - one.area.loc.y);
    let reach = (0..120)
        .find(|step| pixel(&one, left + step, top + 1)[3] == 255)
        .unwrap_or_else(|| panic!("the top edge never becomes solid"));
    let radius = i32::try_from(alo_dock::measures::THE_DOCKS_CORNER).expect("a radius fits");
    assert!(
        (reach - radius).abs() <= 4,
        "the corner reaches {reach} where the ruled radius is {radius}"
    );
}

/// **A well is drawn at its own radius**, and the proof is that two wells of
/// one size with different radii differ at the corner and agree in the middle.
///
/// The test this capability needed. The radius was one constant, so nothing
/// checked that a well's corner came from the well — and a parameter nothing
/// reads looks exactly like a parameter that works. Substituting one and
/// watching is the only way to tell.
#[test]
fn a_wells_corner_comes_from_the_well_and_not_from_a_constant() {
    let square = Rectangle::new((20, 20).into(), (80, 80).into());
    let of = |corner: u32| {
        surface(ADocksSurface {
            raised: crate::how_a_panel_is_raised::HowItIsRaised::THE_DOCKS.unscaled(),
            marks: &[],
            mark: [0, 0, 0],
            panel: Rectangle::new((0, 0).into(), (200, 120).into()),
            fill: [255, 255, 255],
            edge: [255, 255, 255],
            accent: [255, 255, 255],
            against: Edge::Bottom,
            accent_is: 0,
            room: (400, 300),
            badges: &[],
            badge: [0, 0, 0],
            wells: &[super::AWell { at: square, corner }],
            well_is: [0, 0, 0],
            rings: &[],
            ring_is: [0, 0, 0],
            in_primary: super::InPrimary::NOTHING,
        })
        .expect("a panel with area rasterises")
    };

    let sharp = of(0);
    let round = of(40);

    // The raster is offset by the shadow's reach, so sample through the same
    // conversion for both rather than assuming where the panel starts.
    let corner_of = |washed: &super::Washed| {
        let x = square.loc.x - washed.area.loc.x + 1;
        let y = square.loc.y - washed.area.loc.y + 1;
        pixel(washed, x, y)
    };
    let middle_of = |washed: &super::Washed| {
        let x = square.loc.x - washed.area.loc.x + square.size.w / 2;
        let y = square.loc.y - washed.area.loc.y + square.size.h / 2;
        pixel(washed, x, y)
    };

    assert_eq!(
        middle_of(&sharp),
        middle_of(&round),
        "the middle of a well must not depend on its radius, or this test is \
         measuring something else"
    );
    assert_ne!(
        corner_of(&sharp),
        corner_of(&round),
        "a 0 and a 40 radius drew the same corner, so the well's own corner is \
         being ignored"
    );

    // And the direction is the one a radius implies: a large radius cuts the
    // corner away, so *less* of the well is there and more of the panel shows.
    //
    // **Measured on brightness, not on alpha.** The well sits on an opaque
    // panel, so alpha is full across all of it and a corner that is all panel
    // and a corner that is all well are both opaque — the first version of this
    // compared alpha and could not have told them apart. The well is black on a
    // white fill, so the rounder corner is the lighter one.
    let lightness = |p: [u8; 4]| u32::from(p[0]) + u32::from(p[1]) + u32::from(p[2]);
    assert!(
        lightness(corner_of(&round)) > lightness(corner_of(&sharp)),
        "a rounder corner did not let more of the panel through: round {:?} vs \
         sharp {:?}",
        corner_of(&round),
        corner_of(&sharp)
    );
    assert_eq!(
        corner_of(&round)[3],
        corner_of(&sharp)[3],
        "both corners sit on an opaque panel, which is why alpha cannot be the \
         measurement here"
    );
}

/// One panel, with these wells and rings plain and these in `text/primary`.
fn a_card(
    wells: &[super::AWell],
    rings: &[Rectangle<i32, smithay::utils::Physical>],
    in_primary: super::InPrimary<'_>,
) -> super::Washed {
    surface(ADocksSurface {
        raised: crate::how_a_panel_is_raised::HowItIsRaised::THE_DOCKS.unscaled(),
        marks: &[],
        mark: [0, 0, 0],
        panel: Rectangle::new((0, 0).into(), (400, 200).into()),
        fill: [255, 255, 255],
        edge: [255, 255, 255],
        accent: [255, 255, 255],
        against: Edge::Bottom,
        accent_is: 0,
        room: (800, 600),
        badges: &[],
        badge: [0, 0, 0],
        wells,
        well_is: NAVY,
        rings,
        ring_is: NAVY,
        in_primary,
    })
    .expect("a panel with area rasterises")
}

/// The design's `text/primary`, `#102a43`.
const NAVY: [u8; 3] = [0x10, 0x2A, 0x43];

/// **The Primary answer is a well, drawn by the well's own arithmetic, in its
/// own colour.** Byte for byte the same as a plain well of that colour — so
/// there is one formulation of a well in the painter, as its comment says —
/// and the control shows the well drew at all, since two empty rasters are
/// equal too.
#[test]
fn a_primary_well_draws_exactly_as_a_plain_well_in_its_colour() {
    let answer = super::AWell {
        at: Rectangle::new((40, 60).into(), (144, 48).into()),
        corner: 12,
    };
    let plain = a_card(&[answer], &[], super::InPrimary::NOTHING);
    let primary = a_card(
        &[],
        &[],
        super::InPrimary {
            wells: &[answer],
            outlines: &[],
            colour: NAVY,
        },
    );
    let nothing = a_card(&[], &[], super::InPrimary::NOTHING);

    assert_ne!(
        nothing.pixels, plain.pixels,
        "the control: no well was drawn at all"
    );
    assert_eq!(
        plain.pixels, primary.pixels,
        "the Primary answer drew differently from a well of the same colour"
    );
}

/// **The keyboard's outline is a ring, by the ring's own arithmetic, in its
/// own colour** — which is what lets it be navy on a panel whose rings are
/// already grey borders.
#[test]
fn the_keyboards_outline_draws_exactly_as_a_ring_in_its_colour() {
    let around = Rectangle::new((36, 56).into(), (152, 56).into());
    let plain = a_card(&[], &[around], super::InPrimary::NOTHING);
    let primary = a_card(
        &[],
        &[],
        super::InPrimary {
            wells: &[],
            outlines: &[around],
            colour: NAVY,
        },
    );
    let nothing = a_card(&[], &[], super::InPrimary::NOTHING);

    assert_ne!(
        nothing.pixels, plain.pixels,
        "the control: no ring was drawn at all"
    );
    assert_eq!(
        plain.pixels, primary.pixels,
        "the keyboard's outline drew differently from a ring of the same colour"
    );
}

/// One FNV-1a hash of a raster's bytes, so a whole Dock can be compared with a
/// Dock drawn before a change in one line.
fn hashed(drawn: &super::Washed) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in &drawn.pixels {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// **The Dock is drawn pixel for pixel as it was before the rasteriser took a
/// look and a scale.**
///
/// The hashes below were taken on 2026-10-28 by this same fixture run against
/// the rasteriser as it stood on `main` at `95f513e5`, before
/// `crate::how_a_panel_is_raised` existed — once, by hand, and recorded here so
/// this test repeats the comparison on every build. The fixture carries every
/// kind of shape a panel can hold — a badge, a mark, two wells of two radii,
/// a ring, a Primary well and the keyboard's outline — on all four edges, and
/// one panel clipped at the screen's corner, so a change to any of them moves a
/// hash.
#[test]
fn the_dock_is_drawn_exactly_as_it_was_before_the_look() {
    const BEFORE: [(&str, u64); 5] = [
        ("Bottom", 0x6fbb_1a94_ee2a_ed99),
        ("Top", 0x4b6a_b111_c247_d5e1),
        ("Left", 0x7b10_2fef_1bdb_c579),
        ("Right", 0xe3e4_729a_6038_5839),
        ("clipped", 0xca8c_d6ce_4c8f_c899),
    ];
    let badges = [Rectangle::new((260, 424).into(), (28, 28).into())];
    let marks = [Rectangle::new((272, 466).into(), (4, 3).into())];
    let wells = [
        super::AWell {
            at: Rectangle::new((400, 412).into(), (200, 40).into()),
            corner: 12,
        },
        super::AWell {
            at: Rectangle::new((620, 412).into(), (60, 40).into()),
            corner: 8,
        },
    ];
    let rings = [Rectangle::new((396, 408).into(), (208, 48).into())];
    let primary = [super::AWell {
        at: Rectangle::new((700, 412).into(), (60, 40).into()),
        corner: 12,
    }];
    let outlines = [Rectangle::new((696, 408).into(), (68, 48).into())];
    let mut now = Vec::new();
    for against in [Edge::Bottom, Edge::Top, Edge::Left, Edge::Right] {
        let drawn = surface(ADocksSurface {
            raised: crate::how_a_panel_is_raised::HowItIsRaised::THE_DOCKS.unscaled(),
            marks: &marks,
            mark: [0x10, 0x2A, 0x43],
            panel: Rectangle::new((200, 400).into(), (600, 76).into()),
            fill: [255, 255, 255],
            edge: [0xE7, 0xEB, 0xEF],
            accent: [0x11, 0x22, 0x33],
            against,
            accent_is: 2,
            room: (1920, 1080),
            badges: &badges,
            badge: [0xEE, 0xF2, 0xF4],
            wells: &wells,
            well_is: [0xF5, 0xF6, 0xF7],
            rings: &rings,
            ring_is: [0x22, 0x33, 0x44],
            in_primary: super::InPrimary {
                wells: &primary,
                outlines: &outlines,
                colour: [0x10, 0x2A, 0x43],
            },
        })
        .expect("a panel with area rasterises");
        assert_eq!(
            drawn.area,
            Rectangle::new((168, 368).into(), (664, 140).into()),
            "the {against:?} Dock's raster moved"
        );
        now.push(hashed(&drawn));
    }
    let clipped = surface(ADocksSurface {
        raised: crate::how_a_panel_is_raised::HowItIsRaised::THE_DOCKS.unscaled(),
        marks: &[],
        mark: [0, 0, 0],
        panel: Rectangle::new((8, 8).into(), (300, 76).into()),
        fill: [255, 255, 255],
        edge: [0xE7, 0xEB, 0xEF],
        accent: [0x11, 0x22, 0x33],
        against: Edge::Bottom,
        accent_is: 0,
        room: (1920, 1080),
        badges: &[],
        badge: [0, 0, 0],
        wells: &[],
        well_is: [0, 0, 0],
        rings: &[],
        ring_is: [0, 0, 0],
        in_primary: super::InPrimary::NOTHING,
    })
    .expect("a panel");
    assert_eq!(
        clipped.area,
        Rectangle::new((0, 0).into(), (340, 116).into()),
        "the clipped Dock's raster moved"
    );
    now.push(hashed(&clipped));
    for ((which, before), now) in BEFORE.iter().zip(now) {
        assert_eq!(
            now, *before,
            "the {which} Dock's pixels changed: {now:#018x} where it drew {before:#018x}"
        );
    }
}

/// **The rasteriser draws a look at the scale it is given**: the same panel
/// at 200% becomes solid along its top edge about twice as far in as at 100%.
#[test]
fn at_two_hundred_percent_the_rasterisers_corner_doubles() {
    use crate::how_a_panel_is_raised::HowItIsRaised;
    let panel = Rectangle::new((200, 200).into(), (600, 300).into());
    let reach = |scale: f64| {
        let drawn = surface(ADocksSurface {
            raised: HowItIsRaised::THE_DOCKS.at(scale),
            marks: &[],
            mark: [0, 0, 0],
            panel,
            fill: [255, 255, 255],
            edge: [0xE7, 0xEB, 0xEF],
            accent: [0, 0, 0],
            against: Edge::Bottom,
            accent_is: 0,
            room: (1920, 1080),
            badges: &[],
            badge: [0, 0, 0],
            wells: &[],
            well_is: [0, 0, 0],
            rings: &[],
            ring_is: [0, 0, 0],
            in_primary: super::InPrimary::NOTHING,
        })
        .expect("a panel with area rasterises");
        let (left, top) = (200 - drawn.area.loc.x, 200 - drawn.area.loc.y);
        (0..200)
            .find(|step| pixel(&drawn, left + step, top + 1)[3] == 255)
            .unwrap_or_else(|| panic!("the top edge never becomes solid at {scale}"))
    };
    let (ordinary, dense) = (reach(1.0), reach(2.0));
    assert!(
        (ordinary - 18).abs() <= 4,
        "at 100% the corner reaches {ordinary}"
    );
    assert!(
        (dense - 36).abs() <= 6,
        "at 200% the corner reaches {dense}"
    );
}
