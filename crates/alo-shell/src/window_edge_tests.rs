//! The edge, held against the design file's own coordinates.
//!
//! **These assert the numbers in `figma-snapshot/398-26305.xml`, not numbers
//! this file chose.** The component is a 600-wide specimen, so feeding 600 in
//! must give back the node positions the designer drew — 456, 504, 552 and the
//! rest. A test that asserted what the code computes would pass whatever the
//! code computed, which is the failure this whole day keeps finding.
//!
//! The specimen is placed so the edge's region lands at the origin: the edge
//! sits 44 above its window, so a window at `y = 44` has a region at `y = 0`
//! and every figure below reads directly against the capture.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or a missing element is the failure being reported"
)]

use super::*;

/// A window whose edge is a 600-wide specimen at the origin.
fn specimen() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((0, 44)), Size::from((600, 400)))
}

/// **The region is 44 and the strip is 32 at y 12.** `402:26313` is named
/// *32px visible edge / 44px interaction* and is exactly that.
#[test]
fn the_region_is_forty_four_and_the_strip_is_thirty_two_at_twelve() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    assert_eq!(
        edge.region,
        Rectangle::new(Point::from((0, 0)), Size::from((600, 44)))
    );
    let strip = edge.strip.expect("a revealed edge has a strip");
    assert_eq!(
        strip,
        Rectangle::new(Point::from((0, 12)), Size::from((600, 32)))
    );
}

/// **Shell-owned controls land on the design's own coordinates.**
///
/// `398:26251` Minimise at 456, `398:26254` Maximise at 504, `398:26257` Close
/// at 552 — each 44 × 44, each holding 14 × 14 artwork at local x 15, y 21.
#[test]
fn the_shells_three_controls_are_where_the_design_drew_them() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    let at: Vec<(OnTheEdge, i32)> = edge
        .controls
        .iter()
        .map(|control| (control.does, control.target.loc.x))
        .collect();
    assert_eq!(
        at,
        vec![
            (OnTheEdge::Minimise, 456),
            (OnTheEdge::Maximise, 504),
            (OnTheEdge::Close, 552),
        ]
    );
    for control in &edge.controls {
        assert_eq!(
            control.target.size,
            Size::from((44, 44)),
            "{:?}",
            control.does
        );
        assert_eq!(
            control.artwork.size,
            Size::from((14, 14)),
            "{:?}",
            control.does
        );
        // Centred across the target, and centred down the *strip*: x 15 of 44,
        // y 21 of the region — not y 15, which centring in the target gives.
        assert_eq!(control.artwork.loc.x - control.target.loc.x, 15);
        assert_eq!(control.artwork.loc.y, 21);
    }
}

/// **An application that draws its own header gets a menu and nothing else.**
///
/// `398:26238`, a lone Menu at 552 — the owner's ruling that there are no
/// duplicate window buttons, measured in the design rather than asserted.
#[test]
fn an_application_with_its_own_header_gets_only_a_menu() {
    let edge = edge_of(specimen(), Decorations::TheApplicationDraws, true);
    assert_eq!(edge.controls.len(), 1, "{:?}", edge.controls);
    assert_eq!(edge.controls[0].does, OnTheEdge::Menu);
    assert_eq!(edge.controls[0].target.loc.x, 552);
    for control in &edge.controls {
        assert_ne!(control.does, OnTheEdge::Minimise);
        assert_ne!(control.does, OnTheEdge::Maximise);
        assert_ne!(control.does, OnTheEdge::Close);
    }
}

/// **The drag region is what the design drew in both cases**: `398:26247` is
/// x 8 width 444 with three controls, `398:26234` is x 8 width 540 with one.
#[test]
fn the_drag_region_takes_everything_the_controls_do_not() {
    let shell = edge_of(specimen(), Decorations::TheShellDraws, true);
    assert_eq!(
        shell.drag,
        Rectangle::new(Point::from((8, 0)), Size::from((444, 44)))
    );
    let app = edge_of(specimen(), Decorations::TheApplicationDraws, true);
    assert_eq!(
        app.drag,
        Rectangle::new(Point::from((8, 0)), Size::from((540, 44)))
    );
}

/// **The grip and the title sit inside the drag region**, so their positions
/// are 8 further right than the region's own: `398:26248` grip at local 8 is
/// 16 of the edge, `398:26250` title at local 30 is 38.
#[test]
fn the_grip_and_the_title_are_inside_the_drag_region() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    assert_eq!(
        edge.grip,
        Rectangle::new(Point::from((16, 21)), Size::from((14, 14)))
    );
    assert_eq!(edge.title, Some(Point::from((38, 19))));
}

/// **At rest there is no strip, no control, and a 24 × 2 grip.**
///
/// `398:26241` holds a drag region at y 28 of 588, and `398:26244` a grip of
/// 24 × 2 at local x 282 — which is 290 of the edge, centred on 302. Centring
/// on the edge rather than on the drag region would give 288, and being wrong
/// by two is exactly what nobody sees and the design does.
#[test]
fn at_rest_the_edge_is_a_quiet_grip_and_nothing_else() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, false);
    assert!(edge.strip.is_none(), "{:?}", edge.strip);
    assert!(edge.controls.is_empty(), "{:?}", edge.controls);
    assert!(edge.title.is_none());
    assert_eq!(
        edge.drag,
        Rectangle::new(Point::from((8, 28)), Size::from((588, 12)))
    );
    assert_eq!(
        edge.grip,
        Rectangle::new(Point::from((290, 33)), Size::from((24, 2)))
    );
}

/// **An application's own header changes nothing at rest.** `398:26228` and
/// `398:26241` are identical node for node, which is the measured claim.
#[test]
fn at_rest_both_owners_are_the_same_edge() {
    let specimen = specimen();
    assert_eq!(
        edge_of(specimen, Decorations::TheApplicationDraws, false),
        edge_of(specimen, Decorations::TheShellDraws, false)
    );
}

/// **Revealing moves nothing of the application's.**
///
/// The contract: *application content starts below that region and stays
/// stationary when the strip reveals.* The region is above the window and
/// identical in both states, so there is nowhere for the content to go.
#[test]
fn revealing_does_not_move_the_application() {
    let window = specimen();
    let at_rest = edge_of(window, Decorations::TheShellDraws, false);
    let revealed = edge_of(window, Decorations::TheShellDraws, true);
    assert_eq!(at_rest.region, revealed.region);
    assert_eq!(
        at_rest.region.loc.y + at_rest.region.size.h,
        window.loc.y,
        "the region must end exactly where the application begins"
    );
}

/// **The edge fits a window of any width, which is what the specimen is for.**
///
/// The contract: *600 is a specimen width, not a fixed window width.* Controls
/// stay at the trailing edge and the drag region takes what is left.
#[test]
fn the_edge_fits_whatever_window_it_is_on() {
    for wide in [320, 600, 936, 1920] {
        let window = Rectangle::new(Point::from((0, 44)), Size::from((wide, 400)));
        let edge = edge_of(window, Decorations::TheShellDraws, true);
        assert_eq!(edge.region.size.w, wide);
        let last = edge.controls.last().expect("three controls");
        assert_eq!(
            last.target.loc.x + last.target.size.w,
            wide - 4,
            "the last control must end one gap from the trailing edge at {wide}"
        );
        assert!(
            edge.drag.size.w > 0,
            "a {wide}-wide window left no room to drag"
        );
    }
}

/// **No two controls overlap, at any width.** The contract asks for
/// non-overlapping targets by name, and an overlap is a press that does the
/// wrong thing rather than something that merely looks wrong.
#[test]
fn no_two_controls_ever_overlap() {
    for wide in [320, 600, 936, 1920] {
        let window = Rectangle::new(Point::from((0, 44)), Size::from((wide, 400)));
        let edge = edge_of(window, Decorations::TheShellDraws, true);
        for (which, control) in edge.controls.iter().enumerate() {
            for other in edge.controls.iter().skip(which + 1) {
                assert!(
                    control.target.intersection(other.target).is_none(),
                    "{:?} and {:?} overlap at {wide}",
                    control.does,
                    other.does
                );
            }
            assert!(
                control.target.intersection(edge.drag).is_none(),
                "{:?} overlaps the drag region at {wide}",
                control.does
            );
        }
    }
}

/// **The edge moves with its window.** It is derived from the window's own
/// rectangle, so there is no state to forget to update.
#[test]
fn the_edge_moves_with_its_window() {
    let moved = Rectangle::new(Point::from((300, 244)), Size::from((600, 400)));
    let edge = edge_of(moved, Decorations::TheShellDraws, true);
    assert_eq!(edge.region.loc, Point::from((300, 200)));
    assert_eq!(edge.controls[0].target.loc.x, 300 + 456);
}
