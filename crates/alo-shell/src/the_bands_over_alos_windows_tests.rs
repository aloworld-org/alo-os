//! **No band over one of alo's own windows draws a control at rest.**
//!
//! Each test drives both real sides of the seam that broke: the pointer
//! road a person's pointer takes (`Server::pointer_motion`, then the once-a-
//! frame `the_edges_were_told_where_the_pointer_is` the draw calls), and the
//! very function the booted machine draws that window's band with
//! (`the_terminals_edge_picture`, `the_records_edge_picture`,
//! `every_task_windows_edge`) — and counts the controls in the picture.
//!
//! **What would have to be true for these to pass while the product is
//! broken:** the draw loop would have to stop calling
//! `the_edges_were_told_where_the_pointer_is` (then nothing reveals, and these
//! still see none at rest — the *in the band* half of each test catches that
//! one, because it would then see none there either); or a fourth of alo's
//! windows would have to draw its band through a road of its own, which
//! `tests/every_band_is_drawn_through_the_band.rs` refuses; or Settings' band
//! would have to differ from the record's — both are drawn by
//! `crate::an_alo_windows_edge::its_picture`, and the record is the one driven
//! here because Settings needs a whole page raster to be placed.

use std::os::unix::fs::PermissionsExt as _;
use std::time::Duration;

use alo_models::InferenceSource;
use alo_strings::Direction;
use alo_tasks::ATask;
use smithay::input::keyboard::XkbConfig;
use smithay::utils::{Physical, Rectangle, Size};

use crate::Server;
use crate::the_terminal_a_person_opened::A_SENSIBLE_FIRST_GRID;

/// The display the design's own frames are the size of.
const A_DISPLAY: (i32, i32) = (1440, 960);

/// A session with a keyboard and a pointer on the design's display.
fn a_session(named: &str) -> (tempfile::TempDir, Server, cosmic_text::FontSystem) {
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
    server.surfaces.popups.output_size = Some(Size::from(A_DISPLAY));
    let fonts = crate::WindowControlLabels::new()
        .expect("the bundled faces load")
        .fonts;
    (runtime, server, fonts)
}

/// The pointer moved to `(x, y)` on the display, and a frame told every band
/// where it is — what the draw does before it lays a band out.
fn the_pointer_at(server: &mut Server, (x, y): (i32, i32)) {
    server
        .pointer_motion(f64::from(x), f64::from(y), 1)
        .expect("the pointer moved");
    server.the_edges_were_told_where_the_pointer_is();
    // **Past the fade in**, so a band revealed is drawn all the way; the fade
    // out is longer, and one at rest has nothing to fade.
    std::thread::sleep(crate::the_controls_fade::THEY_GO_IN + Duration::from_millis(20));
}

/// The far corner of the display, in no band.
const FAR_AWAY: (i32, i32) = (A_DISPLAY.0 - 2, A_DISPLAY.1 - 2);

/// A point on the band over a window drawn at `window`, at 100 per cent: on
/// its leading half, clear of where any control would be.
fn on_the_band_over(window: Rectangle<i32, Physical>) -> (i32, i32) {
    (
        window.loc.x + 30,
        window.loc.y - crate::window_edge::THE_REGION_IS_TALL / 2,
    )
}

/// **The terminal**: nothing at rest, Minimise and Close in its band, and both
/// at rest once the person keeps the controls always shown.
#[test]
fn the_terminals_band_draws_no_controls_at_rest() {
    let (_runtime, mut server, mut fonts) = a_session("alo-band-terminal-at-rest");
    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    let _ = server.the_terminal_fits_its_window(&mut fonts, A_DISPLAY, 1.0, Direction::LeftToRight);
    let mut vocabulary = alo_strings::Vocabulary::empty();
    alo_terminal::words::declare_into(&mut vocabulary).expect("the terminal's words");
    let words = alo_strings::Strings::of(vocabulary);
    let window = server
        .where_the_terminal_is_seen()
        .expect("the terminal is drawn");
    let mut drawn = |server: &Server| {
        server
            .the_terminals_edge_picture(&mut fonts, &words, A_DISPLAY, 100)
            .expect("the band is on the display")
            .how_many_controls_are_drawn()
    };

    the_pointer_at(&mut server, FAR_AWAY);
    assert_eq!(
        drawn(&server),
        0,
        "the terminal's band drew controls at rest"
    );

    the_pointer_at(&mut server, on_the_band_over(window));
    assert_eq!(drawn(&server), 3, "the control: its band shows three in it");

    the_pointer_at(&mut server, FAR_AWAY);
    assert_eq!(drawn(&server), 0, "they stayed after the pointer left");

    server.the_window_controls_are(alo_appearance::WindowControls::Always);
    the_pointer_at(&mut server, FAR_AWAY);
    assert_eq!(drawn(&server), 3, "Always show window controls was ignored");
}

/// **A press where a hidden control would be reaches no control**: Close is
/// pressable only while it is shown.
#[test]
fn a_hidden_close_on_the_terminal_is_not_pressed() {
    let (_runtime, mut server, mut fonts) = a_session("alo-band-terminal-hidden-close");
    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    let _ = server.the_terminal_fits_its_window(&mut fonts, A_DISPLAY, 1.0, Direction::LeftToRight);
    the_pointer_at(&mut server, FAR_AWAY);
    let close = crate::window_edge::edge_of(
        crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(
            server
                .where_the_terminal_is_seen()
                .expect("the terminal is drawn"),
            100,
        ),
        crate::window_edge::Decorations::AlosOwnWindowThatIsPlaced,
        true,
        Size::from((10_000, 10_000)),
    )
    .controls
    .last()
    .expect("the control: Close where it would be revealed")
    .target;
    // **Pressed without a frame between**, so the band is still at rest.
    assert!(server.a_press_on_the_band_over_alos(
        crate::which_window_is_in_front::AKindOfWindow::TheTerminal,
        (close.loc.x + 22, close.loc.y + 22).into()
    ));
    assert!(
        server.alos_own_terminal().is_some() && server.the_question_closing_asks().is_none(),
        "a press reached a Close nobody could see"
    );
}

/// **The record**: nothing at rest, Minimise and Close in its band.
#[test]
fn the_records_band_draws_no_controls_at_rest() {
    let (_runtime, mut server, mut fonts) = a_session("alo-band-record-at-rest");
    let kept = crate::record_testing::an_afternoon_kept();
    server.the_record_is_kept(Ok(kept.recounting()));
    let _ = server.the_record_is_opened();
    let _ = server.the_record_is_placed((880, 796), A_DISPLAY, Direction::LeftToRight);
    let body = Rectangle::<i32, Physical>::new((280, 100).into(), (880, 752).into());
    server.where_the_records_body_was_drawn = Some(body);
    let words = crate::record_testing::words();
    let mut drawn = |server: &Server| {
        server
            .the_records_edge_picture(&mut fonts, &words, (A_DISPLAY, 100))
            .expect("the band is on the display")
            .how_many_controls_are_drawn()
    };

    the_pointer_at(&mut server, FAR_AWAY);
    assert_eq!(drawn(&server), 0, "the record's band drew controls at rest");

    the_pointer_at(&mut server, on_the_band_over(body));
    assert_eq!(drawn(&server), 2, "the control: its band shows two in it");
}

/// **A task window — the alo chat window**: nothing at rest, Minimise and
/// Close in its band.
#[test]
fn a_task_windows_band_draws_no_controls_at_rest() {
    let (_runtime, mut server, mut fonts) = a_session("alo-band-task-at-rest");
    let which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "tidy my invoices",
        InferenceSource::ThisMachine,
    ));
    server.tasks_mut().put_whatever_has_nowhere_to_be(
        Rectangle::new((0, 0).into(), A_DISPLAY.into()),
        Direction::LeftToRight,
        &[],
        100,
    );
    let window = server
        .the_task_windows_on_the_screen()
        .into_iter()
        .find(|(one, _)| *one == which)
        .map(|(_, window)| window)
        .expect("the task window is on the screen");
    let mut drawn = |server: &Server| {
        let pictures = crate::a_task_windows_edge::every_task_windows_edge(
            server,
            &mut fonts,
            &crate::approval_testing::words(),
            A_DISPLAY,
            100,
        );
        assert_eq!(pictures.len(), 1, "the control: one band drawn");
        pictures
            .iter()
            .map(crate::window_edge_picture::EdgePicture::how_many_controls_are_drawn)
            .sum::<usize>()
    };

    the_pointer_at(&mut server, FAR_AWAY);
    assert_eq!(
        drawn(&server),
        0,
        "a task window's band drew controls at rest"
    );

    the_pointer_at(&mut server, on_the_band_over(window));
    assert_eq!(drawn(&server), 3, "the control: its band shows three in it");
}
