//! The terminal's body, held to the colours the design names and to a
//! hairline that survives every display scale.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected shape is the failure being reported"
)]

use alo_appearance::{Role, Scheme};
use smithay::utils::{Physical, Rectangle};

use super::{TheTerminalsColours, the_body};

/// A role as the three bytes a picture is painted in.
fn rgb(role: Role) -> [u8; 3] {
    let it = role.colour();
    [it.red(), it.green(), it.blue()]
}

/// A window the size the design drew one at.
fn a_window() -> Rectangle<i32, Physical> {
    Rectangle::new((200, 120).into(), (1040, 800).into())
}

/// **The panel is not the canvas.** The fault this file ends was a window
/// painted in `bg/canvas` on a canvas painted in `bg/canvas`, which is no window
/// at all; this holds the light panel to `bg/surface` and away from the canvas.
#[test]
fn the_light_panel_is_the_designs_surface_and_not_the_canvas() {
    let colours = TheTerminalsColours::of(Scheme::Light);
    assert_eq!(colours.ground, rgb(Role::BgSurface));
    assert_ne!(colours.ground, rgb(Role::BgCanvas));
    assert_eq!(colours.ink, rgb(Role::TextPrimary));
    assert_eq!(colours.hairline, rgb(Role::BorderDefault));
}

/// **The hairline is a different colour from the panel in both appearances**,
/// or there is no edge to see.
#[test]
fn the_hairline_can_be_told_from_the_panel_in_both_appearances() {
    for scheme in [Scheme::Light, Scheme::Dark] {
        let colours = TheTerminalsColours::of(scheme);
        assert_ne!(colours.hairline, colours.ground, "{scheme:?}");
    }
}

/// The body is the hairline round the whole window, then the panel one
/// hairline in from every side — so the outermost pixel is the hairline's.
#[test]
fn the_body_is_a_hairline_with_the_panel_inside_it() {
    let colours = TheTerminalsColours::of(Scheme::Light);
    let body = the_body(a_window(), 1.0, colours);
    let [edge, panel] = body.as_slice() else {
        panic!(
            "the body is a hairline and a panel, and drew {} shapes",
            body.len()
        );
    };
    assert_eq!(edge.area, a_window());
    assert_eq!(edge.colour, colours.hairline);
    assert_eq!(
        panel.area,
        Rectangle::new((201, 121).into(), (1038, 798).into()),
        "the design's children start at (1, 1) and are 1038 wide in a 1040 body"
    );
    assert_eq!(panel.colour, colours.ground);
}

/// **The hairline goes through the display's scale**: two units on a display at
/// twice the density, not one pixel that halves on it.
#[test]
fn the_hairline_is_scaled_with_the_display() {
    let body = the_body(a_window(), 2.0, TheTerminalsColours::of(Scheme::Light));
    let [edge, panel] = body.as_slice() else {
        panic!("the body is a hairline and a panel");
    };
    assert_eq!(panel.area.loc.x - edge.area.loc.x, 2);
    assert_eq!(edge.area.size.w - panel.area.size.w, 4);
}

/// **A display scaled below one still gets an edge.** Rounded honestly, one
/// unit at three quarters is one pixel; at less it would round to none.
#[test]
fn a_hairline_never_rounds_to_nothing() {
    let body = the_body(a_window(), 0.4, TheTerminalsColours::of(Scheme::Light));
    let [edge, panel] = body.as_slice() else {
        panic!("the body is a hairline and a panel");
    };
    assert_eq!(panel.area.loc.x - edge.area.loc.x, 1);
}

/// **The refusals.** A window with no area draws nothing, and one too small to
/// have anything inside its hairline draws the hairline alone rather than a
/// panel with a negative size.
#[test]
fn a_window_with_no_room_draws_no_panel() {
    let colours = TheTerminalsColours::of(Scheme::Light);
    assert!(the_body(Rectangle::new((0, 0).into(), (0, 800).into()), 1.0, colours).is_empty());
    let a_sliver = the_body(Rectangle::new((0, 0).into(), (2, 800).into()), 1.0, colours);
    let [only] = a_sliver.as_slice() else {
        panic!("a two-pixel window is all hairline");
    };
    assert_eq!(only.colour, colours.hairline);
}
