//! What the edge draws, held against what the contract forbids.
//!
//! Most of these assert an **absence** — no tile at rest, no highlight on a
//! control nobody is pointing at, no teal anywhere. The contract is mostly a
//! list of things not to do, and a test that only checked the happy drawing
//! would pass while the forbidden things were drawn alongside it.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or a missing element is the failure being reported"
)]

use super::*;
use crate::window_edge::{Decorations, edge_of};
use smithay::utils::{Point, Size};

/// A window whose edge is the design's 600 specimen at the origin.
fn specimen() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((0, 44)), Size::from((600, 400)))
}

/// Every colour this draws, as hexes, so a test can say *no teal* without
/// knowing which role teal is.
fn colours(solids: &[Solid]) -> Vec<(u8, u8, u8)> {
    solids
        .iter()
        .map(|(_, colour)| (colour.red(), colour.green(), colour.blue()))
        .collect()
}

/// **At rest the edge is one quiet bar and nothing else.**
///
/// The contract: *at rest, the strip is concealed and the subtle centred grip
/// remains*, and *no permanent square button tiles*. So a resting edge draws
/// exactly one rectangle.
#[test]
fn at_rest_the_edge_draws_one_quiet_bar() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, false);
    let drawn = solids(&edge, Pointing::default());
    assert_eq!(drawn.len(), 1, "{drawn:?}");
    assert_eq!(drawn[0].0.size, Size::from((24, 2)));
    assert_eq!(colours(&drawn), vec![(0x59, 0x6B, 0x78)], "text/muted");
}

/// **A control at rest contributes no background at all.**
///
/// *Transparent button backgrounds at rest.* So nothing of a control's 44 × 44
/// is ever filled unless it is hovered — only its artwork is drawn.
#[test]
fn an_unpointed_control_draws_only_its_artwork() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    let drawn = solids(&edge, Pointing::default());
    for control in &edge.controls {
        assert!(
            !drawn.iter().any(|(rect, _)| *rect == control.target),
            "{:?} filled its whole target",
            control.does
        );
        assert!(
            !drawn.iter().any(|(rect, _)| *rect == control.highlight),
            "{:?} drew a highlight nobody asked for",
            control.does
        );
    }
}

/// **The highlight appears on the hovered control and on no other**, and it is
/// the 32 × 28 pill rather than the target.
#[test]
fn only_the_hovered_control_is_highlighted() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    let drawn = solids(
        &edge,
        Pointing {
            hovered: Some(OnTheEdge::Maximise),
            focused: None,
        },
    );
    let highlights: Vec<_> = edge
        .controls
        .iter()
        .filter(|control| drawn.iter().any(|(rect, _)| *rect == control.highlight))
        .map(|control| control.does)
        .collect();
    assert_eq!(highlights, vec![OnTheEdge::Maximise]);
    assert!(
        colours(&drawn).contains(&(0xEE, 0xF2, 0xF4)),
        "the highlight is bg/cool"
    );
}

/// **Close turns danger-coloured when hovered, and only then, and only
/// Close.**
///
/// Measured: `status/danger` resolves on the Close-hover variant and on no
/// other of the seven.
#[test]
fn only_close_and_only_hovered_is_danger_coloured() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    const DANGER: (u8, u8, u8) = (0xB4, 0x23, 0x18);

    let resting = solids(&edge, Pointing::default());
    assert!(!colours(&resting).contains(&DANGER), "danger at rest");

    for hovered in [OnTheEdge::Minimise, OnTheEdge::Maximise] {
        let drawn = solids(
            &edge,
            Pointing {
                hovered: Some(hovered),
                focused: None,
            },
        );
        assert!(
            !colours(&drawn).contains(&DANGER),
            "{hovered:?} hovered turned something danger-coloured"
        );
    }

    let closing = solids(
        &edge,
        Pointing {
            hovered: Some(OnTheEdge::Close),
            focused: None,
        },
    );
    assert!(
        colours(&closing).contains(&DANGER),
        "Close hovered is danger"
    );
}

/// **No teal, in any state.** The contract rules it out for ordinary window
/// controls and `the-canvas-as-a-workspace.md` §6 reserves it for alo acting.
#[test]
fn nothing_on_the_edge_is_ever_teal() {
    const TEAL: [(u8, u8, u8); 2] = [(0x0F, 0x6B, 0x72), (0x77, 0xC8, 0xC6)];
    let specimen = specimen();
    for decorations in [Decorations::TheShellDraws, Decorations::TheApplicationDraws] {
        for revealed in [true, false] {
            let edge = edge_of(specimen, decorations, revealed);
            for hovered in [
                None,
                Some(OnTheEdge::Minimise),
                Some(OnTheEdge::Maximise),
                Some(OnTheEdge::Close),
                Some(OnTheEdge::Menu),
            ] {
                let drawn = solids(
                    &edge,
                    Pointing {
                        hovered,
                        focused: hovered,
                    },
                );
                for teal in TEAL {
                    assert!(
                        !colours(&drawn).contains(&teal),
                        "{teal:?} drawn with {decorations:?} revealed={revealed} hovered={hovered:?}"
                    );
                }
            }
        }
    }
}

/// **The revealed strip is white inside a `border/default` frame.**
#[test]
fn the_revealed_strip_is_a_surface_in_a_border() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    let drawn = solids(&edge, Pointing::default());
    let strip = edge.strip.expect("revealed");
    assert_eq!(drawn[0].0, strip);
    assert_eq!(colours(&drawn)[0], (0xE7, 0xEB, 0xEF), "border/default");
    assert_eq!(drawn[1].0.size, Size::from((598, 30)), "one unit inside");
    assert_eq!(colours(&drawn)[1], (0xFF, 0xFF, 0xFF), "bg/surface");
}

/// **Nothing is drawn outside the edge's own region.** The contract's *never
/// crop, cover or recolour application content* is the rule, and the region
/// ends exactly where the application begins.
#[test]
fn nothing_is_drawn_over_the_application() {
    let window = specimen();
    for revealed in [true, false] {
        let edge = edge_of(window, Decorations::TheShellDraws, revealed);
        let drawn = solids(
            &edge,
            Pointing {
                hovered: Some(OnTheEdge::Close),
                focused: Some(OnTheEdge::Close),
            },
        );
        for (rect, _) in &drawn {
            assert!(
                rect.loc.y + rect.size.h <= window.loc.y,
                "{rect:?} reaches into the application, which starts at {}",
                window.loc.y
            );
        }
    }
}

/// **An application that draws its own header gets no window buttons drawn**,
/// which is the owner's ruling checked at the paint rather than only at the
/// geometry.
#[test]
fn no_window_buttons_are_drawn_for_an_application_that_has_its_own() {
    let edge = edge_of(specimen(), Decorations::TheApplicationDraws, true);
    assert_eq!(edge.controls.len(), 1);
    assert_eq!(edge.controls[0].does, OnTheEdge::Menu);
    // And hovering what is not there changes nothing.
    let drawn = solids(&edge, Pointing::default());
    let closing = solids(
        &edge,
        Pointing {
            hovered: Some(OnTheEdge::Close),
            focused: None,
        },
    );
    assert_eq!(drawn, closing, "a control that is not there was drawn");
}
