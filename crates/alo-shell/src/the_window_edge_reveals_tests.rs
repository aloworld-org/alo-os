//! §5 held against the laid-out edge, including the two ways it fails silently.
//!
//! **The refusals are the point of most of these.** A classifier that returns
//! `OnTheSurface` for every point inside the window would pass a happy-path
//! test about controls and would reveal an edge from the middle of an
//! application's content. So *what is not the edge* is asserted as carefully as
//! what is.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported — the words \
              `crate::window_edge`'s own test module uses"
)]

use super::*;
use crate::window_edge::{Decorations, THE_REGION_IS_TALL, THE_STRIP_IS_TALL, edge_of};
use alo_dock::revealing::Revealing;
use smithay::utils::Size;

/// The same specimen `crate::window_edge`'s tests use: a 600-wide edge whose
/// region lands at the origin, so every figure below reads against the capture.
fn specimen() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((0, THE_REGION_IS_TALL)), Size::from((600, 400)))
}

fn at(x: i32, y: i32) -> Point<i32, Logical> {
    Point::from((x, y))
}

/// **A concealed edge is all region**, which is what makes the order in
/// `what_the_pointer_is` safe rather than lucky.
///
/// Every point of it answers `AtTheEdge` — the one answer that reveals — and in
/// particular the band where the strip *would* be once revealed. A classifier
/// that asked the strip first and found `None` would still arrive here, so this
/// is checked across the whole height rather than at one point.
#[test]
fn a_concealed_edge_is_all_region_and_every_point_of_it_asks() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, false);
    assert!(
        edge.strip.is_none(),
        "a concealed edge draws no strip; this test's premise is that there is nothing more \
         specific to find"
    );
    assert!(
        edge.controls.is_empty(),
        "a concealed edge lays out no control. If this ever holds controls, read this file's \
         header before changing it: a pointer arriving directly on one would classify as \
         OnTheSurface, and `Revealing` sets *on the surface* from its own current answer, so an \
         edge nothing had revealed would refuse to reveal"
    );

    for y in 0..THE_REGION_IS_TALL {
        for x in [0, 1, 299, 598, 599] {
            assert_eq!(
                what_the_pointer_is(&edge, at(x, y)),
                ThePointer::AtTheEdge,
                "({x}, {y}) is inside a concealed edge's region and must ask for it"
            );
        }
    }
}

/// **Every control answers on its own target**, asked at each one's centre.
#[test]
fn a_pointer_on_any_control_is_on_the_surface() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    assert!(
        !edge.controls.is_empty(),
        "a revealed shell-drawn edge has controls; without this the loop below proves nothing"
    );
    for control in &edge.controls {
        let middle = at(
            control.target.loc.x + control.target.size.w / 2,
            control.target.loc.y + control.target.size.h / 2,
        );
        assert_eq!(
            what_the_pointer_is(&edge, middle),
            ThePointer::OnTheSurface,
            "the centre of {:?} must be on the surface",
            control.does
        );
    }
}

/// **The 44 × 44 answers, not the 32 × 28 drawn behind it.**
///
/// This is the one test here that a correct-looking change breaks. `EdgeControl`
/// carries `target` and `highlight` as the same type on the same value, so
/// reading the wrong one compiles, draws identically, and silently shrinks every
/// control by six pixels each side and fourteen at the top — against a
/// specification whose one hard floor is *44 × 44, non-overlapping*.
///
/// The corners of the target that lie outside the highlight are asserted to be
/// outside it first, so this cannot pass by the two rectangles having become the
/// same thing.
///
/// **Mutation-tested, 2026-10-10.** Substituting `highlight` for `target` in
/// `what_the_pointer_is` fails this test, the walk, and the application-drawn
/// case — three, named, out of eleven. The other eight stay green, and one of
/// them is the reason this test exists:
/// `a_pointer_on_any_control_is_on_the_surface` **passes with the fault
/// present**, because it probes each control's *centre*, which is inside the
/// highlight as well. A suite of happy paths would have shipped a six-pixel
/// shrink on every control and reported eleven of eleven.
#[test]
fn the_target_answers_and_not_the_highlight_drawn_inside_it() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    let control = edge.controls.first().expect("a revealed edge has controls");

    let corner = at(control.target.loc.x, control.target.loc.y);
    assert!(
        !holds(control.highlight, corner),
        "the premise: a target's top-left corner lies outside the highlight drawn inside it. \
         target {:?}, highlight {:?}",
        control.target,
        control.highlight
    );
    assert!(
        holds(control.target, corner),
        "and inside the target itself, which is what must answer"
    );
    assert_eq!(
        what_the_pointer_is(&edge, corner),
        ThePointer::OnTheSurface,
        "a press at the very corner of a control's target reaches that control. If this fails, \
         something is hit-testing `highlight` — the field that says what is *filled* while \
         hovered, not the field that says what *answers*"
    );
}

/// **The strip between the controls holds the edge**, which is where the title
/// and the drag region are and where a person's pointer spends most of its time.
#[test]
fn the_strip_away_from_every_control_is_on_the_surface() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    let strip = edge.strip.expect("a revealed edge draws a strip");
    let left = at(strip.loc.x + 4, strip.loc.y + THE_STRIP_IS_TALL / 2);
    assert!(
        edge.controls.iter().all(|it| !holds(it.target, left)),
        "the premise: the strip's left end is on no control's target"
    );
    assert_eq!(what_the_pointer_is(&edge, left), ThePointer::OnTheSurface);
}

/// **The application's own content is not the edge**, revealed or not.
///
/// The region sits *above* the window. A classifier that tested the window
/// instead would reveal an edge from a pointer resting in a document, and *hover
/// must never move, restore, hide or close the application*.
#[test]
fn a_pointer_in_the_application_is_elsewhere_in_both_states() {
    for revealed in [false, true] {
        let edge = edge_of(specimen(), Decorations::TheShellDraws, revealed);
        // One below the region's last row, which is the application's first.
        let inside = at(300, THE_REGION_IS_TALL);
        assert_eq!(
            what_the_pointer_is(&edge, inside),
            ThePointer::Elsewhere,
            "revealed = {revealed}: the first row of the application is not the edge"
        );
        // And well inside it.
        assert_eq!(
            what_the_pointer_is(&edge, at(300, 300)),
            ThePointer::Elsewhere,
            "revealed = {revealed}"
        );
    }
}

/// **Half-open, so a region and whatever abuts it never both claim a point.**
#[test]
fn the_region_does_not_claim_the_column_past_its_end() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, false);
    assert_eq!(
        what_the_pointer_is(&edge, at(599, 0)),
        ThePointer::AtTheEdge
    );
    assert_eq!(
        what_the_pointer_is(&edge, at(600, 0)),
        ThePointer::Elsewhere,
        "600 is past a 600-wide region's end"
    );
    assert_eq!(
        what_the_pointer_is(&edge, at(0, -1)),
        ThePointer::Elsewhere,
        "and the row above its top belongs to nothing of this window"
    );
}

/// **A window with no width claims nothing**, rather than claiming everything.
#[test]
fn an_edge_of_no_extent_claims_no_point() {
    let flat = Rectangle::new(Point::from((0, THE_REGION_IS_TALL)), Size::from((0, 400)));
    let edge = edge_of(flat, Decorations::TheShellDraws, true);
    for x in [-1, 0, 1] {
        for y in [-1, 0, 1] {
            assert_eq!(
                what_the_pointer_is(&edge, at(x, y)),
                ThePointer::Elsewhere,
                "({x}, {y}) against an edge of no width"
            );
        }
    }
}

/// **A continuous pointer path, and no timed grace period** — the specification's
/// own words, as the behaviour rather than as a promise.
///
/// A person comes from above the window, crosses the region, and travels along it
/// to the furthest control. Every step is fed to the real `Revealing`, and the
/// edge must be revealed at every one of them after the first. There is nothing
/// to wait for and nothing that depends on how fast the walk is, because the
/// machine holds no instant — the walk could take an hour between steps and
/// produce the same answers.
#[test]
fn a_pointer_walking_to_the_furthest_control_never_loses_the_edge() {
    let mut machine = Revealing::covered();
    assert!(
        !machine.is_revealed(),
        "the premise: it starts concealed, so the first step below is doing the revealing"
    );

    // Entering at the region's top row, as somebody coming from above does.
    let first = edge_of(
        specimen(),
        Decorations::TheShellDraws,
        machine.is_revealed(),
    );
    machine = machine.the_pointer_is(what_the_pointer_is(&first, at(0, 0)));
    assert!(
        machine.is_revealed(),
        "the region is what reveals the edge, and a concealed edge is all region"
    );

    // Then along it, one logical pixel at a time, laying the edge out afresh at
    // each step **from the machine's own answer** — which is the loop a frame
    // actually runs, and the only way a reveal that depended on the previous
    // geometry would show up.
    let mut steps = 0_u32;
    for x in 0..600 {
        let edge = edge_of(
            specimen(),
            Decorations::TheShellDraws,
            machine.is_revealed(),
        );
        let was = what_the_pointer_is(&edge, at(x, 0));
        assert_ne!(
            was,
            ThePointer::Elsewhere,
            "({x}, 0) is on the path along a 600-wide edge and must not be nothing"
        );
        machine = machine.the_pointer_is(was);
        assert!(
            machine.is_revealed(),
            "the edge went away at x = {x} while the pointer was still on it. That is the \
             failure *a continuous pointer path* forbids: a person moving towards a control \
             must not have it disappear"
        );
        steps += 1;
    }
    assert_eq!(steps, 600, "the walk ran");

    // And the furthest control is reachable at the end of it.
    let edge = edge_of(
        specimen(),
        Decorations::TheShellDraws,
        machine.is_revealed(),
    );
    let last = edge.controls.last().expect("a revealed edge has controls");
    assert_eq!(
        what_the_pointer_is(&edge, at(last.target.loc.x, last.target.loc.y)),
        ThePointer::OnTheSurface
    );
}

/// **Leaving conceals it**, with no delay, which is the other half of no timer.
#[test]
fn a_pointer_leaving_conceals_the_edge_at_once() {
    let mut machine = Revealing::covered();
    let concealed = edge_of(specimen(), Decorations::TheShellDraws, false);
    machine = machine.the_pointer_is(what_the_pointer_is(&concealed, at(10, 10)));
    assert!(machine.is_revealed());

    let revealed = edge_of(specimen(), Decorations::TheShellDraws, true);
    let away = what_the_pointer_is(&revealed, at(300, 300));
    assert_eq!(away, ThePointer::Elsewhere);
    machine = machine.the_pointer_is(away);
    assert!(
        !machine.is_revealed(),
        "it conceals on the step the pointer left, not a step later"
    );
}

/// **An application that draws its own header still has an edge to reveal.**
///
/// The specification keeps movement and the menu for those, so the region is the
/// same and only what is on it differs. A classifier that worked for one
/// decoration mode and not the other would leave those applications unmovable,
/// which is the clause *borderless applications stay movable and recoverable*.
#[test]
fn an_application_drawn_edge_classifies_the_same_way() {
    let edge = edge_of(specimen(), Decorations::TheApplicationDraws, true);
    assert_eq!(what_the_pointer_is(&edge, at(0, 0)), ThePointer::AtTheEdge);
    assert!(
        !edge.controls.is_empty(),
        "alo adds movement and the menu even here; without this the loop proves nothing"
    );
    for control in &edge.controls {
        assert_eq!(
            what_the_pointer_is(&edge, at(control.target.loc.x, control.target.loc.y)),
            ThePointer::OnTheSurface,
            "{:?} must answer on its own target",
            control.does
        );
    }
}

/// **`THE_STRIP_STARTS_AT` is used, so the region above the strip is still the
/// edge.** The twelve rows between the region's top and the strip's are the
/// approach, and a person crossing them has not reached the strip yet.
#[test]
fn the_rows_above_the_strip_are_the_approach_and_not_nothing() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    let strip = edge.strip.expect("a revealed edge draws a strip");
    // **The premise, read off the edge rather than off the constants.** Two
    // `const`s compared is an assertion clippy rejects as constant-valued, and
    // it would be the weaker claim anyway: what matters is that the edge *as
    // laid out* has rows above its strip, not that two numbers differ.
    assert!(
        strip.loc.y > edge.region.loc.y,
        "the premise: the strip starts below the region's top. region {:?}, strip {strip:?}",
        edge.region
    );
    // Left of every control, above the strip.
    let approach = at(4, strip.loc.y - 1);
    assert!(
        !holds(strip, approach),
        "the premise: this point is above the strip"
    );
    assert!(
        edge.controls.iter().all(|it| !holds(it.target, approach)),
        "and left of every control — a control's target is the region's full height, which is \
         why the approach above the strip is only found away from them"
    );
    assert_eq!(
        what_the_pointer_is(&edge, approach),
        ThePointer::AtTheEdge,
        "above the strip and left of the controls is the region, which keeps the edge revealed"
    );
}

/// **Each control's target reaches that control**, asked at every one.
#[test]
fn a_press_on_a_control_is_that_controls_action() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    assert!(
        !edge.controls.is_empty(),
        "a revealed shell-drawn edge has controls; without this the loop proves nothing"
    );
    for control in &edge.controls {
        let middle = at(
            control.target.loc.x + control.target.size.w / 2,
            control.target.loc.y + control.target.size.h / 2,
        );
        assert_eq!(
            what_a_press_does(&edge, middle),
            WhatAPressDoes::This(control.does),
            "a press at the centre of {:?} must reach it",
            control.does
        );
    }
}

/// **Buttons do not start a drag** — the specification's own clause, and the
/// one a caller could most easily get wrong by hit-testing in the other order.
///
/// Asserted at each control's **corner**, not its centre: the corner is the
/// part of a 44 × 44 target that lies outside the 32 × 28 highlight, so this
/// also fails if anything starts reading the wrong rectangle.
#[test]
fn no_press_anywhere_on_a_control_moves_the_window() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    for control in &edge.controls {
        for corner in [
            at(control.target.loc.x, control.target.loc.y),
            at(
                control.target.loc.x + control.target.size.w - 1,
                control.target.loc.y + control.target.size.h - 1,
            ),
        ] {
            assert_ne!(
                what_a_press_does(&edge, corner),
                WhatAPressDoes::MovesTheWindow,
                "a press at {corner:?} on {:?} would drag the window. *Buttons and menus do not \
                 start a drag*, and the only thing keeping that true is that one classification \
                 answers once and the control is its more specific answer",
                control.does
            );
            assert_eq!(
                what_a_press_does(&edge, corner),
                WhatAPressDoes::This(control.does)
            );
        }
    }
}

/// **The movement region moves the window**, and it is `drag` rather than the
/// strip.
#[test]
fn a_press_on_the_movement_region_moves_the_window() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    let middle = at(
        edge.drag.loc.x + edge.drag.size.w / 2,
        edge.drag.loc.y + edge.drag.size.h / 2,
    );
    assert!(
        edge.controls.iter().all(|it| !holds(it.target, middle)),
        "the premise: the drag region's middle is on no control. `edge_of` ends the revealed \
         drag one gap before the first control, and if that ever changes this test is measuring \
         something else"
    );
    assert_eq!(
        what_a_press_does(&edge, middle),
        WhatAPressDoes::MovesTheWindow
    );
}

/// **A window whose edge is concealed is still movable.**
///
/// The clause *borderless applications stay movable and recoverable*. At rest
/// the drag region is the shallow band the grip is drawn in, and a press there
/// moves the window without anybody having revealed the edge first — so a
/// person who cannot see a title bar can still move the window by aiming at the
/// grip.
#[test]
fn a_concealed_edge_can_still_be_dragged_by_its_grip() {
    let edge = edge_of(specimen(), Decorations::TheShellDraws, false);
    assert!(
        edge.strip.is_none() && edge.controls.is_empty(),
        "the premise: this edge is concealed"
    );
    let on_the_grip = at(
        edge.drag.loc.x + edge.drag.size.w / 2,
        edge.drag.loc.y + edge.drag.size.h / 2,
    );
    assert_eq!(
        what_a_press_does(&edge, on_the_grip),
        WhatAPressDoes::MovesTheWindow,
        "a concealed edge's resting band does not drag, so a borderless window cannot be moved"
    );
    // **And it still reveals from there.** The two answers are read off one
    // classification, and a band that drags but does not reveal would be a
    // person pressing to move a window they cannot see the edge of.
    assert_eq!(
        what_the_pointer_is(&edge, on_the_grip),
        ThePointer::AtTheEdge,
        "the resting band is inside the interaction region, so it asks for the edge too"
    );
}

/// **A press the edge has no business with does nothing**, in both states.
///
/// The application's content, and the approach rows above the strip. A
/// classifier that answered `MovesTheWindow` for the whole region would make
/// every press near the top of a window a window move.
#[test]
fn a_press_off_the_movement_region_does_nothing() {
    for revealed in [false, true] {
        let edge = edge_of(specimen(), Decorations::TheShellDraws, revealed);
        assert_eq!(
            what_a_press_does(&edge, at(300, 300)),
            WhatAPressDoes::Nothing,
            "revealed = {revealed}: a press inside the application is not the edge's"
        );
        assert_eq!(
            what_a_press_does(&edge, at(300, THE_REGION_IS_TALL)),
            WhatAPressDoes::Nothing,
            "revealed = {revealed}: the application's first row is not the edge's"
        );
    }
    // The approach, above a revealed edge's strip and clear of its drag.
    let edge = edge_of(specimen(), Decorations::TheShellDraws, true);
    let above = at(4, 0);
    assert!(
        !holds(edge.drag, above),
        "the premise: this point is outside the movement region"
    );
    assert_eq!(what_a_press_does(&edge, above), WhatAPressDoes::Nothing);
}

/// **An application that draws its own header keeps movement and the menu**,
/// and both are pressable.
#[test]
fn an_application_drawn_edge_is_movable_and_has_its_menu() {
    let edge = edge_of(specimen(), Decorations::TheApplicationDraws, true);
    let middle = at(
        edge.drag.loc.x + edge.drag.size.w / 2,
        edge.drag.loc.y + edge.drag.size.h / 2,
    );
    assert_eq!(
        what_a_press_does(&edge, middle),
        WhatAPressDoes::MovesTheWindow
    );
    let menu = edge
        .controls
        .iter()
        .find(|it| matches!(it.does, crate::window_edge::OnTheEdge::Menu))
        .expect("alo adds the menu to an application-drawn edge");
    assert_eq!(
        what_a_press_does(&edge, at(menu.target.loc.x, menu.target.loc.y)),
        WhatAPressDoes::This(crate::window_edge::OnTheEdge::Menu)
    );
}

/// **A control wins over a drag region that overlaps it**, held against an edge
/// built to overlap on purpose.
///
/// `edge_of` ends the revealed movement region one gap **before** the first
/// control, so the two do not overlap in anything this repository lays out —
/// which was measured rather than assumed: swapping the two questions in
/// `where_on_the_edge` on 2026-10-10 left **all 742 lib tests passing**. The
/// order was defensive and nothing could tell.
///
/// That is a claim about today's geometry, not about the rule. If `edge_of` ever
/// extends the drag under the controls — a wider title, a different inset — the
/// order becomes load-bearing and *buttons and menus do not start a drag* would
/// be broken with every test still green. So this builds the overlap by hand
/// instead of waiting for one: `WindowEdge`'s fields are public, and a drag
/// spanning the whole region is exactly what a later change might produce.
#[test]
fn a_control_beats_a_drag_region_that_covers_it() {
    let ordinary = edge_of(specimen(), Decorations::TheShellDraws, true);
    let control = *ordinary
        .controls
        .first()
        .expect("a revealed edge has controls");
    // The same edge, with the movement region widened across everything.
    let overlapping = crate::window_edge::WindowEdge {
        drag: ordinary.region,
        ..ordinary.clone()
    };
    assert!(
        holds(
            overlapping.drag,
            at(control.target.loc.x, control.target.loc.y)
        ),
        "the premise: this edge's drag region really does cover the control. Without it this \
         test asserts nothing and would pass whatever order the questions are asked in"
    );

    let corner = at(control.target.loc.x, control.target.loc.y);
    assert_eq!(
        where_on_the_edge(&overlapping, corner),
        WhereOnTheEdge::AControl(control.does),
        "a drag region covering a control took the press. The control is the more specific \
         answer and has to win, or pressing Close would move the window"
    );
    assert_eq!(
        what_a_press_does(&overlapping, corner),
        WhatAPressDoes::This(control.does)
    );
    assert_ne!(
        what_a_press_does(&overlapping, corner),
        WhatAPressDoes::MovesTheWindow
    );

    // And the drag still answers where no control is.
    let clear = at(ordinary.region.loc.x + 2, ordinary.region.loc.y + 2);
    assert!(
        overlapping
            .controls
            .iter()
            .all(|it| !holds(it.target, clear)),
        "the premise: this point is on no control"
    );
    assert_eq!(
        where_on_the_edge(&overlapping, clear),
        WhereOnTheEdge::TheDrag
    );
}
