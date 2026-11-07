//! The record as a window on the canvas: the approved size for the display it
//! is on, laid out unchanged in a room of its own, placed once, counted by
//! *Show all*, out of the way on another Place, pressed where the camera drew
//! it, and taking the keys only while it is in front.

use std::os::unix::fs::PermissionsExt as _;

use alo_canvas::{Place, Zoom};
use alo_strings::Direction;
use smithay::backend::input::KeyState;
use smithay::input::keyboard::XkbConfig;
use smithay::utils::{Physical, Point, Rectangle, Size};

use super::how_big_the_record_is;
use crate::a_press_on_the_record::TheRecordsPlaces;
use crate::display_scale::DisplayScale;
use crate::record_frame_drawn::TheStrip;
use crate::record_raster::{TheRecordsRoom, picture_with};
use crate::record_testing::{a_clock, an_afternoon_kept, light, words};
use crate::which_window_is_in_front::AKindOfWindow;
use crate::{RecordWindow, Server};

/// **Both approved frames come out of the rule**, as they did fixed to the
/// display: 880 × 796 on 1440 × 960 and 880 × 652 on 1280 × 800.
#[test]
fn the_record_is_the_size_the_owner_approved() {
    assert_eq!(
        how_big_the_record_is((1440, 960), light(), DisplayScale::ONE),
        Some((880, 796))
    );
    assert_eq!(
        how_big_the_record_is((1280, 800), light(), DisplayScale::ONE),
        Some((880, 652))
    );
    assert_eq!(
        how_big_the_record_is((2880, 1920), light(), DisplayScale::of_per_cent(200)),
        Some((1760, 1592)),
        "twice the pixels draws the same window sharper"
    );
}

/// **In a room of its own, #1071's layout is unchanged**: the window is the
/// size the rule gives, at the shadow's reach from the room's corner, its body
/// the band below its top — and the strip is the canvas edge's, so none of the
/// window's own Close is drawn.
#[test]
fn in_its_own_room_the_layout_is_unchanged() {
    let kept = an_afternoon_kept();
    let mut window = RecordWindow::on_an_output();
    window.opened_by_hand(&kept.recounting());
    let mut labels = crate::WindowControlLabels::new().expect("the bundled faces");
    let picture = picture_with(
        window.shows(),
        &words(),
        &mut labels,
        (
            TheRecordsRoom::ItsOwn((880, 796)),
            light(),
            DisplayScale::ONE,
        ),
        a_clock(),
        TheStrip::TheCanvasEdge { focused: true },
    )
    .expect("the record is drawn in its room");
    assert_eq!(picture.size, (976, 892));
    assert_eq!(
        picture.panel,
        Some(Rectangle::new((48, 48).into(), (880, 796).into()))
    );
    assert_eq!(
        picture.body,
        Some(Rectangle::new((48, 92).into(), (880, 752).into()))
    );
}

/// A session with a keyboard, a pointer and the record open on it.
fn with_the_record_open(named: &str) -> (tempfile::TempDir, Server, crate::record_testing::Kept) {
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
    server.surfaces.popups.output_size = Some(Size::from((1440, 960)));
    let kept = an_afternoon_kept();
    server.the_record_is_kept(Ok(kept.recounting()));
    let _ = server.the_record_is_opened();
    (runtime, server, kept)
}

/// **Placed once, in the view, and counted** under a number of its own.
#[test]
fn it_is_placed_once_and_counted() {
    let (_runtime, mut server, _kept) = with_the_record_open("record-placed");
    assert!(server.the_record_is_placed((880, 796), (1440, 960), Direction::LeftToRight));
    let placed = server.where_the_record_is_on_the_plane().expect("placed");
    assert_eq!((placed.size.w, placed.size.h), (880, 796));
    assert!(!server.the_record_is_placed((880, 796), (1440, 960), Direction::LeftToRight));
    let frame = server.the_record_as_a_frame().expect("a frame");
    assert_eq!(frame.id(), server.the_records_number);
    assert!(
        server.the_record_is_taking_the_keys(),
        "opening it did not go to it"
    );
}

/// **On another Place it is out of the way** — not open here, no keys — until
/// the person goes to it, which takes them to its Place without moving it.
#[test]
fn on_another_place_it_is_out_of_the_way_until_the_person_goes_to_it() {
    let (_runtime, mut server, _kept) = with_the_record_open("record-elsewhere");
    let _ = server.the_record_is_placed((880, 796), (1440, 960), Direction::LeftToRight);
    let placed = server.where_the_record_is_on_the_plane();
    server.surfaces.place = Place::FIRST.next().expect("a second Place");
    assert!(!server.the_record_is_open_here());
    assert!(!server.the_record_is_taking_the_keys());
    server.the_person_went_to_the_record();
    assert_eq!(server.the_place_now(), Place::FIRST);
    assert_eq!(server.where_the_record_is_on_the_plane(), placed);
    assert!(server.the_record_is_taking_the_keys());
}

/// **A press finds Put back where the camera drew it**, at half zoom — and
/// asks for its plan, as Enter on it does — and not where it would be at life
/// size.
#[test]
fn a_press_finds_put_back_where_the_camera_drew_it() {
    let (_runtime, mut server, _kept) = with_the_record_open("record-press-zoomed");
    let laid_out = TheRecordsPlaces {
        window: Rectangle::new((48, 48).into(), (880, 796).into()),
        close: Rectangle::new((880, 48).into(), (44, 44).into()),
        buttons: vec![(3, Rectangle::new((700, 300).into(), (128, 40).into()))],
        put_back: None,
        cancel: None,
    };
    let room_on_screen = Rectangle::<i32, Physical>::new((100, 60).into(), (488, 446).into());
    server.the_record_was_drawn(Some(laid_out.as_seen((976, 892), room_on_screen)));
    // Put back's middle, (764, 320), halved and moved by (100, 60).
    assert!(
        !server.a_press_on_the_record(Point::from((764, 320))),
        "a press found the window where it would be at life size"
    );
    assert_eq!(server.take_the_plan_asked(), None);
    assert!(server.a_press_on_the_record(Point::from((482, 220))));
    assert_eq!(server.take_the_plan_asked(), Some(3));
}

/// **Behind another window it takes no key**, and Escape reaches whatever
/// is in front instead; brought back, Escape closes it.
#[test]
fn behind_another_window_it_takes_no_key() {
    let (_runtime, mut server, _kept) = with_the_record_open("record-behind");
    server.the_person_went_to(AKindOfWindow::Applications);
    assert!(!server.the_record_is_taking_the_keys());
    server.the_person_went_to_the_record();
    assert!(server.the_record_is_taking_the_keys());
    for state in [KeyState::Pressed, KeyState::Released] {
        crate::a_key_reaches_the_record::the_key_the_record_took(
            &mut server,
            crate::DirectKeyEvent {
                code: 1,
                state,
                time: 1,
            },
        )
        .expect("a key on a seat with a keyboard");
    }
    assert!(!server.record.is_open(), "Escape left it open");
}

/// **Shut, it leaves the canvas.**
#[test]
fn shut_it_leaves_the_canvas() {
    let (_runtime, mut server, _kept) = with_the_record_open("record-shut");
    let _ = server.the_record_is_placed((880, 796), (1440, 960), Direction::LeftToRight);
    assert!(
        server
            .zoom_the_canvas(Zoom::of(500).expect("a zoom"), (0, 0))
            .is_some()
    );
    server.record.closed();
    let _ = server.the_record_is_placed((880, 796), (1440, 960), Direction::LeftToRight);
    assert!(server.where_the_record_is_on_the_plane().is_none());
    assert!(server.the_record_as_a_frame().is_none());
}
