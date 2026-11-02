//! Settings as a window on the canvas: the design's size for the display it is
//! on, a room that draws exactly that window back, placed once in the view,
//! counted by *Show all*, out of the way on another Place, and reached by the
//! view travelling to it.

use std::os::unix::fs::PermissionsExt as _;

use alo_appearance::TextScale;
use alo_canvas::Place;
use alo_strings::Direction;
use smithay::utils::{Physical, Rectangle, Size};

use super::how_big_settings_is;
use crate::display_scale::DisplayScale;
use crate::settings_testing::{a_persons_machine, light, noon, settings_words};

/// **The design's two frames, and a dense display**: 1200 × 816 on 1440 × 960,
/// 1152 × 688 on 1280 × 800, and twice the pixels at 200 per cent — the window
/// drawn sharper, not smaller.
#[test]
fn settings_is_the_size_the_design_draws_it_at() {
    let ordinary = TextScale::ordinary();
    let large = how_big_settings_is((1440, 960), ordinary, DisplayScale::ONE).expect("room");
    assert_eq!(large.window, (1200, 816));
    assert_eq!(large.room, (1328, 928));
    assert_eq!(large.margin, (64, 56));
    let small = how_big_settings_is((1280, 800), ordinary, DisplayScale::ONE).expect("room");
    assert_eq!(small.window, (1152, 688));
    let dense =
        how_big_settings_is((2880, 1920), ordinary, DisplayScale::of_per_cent(200)).expect("room");
    assert_eq!(dense.window, (2400, 1632));
    assert_eq!(dense.margin, (128, 112));
    assert!(
        how_big_settings_is((500, 400), ordinary, DisplayScale::ONE).is_none(),
        "a display with no room for Settings was given one"
    );
}

/// **The room draws exactly the window back**, at the margin, at the ordinary
/// size and at a text size and display scale whose pixels do not divide
/// evenly — where a room worked out in pixels alone came out one unit short.
#[test]
fn the_room_draws_exactly_the_window_back() {
    let machine = a_persons_machine("the-room");
    let window = machine.opened();
    for (text, display, size) in [
        (100, 100, (1440, 960)),
        (125, 100, (1920, 1200)),
        (100, 150, (2160, 1440)),
    ] {
        let text = TextScale::percent(text).expect("a text size");
        let display = DisplayScale::of_per_cent(display);
        let sizes = how_big_settings_is(size, text, display).expect("room");
        let mut labels = crate::WindowControlLabels::new().expect("the bundled faces");
        let picture = crate::settings_raster::picture(
            &window,
            &settings_words(),
            &mut labels,
            sizes.room,
            crate::SettingsLook {
                scale: text,
                ..light()
            },
            noon(),
            None,
            display,
        )
        .expect("Settings is drawn in its room");
        assert_eq!(
            picture.panel,
            Some(Rectangle::<i32, Physical>::new(
                sizes.margin.into(),
                sizes.window.into()
            )),
            "at {text:?} on {display:?}"
        );
    }
}

/// A session with Settings open on it.
fn with_settings_open(named: &str) -> (tempfile::TempDir, crate::Server) {
    let directory = tempfile::tempdir().expect("a temporary directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = crate::Server::bind(directory.path(), named).expect("a bound socket");
    let machine = a_persons_machine(named);
    server.the_settings_places_are(machine.places.clone());
    server.settings = machine.opened();
    server.surfaces.popups.output_size = Some(Size::from((1440, 960)));
    (directory, server)
}

/// **Placed once, in the view, the size the rule gives** — and counted by
/// *Show all* under a number of its own.
#[test]
fn it_is_placed_once_and_counted() {
    let (_directory, mut server) = with_settings_open("settings-placed");
    assert!(server.settings_is_placed((1200, 816), (1440, 960), Direction::LeftToRight));
    let placed = server
        .where_settings_is_on_the_plane()
        .expect("placed on the Place being looked at");
    assert_eq!((placed.size.w, placed.size.h), (1200, 816));
    assert!(
        !server.settings_is_placed((1200, 816), (1440, 960), Direction::LeftToRight),
        "it was placed a second time"
    );
    let frame = server.settings_as_a_frame().expect("a frame");
    assert_eq!(frame.id(), server.the_settings_number);
    assert_ne!(frame.id(), server.the_terminals_number);
}

/// **On another Place it is out of the way**, and going to it takes the person
/// to its Place, without moving it.
#[test]
fn on_another_place_it_is_out_of_the_way_until_the_person_goes_to_it() {
    let (_directory, mut server) = with_settings_open("settings-elsewhere");
    let _ = server.settings_is_placed((1200, 816), (1440, 960), Direction::LeftToRight);
    let placed = server.where_settings_is_on_the_plane();
    server.surfaces.place = Place::FIRST.next().expect("a second Place");
    assert!(!server.settings_is_open_here());
    assert!(server.where_settings_is_on_the_plane().is_none());
    assert!(
        !server.settings_is_taking_the_keys(),
        "keys went to Settings on a Place nobody is looking at"
    );
    server.the_person_went_to_settings();
    assert_eq!(server.the_place_now(), Place::FIRST);
    assert_eq!(server.where_settings_is_on_the_plane(), placed);
    assert!(server.settings_is_taking_the_keys());
}

/// **Going to it out of sight brings the view**, and Settings stays put.
#[test]
fn going_to_settings_out_of_sight_brings_the_view_to_it() {
    let (_directory, mut server) = with_settings_open("settings-travelled-to");
    let _ = server.settings_is_placed((1200, 816), (1440, 960), Direction::LeftToRight);
    let placed = server.where_settings_is_on_the_plane().expect("placed");
    assert!(server.pan_the_canvas(6_000, 6_000).is_some());
    assert!(server.the_view_goes_to_settings(), "the view stayed away");
    assert_eq!(server.where_settings_is_on_the_plane(), Some(placed));
    let seen = crate::the_terminal_on_the_plane::on_the_screen(placed, server.the_camera());
    assert!(Rectangle::<i32, Physical>::from_size((1440, 960).into()).contains_rect(seen));
}

/// **Shut, it leaves the canvas**: no place, no frame, nothing counted.
#[test]
fn shut_it_leaves_the_canvas() {
    let (_directory, mut server) = with_settings_open("settings-shut");
    let _ = server.settings_is_placed((1200, 816), (1440, 960), Direction::LeftToRight);
    server.settings.close();
    let _ = server.settings_is_placed((1200, 816), (1440, 960), Direction::LeftToRight);
    assert!(server.where_settings_is_on_the_plane().is_none());
    assert!(server.settings_as_a_frame().is_none());
    assert!(server.settings_on_the_plane.is_none());
}
