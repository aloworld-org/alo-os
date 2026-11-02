//! alo's terminal's edge: its title, icon and Close, laid out on the terminal
//! where the camera draws it, and pressed through the pointer road a person's
//! press takes.

use std::os::unix::fs::PermissionsExt as _;
use std::time::Duration;

use alo_canvas::Zoom;
use alo_strings::Direction;
use smithay::backend::input::ButtonState;
use smithay::utils::{Logical, Size};

use crate::Server;
use crate::the_terminal_a_person_opened::A_SENSIBLE_FIRST_GRID;
use crate::which_window_is_in_front::AKindOfWindow;
use crate::window_edge::{OnTheEdge, WindowEdge};

/// The display the design's own frames are the size of.
const A_DISPLAY: (i32, i32) = (1440, 1040);

/// A room no edge is clamped by.
fn anywhere() -> Size<i32, Logical> {
    Size::from((10_000, 10_000))
}

/// A session with a keyboard and a pointer, alo's terminal open and laid out
/// on the design's display.
fn a_terminal(named: &str) -> (tempfile::TempDir, Server, cosmic_text::FontSystem) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        named,
        smithay::input::keyboard::XkbConfig {
            layout: "us",
            ..smithay::input::keyboard::XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.enable_pointer().expect("a pointer on this seat");
    server.surfaces.popups.output_size = Some(Size::from(A_DISPLAY));
    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    let mut fonts = crate::WindowControlLabels::new()
        .expect("the bundled faces load")
        .fonts;
    let _ = server.the_terminal_fits_its_window(&mut fonts, A_DISPLAY, 1.0, Direction::LeftToRight);
    (runtime, server, fonts)
}

/// The terminal's edge as it is laid out now, at 100 per cent.
fn its_edge(server: &Server) -> WindowEdge {
    server
        .the_terminals_edge(anywhere(), 100)
        .expect("a terminal drawn has an edge")
}

/// Press the primary button at the middle of `on` on the terminal's edge.
fn press(server: &mut Server, on: OnTheEdge) {
    let target = its_edge(server)
        .controls
        .iter()
        .find(|control| control.does == on)
        .expect("the control is on the edge")
        .target;
    press_at(
        server,
        target.loc.x + target.size.w / 2,
        target.loc.y + target.size.h / 2,
    );
}

/// Press the primary button at this point of the display.
fn press_at(server: &mut Server, x: i32, y: i32) {
    server
        .pointer_motion(f64::from(x), f64::from(y), 1)
        .expect("the pointer moved");
    let _ = server
        .pointer_button(0x110, ButtonState::Pressed, 2)
        .expect("the press was handled");
    let _ = server
        .pointer_button(0x110, ButtonState::Released, 3)
        .expect("the release was handled");
}

/// **The design's edge, with Close alone**: the title and the icon in their
/// places, and of its controls only Close — minimise and maximise wait until
/// something can do either to the terminal.
#[test]
fn the_edge_carries_the_title_the_icon_and_close_alone() {
    let (_runtime, server, _fonts) = a_terminal("alo-terminal-edge-parts");
    let edge = its_edge(&server);
    let controls: Vec<OnTheEdge> = edge.controls.iter().map(|control| control.does).collect();
    assert_eq!(controls, vec![OnTheEdge::Close]);
    assert!(edge.title.is_some(), "the title is not on the edge");
    assert!(edge.icon.is_some(), "the icon is not on the edge");
    let drawn = server.where_the_terminal_is.expect("the terminal is drawn");
    assert_eq!(
        edge.region.loc.y + edge.region.size.h,
        drawn.loc.y,
        "the edge does not sit on the terminal"
    );
    assert_eq!(edge.region.loc.x, drawn.loc.x);
    assert_eq!(edge.region.size.w, drawn.size.w);
}

/// **The edge follows the camera**: zoomed out to half and panned, it is laid
/// out on the terminal where it is drawn now, and not where it was.
#[test]
fn the_edge_follows_the_terminal_when_the_canvas_pans_and_zooms() {
    let (_runtime, mut server, mut fonts) = a_terminal("alo-terminal-edge-camera");
    let before = its_edge(&server).region;
    assert!(
        server
            .zoom_the_canvas(Zoom::of(500).expect("a zoom"), (0, 0))
            .is_some()
    );
    assert!(server.pan_the_canvas(-120, -60).is_some());
    let _ = server.the_terminal_fits_its_window(&mut fonts, A_DISPLAY, 1.0, Direction::LeftToRight);
    let drawn = server.where_the_terminal_is.expect("still drawn");
    let after = its_edge(&server).region;
    assert_ne!(after, before, "the edge stayed where the terminal was");
    assert_eq!(after.loc.y + after.size.h, drawn.loc.y);
    assert_eq!((after.loc.x, after.size.w), (drawn.loc.x, drawn.size.w));
    // **Its own furniture is not zoomed**: the band is the design's 44 tall at
    // any zoom, as an application's edge is.
    assert_eq!(after.size.h, before.size.h);
}

/// **A press on the edge is the terminal's**: the kind under the pointer
/// there is the terminal, and a press on its title brings it in front with
/// the keys.
#[test]
fn a_press_on_the_edge_brings_the_terminal_in_front() {
    let (_runtime, mut server, _fonts) = a_terminal("alo-terminal-edge-front");
    server.the_person_went_to(AKindOfWindow::Applications);
    assert!(!server.the_terminal_is_taking_the_keys());
    let title = its_edge(&server).title.expect("a title");
    assert_eq!(
        server.the_kind_under((f64::from(title.x + 8), f64::from(title.y + 6)).into()),
        Some(AKindOfWindow::TheTerminal)
    );
    press_at(&mut server, title.x + 8, title.y + 6);
    assert!(
        server.alos_own_terminal().is_some(),
        "a press on the title closed it"
    );
    assert!(server.the_terminal_is_taking_the_keys());
}

/// **Close on an idle terminal ends it**, through the pointer road.
#[test]
fn close_on_an_idle_terminal_ends_it() {
    let (_runtime, mut server, _fonts) = a_terminal("alo-terminal-edge-close");
    let mut idle = false;
    for _ in 0..400 {
        let _ = server.the_terminal_was_written_to();
        if server.alos_own_terminal().is_some_and(|terminal| {
            terminal.is_a_command_running() == alo_terminal::IsACommandRunning::No
        }) {
            idle = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(idle, "the shell never reached its prompt");
    press(&mut server, OnTheEdge::Close);
    assert!(server.alos_own_terminal().is_none(), "Close left it open");
    assert!(server.the_question_closing_asks().is_none());
}

/// **The picture is drawn**, at 100 and at 200 per cent, with the bundled faces
/// and the terminal's own words.
#[test]
fn the_edge_is_drawn_at_both_scales() {
    let (_runtime, server, mut fonts) = a_terminal("alo-terminal-edge-drawn");
    let mut vocabulary = alo_strings::Vocabulary::empty();
    alo_terminal::words::declare_into(&mut vocabulary).expect("the terminal's words");
    let words = alo_strings::Strings::of(vocabulary);
    assert!(
        server
            .the_terminals_edge_picture(&mut fonts, &words, A_DISPLAY, 100)
            .is_some()
    );
    assert!(
        server
            .the_terminals_edge_picture(&mut fonts, &words, (2880, 2080), 200)
            .is_some()
    );
}
