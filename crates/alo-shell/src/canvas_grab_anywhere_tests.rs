//! `⊞` and a drag take hold of the canvas — every case driven through the
//! seat's own entry points (`keyboard_key`, `pointer_motion`,
//! `pointer_button`), never by setting the held state directly.
//!
//! The case a person meets most — a press over an application, which must not
//! reach it — needs a real client, and is in
//! `tests/grabbing_the_canvas_anywhere`.

use crate::Server;
use alo_canvas::{Camera, Zoom};
use alo_displays::{Reported, Socket};
use smithay::backend::input::{ButtonState, KeyState};
use smithay::input::keyboard::XkbConfig;
use smithay::utils::{Physical, Rectangle};
use std::os::unix::fs::PermissionsExt as _;

/// `KEY_LEFTMETA`, the `⊞` key, as a keyboard sends it.
const SUPER: u32 = 125;
/// `BTN_LEFT`.
const LEFT: u32 = 0x110;
/// `BTN_RIGHT`.
const RIGHT: u32 = 0x111;

/// A session with a keyboard and a pointer, in a private directory.
fn a_seat(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        what,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server
        .enable_pointer()
        .expect("a seat that takes a pointer");
    (runtime, server)
}

/// A screen of this size in this socket.
fn a_screen(socket: &str) -> Reported {
    Reported::of(
        Socket::named(socket).expect("a socket"),
        None,
        (1920, 1080),
        Some((597, 336)),
    )
    .expect("a screen")
}

/// Two side-by-side displays as the session names them.
struct TwoDisplays {
    /// The one at the left edge of the desk.
    first: String,
    /// The one beside it.
    second: String,
    /// Where the second begins, in the desk's pixels.
    boundary: i32,
}

/// A seat on two side-by-side displays.
///
/// The names are read back from the arrangement rather than assumed from the
/// sockets, because the camera is keyed by what the arrangement calls a
/// display, and a test that guessed it would be asserting about a key nothing
/// writes.
fn a_seat_on_two_displays() -> (tempfile::TempDir, Server, TwoDisplays) {
    let (directory, mut server) = a_seat("two-displays");
    server.these_screens_are(Some(crate::screens_testing::the_screens(
        vec![a_screen("DP-1"), a_screen("DP-2")],
        &alo_displays::Changes::untouched(),
        &alo_appearance::Appearance::shipped(),
        &crate::screens_testing::a_cold_evening(),
    )));
    let screens = server.the_screens().expect("an arrangement");
    let named = |second: bool| {
        screens
            .each()
            .find(|place| (place.at().across() != 0) == second)
            .map(|place| (place.name().name().to_owned(), place.at().across()))
            .expect("two screens")
    };
    let (first, _) = named(false);
    let (second, boundary) = named(true);
    (
        directory,
        server,
        TwoDisplays {
            first,
            second,
            boundary,
        },
    )
}

/// Press or release one key.
fn key(server: &mut Server, code: u32, state: KeyState) {
    server.keyboard_key(code, state, 5).expect("a real key");
}

/// Move the pointer here, in the screen's pixels.
fn to(server: &mut Server, at: (f64, f64)) {
    server
        .pointer_motion(at.0, at.1, 7)
        .expect("a real position");
}

/// Press or release one button, and say whether a client was told.
fn button(server: &mut Server, code: u32, state: ButtonState) -> bool {
    server
        .pointer_button(code, state, 9)
        .expect("a real button")
}

/// A Dock along the bottom of a 1920 × 1080 display, and a put-aside panel
/// down its right-hand side.
fn the_furniture() -> crate::FixedControlsDrawn {
    crate::FixedControlsDrawn {
        dock_band: Some(Rectangle::<i32, Physical>::new(
            (0, 1020).into(),
            (1920, 60).into(),
        )),
        panel_reserved: Rectangle::new((1800, 0).into(), (120, 1020).into()),
        what_is_leaving: None,
        top_controls: None,
    }
}

/// **Over empty canvas, `⊞` and a drag pan one for one, and only while held.**
///
/// Held to `Camera::panned_by` itself, at three zooms: the expected camera is
/// the one that contract gives for the same screen pixels, so a gesture that
/// panned in plane units would pass at life size and fail the other two.
#[test]
fn a_drag_with_super_held_pans_by_what_the_hand_moved_at_every_zoom() {
    for zoom in [400, 1000, 2500] {
        let (_directory, mut server) = a_seat("pans");
        let zoom = Zoom::of(zoom).expect("a zoom the canvas has");
        assert!(server.zoom_the_canvas(zoom, (0, 0)).is_some());
        to(&mut server, (400.0, 300.0));
        key(&mut server, SUPER, KeyState::Pressed);

        assert!(!button(&mut server, LEFT, ButtonState::Pressed));
        assert!(server.the_canvas_is_held(), "the press did not take hold");
        let before = server.the_camera();
        to(&mut server, (500.0, 330.0));

        assert_eq!(
            server.the_camera(),
            before.panned_by(-100, -30).expect("room to pan"),
            "at {} thousandths a drag of (100, 30) pixels did not pan by them",
            zoom.thousandths()
        );

        assert!(!button(&mut server, LEFT, ButtonState::Released));
        assert!(!server.the_canvas_is_held(), "the release did not let go");
        let after = server.the_camera();
        to(&mut server, (600.0, 330.0));
        assert_eq!(
            server.the_camera(),
            after,
            "the canvas moved after it was let go"
        );
    }
}

/// **Without `⊞`, a press takes nothing**, which is the control on the case
/// above: a gesture that held the canvas on every press would pass it too.
#[test]
fn without_super_a_press_takes_no_hold() {
    let (_directory, mut server) = a_seat("no-super");
    to(&mut server, (400.0, 300.0));
    let before = server.the_camera();
    let _ = button(&mut server, LEFT, ButtonState::Pressed);
    to(&mut server, (500.0, 330.0));
    assert!(!server.the_canvas_is_held());
    assert_eq!(server.the_camera(), before);
}

/// **A press on the Dock or the panel is theirs**, `⊞` or not: they lie over
/// the canvas and do not move with it.
#[test]
fn over_the_dock_or_the_panel_super_and_a_drag_does_nothing() {
    for (on, what) in [
        ((900.0, 1050.0), "the Dock"),
        ((1850.0, 500.0), "the panel"),
    ] {
        let (_directory, mut server) = a_seat("furniture");
        server.the_fixed_controls_were_drawn(
            "",
            the_furniture(),
            alo_appearance::TextScale::ordinary(),
        );
        to(&mut server, on);
        key(&mut server, SUPER, KeyState::Pressed);
        assert_eq!(
            server.the_hand_for_holding_the_canvas(),
            None,
            "the pointer offered to take hold over {what}"
        );
        let before = server.the_camera();

        let _ = button(&mut server, LEFT, ButtonState::Pressed);
        to(&mut server, (on.0 - 100.0, on.1 - 30.0));

        assert!(
            !server.the_canvas_is_held(),
            "a press on {what} took hold of the canvas"
        );
        assert_eq!(
            server.the_camera(),
            before,
            "a drag from {what} panned the canvas"
        );
    }
}

/// **Letting go of `⊞` mid-drag ends it**, and cleanly: the movement after is
/// an ordinary movement, and the button's release — for a press no window was
/// ever told about — reaches nobody.
#[test]
fn letting_go_of_super_mid_drag_ends_the_drag() {
    let (_directory, mut server) = a_seat("let-go");
    to(&mut server, (400.0, 300.0));
    key(&mut server, SUPER, KeyState::Pressed);
    let _ = button(&mut server, LEFT, ButtonState::Pressed);
    to(&mut server, (450.0, 300.0));
    let panned = server.the_camera();
    assert_ne!(panned, Camera::new(), "the control: the drag did not pan");

    key(&mut server, SUPER, KeyState::Released);
    assert!(
        !server.the_canvas_is_held(),
        "letting go of Super left the canvas held"
    );
    to(&mut server, (550.0, 300.0));
    assert_eq!(
        server.the_camera(),
        panned,
        "the canvas kept moving without Super"
    );

    assert!(
        !button(&mut server, LEFT, ButtonState::Released),
        "the release of a press nobody was told about was delivered"
    );
    assert!(!server.the_canvas_is_held());
}

/// **While the canvas is held, every other button is its own too**, so a
/// second finger cannot click an application mid-pan.
#[test]
fn another_button_during_a_hold_reaches_nobody() {
    let (_directory, mut server) = a_seat("second-button");
    to(&mut server, (400.0, 300.0));
    key(&mut server, SUPER, KeyState::Pressed);
    let _ = button(&mut server, LEFT, ButtonState::Pressed);

    assert!(!button(&mut server, RIGHT, ButtonState::Pressed));
    assert!(!button(&mut server, RIGHT, ButtonState::Released));
    assert!(
        server.the_canvas_is_held(),
        "another button let go of the canvas"
    );
}

/// **Nothing carries on after the release, with less motion or without it.**
///
/// The gesture adds no glide and no inertia, so a person who asked for less
/// motion has nothing to switch off — and one who did not gets the same.
#[test]
fn nothing_glides_after_a_release_with_or_without_less_motion() {
    for reduced in [false, true] {
        let (_directory, mut server) = a_seat("no-inertia");
        server.the_motion_is_reduced(reduced);
        to(&mut server, (400.0, 300.0));
        key(&mut server, SUPER, KeyState::Pressed);
        let _ = button(&mut server, LEFT, ButtonState::Pressed);
        to(&mut server, (480.0, 300.0));
        let released_at = server.the_camera();
        let _ = button(&mut server, LEFT, ButtonState::Released);

        assert!(
            server.the_glide.is_none(),
            "with reduced motion {reduced}, a release set the view gliding"
        );
        assert_eq!(server.the_camera(), released_at);
    }
}

/// **The view the press landed on is the one that pans**, to the end of the
/// stroke — even after the pointer crosses onto the other display.
#[test]
fn the_grab_pans_the_display_it_started_on_and_no_other() {
    let (_directory, mut server, displays) = a_seat_on_two_displays();
    let on_the_second = f64::from(displays.boundary) + 400.0;
    to(&mut server, (on_the_second, 300.0));
    key(&mut server, SUPER, KeyState::Pressed);
    let _ = button(&mut server, LEFT, ButtonState::Pressed);

    to(&mut server, (on_the_second + 100.0, 300.0));
    let first = server.surfaces.camera_of(&displays.first);
    let moved = server.surfaces.camera_of(&displays.second);
    assert_eq!(
        moved,
        Camera::new().panned_by(-100, 0).expect("room to pan"),
        "the display the press landed on did not pan"
    );
    assert_eq!(first, Camera::new(), "the other display panned too");

    // Across the boundary onto the first display: still the second's view.
    to(&mut server, (f64::from(displays.boundary) - 100.0, 300.0));
    assert_eq!(
        server.surfaces.camera_of(&displays.first),
        Camera::new(),
        "crossing onto the first display moved its view mid-stroke"
    );
    assert_ne!(
        server.surfaces.camera_of(&displays.second),
        moved,
        "the stroke stopped panning"
    );
}

/// **The pointer shows an open hand while `⊞` is held, a closed one while the
/// canvas is held, and its own shape otherwise.**
#[test]
fn the_pointer_shows_the_hand_for_the_gesture() {
    let (_directory, mut server) = a_seat("the-hand");
    to(&mut server, (400.0, 300.0));
    assert_eq!(server.the_hand_for_holding_the_canvas(), None);

    key(&mut server, SUPER, KeyState::Pressed);
    assert_eq!(
        server.the_hand_for_holding_the_canvas(),
        Some(alo_cursors::CursorIcon::Grab)
    );
    let _ = button(&mut server, LEFT, ButtonState::Pressed);
    assert_eq!(
        server.the_hand_for_holding_the_canvas(),
        Some(alo_cursors::CursorIcon::Grabbing)
    );
    assert!(matches!(
        server.cursor(),
        crate::Cursor::Named {
            shape: alo_cursors::CursorIcon::Grabbing,
            ..
        }
    ));

    let _ = button(&mut server, LEFT, ButtonState::Released);
    key(&mut server, SUPER, KeyState::Released);
    assert_eq!(server.the_hand_for_holding_the_canvas(), None);
}
