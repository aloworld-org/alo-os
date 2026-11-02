//! The terminal's body, held to the colours the design names, to its rounded
//! corners and shadow, and to a display's scale.
//!
//! **Read back out of the raster's pixels**, because *the corner is rounded*
//! and *the shadow falls below* are claims about coverage, and coverage is a
//! number here.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected shape is the failure being reported, and a \
              formatted panic names the pixel it was reading"
)]

use alo_appearance::{Role, Scheme};
use smithay::utils::{Physical, Rectangle};

use super::{TheTerminalsColours, the_body_in};
use crate::painted::Washed;
use crate::window_edge_look::WhichLook;

/// A role as the three bytes a picture is painted in.
fn rgb(role: Role) -> [u8; 3] {
    let it = role.colour();
    [it.red(), it.green(), it.blue()]
}

/// A display the size of the design's own frames.
const A_DISPLAY: (i32, i32) = (1440, 1040);

/// A window the size the design drew one at, where the design drew it.
fn a_window() -> Rectangle<i32, Physical> {
    Rectangle::new((200, 120).into(), (1040, 800).into())
}

/// The light body of a window the keys are not going to, at this scale.
fn a_body(window: Rectangle<i32, Physical>, room: (i32, i32), scale: f64) -> Washed {
    the_body_in(
        window,
        room,
        scale,
        TheTerminalsColours::of(Scheme::Light),
        WhichLook::NotFocused,
    )
    .expect("a window with area has a body")
}

/// The four bytes at this point of the display, read out of the raster.
fn pixel(of: &Washed, x: i32, y: i32) -> [u8; 4] {
    let (rx, ry) = (x - of.area.loc.x, y - of.area.loc.y);
    assert!(
        rx >= 0 && ry >= 0 && rx < of.area.size.w && ry < of.area.size.h,
        "({x}, {y}) is outside the raster at {:?}",
        of.area
    );
    let at = usize::try_from((ry * of.area.size.w + rx) * 4).expect("a place in the raster");
    let bytes = of
        .pixels
        .get(at..at + 4)
        .unwrap_or_else(|| panic!("no pixel at ({x}, {y})"));
    let mut out = [0_u8; 4];
    out.copy_from_slice(bytes);
    out
}

/// How far along the window's top row the panel becomes solid, from its left
/// edge — which is the corner's reach.
fn where_the_top_edge_becomes_solid(body: &Washed, window: Rectangle<i32, Physical>) -> i32 {
    (0..window.size.w / 2)
        .find(|step| pixel(body, window.loc.x + step, window.loc.y)[3] == 255)
        .unwrap_or_else(|| panic!("the top edge never becomes solid"))
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

/// The outermost row of the window is the hairline and the row inside it is
/// the panel, both fully opaque — so the window has an edge and a body.
#[test]
fn the_body_is_a_hairline_with_the_panel_inside_it() {
    let colours = TheTerminalsColours::of(Scheme::Light);
    let window = a_window();
    let body = a_body(window, A_DISPLAY, 1.0);
    let middle = window.loc.x + window.size.w / 2;
    let [r, g, b, a] = pixel(&body, middle, window.loc.y);
    assert_eq!(
        (a, [r, g, b]),
        (255, colours.hairline),
        "the top row is not the hairline"
    );
    let [r, g, b, a] = pixel(&body, middle, window.loc.y + 1);
    assert_eq!(
        (a, [r, g, b]),
        (255, colours.ground),
        "one row in is not the panel"
    );
}

/// **The corners are rounded to `radius/lg`, 18**: the window's very corner is
/// empty, the top row becomes solid about eighteen in, and the middle of an
/// edge is solid at once.
#[test]
fn the_corners_are_rounded_to_the_designs_radius() {
    let window = a_window();
    let body = a_body(window, A_DISPLAY, 1.0);
    let [.., corner] = pixel(&body, window.loc.x, window.loc.y);
    assert!(
        corner < 64,
        "the window's corner pixel is {corner} opaque, so the corner is square"
    );
    let reach = where_the_top_edge_becomes_solid(&body, window);
    assert!(
        (reach - 18).abs() <= 4,
        "the top edge becomes solid {reach} in, where the design's radius is 18"
    );
}

/// **At 200% the radius doubles**, and the same window drawn at 100% does not
/// — the control, without which a radius that ignored the scale would pass.
#[test]
fn at_two_hundred_percent_the_corner_is_twice_as_round() {
    let window = a_window();
    let ordinary = where_the_top_edge_becomes_solid(&a_body(window, A_DISPLAY, 1.0), window);
    let dense = where_the_top_edge_becomes_solid(&a_body(window, A_DISPLAY, 2.0), window);
    assert!(
        (dense - 36).abs() <= 6,
        "at 200% the top edge becomes solid {dense} in, where twice the radius is 36"
    );
    assert!(
        dense > ordinary + 12,
        "the corner at 200% ({dense}) is not clearly rounder than at 100% ({ordinary})"
    );
}

/// **A shadow falls below the window and fades**, darker just under it than
/// further away, and there is no shadow above it to speak of — the design's
/// offset is (0, 16), straight down. Held in the focused look, whose shadow
/// this is.
#[test]
fn a_shadow_falls_below_the_window_and_fades() {
    let window = a_window();
    let body = the_body_in(
        window,
        A_DISPLAY,
        1.0,
        TheTerminalsColours::of(Scheme::Light),
        WhichLook::Focused,
    )
    .expect("a window with area has a body");
    let middle = window.loc.x + window.size.w / 2;
    let below = window.loc.y + window.size.h;
    let near = pixel(&body, middle, below + 4)[3];
    let far = pixel(&body, middle, below + 40)[3];
    let above = pixel(&body, middle, window.loc.y - 4)[3];
    assert!(near > 0, "nothing is drawn just under the window");
    assert!(
        near > far,
        "the shadow does not fade: {near} near and {far} far"
    );
    assert!(
        above < near,
        "the shadow is as dark above the window ({above}) as below it ({near})"
    );
    // At its darkest it is `Elevation/Floating`'s 0x24 — never more.
    assert!(
        near <= 0x24,
        "the shadow is {near} dark, past the design's 36"
    );
}

/// **The shadow goes through the scale too**: at 200% it still reaches
/// further below the window than at 100%, because the fall and the blur doubled
/// with the corner.
#[test]
fn the_shadow_reaches_twice_as_far_at_two_hundred_percent() {
    let window = a_window();
    let middle = window.loc.x + window.size.w / 2;
    let below = window.loc.y + window.size.h;
    let reaches = |scale: f64| {
        let body = a_body(window, A_DISPLAY, scale);
        let last = body.area.loc.y + body.area.size.h;
        (below..last)
            .take_while(|y| pixel(&body, middle, *y)[3] > 0)
            .count()
    };
    let (ordinary, dense) = (reaches(1.0), reaches(2.0));
    assert!(
        dense > ordinary * 3 / 2,
        "the shadow reaches {dense} at 200% and {ordinary} at 100%"
    );
}

/// **The raster is kept on the display**: a window against the screen's edge
/// loses the shadow off the screen, not a frame drawn past it.
#[test]
fn the_body_is_kept_on_the_display() {
    let window = Rectangle::new((0, 0).into(), (400, 300).into());
    let body = a_body(window, (400, 300), 1.0);
    assert_eq!(
        body.area, window,
        "the raster hangs off a display the window fills"
    );
}

/// **The refusal.** A window with no area draws no body rather than a raster
/// of no size.
#[test]
fn a_window_with_no_room_draws_no_body() {
    let colours = TheTerminalsColours::of(Scheme::Light);
    assert!(
        the_body_in(
            Rectangle::new((0, 0).into(), (0, 800).into()),
            A_DISPLAY,
            1.0,
            colours,
            WhichLook::Focused
        )
        .is_none()
    );
}

/// **The focused look is the approved one**: the hairline is `border/strong`
/// and the shadow just under the window is darker than the unfocused one's —
/// the two looks every window wears (`557:213`, `557:149`).
#[test]
fn the_focused_body_has_the_strong_hairline_and_the_deeper_shadow() {
    let window = a_window();
    let colours = TheTerminalsColours::of(Scheme::Light);
    let focused = the_body_in(window, A_DISPLAY, 1.0, colours, WhichLook::Focused)
        .expect("a window with area has a body");
    let quiet = a_body(window, A_DISPLAY, 1.0);
    let middle = window.loc.x + window.size.w / 2;
    let [r, g, b, _] = pixel(&focused, middle, window.loc.y);
    assert_eq!([r, g, b], crate::window_edge_look::BORDER_STRONG);
    let [r, g, b, _] = pixel(&quiet, middle, window.loc.y);
    assert_eq!([r, g, b], colours.hairline);
    let below = window.loc.y + window.size.h + 12;
    assert!(
        pixel(&focused, middle, below)[3] > pixel(&quiet, middle, below)[3],
        "the focused window is not raised higher than the unfocused one"
    );
}
