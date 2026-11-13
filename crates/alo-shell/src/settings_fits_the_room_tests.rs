//! Settings sized for the room above the Dock and clear of the panel, held to
//! the owner's *"Yes, fit above the Dock"* of 2026-11-13: on the design's small
//! frame with the walk's Dock, on a large display, and at 200 per cent — and
//! in each, **the size the rule gives is a size the placement room holds**,
//! which is where the two ends of this change meet.
//!
//! The Docks and panels here are fixtures in the shapes production draws (a
//! centred band floating off the bottom, a full-height column on the right),
//! not measurements of a machine; the 1 280 × 800 Dock is the walk's (`w26-43`).

use alo_appearance::TextScale;
use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

use super::the_room_settings_is_sized_for;
use crate::a_window_and_its_strip::ItsStrip;
use crate::display_scale::DisplayScale;
use crate::settings_on_the_plane::how_big_settings_is;
use crate::the_room_alo_places_a_window_in::{
    kept_inside, lies_wholly_in, the_room_the_controls_leave,
};

/// One display: its size, its Dock's band and its panel's column, each as
/// `(x, y, w, h)` in its pixels, and its scale in per cent.
struct ADisplay {
    /// Its size in pixels.
    size: (i32, i32),
    /// Its Dock's band.
    dock: (i32, i32, i32, i32),
    /// Its panel's column.
    panel: (i32, i32, i32, i32),
    /// Its scale in per cent.
    scale: u16,
}

/// The design's small frame with the walk's Dock, its top at 717, and a
/// panel column the width `crate::panel_raster` gives at 100 per cent.
const THE_SMALL_FRAME: ADisplay = ADisplay {
    size: (1280, 800),
    dock: (272, 717, 736, 74),
    panel: (1168, 0, 112, 800),
    scale: 100,
};

/// A rectangle of the display's pixels.
fn physical((x, y, w, h): (i32, i32, i32, i32)) -> Rectangle<i32, Physical> {
    Rectangle::new((x, y).into(), (w, h).into())
}

/// At life size with the camera at the origin, the plane is the display.
fn logical((x, y, w, h): (i32, i32, i32, i32)) -> Rectangle<i32, Logical> {
    Rectangle::new((x, y).into(), (w, h).into())
}

/// **Settings' window on `display`**, by the rule asked of `asked`, and
/// whether — placed by `kept_inside` from the view's corner — it lies wholly
/// in the room a window alo places opens in, band and gap included.
fn sized_and_placed(display: &ADisplay, asked: (i32, i32)) -> Option<(Size<i32, Logical>, bool)> {
    let strip = ItsStrip::at(u32::from(display.scale));
    let sizes = how_big_settings_is(
        asked,
        TextScale::ordinary(),
        DisplayScale::of_per_cent(display.scale),
    )?;
    let window = Size::from(sizes.window);
    let room = the_room_the_controls_leave(
        logical((0, 0, display.size.0, display.size.1)),
        Some(logical(display.dock)),
        Some(logical(display.panel)),
    );
    let at = kept_inside(Point::from((0, 0)), window, room, strip);
    Some((window, lies_wholly_in(at, window, room, strip)))
}

/// What the rule is asked of on `display`: the room, less the band at its
/// scale.
fn the_room_on(display: &ADisplay) -> (i32, i32) {
    let band = ItsStrip::at(u32::from(display.scale))
        .round_a_size(Size::from((1, 1)))
        .h
        - 1;
    the_room_settings_is_sized_for(
        display.size,
        Some(physical(display.dock)),
        Some(physical(display.panel)),
        band,
    )
}

/// **On 1 280 × 800 with the walk's Dock, the whole window fits** — band
/// included, above the Dock and left of the panel — and is shorter than the
/// design's 688. The control is the rule asked of the whole display, which is
/// what #1217 left: its window does not fit.
#[test]
fn on_the_small_frame_the_whole_window_fits_above_the_dock() {
    let asked = the_room_on(&THE_SMALL_FRAME);
    assert_eq!(
        asked,
        (1168, 717 - 54),
        "the room is the display left of the panel and above the Dock, less the 54 band"
    );
    let (window, fits) = sized_and_placed(&THE_SMALL_FRAME, asked).expect("room for Settings");
    assert!(fits, "Settings {window:?} does not lie wholly in the room");
    assert!(
        window.h < 688,
        "it is shorter than the design's frame: {window:?}"
    );
    let (whole, whole_fits) =
        sized_and_placed(&THE_SMALL_FRAME, THE_SMALL_FRAME.size).expect("room for Settings");
    assert_eq!(
        (whole.w, whole.h),
        (1152, 688),
        "the control is the design's rule"
    );
    assert!(
        !whole_fits,
        "the control: asked of the whole display, Settings would fit, so this test could \
         not tell the two rules apart"
    );
}

/// **On a large display it is the design's full size**, because the room
/// holds it: fitting the room never makes Settings smaller than the design
/// draws it where there is space.
#[test]
fn on_a_large_display_it_is_the_designs_full_size() {
    let large = ADisplay {
        size: (2560, 1440),
        dock: (912, 1357, 736, 74),
        panel: (2448, 0, 112, 1440),
        scale: 100,
    };
    let (window, fits) = sized_and_placed(&large, the_room_on(&large)).expect("room");
    assert_eq!((window.w, window.h), (1200, 816));
    assert!(fits, "Settings {window:?} does not lie wholly in the room");
}

/// **At 200 per cent it is the small frame's window drawn sharper**, twice the
/// pixels and not half the size, and it still fits.
#[test]
fn at_twice_the_scale_it_is_drawn_sharper_and_fits() {
    let dense = ADisplay {
        size: (2560, 1600),
        dock: (544, 1434, 1472, 148),
        panel: (2336, 0, 224, 1600),
        scale: 200,
    };
    let (window, fits) = sized_and_placed(&dense, the_room_on(&dense)).expect("room");
    let (small, _) =
        sized_and_placed(&THE_SMALL_FRAME, the_room_on(&THE_SMALL_FRAME)).expect("room");
    assert!(
        (window.w - 2 * small.w).abs() <= 2 && (window.h - 2 * small.h).abs() <= 2,
        "at 200 per cent {window:?} is not twice the 100 per cent {small:?}"
    );
    assert!(fits, "Settings {window:?} does not lie wholly in the room");
}

/// **A room with no space for Settings is refused, not drawn cramped**: the
/// rule's own refusal, asked of the room. And a Dock or panel that covers
/// nothing takes nothing.
#[test]
fn a_room_too_small_is_refused_and_an_empty_control_takes_nothing() {
    let tiny = ADisplay {
        size: (640, 480),
        dock: (100, 400, 440, 74),
        panel: (528, 0, 112, 480),
        scale: 100,
    };
    assert!(
        sized_and_placed(&tiny, the_room_on(&tiny)).is_none(),
        "a room with no space for the content's column was given a Settings"
    );
    assert_eq!(
        the_room_settings_is_sized_for((1280, 800), None, Some(Rectangle::default()), 54),
        (1280, 746)
    );
}
