//! What the panel does with a pointer button, every case.
//!
//! Asked of [`whether_the_panel_takes_this_click`] rather than of a seat, because the
//! decision is the thing being tested and a seat would need a display, a pointing device
//! and a drawn panel to reach it. The geometry it consults is `crate::the_panel_reveals`'
//! own, tested there against real draws.

use super::{TheClick, whether_the_panel_takes_this_click};
use alo_dock::revealing::ThePointer;
use smithay::backend::input::ButtonState;

/// **A press on the rail is taken, and where it happened is remembered.**
#[test]
fn a_press_on_the_rail_is_taken_and_remembered() {
    assert_eq!(
        whether_the_panel_takes_this_click(
            Some(ThePointer::OnTheSurface),
            ButtonState::Pressed,
            false
        ),
        TheClick::TakeItAndRememberWhere
    );
}

/// **A press in the reserved column but off the rail passes on.**
///
/// There is no preview there to bring back. Swallowing it would lose a click to perform
/// nothing — and this is the case a concealed panel gives for its *whole* column, because
/// `crate::panel_raster` gives a concealed panel a rail of no height. So this one
/// assertion is also what stops an invisible panel eating clicks.
#[test]
fn a_press_at_the_edge_passes_on_which_is_also_a_concealed_panel() {
    assert_eq!(
        whether_the_panel_takes_this_click(
            Some(ThePointer::AtTheEdge),
            ButtonState::Pressed,
            false
        ),
        TheClick::PassItOn
    );
}

/// **A press anywhere else passes on.**
#[test]
fn a_press_elsewhere_passes_on() {
    assert_eq!(
        whether_the_panel_takes_this_click(
            Some(ThePointer::Elsewhere),
            ButtonState::Pressed,
            false
        ),
        TheClick::PassItOn
    );
}

/// **A press that cannot be placed passes on.**
///
/// `None` is before the first draw, or before the seat has a pointer. A compositor that
/// treated *I do not know where this is* as *it is mine* would swallow every click on a
/// machine that had not painted yet.
#[test]
fn a_press_that_cannot_be_placed_passes_on() {
    assert_eq!(
        whether_the_panel_takes_this_click(None, ButtonState::Pressed, false),
        TheClick::PassItOn
    );
}

/// **The release of a press the panel took is taken too.**
///
/// A client that never saw the press must not see the release. Otherwise a person who
/// clicks a preview leaves whichever client had the pointer believing a button it never
/// saw pressed has just come up.
#[test]
fn the_release_of_a_taken_press_is_taken() {
    assert_eq!(
        whether_the_panel_takes_this_click(
            Some(ThePointer::OnTheSurface),
            ButtonState::Released,
            true
        ),
        TheClick::TakeItToKeepTheButtonsPaired
    );
}

/// **The release of a press the panel did not take passes on, even over the rail.**
///
/// This is the case the geometry must not decide. A person presses on a window, moves
/// onto the panel, releases: the pointer is on the rail, and the release belongs to the
/// client that got the press. Asking *where is the pointer now* would take it and leave
/// that client holding the button for ever — a drag that never ends.
#[test]
fn a_release_over_the_rail_whose_press_was_not_taken_still_passes_on() {
    assert_eq!(
        whether_the_panel_takes_this_click(
            Some(ThePointer::OnTheSurface),
            ButtonState::Released,
            false
        ),
        TheClick::PassItOn,
        "a release was taken on where the pointer is rather than on whose press it was"
    );
}

/// **And the mirror: a release off the panel whose press the panel took is still taken.**
///
/// A person presses a preview and releases having moved away. The press was the panel's,
/// so the release is too — no client saw either, and none should see half of one.
#[test]
fn a_release_away_from_the_panel_whose_press_was_taken_is_still_taken() {
    assert_eq!(
        whether_the_panel_takes_this_click(
            Some(ThePointer::Elsewhere),
            ButtonState::Released,
            true
        ),
        TheClick::TakeItToKeepTheButtonsPaired,
        "the panel let go of a button it had taken, because the pointer had moved off it"
    );
}
