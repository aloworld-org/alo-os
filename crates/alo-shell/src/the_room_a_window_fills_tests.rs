//! The room a maximised or full-screen window fills, and its refusals.

use smithay::utils::{Physical, Rectangle};

use super::*;

fn rect(x: i32, y: i32, w: i32, h: i32) -> Rectangle<i32, Physical> {
    Rectangle::new((x, y).into(), (w, h).into())
}

/// A 1440 × 960 display drawn as the design's frames draw it: the top
/// controls 84 tall, the panel's column 112 wide on the right, the Dock
/// floating 16 above the bottom.
fn as_drawn() -> WhatIsReserved {
    WhatIsReserved {
        dock: Some(rect(400, 872, 528, 72)),
        panel: Some(rect(1328, 0, 112, 960)),
        top: Some(rect(48, 0, 1280, 84)),
        leaving: Some(rect(0, 0, 48, 48)),
    }
}

#[test]
fn the_work_area_is_the_display_less_every_reserved_edge() {
    assert_eq!(
        the_work_area((1440, 960), as_drawn()),
        rect(0, 84, 1328, 788),
        "top controls take the top, the panel the right, the Dock everything below its top"
    );
}

#[test]
fn reading_right_to_left_the_panel_takes_the_left() {
    let reserved = WhatIsReserved {
        panel: Some(rect(0, 0, 112, 960)),
        ..WhatIsReserved::default()
    };
    assert_eq!(
        the_work_area((1440, 960), reserved),
        rect(112, 0, 1328, 960)
    );
}

#[test]
fn nothing_reserved_is_the_whole_display() {
    assert_eq!(
        the_work_area((800, 600), WhatIsReserved::default()),
        rect(0, 0, 800, 600)
    );
}

#[test]
fn a_dock_that_gave_way_still_bounds_the_work_area() {
    let before = as_drawn();
    let drawn = crate::canvas_fixed_controls::FixedControlsDrawn {
        dock_band: None,
        panel_reserved: rect(1328, 0, 112, 960),
        what_is_leaving: None,
        top_controls: None,
    };
    let now = WhatIsReserved::drawn(Some(drawn), before);
    assert_eq!(now.dock, before.dock, "a hidden Dock must not grow the work area");
    assert_eq!(now.top, before.top);
    assert_eq!(now.leaving, None);
}

#[test]
fn a_maximised_window_keeps_its_band_inside_the_work_area() {
    let room = ADisplaysRoom::of((1440, 960), as_drawn(), alo_canvas::Camera::new(), 100)
        .expect("a room");
    let share = room.maximised().expect("room for a window");
    assert_eq!(share.at, (0, 84 + 54), "the band sits at the top of the work area");
    assert_eq!(share.size, (1328, 788 - 54));
}

#[test]
fn the_band_grows_with_the_displays_scale() {
    let room = ADisplaysRoom::of((2880, 1920), WhatIsReserved::default(), alo_canvas::Camera::new(), 200)
        .expect("a room");
    assert_eq!(room.band, 108);
    assert_eq!(room.maximised().expect("room").at, (0, 108));
}

#[test]
fn a_work_area_too_short_for_a_band_refuses_to_maximise() {
    let room = ADisplaysRoom::of((400, 54), WhatIsReserved::default(), alo_canvas::Camera::new(), 100)
        .expect("a room");
    assert_eq!(
        room.maximised(),
        None,
        "a window with no room under its band is refused, never drawn with the band off screen"
    );
}

#[test]
fn full_screen_is_everything_the_display_shows() {
    let room = ADisplaysRoom::of((1440, 960), as_drawn(), alo_canvas::Camera::new(), 100)
        .expect("a room");
    let share = room.full_screen().expect("a display");
    assert_eq!((share.at, share.size), ((0, 0), (1440, 960)));
}

#[test]
fn a_panned_camera_puts_the_room_where_the_person_is_looking() {
    let camera = alo_canvas::Camera::new()
        .looking_at(alo_canvas::At::checked(3000, -500).expect("on the plane"))
        .expect("a camera");
    let room = ADisplaysRoom::of((1440, 960), WhatIsReserved::default(), camera, 100)
        .expect("a room");
    assert_eq!(room.maximised().expect("room").at, (3000, -500 + 54));
    assert_eq!(room.full_screen().expect("room").at, (3000, -500));
}

#[test]
fn a_window_takes_the_room_of_the_display_showing_most_of_it() {
    let left = ADisplaysRoom {
        whole: Rectangle::new((0, 0).into(), (1000, 800).into()),
        work: Rectangle::new((0, 0).into(), (1000, 800).into()),
        band: 54,
    };
    let right = ADisplaysRoom {
        whole: Rectangle::new((1000, 0).into(), (1000, 800).into()),
        work: Rectangle::new((1000, 0).into(), (1000, 800).into()),
        band: 54,
    };
    let mostly_right = Rectangle::new((900, 100).into(), (400, 300).into());
    assert_eq!(the_room_of(&[left, right], mostly_right), Some(right));
    let nowhere = Rectangle::new((-5000, 0).into(), (10, 10).into());
    assert_eq!(the_room_of(&[left, right], nowhere), Some(left));
    let split = Rectangle::new((900, 0).into(), (200, 10).into());
    assert_eq!(
        the_room_of(&[left, right], split),
        Some(left),
        "at equal shares the display drawn first"
    );
    assert_eq!(the_room_of(&[], split), None);
}
