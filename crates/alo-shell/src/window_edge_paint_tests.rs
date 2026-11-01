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
use crate::window_edge_look::WhichLook;
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
    let edge = edge_of(
        specimen(),
        Decorations::TheShellDraws,
        false,
        smithay::utils::Size::from((1920, 1080)),
    );
    let drawn = solids(&edge, Pointing::default(), WhichLook::Focused);
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
    let edge = edge_of(
        specimen(),
        Decorations::TheShellDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    );
    let drawn = solids(&edge, Pointing::default(), WhichLook::Focused);
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
    let edge = edge_of(
        specimen(),
        Decorations::TheShellDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    );
    let drawn = solids(
        &edge,
        Pointing {
            hovered: Some(OnTheEdge::Maximise),
            focused: None,
        },
        WhichLook::Focused,
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
    let edge = edge_of(
        specimen(),
        Decorations::TheShellDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    );
    const DANGER: (u8, u8, u8) = (0xB4, 0x23, 0x18);

    let resting = solids(&edge, Pointing::default(), WhichLook::Focused);
    assert!(!colours(&resting).contains(&DANGER), "danger at rest");

    for hovered in [OnTheEdge::Minimise, OnTheEdge::Maximise] {
        let drawn = solids(
            &edge,
            Pointing {
                hovered: Some(hovered),
                focused: None,
            },
            WhichLook::Focused,
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
        WhichLook::Focused,
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
            let edge = edge_of(
                specimen,
                decorations,
                revealed,
                smithay::utils::Size::from((1920, 1080)),
            );
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
                    WhichLook::Focused,
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

/// **The strip is no longer a flat rectangle here.** Its top corners are
/// rounded at 10 (`554:25`), so it is drawn by `crate::the_title_strip_drawn`
/// and this draws only what sits on it. A square strip filled here as well
/// would be painted over the rounded one and undo its corners.
#[test]
fn the_revealed_strip_is_left_to_the_raster_that_rounds_it() {
    let edge = edge_of(
        specimen(),
        Decorations::TheShellDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    );
    let drawn = solids(&edge, Pointing::default(), WhichLook::Focused);
    let strip = edge.strip.expect("revealed");
    assert!(
        !drawn
            .iter()
            .any(|(rect, _)| rect.size.w >= strip.size.w - 2),
        "a strip-wide rectangle was filled over the rounded strip: {drawn:?}"
    );
    assert!(!drawn.is_empty(), "the controls' artwork is still drawn");
}

/// **The focused window's controls are navy, every other's muted** — the
/// board `557:456`, *title, icon and controls in text/muted — still readable,
/// still 44 × 44*. Their targets do not move or shrink.
#[test]
fn an_unfocused_windows_controls_are_muted_and_keep_their_size() {
    let edge = edge_of(
        specimen(),
        Decorations::TheShellDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    );
    let focused = solids(&edge, Pointing::default(), WhichLook::Focused);
    let quiet = solids(&edge, Pointing::default(), WhichLook::NotFocused);
    assert!(
        colours(&focused).contains(&(0x10, 0x2A, 0x43)),
        "text/primary"
    );
    assert!(!colours(&quiet).contains(&(0x10, 0x2A, 0x43)), "no navy");
    assert!(colours(&quiet).contains(&(0x59, 0x6B, 0x78)), "text/muted");
    let shapes = |drawn: &[Solid]| drawn.iter().map(|(rect, _)| *rect).collect::<Vec<_>>();
    assert_eq!(shapes(&focused), shapes(&quiet), "only the colour changes");
    for control in &edge.controls {
        assert_eq!(control.target.size, Size::from((44, 44)));
    }
}

/// **Nothing is drawn outside the edge's own region.** The contract's *never
/// crop, cover or recolour application content* is the rule, and the region
/// ends exactly where the application begins.
#[test]
fn nothing_is_drawn_over_the_application() {
    let window = specimen();
    for revealed in [true, false] {
        let edge = edge_of(
            window,
            Decorations::TheShellDraws,
            revealed,
            smithay::utils::Size::from((1920, 1080)),
        );
        let drawn = solids(
            &edge,
            Pointing {
                hovered: Some(OnTheEdge::Close),
                focused: Some(OnTheEdge::Close),
            },
            WhichLook::Focused,
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
    let edge = edge_of(
        specimen(),
        Decorations::TheApplicationDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    );
    // Movement and the menu, and no window button of alo's own.
    assert_eq!(
        edge.controls.iter().map(|it| it.does).collect::<Vec<_>>(),
        vec![OnTheEdge::Move, OnTheEdge::Menu]
    );
    // And hovering what is not there changes nothing.
    let drawn = solids(&edge, Pointing::default(), WhichLook::Focused);
    let closing = solids(
        &edge,
        Pointing {
            hovered: Some(OnTheEdge::Close),
            focused: None,
        },
        WhichLook::Focused,
    );
    assert_eq!(drawn, closing, "a control that is not there was drawn");
}

/// **The focus ring is the interaction target, measured from `414:27609`.**
///
/// This rang `control.highlight` — the 32 × 28 at `(+6, +14)` — until
/// 2026-10-12. That rectangle is right for hover and is byte-for-byte what
/// `State=Menu open` draws; the design rings the whole 44 × 44, so the real
/// ring stands 12 units above the visible strip and the drawn one sat inside
/// it.
///
/// The node id is in this comment so that the next person to doubt the number
/// re-reads the file rather than re-measuring a PNG, which is what it took to
/// find: a stroke on a frame is not a child layer, so `get_metadata` returns
/// that state identical to `State=Reveal`.
#[test]
fn the_focus_ring_is_the_whole_target_and_two_units_thick() {
    let edge = edge_of(
        specimen(),
        Decorations::TheShellDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    );
    let close = edge
        .controls
        .iter()
        .find(|control| control.does == OnTheEdge::Close)
        .expect("a revealed edge has a Close control");
    // The control: the two rectangles really are different, or this proves
    // nothing about which one was rung.
    assert_ne!(close.target, close.highlight);
    assert_eq!(close.target.size, Size::from((44, 44)));

    let ringed = solids(
        &edge,
        Pointing {
            hovered: None,
            focused: Some(OnTheEdge::Close),
        },
        WhichLook::Focused,
    );
    let quiet = solids(
        &edge,
        Pointing {
            hovered: None,
            focused: None,
        },
        WhichLook::Focused,
    );
    // Everything the ring added, and nothing else: the difference between the
    // two draws rather than a guess at which solids are the ring.
    let ring: Vec<_> = ringed
        .iter()
        .filter(|solid| !quiet.contains(solid))
        .copied()
        .collect();
    assert_eq!(ring.len(), 4, "a ring is four bars: {ring:?}");

    // Its outer bounds are the target's, which is the whole finding.
    let left = ring.iter().map(|(r, _)| r.loc.x).min().expect("four bars");
    let top = ring.iter().map(|(r, _)| r.loc.y).min().expect("four bars");
    let right = ring
        .iter()
        .map(|(r, _)| r.loc.x + r.size.w)
        .max()
        .expect("four bars");
    let bottom = ring
        .iter()
        .map(|(r, _)| r.loc.y + r.size.h)
        .max()
        .expect("four bars");
    assert_eq!(
        (left, top, right - left, bottom - top),
        (
            close.target.loc.x,
            close.target.loc.y,
            close.target.size.w,
            close.target.size.h
        ),
        "the ring is not the target it rings"
    );

    // And two units, which was one until the PNG was measured.
    for (bar, _) in &ring {
        assert!(
            bar.size.w == A_FOCUS_RING_IS || bar.size.h == A_FOCUS_RING_IS,
            "a bar of the ring is not two units thick: {bar:?}"
        );
    }

    // **The strip's own border is a different line and did not move.** The two
    // shared a constant before this, so a change to either moved the other.
    assert_eq!(THE_BORDER_IS, 1);
    assert_eq!(A_FOCUS_RING_IS, 2);
}

/// **The hover highlight stays where the design puts it**, which is the half
/// that was already right and must not be dragged along by the fix above.
#[test]
fn hover_still_fills_the_thirty_two_by_twenty_eight() {
    let edge = edge_of(
        specimen(),
        Decorations::TheShellDraws,
        true,
        smithay::utils::Size::from((1920, 1080)),
    );
    let close = edge
        .controls
        .iter()
        .find(|control| control.does == OnTheEdge::Close)
        .expect("a revealed edge has a Close control");
    assert_eq!(close.highlight.size, Size::from((32, 28)));
    assert_eq!(
        (
            close.highlight.loc.x - close.target.loc.x,
            close.highlight.loc.y - close.target.loc.y
        ),
        (6, 14),
        "`State=Menu open` draws it at (6, 14) inside the target"
    );

    let drawn = solids(
        &edge,
        Pointing {
            hovered: Some(OnTheEdge::Close),
            focused: None,
        },
        WhichLook::Focused,
    );
    assert!(
        drawn
            .iter()
            .any(|(rectangle, _)| *rectangle == close.highlight),
        "the hovered control's 32 x 28 is not filled"
    );
}
