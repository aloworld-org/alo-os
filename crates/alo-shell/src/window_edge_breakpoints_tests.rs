//! The widths the design says to check, each checked.
//!
//! The owner named eight: **80, 143, 144, 145, 148, 195, 196** and the
//! smallest window there is. They are here as one table rather than scattered
//! through other tests, because the thing being verified is the *boundaries* —
//! and a boundary is only tested by the pair of values either side of it.
//!
//! 144 is in the list and is no longer special. It was where this lane found
//! controls laid out outside the edge, and the design's answer is that it was
//! *a reproduction boundary, not evidence that all full controls fit at 144*.
//! So 143, 144 and 145 are three ordinary members of the compact range now,
//! and the test that would have caught the original fault —
//! [`every_control_sits_inside_the_edge`] — runs across all of them.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected shape is the failure being reported"
)]

use super::{
    A_TARGET_IS, BETWEEN_TARGETS, Decorations, FULL_CONTROLS_NEED, HowItFits, OnTheEdge,
    THE_COMPACT_EDGE_IS, THE_PADDING_IS, THE_REGION_IS_TALL, THE_STRIP_IS_TALL, WindowEdge,
    edge_of, the_usable_output,
};
use smithay::utils::{Logical, Point, Rectangle, Size};

/// A roomy output, so a width test is about the width and not about clamping.
fn a_big_output() -> Size<i32, Logical> {
    Size::from((1920, 1080))
}

/// A window of this content width, somewhere that is not the origin.
fn a_window(wide: i32) -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((400, 300)), Size::from((wide, 240)))
}

fn revealed(wide: i32) -> WindowEdge {
    edge_of(
        a_window(wide),
        Decorations::TheShellDraws,
        true,
        a_big_output(),
    )
}

/// The eight widths, and what each is for.
const THE_WIDTHS: [(i32, &str); 8] = [
    (1, "the smallest window there is"),
    (80, "the design's tiny case, drawn in the review"),
    (143, "one below the old fault"),
    (144, "the old fault itself, now ordinary"),
    (145, "one above it"),
    (148, "the compact minimum exactly"),
    (195, "one below the full breakpoint"),
    (196, "the full breakpoint exactly"),
];

/// **Which layout each width gets**, which is the whole ruling in one table.
#[test]
fn each_width_gets_the_layout_the_design_names() {
    for (wide, why) in THE_WIDTHS {
        let expected = if wide >= FULL_CONTROLS_NEED {
            HowItFits::Full
        } else {
            HowItFits::Compact
        };
        assert_eq!(HowItFits::at(wide), expected, "{wide}: {why}");
    }
    // The boundary, stated as the pair either side of it rather than as one
    // value: 195 is compact and 196 is full, and a test of only one of them
    // would pass against an off-by-one.
    assert_eq!(HowItFits::at(195), HowItFits::Compact);
    assert_eq!(HowItFits::at(196), HowItFits::Full);
}

/// **The edge is never narrower than 148, and never narrower than its
/// window.**
#[test]
fn the_edge_is_at_least_the_compact_width() {
    for (wide, why) in THE_WIDTHS {
        let edge = revealed(wide);
        assert!(
            edge.region.size.w >= THE_COMPACT_EDGE_IS,
            "{wide} ({why}) gave an edge {} wide",
            edge.region.size.w
        );
        assert_eq!(
            edge.region.size.w,
            wide.max(THE_COMPACT_EDGE_IS),
            "{wide}: the edge follows the window above 148 and stops at 148 below"
        );
    }
}

/// **The application keeps its content width**, which the contract puts first:
/// *do not resize or relocate application content merely to make chrome fit.*
///
/// Asserted by the only means this function has — the window it was given is
/// not altered, and the edge is the only thing that changed size.
#[test]
fn the_application_keeps_its_width_at_every_size() {
    for (wide, _) in THE_WIDTHS {
        let window = a_window(wide);
        let edge = revealed(wide);
        assert_eq!(window.size.w, wide, "the window itself is untouched");
        assert_eq!(
            edge.region.loc.y,
            window.loc.y - THE_REGION_IS_TALL,
            "{wide}: the edge sits above the window, never inside it"
        );
    }
}

/// **A narrow edge is centred over its window and overhangs both sides.**
#[test]
fn a_narrow_window_gets_a_centred_edge_that_overhangs() {
    for wide in [1, 80, 143, 144, 145, 147] {
        let window = a_window(wide);
        let edge = revealed(wide);
        let over = THE_COMPACT_EDGE_IS - wide;
        assert_eq!(edge.region.size.w, THE_COMPACT_EDGE_IS);
        assert_eq!(
            edge.region.loc.x,
            window.loc.x - over / 2,
            "{wide}: half the difference to the left"
        );
        // **Overhanging, and the two sides are not always equal.** Integer
        // division puts the odd unit on the trailing side, so at 147 the
        // difference is 1 and the whole of it is on the right - the left
        // overhang is 0 there. Asserted as *not inside the window* on the left
        // and *strictly beyond it* on the right, which is what centring with
        // truncation actually gives.
        assert!(
            edge.region.loc.x <= window.loc.x,
            "{wide}: the edge starts inside the window"
        );
        assert!(
            edge.region.loc.x + edge.region.size.w > window.loc.x + wide,
            "{wide}: no right overhang"
        );
        assert_eq!(
            (window.loc.x - edge.region.loc.x)
                + (edge.region.loc.x + edge.region.size.w - (window.loc.x + wide)),
            over,
            "{wide}: the two overhangs do not add up to the difference"
        );
    }
}

/// **At 148 and above the edge is exactly the window**, with no overhang.
#[test]
fn from_the_compact_width_up_the_edge_is_the_window() {
    for wide in [148, 149, 195, 196, 400, 1600] {
        let window = a_window(wide);
        let edge = revealed(wide);
        assert_eq!(edge.region.loc.x, window.loc.x, "{wide}");
        assert_eq!(edge.region.size.w, wide, "{wide}");
    }
}

/// **The edge is clamped inside the output**, after being centred.
#[test]
fn a_narrow_edge_is_clamped_into_the_output() {
    let output = Size::from((800, 600));
    // Hard against the left: centring would put the edge at a negative x.
    let at_the_left = Rectangle::new(Point::from((0, 300)), Size::from((80, 240)));
    let edge = edge_of(at_the_left, Decorations::TheShellDraws, true, output);
    assert_eq!(edge.region.loc.x, 0, "an edge was allowed off the left");

    // Hard against the right.
    let at_the_right = Rectangle::new(Point::from((720, 300)), Size::from((80, 240)));
    let edge = edge_of(at_the_right, Decorations::TheShellDraws, true, output);
    assert_eq!(
        edge.region.loc.x + edge.region.size.w,
        800,
        "an edge was allowed off the right"
    );

    // And the control: the same window in the middle is not clamped, so the
    // two assertions above are about clamping and not about centring.
    let middle = Rectangle::new(Point::from((360, 300)), Size::from((80, 240)));
    let edge = edge_of(middle, Decorations::TheShellDraws, true, output);
    assert_eq!(edge.region.loc.x, 360 - (THE_COMPACT_EDGE_IS - 80) / 2);
}

/// **Every control's target is inside the edge, at every width** — the
/// invariant the Mac lane wrote and the original fault broke.
///
/// Carried here from that lane with its sizes: the fault was that
/// `first_target_at` went negative below 144 and laid controls to the left of
/// the window they belonged to, where a press would have reached whatever was
/// beneath. It cannot now, because the edge is never narrower than the
/// controls need.
///
/// The control count is asserted too, so that an edge laying out *no*
/// controls cannot pass by having nothing to check.
#[test]
fn every_control_sits_inside_the_edge() {
    // The Mac lane's own shapes, which is where this invariant came from: a
    // window one unit square, one 120 by 80, and one 200 by 1. 320 and up were
    // always clean, so a test of only roomy windows would have passed against
    // the fault. 120 is the named case because 120 is a window somebody might
    // really have and 1 is not.
    for (wide, tall, why) in [
        (1, 1, "the Mac lane's smallest"),
        (120, 80, "the plausible narrow window"),
        (200, 1, "wide and one unit tall"),
    ] {
        for revealed in [false, true] {
            let window = Rectangle::new(Point::from((37, 59)), Size::from((wide, tall)));
            let edge = edge_of(window, Decorations::TheShellDraws, revealed, a_big_output());
            for control in &edge.controls {
                assert_eq!(
                    edge.region.intersection(control.target),
                    Some(control.target),
                    "{wide}x{tall} ({why}, revealed={revealed}): {:?} at x={} escapes region {:?}",
                    control.does,
                    control.target.loc.x,
                    edge.region
                );
            }
        }
    }
    for (wide, why) in THE_WIDTHS {
        for decorations in [Decorations::TheShellDraws, Decorations::TheApplicationDraws] {
            let edge = edge_of(a_window(wide), decorations, true, a_big_output());
            assert!(
                !edge.controls.is_empty(),
                "{wide} ({why}, {decorations:?}) laid out no control, so this proves nothing"
            );
            for control in &edge.controls {
                assert_eq!(
                    edge.region.intersection(control.target),
                    Some(control.target),
                    "{wide} ({why}, {decorations:?}): {:?} at x={} is not inside the edge",
                    control.does,
                    control.target.loc.x
                );
            }
        }
    }
}

/// **No two targets overlap**, at any width — *adjacent hit areas never
/// overlap*.
#[test]
fn no_two_targets_overlap_at_any_width() {
    for (wide, why) in THE_WIDTHS {
        for decorations in [Decorations::TheShellDraws, Decorations::TheApplicationDraws] {
            let edge = edge_of(a_window(wide), decorations, true, a_big_output());
            for (one, two) in pairs(&edge.controls) {
                assert!(
                    one.target.intersection(two.target).is_none(),
                    "{wide} ({why}): {:?} and {:?} overlap",
                    one.does,
                    two.does
                );
            }
        }
    }
}

fn pairs(controls: &[super::EdgeControl]) -> Vec<(super::EdgeControl, super::EdgeControl)> {
    let mut out = Vec::new();
    for (at, one) in controls.iter().enumerate() {
        for two in controls.iter().skip(at + 1) {
            out.push((*one, *two));
        }
    }
    out
}

/// **Close is reachable at every width**, which is the ruling's own sentence:
/// *keep Close directly available whenever the edge's controls are revealed*.
#[test]
fn close_is_on_the_edge_at_every_width() {
    for (wide, why) in THE_WIDTHS {
        let edge = revealed(wide);
        assert!(
            edge.controls.iter().any(|it| it.does == OnTheEdge::Close),
            "{wide} ({why}) has no Close"
        );
    }
}

/// **And Move is**, so dragging never depends on finding bare strip.
#[test]
fn move_is_on_the_edge_at_every_width() {
    for (wide, _why) in THE_WIDTHS {
        for decorations in [Decorations::TheShellDraws, Decorations::TheApplicationDraws] {
            let edge = edge_of(a_window(wide), decorations, true, a_big_output());
            let moves = edge.controls.iter().find(|it| it.does == OnTheEdge::Move);
            let moves = moves.expect("every edge lays out a Move control");
            assert_eq!(
                moves.target.loc.x,
                edge.region.loc.x + THE_PADDING_IS,
                "{wide}: Move is not at the leading padding"
            );
        }
    }
}

/// **The compact layout is Move, Menu, Close and the full one swaps Menu for
/// the two actions** — the lists, at the two boundary widths.
#[test]
fn the_two_layouts_hold_what_the_design_draws() {
    let compact: Vec<_> = revealed(195).controls.iter().map(|it| it.does).collect();
    assert_eq!(
        compact,
        vec![OnTheEdge::Move, OnTheEdge::Menu, OnTheEdge::Close],
        "195 is compact"
    );

    let full: Vec<_> = revealed(196).controls.iter().map(|it| it.does).collect();
    assert_eq!(
        full,
        vec![
            OnTheEdge::Move,
            OnTheEdge::Minimise,
            OnTheEdge::Maximise,
            OnTheEdge::Close
        ],
        "196 is full"
    );
    assert!(
        !full.contains(&OnTheEdge::Menu),
        "the full layout needs no menu control to reach its actions"
    );
}

/// **An application drawing its own header never gets a Close**, at any width.
///
/// The owner's ruling, repeated by the compact contract. Checked at every
/// width because the compact layout is where a second Close would be most
/// tempting: it is the one control the shell keeps.
#[test]
fn an_application_that_draws_its_own_header_gets_no_close() {
    for (wide, why) in THE_WIDTHS {
        let edge = edge_of(
            a_window(wide),
            Decorations::TheApplicationDraws,
            true,
            a_big_output(),
        );
        let does: Vec<_> = edge.controls.iter().map(|it| it.does).collect();
        assert!(
            !does.contains(&OnTheEdge::Close),
            "{wide} ({why}) gave an app-drawn window a duplicate Close: {does:?}"
        );
        assert_eq!(does, vec![OnTheEdge::Move, OnTheEdge::Menu], "{wide}");
    }
}

/// **The positions the design draws, at the two exact minimums.**
///
/// `414:27583` for compact and `414:27626` for full. Spelled as literals
/// because they are a measurement of the file, and a test that recomputed them
/// from the same constants the code uses would agree with a wrong layout.
#[test]
fn the_minimums_match_the_masters() {
    let compact = revealed(THE_COMPACT_EDGE_IS);
    let at = |edge: &WindowEdge, does: OnTheEdge| {
        edge.controls
            .iter()
            .find(|it| it.does == does)
            .map(|it| it.target.loc.x - edge.region.loc.x)
    };
    assert_eq!(at(&compact, OnTheEdge::Move), Some(4), "Move at 4");
    assert_eq!(at(&compact, OnTheEdge::Menu), Some(52), "Menu at 52");
    assert_eq!(at(&compact, OnTheEdge::Close), Some(100), "Close at 100");

    let full = revealed(FULL_CONTROLS_NEED);
    assert_eq!(at(&full, OnTheEdge::Move), Some(4), "Move at 4");
    assert_eq!(at(&full, OnTheEdge::Minimise), Some(52), "Minimise at 52");
    assert_eq!(at(&full, OnTheEdge::Maximise), Some(100), "Maximise at 100");
    assert_eq!(at(&full, OnTheEdge::Close), Some(148), "Close at 148");
}

/// **The strip stays 32 inside a 44 envelope at every width**, which the
/// contract asks for and a new layout is the obvious place to lose.
#[test]
fn the_strip_and_the_envelope_do_not_change_with_width() {
    for (wide, why) in THE_WIDTHS {
        let edge = revealed(wide);
        assert_eq!(edge.region.size.h, THE_REGION_IS_TALL, "{wide} ({why})");
        let strip = edge.strip.expect("a revealed edge draws a strip");
        assert_eq!(strip.size.h, THE_STRIP_IS_TALL, "{wide} ({why})");
        assert_eq!(
            strip.size.w, edge.region.size.w,
            "{wide}: strip spans the edge"
        );
        for control in &edge.controls {
            assert_eq!(control.target.size.w, A_TARGET_IS, "{wide}");
            assert_eq!(control.target.size.h, A_TARGET_IS, "{wide}");
        }
    }
}

/// **The drag area is at least one target wide**, at every width — *the
/// available drag area is at least 44×44*.
#[test]
fn there_is_always_room_to_drag() {
    for (wide, why) in THE_WIDTHS {
        let edge = revealed(wide);
        assert!(
            edge.drag.size.w >= A_TARGET_IS,
            "{wide} ({why}): the drag area is {} wide",
            edge.drag.size.w
        );
        assert!(edge.drag.size.h >= A_TARGET_IS, "{wide} ({why})");
    }
}

/// **Extra width goes to dragging**, which is where the contract puts it.
#[test]
fn width_above_the_minimum_belongs_to_dragging() {
    // **Within one layout**, because the drag area also changes when the
    // layout does: 148 + 100 is 248, which is Full and has a third action
    // target. Comparing across the breakpoint measured two different things
    // and is the first thing this test got wrong.
    for (from, by) in [(THE_COMPACT_EDGE_IS, 47), (FULL_CONTROLS_NEED, 200)] {
        let narrow = revealed(from).drag.size.w;
        let wider = revealed(from + by).drag.size.w;
        assert_eq!(
            wider - narrow,
            by,
            "from {from}, {by} more units of window gave {} more units of drag",
            wider - narrow
        );
    }
}

/// **The gap between targets is the measured one**, so the layout cannot drift
/// to a different spacing while every other test still passes.
#[test]
fn the_targets_are_one_gap_apart() {
    let full = revealed(FULL_CONTROLS_NEED);
    let actions: Vec<_> = full
        .controls
        .iter()
        .filter(|it| it.does != OnTheEdge::Move)
        .collect();
    for pair in actions.windows(2) {
        let [one, two] = pair else { continue };
        assert_eq!(
            two.target.loc.x - (one.target.loc.x + A_TARGET_IS),
            BETWEEN_TARGETS,
            "{:?} to {:?}",
            one.does,
            two.does
        );
    }
}

/// **The output converter divides by the scale**, including the three the
/// design names.
#[test]
fn the_output_is_read_in_logical_units() {
    let pixels = Size::from((1920, 1080));
    assert_eq!(the_usable_output(pixels, 100), Size::from((1920, 1080)));
    assert_eq!(the_usable_output(pixels, 125), Size::from((1536, 864)));
    assert_eq!(the_usable_output(pixels, 150), Size::from((1280, 720)));
    assert_eq!(the_usable_output(pixels, 200), Size::from((960, 540)));
    // A scale of nothing is read as one-to-one rather than dividing by zero.
    assert_eq!(the_usable_output(pixels, 0), Size::from((1920, 1080)));
}

/// **A scaled display clamps in logical units**, which is the whole reason the
/// converter exists: at 200% a 1920-pixel display is 960 logical units wide,
/// and an edge clamped against 1920 would sit half off it.
#[test]
fn clamping_happens_in_the_units_the_edge_is_laid_out_in() {
    let doubled = the_usable_output(Size::from((1920, 1080)), 200);
    assert_eq!(doubled.w, 960);
    let at_the_right = Rectangle::new(Point::from((900, 300)), Size::from((80, 240)));
    let edge = edge_of(at_the_right, Decorations::TheShellDraws, true, doubled);
    assert_eq!(
        edge.region.loc.x + edge.region.size.w,
        960,
        "the edge was clamped against the pixel width instead of the logical one"
    );
}
