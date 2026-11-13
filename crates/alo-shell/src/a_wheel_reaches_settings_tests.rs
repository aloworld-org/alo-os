//! **A wheel over Settings' side column, down the pointer road a person's
//! wheel takes**: where the column runs over it moves the column and never the
//! canvas, even at its end; where the column fits — the design's 1440 × 960 —
//! the canvas pans as it did before the column could scroll.

use std::os::unix::fs::PermissionsExt as _;

use alo_appearance::TextScale;
use alo_strings::Direction;
use smithay::backend::input::{Axis, AxisSource};
use smithay::input::keyboard::XkbConfig;
use smithay::input::pointer::AxisFrame;
use smithay::utils::{Physical, Rectangle, Size};

use crate::Server;
use crate::display_scale::DisplayScale;
use crate::settings_categories::SettingsCategory;
use crate::settings_raster::SettingsLook;
use crate::settings_testing::{Machine, a_persons_machine, light, settings_words};

/// A wheel turned this far, positive downwards.
fn a_scroll_of(down: f64) -> AxisFrame {
    AxisFrame::new(0)
        .source(AxisSource::Wheel)
        .value(Axis::Vertical, down)
}

/// A session with a keyboard and a pointer, and Settings open on a display
/// of `size`.
fn with_settings_open(named: &str, size: (i32, i32)) -> (tempfile::TempDir, Server, Machine) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        named,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.enable_pointer().expect("a pointer on this seat");
    server.surfaces.popups.output_size = Some(Size::from(size));
    let machine = a_persons_machine(named);
    server.the_settings_places_are(machine.places.clone());
    server.settings = machine.opened();
    server.the_person_went_to_settings();
    (runtime, server, machine)
}

/// What a frame does with Settings on a display of `size`, for a person whose
/// text is `percent` large: place it, lay it out in its room, and keep where it
/// drew everything — the steps `crate::direct_desktop` takes.
fn a_frame(server: &mut Server, size: (i32, i32), percent: u16) {
    let text = TextScale::percent(percent).expect("a text size");
    let sizes = crate::settings_on_the_plane::how_big_settings_is(size, text, DisplayScale::ONE)
        .expect("room");
    let _ = server.settings_is_placed(sizes.window, size, Direction::LeftToRight);
    let room = server
        .where_the_settings_room_is_drawn("", sizes)
        .expect("placed");
    let mut labels = crate::WindowControlLabels::new().expect("the bundled faces");
    let picture = crate::settings_raster::picture(
        server.the_settings_window(),
        &settings_words(),
        &mut labels,
        sizes.room,
        SettingsLook {
            scale: text,
            ..light()
        },
        std::time::SystemTime::now(),
        None,
        DisplayScale::ONE,
    )
    .expect("Settings is drawn");
    let drawn = crate::settings_as_drawn::SettingsAsDrawn::of(&picture, room);
    server.settings_was_drawn(drawn);
}

/// Where the first category is seen on the display.
fn the_first_category(server: &Server) -> Rectangle<i32, Physical> {
    server
        .settings_as_drawn
        .as_ref()
        .and_then(|drawn| drawn.where_the_category_is(SettingsCategory::AloAndAi))
        .expect("the first category is drawn")
}

/// How far the side column was last drawn scrolled.
fn scrolled(server: &Server) -> i32 {
    server
        .settings_as_drawn
        .as_ref()
        .and_then(crate::settings_as_drawn::SettingsAsDrawn::the_side_column)
        .expect("the side column is drawn")
        .scrolled
}

/// Move the pointer to the middle of `of`, then turn the wheel `down`;
/// answers whether anything moved.
fn a_wheel_over(server: &mut Server, of: Rectangle<i32, Physical>, down: f64) -> bool {
    server
        .pointer_motion(
            f64::from(of.loc.x + of.size.w / 2),
            f64::from(of.loc.y + of.size.h / 2),
            1,
        )
        .expect("the pointer moved");
    server
        .pointer_axis(a_scroll_of(down))
        .expect("the wheel was handled")
}

/// **Where the column runs over, the wheel moves it and never pans the
/// canvas** — and at its end it is still the column's, and moves nothing.
#[test]
fn a_wheel_over_a_column_that_runs_over_moves_it_and_not_the_canvas() {
    let size = (1280, 800);
    let (_runtime, mut server, _machine) = with_settings_open("wheel-runs-over", size);
    a_frame(&mut server, size, 150);
    assert_eq!(scrolled(&server), 0);
    let camera = server.the_camera();
    let over = the_first_category(&server);
    assert!(a_wheel_over(&mut server, over, 15.0));
    a_frame(&mut server, size, 150);
    assert!(scrolled(&server) > 0, "the wheel did not move the column");
    assert_eq!(server.the_camera(), camera, "the wheel panned the canvas");
    for _ in 0..40 {
        let _ = a_wheel_over(&mut server, over, 15.0);
    }
    a_frame(&mut server, size, 150);
    let at_the_end = scrolled(&server);
    assert!(!a_wheel_over(&mut server, over, 15.0));
    a_frame(&mut server, size, 150);
    assert_eq!(scrolled(&server), at_the_end);
    assert_eq!(server.the_camera(), camera, "the wheel panned the canvas");
}

/// **Where the column fits, the wheel goes where it went before**: the canvas
/// pans, and the column never moves.
#[test]
fn a_wheel_over_a_column_that_fits_pans_the_canvas_as_before() {
    let size = (1440, 960);
    let (_runtime, mut server, _machine) = with_settings_open("wheel-fits", size);
    a_frame(&mut server, size, 100);
    let camera = server.the_camera();
    let over = the_first_category(&server);
    let _ = a_wheel_over(&mut server, over, 15.0);
    a_frame(&mut server, size, 100);
    assert_eq!(scrolled(&server), 0);
    assert_ne!(server.the_camera(), camera, "the canvas did not pan");
}
