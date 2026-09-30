//! Task 6's region contract, from `docs/autonomy/putting-a-window-aside.md`, and the owner's
//! invariant in their own words:
//!
//! > Activation strips and pointer paths must follow those bounds too, so **one pointer
//! > position cannot reveal two surfaces.**
//!
//! # What this file can and cannot demonstrate
//!
//! It holds the contract: that every part of the panel keeps it open, that the strip and the
//! surface are different answers, that the path between parts is not a gap, and that the
//! at-most-one rule is checkable rather than asserted.
//!
//! **It is not evidence that edge reveal works.** Nothing on a machine reaches any of this:
//! `alo-shell` has no full screen, so the situation the reveal exists for cannot be created,
//! and the ledger entry for this promise says *no evidence* until something does. A crate with
//! passing tests and no caller is reusable groundwork — citing it would make the record claim a
//! person can reach something nobody can.
//!
//! # The grid, and why it is at two widths
//!
//! The invariant is checked over a grid of points rather than at a few chosen ones, because a
//! rule about overlap tested where the surfaces are far apart is tested where it cannot fail.
//! And at **two panel widths**, collapsed and expanded, because the reserved area follows the
//! panel's current width — so the top and bottom bounds move with it, and the expanded width is
//! where the top region is smallest and the shared corner nearest.
//!
//! The classifier under test here is a **stand-in**, written in this file, not the real one —
//! the real one lives where the coordinates do. What is being tested is that the contract can
//! express a correct classifier and catch an incorrect one, which is the most a crate holding
//! no coordinates can honestly claim.
//! # It asks for no lint exemption, because it needs none
//!
//! Every other test file in this crate carries `#![expect(clippy::unwrap_used, ...)]`, and this
//! one was written with the same block out of habit. Clippy reported the expectation as
//! **unfulfilled**: nothing here unwraps, expects or panics, because the contract is values and
//! the assertions compare them.
//!
//! The block is gone rather than kept for symmetry. **Carrying a permission nothing needs is
//! how a file ends up permitting more than its author read** — the same finding as a forbidden
//! name nobody could write, and clippy caught this one the way a check that can fail is
//! supposed to.

use alo_put_aside::the_region_the_panel_claims::{
    PartOfThePanel, WhatEachSurfaceSaid, WhereThePointerIs, WhichEdge,
    at_most_one_surface_claims_it,
};

/// **Every part of the panel keeps it open, and the path between them is one of them.**
///
/// The path is the part that is easy to leave out, and leaving it out is the flicker: a pointer
/// travelling from a preview to a control crosses it, and answering *elsewhere* there conceals
/// the panel under a pointer that never left it.
#[test]
fn every_part_of_the_panel_keeps_it_open_including_the_path_between() {
    for part in [
        PartOfThePanel::APreview,
        PartOfThePanel::AControl,
        PartOfThePanel::AMenu,
        PartOfThePanel::ThePathBetweenThem,
    ] {
        let where_it_is = WhereThePointerIs::OnThePanel(part);
        assert!(
            where_it_is.holds_the_panel_open(),
            "{part:?} does not keep the panel open, so a pointer over it conceals the panel"
        );
        assert!(
            !where_it_is.is_the_asking_strip(),
            "{part:?} was reported as the asking strip, which would reveal a concealed panel"
        );
        assert_eq!(where_it_is.part(), Some(part));
    }
}

/// **The strip and the surface are different answers**, because they do different things.
///
/// The strip reveals from concealed; the surface only keeps what is already revealed. A
/// classifier reporting the panel's own area as the strip would reveal a concealed panel from a
/// pointer resting where the panel *would* be.
#[test]
fn the_asking_strip_is_not_the_panel_itself() {
    let strip = WhereThePointerIs::InTheStrip;
    assert!(strip.is_the_asking_strip());
    assert!(strip.holds_the_panel_open());
    assert_eq!(strip.part(), None, "the strip is not a part of the panel");

    let on_it = WhereThePointerIs::OnThePanel(PartOfThePanel::APreview);
    assert!(!on_it.is_the_asking_strip());
    assert_ne!(strip, on_it);
}

/// **Elsewhere keeps nothing open**, which is the only thing that conceals.
#[test]
fn elsewhere_is_the_one_answer_that_lets_it_go() {
    let away = WhereThePointerIs::Elsewhere;
    assert!(!away.holds_the_panel_open());
    assert!(!away.is_the_asking_strip());
    assert_eq!(away.part(), None);
}

/// **The edge is data**, so nothing here reads as *right*.
///
/// Costs nothing if the mirrored frame in the design is a stray, and is the difference between
/// a rename and a rewrite if it is a right-to-left variant. The same correction `alo-dock`'s
/// reveal machine already went through.
#[test]
fn the_panel_does_not_know_which_edge_it_is_on() {
    assert_ne!(WhichEdge::Right, WhichEdge::Left);
    // Both are ordinary values of one type; neither is a default and neither is assumed.
    for edge in [WhichEdge::Right, WhichEdge::Left] {
        let _: WhichEdge = edge;
    }
}

/// **At most one surface claims a point**, over a grid, at both panel widths.
///
/// The owner's invariant. The stand-in classifier follows their rule — the panel owns the
/// reserved area including the shared corner, and the top controls and the Dock stop before it
/// — and the grid is dense enough to cross every boundary rather than sampling between them.
#[test]
fn a_correct_classifier_never_lets_two_surfaces_claim_one_point() {
    for reserved in [COLLAPSED, EXPANDED] {
        for x in 0..=WIDE {
            for y in 0..=TALL {
                let said = who_claims(x, y, reserved);
                assert!(
                    at_most_one_surface_claims_it(said),
                    "at ({x}, {y}) with the panel {reserved} wide, more than one surface \
                     claimed the point: {said:?}"
                );
            }
        }
    }
}

/// **And the corner belongs to the panel at both widths**, which is the decision itself.
#[test]
fn the_shared_corner_belongs_to_the_panel() {
    for reserved in [COLLAPSED, EXPANDED] {
        // The top-right corner, inside the top strip's height and inside the reserved area.
        let said = who_claims(WIDE - 1, 0, reserved);
        assert!(said.the_panel, "the panel does not own the shared corner");
        assert!(
            !said.the_top_controls,
            "the top controls reach into the reserved area, which the owner's rule forbids"
        );
    }
}

/// **A classifier that lets the top controls run the full width is caught.**
///
/// The design as drawn does exactly this — the top region is the full 1440 — and the owner's
/// decision supersedes it. So this is not a hypothetical mistake: it is the state the design
/// was in, and the invariant has to catch it or it is checking nothing.
#[test]
fn the_design_as_drawn_would_fail_this_invariant() {
    let mut caught = false;
    for x in 0..=WIDE {
        for y in 0..=TALL {
            let said = WhatEachSurfaceSaid {
                // The top controls, unbounded, as the frames have them.
                the_top_controls: y < TOP_STRIP,
                the_dock: false,
                the_panel: x >= WIDE - COLLAPSED,
            };
            if !at_most_one_surface_claims_it(said) {
                caught = true;
            }
        }
    }
    assert!(
        caught,
        "an unbounded top region did not violate the invariant, so the invariant is not \
         checking what it claims to — the design as drawn overlaps the panel's region at the \
         top-right corner over the full width of the reserved area"
    );
}

/// **Zero surfaces claiming a point is correct**, not a gap.
///
/// Most of the screen belongs to no edge surface. An invariant that required exactly one would
/// fail in the middle of the canvas, which is where a person does their work.
#[test]
fn the_middle_of_the_screen_belongs_to_nobody() {
    let said = who_claims(WIDE / 2, TALL / 2, COLLAPSED);
    assert!(!said.the_top_controls && !said.the_dock && !said.the_panel);
    assert!(at_most_one_surface_claims_it(said));
}

// ---------------------------------------------------------------- the stand-in classifier
//
// **Not the real one.** The real classifier lives where the coordinates do, in whoever draws,
// and these figures are this test's own fixture rather than the design's numbers — the crate
// under test holds no numbers at all, which is how the one-screen-size rule is kept there.
//
// They are chosen only to be distinguishable: a screen, a top strip, a dock that stops short,
// and two reserved widths so the invariant is exercised where the corner is nearest.

/// The fixture screen. Any two numbers would do; these are not a specification.
const WIDE: i32 = 200;
const TALL: i32 = 120;
/// How far down the top controls reach.
const TOP_STRIP: i32 = 10;
/// The reserved width when the panel is collapsed, and when it is expanded.
const COLLAPSED: i32 = 16;
const EXPANDED: i32 = 40;
/// Where the Dock's own region begins and ends, before the reserved area.
const DOCK_FROM: i32 = 40;
const DOCK_STRIP: i32 = 14;

/// Who claims this point, following the owner's rule.
///
/// > Top controls: span the screen **up to** the reserved right-panel area.
/// > Right panel: **owns that area**, including the top-right corner.
/// > Bottom Dock: stops before the same area, as it already does.
fn who_claims(x: i32, y: i32, reserved: i32) -> WhatEachSurfaceSaid {
    let reserved_from = WIDE - reserved;
    let in_the_reserved_area = x >= reserved_from;
    WhatEachSurfaceSaid {
        the_top_controls: y < TOP_STRIP && !in_the_reserved_area,
        the_dock: y >= TALL - DOCK_STRIP && x >= DOCK_FROM && !in_the_reserved_area,
        the_panel: in_the_reserved_area,
    }
}
