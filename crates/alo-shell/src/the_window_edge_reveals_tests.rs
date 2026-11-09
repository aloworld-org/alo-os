//! The band's reveal, held to the specification's continuous path.
//!
//! These drive [`what_the_pointer_is`] and the shared
//! [`alo_dock::revealing::Revealing`] machine together, so a pointer is walked
//! across a band the way a person walks it.

use super::*;
use crate::window_edge::{Decorations, OnTheEdge, edge_of};
use alo_dock::revealing::Revealing;
use smithay::utils::Size;

/// The Notes window of `11:24`, its band at the origin.
fn specimen() -> Rectangle<i32, Logical> {
    Rectangle::new(Point::from((0, 54)), Size::from((1056, 466)))
}

/// A point.
fn at(x: i32, y: i32) -> Point<i32, Logical> {
    Point::from((x, y))
}

/// A display large enough that nothing is clamped.
fn roomy() -> Size<i32, Logical> {
    Size::from((4000, 3000))
}

/// The band as `revealing` lays it out.
fn band(revealing: Revealing) -> WindowEdge {
    edge_of(
        specimen(),
        Decorations::TheShellDraws,
        revealing.is_revealed(),
        roomy(),
    )
}

/// **Anywhere in the band reveals** — on the title, on the empty middle, and
/// where a control will appear.
#[test]
fn a_pointer_anywhere_in_the_band_reveals_the_controls() {
    for point in [at(30, 25), at(500, 2), at(1000, 30), at(1050, 53)] {
        let after = Revealing::covered()
            .the_pointer_is(what_the_pointer_is(&band(Revealing::covered()), point));
        assert!(after.is_revealed(), "{point:?} did not reveal");
    }
}

/// **A pointer in the application is not in the band**, whether the controls
/// are shown or not — the content below is never part of the reveal.
#[test]
fn a_pointer_in_the_application_is_elsewhere() {
    let shown = Revealing::covered().the_pointer_is(ThePointer::AtTheEdge);
    for revealing in [Revealing::covered(), shown] {
        assert_eq!(
            what_the_pointer_is(&band(revealing), at(500, 54)),
            ThePointer::Elsewhere
        );
    }
}

/// **The continuous path**: from the title along the band onto Close, the
/// controls never go — no gap anywhere for them to drop through.
#[test]
fn a_pointer_walking_from_the_title_to_close_never_loses_the_controls() {
    let mut revealing = Revealing::covered();
    for x in (22..1038).step_by(3) {
        let point = at(x, 27);
        revealing = revealing.the_pointer_is(what_the_pointer_is(&band(revealing), point));
        assert!(revealing.is_revealed(), "lost at x = {x}");
    }
    let close = band(revealing)
        .the_control(OnTheEdge::Close)
        .map(|it| it.target);
    assert!(
        close.is_some_and(|it| it.contains(at(1010, 27))),
        "the walk ends on Close"
    );
}

/// **Leaving the band conceals them at once** — there is no wait; the fade is
/// how it is drawn, not whether it happened.
#[test]
fn leaving_the_band_conceals_them_with_no_wait() {
    let revealing = Revealing::covered().the_pointer_is(ThePointer::AtTheEdge);
    let left = revealing.the_pointer_is(what_the_pointer_is(&band(revealing), at(500, 300)));
    assert!(!left.is_revealed());
    assert!(
        band(left).controls.is_empty(),
        "a control that has gone cannot be pressed"
    );
}

/// **An open panel holds them**, with the pointer gone from the band.
#[test]
fn an_open_panel_holds_the_controls_with_the_pointer_elsewhere() {
    let held = Revealing::covered()
        .the_pointer_is(ThePointer::AtTheEdge)
        .a_menu(alo_dock::revealing::AMenu::IsOpen)
        .the_pointer_is(ThePointer::Elsewhere);
    assert!(held.is_revealed());
}

/// **The keyboard holds them**, so focus is never on a control that cannot be
/// seen.
#[test]
fn the_keyboard_holds_the_controls() {
    let held = Revealing::covered().the_keyboard(alo_dock::revealing::TheKeyboard::IsOnIt);
    assert!(held.is_revealed());
}

/// **A band of no extent claims nothing**, so a window of no width reveals
/// nothing from wherever the pointer is.
#[test]
fn a_band_of_no_extent_claims_no_point() {
    let mut edge = band(Revealing::covered());
    edge.region = Rectangle::new(at(0, 0), Size::from((0, 54)));
    assert_eq!(what_the_pointer_is(&edge, at(0, 0)), ThePointer::Elsewhere);
}

/// **The band does not claim the column just past its end**, so two bands side
/// by side do not both reveal on the line they share.
#[test]
fn the_band_does_not_claim_the_column_past_its_end() {
    let edge = band(Revealing::covered());
    assert_eq!(
        what_the_pointer_is(&edge, at(1055, 10)),
        ThePointer::AtTheEdge
    );
    assert_eq!(
        what_the_pointer_is(&edge, at(1056, 10)),
        ThePointer::Elsewhere
    );
}
