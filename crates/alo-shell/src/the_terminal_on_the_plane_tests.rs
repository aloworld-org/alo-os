//! That alo's terminal is a window on the canvas: placed once in the view, moved
//! and scaled by the camera like an application's window, counted by *Show all*,
//! and out of the way on another Place — and that its picture is one raster.

use alo_canvas::{At, Camera, Place, Zoom};
use alo_strings::Direction;
use smithay::utils::{Logical, Physical, Rectangle, Size};

use super::{flattened, on_the_screen};
use crate::painted::{Inked, Solid, Washed};
use crate::the_terminal_a_person_opened::A_SENSIBLE_FIRST_GRID;
use crate::the_terminals_cells_are_drawn::TerminalPicture;

/// The display the design's own frames are the size of.
const A_DISPLAY: (i32, i32) = (1440, 1040);

fn a_window() -> Rectangle<i32, Logical> {
    Rectangle::new((100, 50).into(), (400, 300).into())
}

/// A server with alo's terminal open on it, and nothing drawn yet.
fn with_a_terminal(named: &str) -> (tempfile::TempDir, crate::Server) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().expect("a temporary directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = crate::Server::bind(directory.path(), named).expect("a bound socket");
    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    (directory, server)
}

/// **At life size and looking at the origin, the plane is the screen**, and a
/// pan and a zoom move and scale the window by exactly the camera's arithmetic.
#[test]
fn the_camera_moves_and_scales_the_window() {
    let still = on_the_screen(a_window(), Camera::new());
    assert_eq!(
        still,
        Rectangle::<i32, Physical>::new((100, 50).into(), (400, 300).into())
    );

    let panned = Camera::new()
        .looking_at(At::checked(50, 0).expect("on the plane"))
        .expect("a camera");
    let halved = panned
        .zoomed_to(Zoom::of(500).expect("a zoom"), (0, 0))
        .expect("a camera");
    let seen = on_the_screen(a_window(), halved);
    assert_eq!(
        (seen.loc.x, seen.loc.y, seen.size.w, seen.size.h),
        (25, 25, 200, 150),
        "at half zoom the window should be half the size, half as far from the corner"
    );
}

/// A picture of this size holding these parts.
fn a_picture(size: (i32, i32), solids: Vec<Solid>, inked: Vec<Inked>) -> TerminalPicture {
    TerminalPicture {
        size,
        panel: Some(Washed {
            area: Rectangle::new((0, 0).into(), size.into()),
            pixels: vec![9; usize::try_from(size.0 * size.1 * 4).expect("a size")],
        }),
        solids,
        inked,
        strip: Vec::new(),
    }
}

/// **The parts are flattened in the order they are painted**: the body, then
/// the shapes over it, then the text over those.
#[test]
fn the_parts_are_flattened_in_painting_order() {
    let picture = a_picture(
        (4, 2),
        vec![Solid {
            area: Rectangle::new((1, 0).into(), (2, 2).into()),
            colour: [1, 2, 3],
        }],
        vec![Inked {
            area: Rectangle::new((2, 1).into(), (2, 1).into()),
            pixels: vec![[7, 7, 7], [8, 8, 8]],
        }],
    );
    let raster = flattened(&picture).expect("a picture inside its own room");
    let pixel = |x: usize, y: usize| -> [u8; 4] {
        let at = (y * 4 + x) * 4;
        let mut out = [0; 4];
        out.copy_from_slice(raster.pixels.get(at..at + 4).expect("a pixel"));
        out
    };
    assert_eq!(pixel(0, 0), [9, 9, 9, 9], "the body is under everything");
    assert_eq!(pixel(1, 0), [1, 2, 3, 255], "a shape covers the body");
    assert_eq!(pixel(2, 1), [7, 7, 7, 255], "text covers a shape");
    assert_eq!(pixel(3, 1), [8, 8, 8, 255]);
    assert_eq!((raster.area.size.w, raster.area.size.h), (4, 2));
}

/// **A part outside the picture's own room is refused**, not cut off: it is a
/// layout fault, and a raster that quietly dropped it would hide one.
#[test]
fn a_part_outside_the_picture_is_refused() {
    let picture = a_picture(
        (4, 2),
        vec![Solid {
            area: Rectangle::new((3, 0).into(), (2, 1).into()),
            colour: [1, 2, 3],
        }],
        Vec::new(),
    );
    assert!(flattened(&picture).is_none());
}

/// **A terminal is placed once, in the view, at the share of the display the
/// design draws it at** — and the second frame leaves it where it is.
#[test]
fn a_terminal_is_placed_once_in_the_view() {
    let (_directory, mut server) = with_a_terminal("alo-terminal-placed");
    assert!(server.the_terminal_is_placed(A_DISPLAY, 1.0, Direction::LeftToRight));
    let placed = server
        .where_the_terminal_is_on_the_plane()
        .expect("it has a place on the canvas");
    assert_eq!((placed.size.w, placed.size.h), (1040, 800));
    let view = Rectangle::<i32, Logical>::new((0, 0).into(), A_DISPLAY.into());
    assert!(
        view.contains_rect(placed),
        "it opened outside the view: {placed:?}"
    );

    assert!(
        !server.the_terminal_is_placed(A_DISPLAY, 1.0, Direction::LeftToRight),
        "a terminal that has a place was placed again"
    );
    assert_eq!(server.where_the_terminal_is_on_the_plane(), Some(placed));
}

/// **With no terminal open there is nothing to place**, and nothing on the
/// plane — the refusal half of the test above.
#[test]
fn with_no_terminal_nothing_is_placed() {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().expect("a temporary directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server =
        crate::Server::bind(directory.path(), "alo-no-terminal-placed").expect("a socket");
    assert!(!server.the_terminal_is_placed(A_DISPLAY, 1.0, Direction::LeftToRight));
    assert_eq!(server.where_the_terminal_is_on_the_plane(), None);
    assert_eq!(server.the_terminal_as_a_frame(), None);
}

/// **Zooming the canvas out shrinks the terminal with it, and its grid stays
/// the same** — the fault the booted machine showed, where the terminal stayed
/// full size over a zoomed-out canvas.
#[test]
fn zooming_out_shrinks_the_terminal_and_keeps_its_grid() {
    let (_directory, mut server) = with_a_terminal("alo-terminal-zoomed");
    let mut fonts = crate::WindowControlLabels::new()
        .expect("the bundled faces load")
        .fonts;
    let _ = server.the_terminal_fits_its_window(&mut fonts, A_DISPLAY, 1.0, Direction::LeftToRight);
    let at_life_size = server.where_the_terminal_is.expect("it is drawn");
    let grid = server
        .alos_own_terminal()
        .expect("it is open")
        .screen()
        .size()
        .columns();

    assert!(
        server
            .zoom_the_canvas(Zoom::of(500).expect("a zoom"), (0, 0))
            .is_some()
    );
    assert!(
        !server.the_terminal_fits_its_window(&mut fonts, A_DISPLAY, 1.0, Direction::LeftToRight),
        "zooming the canvas resized the shell's grid"
    );
    let zoomed = server.where_the_terminal_is.expect("it is still drawn");
    assert_eq!(
        (zoomed.size.w, zoomed.size.h),
        (at_life_size.size.w / 2, at_life_size.size.h / 2),
        "at half zoom the terminal should be drawn half the size"
    );
    assert_eq!(
        server
            .alos_own_terminal()
            .expect("it is open")
            .screen()
            .size()
            .columns(),
        grid
    );
}

/// **Show all counts the terminal**, under the number it was given.
#[test]
fn show_all_counts_the_terminal() {
    let (_directory, mut server) = with_a_terminal("alo-terminal-counted");
    assert!(server.the_terminal_is_placed(A_DISPLAY, 1.0, Direction::LeftToRight));
    let frame = server.the_terminal_as_a_frame().expect("a frame");
    assert_eq!(frame.id(), server.the_terminals_number);
    assert_eq!(
        frame.size(),
        alo_canvas::Size::checked(1040, 800).expect("a size")
    );
}

/// **On another Place it is out of the way**: not on the plane being looked at,
/// not drawn, not pressed and not taking keys — and going to it takes the
/// person to its Place.
#[test]
fn on_another_place_it_is_out_of_the_way_until_the_person_goes_to_it() {
    let (_directory, mut server) = with_a_terminal("alo-terminal-elsewhere");
    let mut fonts = crate::WindowControlLabels::new()
        .expect("the bundled faces load")
        .fonts;
    let _ = server.the_terminal_fits_its_window(&mut fonts, A_DISPLAY, 1.0, Direction::LeftToRight);
    server.the_person_went_to(crate::which_window_is_in_front::AKindOfWindow::TheTerminal);
    assert!(server.the_terminal_is_taking_the_keys());

    let elsewhere = Place::FIRST.next().expect("a second Place");
    server.surfaces.place = elsewhere;
    let _ = server.the_terminal_fits_its_window(&mut fonts, A_DISPLAY, 1.0, Direction::LeftToRight);
    assert_eq!(server.where_the_terminal_is_on_the_plane(), None);
    assert_eq!(server.where_the_terminal_is, None, "a press could reach it");
    assert!(
        !server.the_terminal_is_taking_the_keys(),
        "keys went to a terminal on a Place nobody is looking at"
    );

    assert!(server.the_view_goes_to_the_terminal());
    assert_eq!(
        server.the_place_now(),
        Place::FIRST,
        "the view did not go to its Place"
    );
    assert!(server.the_terminal_is_taking_the_keys());
}

/// **Going to a terminal out of sight brings the view to it**, without moving
/// the terminal.
#[test]
fn going_to_a_terminal_out_of_sight_brings_the_view_to_it() {
    let (_directory, mut server) = with_a_terminal("alo-terminal-travelled-to");
    server.surfaces.popups.output_size = Some(Size::from(A_DISPLAY));
    assert!(server.the_terminal_is_placed(A_DISPLAY, 1.0, Direction::LeftToRight));
    let placed = server.where_the_terminal_is_on_the_plane().expect("placed");

    assert!(server.pan_the_canvas(5_000, 5_000).is_some());
    assert!(
        server.the_view_goes_to_the_terminal(),
        "the view stayed away"
    );
    assert_eq!(
        server.where_the_terminal_is_on_the_plane(),
        Some(placed),
        "the terminal was moved to the view instead"
    );
    let seen = on_the_screen(placed, server.the_camera());
    assert!(
        Rectangle::<i32, Physical>::from_size(A_DISPLAY.into()).contains_rect(seen),
        "the terminal is still out of sight at {seen:?}"
    );

    assert!(
        !server.the_view_goes_to_the_terminal(),
        "a terminal already in sight moved the view"
    );
}
