//! That History is the design's drawing on a side Dock and the word on the
//! others, and that the drawing stays inside its own slot at any scale.

#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported, and \
              a formatted panic names the edge it was on"
)]

use alo_dock::places::WhatIsHere;
use alo_dock::{Dock, Edge};
use alo_strings::Direction;
use smithay::utils::{Physical, Rectangle};

use super::{history_is_a_drawing_on, the_history_mark};
use crate::desktop_testing::{an_appearance, noon_look};
use crate::display_scale::DisplayScale;
use crate::dock_raster::DockPicture;

/// The faces the shell draws with, and only those.
fn fonts() -> cosmic_text::FontSystem {
    crate::WindowControlLabels::new()
        .expect("the bundled faces")
        .fonts
}

/// The Dock on `edge` of a display of `size` at `scale`, holding nothing.
fn drawn(edge: Edge, size: (i32, i32), scale: DisplayScale) -> DockPicture {
    let mut dock = Dock::shipped();
    dock.set_edge(edge);
    let strings = alo_strings::Strings::of(
        alo_saying::everything_this_machine_can_say().expect("the vocabulary"),
    );
    let words =
        crate::the_docks_words::TheDocksWords::said(&strings, &crate::WhatTheDockHasOpen::Nothing);
    crate::dock_raster::picture(
        &dock,
        noon_look(&an_appearance(), Direction::LeftToRight),
        size,
        &[],
        &[],
        (&crate::WhatTheDockHasOpen::Nothing, &words),
        None,
        &alo_dock::artwork::NOTHING,
        &mut fonts(),
        scale,
    )
    .unwrap_or_else(|why| panic!("{edge:?} would not draw: {why:?}"))
}

/// History's slot on `picture`, in output pixels.
fn historys_slot(picture: &DockPicture) -> Rectangle<i32, Physical> {
    let dock = &picture.standing;
    let place = dock
        .places
        .each()
        .iter()
        .find(|place| matches!(place.what(), WhatIsHere::History))
        .expect("History is on this Dock");
    let icon = dock.scale.px(alo_dock::measures::ICON);
    let along = dock.scale.px(place.from_the_start());
    let (left, top) = crate::dock_raster::a_slot_at(dock.band, dock.layout.edge(), along, icon);
    Rectangle::new(
        (left, top).into(),
        (dock.scale.px(place.across()), icon).into(),
    )
}

/// Every box of ink that touches `slot`.
fn ink_on(picture: &DockPicture, slot: Rectangle<i32, Physical>) -> Vec<Rectangle<i32, Physical>> {
    picture
        .inked
        .iter()
        .map(|inked| inked.area)
        .filter(|area| {
            area.loc.x < slot.loc.x + slot.size.w
                && slot.loc.x < area.loc.x + area.size.w
                && area.loc.y < slot.loc.y + slot.size.h
                && slot.loc.y < area.loc.y + area.size.h
        })
        .collect()
}

/// **Only the side Docks draw it.**
#[test]
fn history_is_a_drawing_on_the_side_docks_only() {
    for edge in Edge::EVERY {
        assert_eq!(
            history_is_a_drawing_on(edge),
            matches!(edge, Edge::Left | Edge::Right),
            "{edge:?}"
        );
    }
}

/// **On a side Dock, History is one square of the artwork's size, inside its
/// own slot, with nothing running over its edges** — at one to one and at
/// twice the pixels.
#[test]
fn on_a_side_dock_history_is_the_drawing_inside_its_slot() {
    for (size, scale) in [
        ((1920, 1080), DisplayScale::ONE),
        ((3840, 2160), DisplayScale::of_per_cent(200)),
    ] {
        for edge in [Edge::Left, Edge::Right] {
            let picture = drawn(edge, size, scale);
            let slot = historys_slot(&picture);
            let ink = ink_on(&picture, slot);
            assert_eq!(ink.len(), 1, "{edge:?}: {ink:?}");
            let side = scale.px(alo_dock::measures::GLYPH);
            assert_eq!(ink[0].size, (side, side).into(), "{edge:?}");
            assert!(
                ink[0].loc.x >= slot.loc.x
                    && ink[0].loc.y >= slot.loc.y
                    && ink[0].loc.x + side <= slot.loc.x + slot.size.w
                    && ink[0].loc.y + side <= slot.loc.y + slot.size.h,
                "{edge:?}: the drawing runs over its slot: {:?} in {slot:?}",
                ink[0]
            );
        }
    }
}

/// **On the top Dock History is still the word**, so nothing drawn on its
/// slot is a square of the artwork's size.
#[test]
fn on_the_top_dock_history_is_still_the_word() {
    let picture = drawn(Edge::Top, (1920, 1080), DisplayScale::ONE);
    let side = i32::try_from(alo_dock::measures::GLYPH).expect("small");
    let ink = ink_on(&picture, historys_slot(&picture));
    assert!(!ink.is_empty(), "History drew nothing on the top Dock");
    assert!(
        ink.iter().all(|area| area.size != (side, side).into()),
        "the top Dock drew the drawing: {ink:?}"
    );
}

/// **The drawing is the design's**: ink on its strokes, ground between them,
/// and about four times the ink at twice the side.
#[test]
fn the_drawing_has_ink_on_its_strokes_and_none_between_them() {
    let ground = [255, 255, 255];
    let ink = [16, 42, 67];
    let mark = the_history_mark(28, ground, ink);
    assert_eq!(mark.len(), 28 * 28);
    let at = |x: usize, y: usize| mark[y * 28 + x];
    // The upright hand runs from (14, 8.17) to (14, 14): pixel 13 spans 13 to
    // 14, so its centre is half a unit from the line, inside the stroke.
    assert!(
        at(13, 11)[0] < 64,
        "the hand is not drawn: {:?}",
        at(13, 11)
    );
    // Inside the face, clear of both hands and the rim.
    assert_eq!(at(18, 20), ground, "ink where the face is empty");
    // The corner of the square, outside the rim.
    assert_eq!(at(27, 27), ground, "ink outside the rim");
    // The rim at its rightmost, near x 23.9 at y 13 to 14.
    assert!(at(23, 13)[0] < 64, "the rim is not drawn: {:?}", at(23, 13));
    let inked = mark.iter().filter(|pixel| **pixel != ground).count();
    let twice = the_history_mark(56, ground, ink);
    let inked_twice = twice.iter().filter(|pixel| **pixel != ground).count();
    assert!(
        (3 * inked..=5 * inked).contains(&inked_twice),
        "twice the side should be about four times the ink: {inked} and {inked_twice}"
    );
    assert!(the_history_mark(0, ground, ink).is_empty());
    assert!(the_history_mark(-3, ground, ink).is_empty());
}
