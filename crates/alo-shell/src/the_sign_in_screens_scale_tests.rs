//! The sign-in screen's scale, from what a screen reports about itself.

use super::*;

/// **A dense laptop panel is drawn larger**, by the same arithmetic a screen
/// nobody has arranged gets after sign-in: 1920 × 1080 on a 13-inch panel is
/// 175 per cent, and a 15-inch 4K panel is the largest size there is.
#[test]
fn a_dense_screen_is_drawn_larger() {
    assert_eq!(
        the_sign_in_screens_scale("eDP-1", (1920, 1080), (294, 165)).per_cent(),
        175
    );
    assert_eq!(
        the_sign_in_screens_scale("eDP-1", (3840, 2160), (345, 194)).per_cent(),
        300
    );
}

/// **An ordinary desktop monitor stays at one-to-one**, so the screen most
/// people meet is drawn exactly as it was.
#[test]
fn an_ordinary_monitor_is_one_to_one() {
    assert_eq!(
        the_sign_in_screens_scale("DP-1", (1920, 1080), (531, 299)),
        DisplayScale::ONE
    );
}

/// **A screen that does not say how large it is cannot be worked out**, and
/// is one-to-one rather than a guess. A virtual machine's display says
/// nothing, and neither does a projector that reports a zero.
#[test]
fn a_screen_that_does_not_say_its_size_is_one_to_one() {
    for glass in [(0, 0), (0, 194), (345, 0), (-1, -1)] {
        assert_eq!(
            the_sign_in_screens_scale("Virtual-1", (3840, 2160), glass),
            DisplayScale::ONE,
            "{glass:?}"
        );
    }
}

/// **Nonsense in is one-to-one out, never a panic**: no pixels, negative
/// pixels, and a connector with no name.
#[test]
fn nonsense_is_one_to_one() {
    for (connector, pixels) in [
        ("eDP-1", (0, 0)),
        ("eDP-1", (-3840, 2160)),
        ("", (3840, 2160)),
    ] {
        assert_eq!(
            the_sign_in_screens_scale(connector, pixels, (345, 194)),
            DisplayScale::ONE,
            "{connector:?} {pixels:?}"
        );
    }
}

/// **A small, very dense panel is held to what still fits**, rather than
/// worked out to a size at which the sign-in screen refuses to draw at all.
/// 800 × 600 on glass 68 millimetres across works out at 300; at that the
/// screen would be 266 logical pixels wide, under the 360 it needs.
#[test]
fn a_small_dense_panel_is_held_to_what_fits() {
    let scale = the_sign_in_screens_scale("DSI-1", (800, 600), (68, 51));
    assert!(scale.per_cent() > 100, "the premise: it is dense");
    assert_eq!(scale.per_cent(), 166, "600 × 100 / 360, rounded down");
    let logical = (scale.logical(800), scale.logical(600));
    assert!(
        logical.0 >= 360 && logical.1 >= 360,
        "{logical:?} is smaller than the sign-in screen lays out for"
    );
}
