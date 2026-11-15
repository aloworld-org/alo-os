//! **Window actions on the terminal and on a task window**: the panel opened by
//! a real press on the band and by Alt+F3, its choices carried out on the
//! window's own rectangle, and the refusals.
//!
//! Each placement is read back from the rectangle the window lays itself out
//! in — `Server::the_terminal_on_the_plane`, the task's in `TheTasks` — and
//! compared with the room every application's window is maximised into
//! (`crate::the_room_a_window_fills`), so the terminal and an application
//! cannot be given two different answers.
//!
//! **What would have to be true for these to pass while the product is
//! broken:** the draw would have to lay the terminal out somewhere other than
//! its rectangle on the plane — `the_terminal_fits_its_window` reads it every
//! frame, which `the_terminals_edge_tests` holds — or a booted machine's room
//! would have to differ from the one noted here. The KVM walk's *the terminal
//! maximised* step is the check of the second.

use std::os::unix::fs::PermissionsExt as _;

use alo_models::InferenceSource;
use alo_put_aside::AlosOwnWindow;
use alo_shortcuts::{Action, Shortcuts};
use alo_strings::Direction;
use alo_tasks::ATask;
use smithay::backend::input::ButtonState;
use smithay::utils::{Logical, Rectangle, Size};

use crate::Server;
use crate::the_bands_over_alos_windows::AlosBand;
use crate::the_terminal_a_person_opened::A_SENSIBLE_FIRST_GRID;
use crate::which_band::ABand;
use crate::which_window_is_in_front::AKindOfWindow;
use crate::window_edge::{Decorations, OnTheEdge, edge_of};
use crate::window_edge_keys::AnEdgeKey;
use crate::window_menu::WhatARowDoes;

/// The display the design's own frames are the size of.
const A_DISPLAY: (i32, i32) = (1440, 960);

/// The terminal's band.
const THE_TERMINAL: AlosBand = AlosBand::Its(AlosOwnWindow::TheTerminal);

/// A session on the design's display with its room noted, as a frame notes
/// it.
fn a_session(named: &str) -> (tempfile::TempDir, Server, cosmic_text::FontSystem) {
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
    server.this_displays_room_was_seen("", A_DISPLAY);
    let fonts = crate::WindowControlLabels::new()
        .expect("the bundled faces load")
        .fonts;
    (runtime, server, fonts)
}

/// The same, with alo's terminal open, laid out, and in front with the keys.
fn a_terminal(named: &str) -> (tempfile::TempDir, Server, cosmic_text::FontSystem) {
    let (runtime, mut server, mut fonts) = a_session(named);
    assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
    let _ = server.the_terminal_fits_its_window(&mut fonts, A_DISPLAY, 1.0, Direction::LeftToRight);
    server.the_person_went_to(AKindOfWindow::TheTerminal);
    (runtime, server, fonts)
}

/// The room every window on this display is maximised into, as
/// `crate::the_room_a_window_fills` gives it.
fn the_maximised_area(server: &Server) -> Rectangle<i32, Logical> {
    let area = server
        .surfaces
        .rooms
        .first()
        .and_then(|(_, room, _)| room.maximised())
        .expect("the display's room was noted");
    Rectangle::new(area.at.into(), area.size.into())
}

/// Where the terminal is on the plane.
fn the_terminal(server: &Server) -> Rectangle<i32, Logical> {
    server
        .alos_window_on_the_plane(THE_TERMINAL)
        .expect("the terminal is open")
}

/// The pointer moved to `(x, y)`, a frame told the bands, and the primary
/// button pressed and let go there.
fn press_at(server: &mut Server, (x, y): (i32, i32)) {
    server
        .pointer_motion(f64::from(x), f64::from(y), 1)
        .expect("the pointer moved");
    server.the_edges_were_told_where_the_pointer_is();
    let _ = server
        .pointer_button(0x110, ButtonState::Pressed, 2)
        .expect("the press was handled");
    let _ = server
        .pointer_button(0x110, ButtonState::Released, 3)
        .expect("the release was handled");
}

/// The middle of a rectangle.
fn middle(of: Rectangle<i32, Logical>) -> (i32, i32) {
    (of.loc.x + of.size.w / 2, of.loc.y + of.size.h / 2)
}

/// **Maximise fills the room any window is maximised into, and Restore puts it
/// back** — and the panel reads Restore in between.
#[test]
fn maximise_fills_the_work_area_and_restore_puts_it_back() {
    let (_runtime, mut server, _fonts) = a_terminal("alo-placed-maximise");
    let before = the_terminal(&server);
    assert!(!server.how_alos_window_is(THE_TERMINAL).maximised);

    assert!(server.place_alos_window(THE_TERMINAL, WhatARowDoes::Maximise));
    assert_eq!(the_terminal(&server), the_maximised_area(&server));
    assert_ne!(before, the_maximised_area(&server), "the control: it moved");
    assert!(server.how_alos_window_is(THE_TERMINAL).maximised);

    assert!(server.place_alos_window(THE_TERMINAL, WhatARowDoes::Restore));
    assert_eq!(the_terminal(&server), before);
    assert!(!server.how_alos_window_is(THE_TERMINAL).maximised);
}

/// **Left, then Maximise, then Restore returns it to where it was before
/// Left**, and the halves are the ones every window gets.
#[test]
fn left_then_maximise_then_restore_returns_to_the_start() {
    let (_runtime, mut server, _fonts) = a_terminal("alo-placed-left");
    let before = the_terminal(&server);
    let half = server
        .where_this_side_would_put_alos(THE_TERMINAL, crate::a_window_on_one_side::OneSide::Left)
        .expect("a display has a left half")
        .share
        .expect("the terminal fits on a half");

    assert!(server.place_alos_window(THE_TERMINAL, WhatARowDoes::Left));
    assert_eq!(
        the_terminal(&server),
        Rectangle::new(half.at.into(), half.size.into())
    );
    assert_eq!(
        the_terminal(&server).loc.x,
        the_maximised_area(&server).loc.x
    );
    assert!(server.place_alos_window(THE_TERMINAL, WhatARowDoes::Maximise));
    assert!(server.place_alos_window(THE_TERMINAL, WhatARowDoes::Restore));
    assert_eq!(the_terminal(&server), before);
}

/// **The preview never moves the window**: Right in hand draws an outline and
/// leaves the terminal where it is.
#[test]
fn a_preview_moves_nothing() {
    let (_runtime, server, _fonts) = a_terminal("alo-placed-preview");
    let before = the_terminal(&server);
    assert!(
        server
            .the_preview_of_alos(THE_TERMINAL, Some(WhatARowDoes::Right), 100)
            .is_some()
    );
    assert!(
        server
            .the_preview_of_alos(THE_TERMINAL, Some(WhatARowDoes::Maximise), 100)
            .is_none(),
        "only Left and Right are previewed"
    );
    assert_eq!(the_terminal(&server), before);
}

/// **The refusals**: Full screen and Help are not offered and do nothing;
/// Restore on a window nothing placed does nothing; Settings has no panel to
/// place it — each counted, none moving anything.
#[test]
fn what_is_not_offered_changes_nothing() {
    let (_runtime, mut server, _fonts) = a_terminal("alo-placed-refused");
    let before = the_terminal(&server);
    let how = server.how_alos_window_is(THE_TERMINAL);
    assert!(how.full_screen_is_not_offered && !how.alo_may_help);
    for does in [
        WhatARowDoes::FullScreen,
        WhatARowDoes::HelpWithThisWindow,
        WhatARowDoes::Restore,
    ] {
        assert!(!server.place_alos_window(THE_TERMINAL, does), "{does:?}");
    }
    assert!(!server.place_alos_window(
        AlosBand::Its(AlosOwnWindow::Settings),
        WhatARowDoes::Maximise
    ));
    let refused = server.how_many_window_panel_choices_were_refused();
    server.carry_out_the_row_on(&ABand::Alos(THE_TERMINAL), WhatARowDoes::FullScreen);
    assert_eq!(
        server.how_many_window_panel_choices_were_refused(),
        refused + 1
    );
    assert_eq!(the_terminal(&server), before);
}

/// **Window actions pressed on the terminal's band opens its panel, and
/// Maximise pressed in the panel maximises it** — both down the pointer road a
/// person's press takes.
#[test]
fn a_press_on_window_actions_then_on_maximise_maximises_the_terminal() {
    let (_runtime, mut server, _fonts) = a_terminal("alo-placed-pressed");
    let band = edge_of(
        crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(
            server.where_the_terminal_is_seen().expect("drawn"),
            100,
        ),
        Decorations::AlosOwnWindowThatIsPlaced,
        true,
        Size::from((10_000, 10_000)),
    );
    let window_actions = band
        .the_control(OnTheEdge::WindowActions)
        .expect("the terminal's band has Window actions")
        .target;
    press_at(&mut server, middle(window_actions));
    assert_eq!(
        server.whose_bands_panel_is_open(),
        Some(ABand::Alos(THE_TERMINAL)),
        "Window actions did not open the terminal's panel"
    );
    let (_, menu) = server
        .alos_band_and_its_panel(THE_TERMINAL)
        .expect("the terminal has a band");
    let menu = menu.expect("its panel is open");
    let rows: Vec<WhatARowDoes> = menu.rows.iter().map(|row| row.does).collect();
    assert_eq!(
        rows,
        vec![
            WhatARowDoes::Left,
            WhatARowDoes::Maximise,
            WhatARowDoes::Right
        ],
        "a choice that does not work was drawn"
    );
    let maximise = menu
        .rows
        .iter()
        .find(|row| row.does == WhatARowDoes::Maximise)
        .expect("Maximise is in the panel")
        .row;
    press_at(&mut server, middle(maximise));
    assert_eq!(the_terminal(&server), the_maximised_area(&server));
    assert_eq!(
        server.whose_bands_panel_is_open(),
        None,
        "the panel stayed open"
    );
}

/// **Alt+F3 opens the terminal's panel on its first choice, the arrow walks
/// it, and Enter carries it out** — the keyboard road, which is the one the
/// KVM walk takes.
#[test]
fn alt_f3_right_and_enter_maximise_the_terminal() {
    let (_runtime, mut server, _fonts) = a_terminal("alo-placed-keys");
    let shortcuts = Shortcuts::shipped();
    let options = shortcuts
        .chord_for(Action::WindowOptions)
        .expect("the release ships Alt+F3");
    assert!(server.a_chord_places_alos_own_window(&shortcuts, options));
    assert_eq!(
        server.whose_bands_panel_is_open(),
        Some(ABand::Alos(THE_TERMINAL))
    );
    let _ = server.a_key_on_a_band(AnEdgeKey::Next);
    let _ = server.a_key_on_a_band(AnEdgeKey::Press);
    assert_eq!(the_terminal(&server), the_maximised_area(&server));
}

/// **Super+Up maximises the terminal with the keys, and again restores it**;
/// with Settings holding the keys the chord is not taken, so it is never
/// carried out on a window the person is not in.
#[test]
fn super_up_maximises_and_restores_and_is_not_taken_from_settings() {
    let (_runtime, mut server, _fonts) = a_terminal("alo-placed-super-up");
    let before = the_terminal(&server);
    let shortcuts = Shortcuts::shipped();
    let maximise = shortcuts
        .chord_for(Action::MaximiseWindow)
        .expect("the release ships Super+Up");
    assert!(server.a_chord_places_alos_own_window(&shortcuts, maximise));
    assert_eq!(the_terminal(&server), the_maximised_area(&server));
    assert!(server.a_chord_places_alos_own_window(&shortcuts, maximise));
    assert_eq!(the_terminal(&server), before);

    server.the_person_went_to(AKindOfWindow::Applications);
    assert!(
        !server.a_chord_places_alos_own_window(&shortcuts, maximise),
        "a chord was taken for a window without the keyboard"
    );
}

/// **A task window — the alo chat window — is maximised the same way**, into
/// the same room.
#[test]
fn a_task_window_is_maximised_into_the_same_room() {
    let (_runtime, mut server, _fonts) = a_session("alo-placed-task");
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
    let task = AlosBand::ATask(which);
    let before = server.alos_window_on_the_plane(task).expect("placed");
    assert!(server.place_alos_window(task, WhatARowDoes::Maximise));
    assert_eq!(
        server.alos_window_on_the_plane(task),
        Some(the_maximised_area(&server))
    );
    assert!(server.place_alos_window(task, WhatARowDoes::Restore));
    assert_eq!(server.alos_window_on_the_plane(task), Some(before));
}

/// **The line the KVM walk waits for is the start of the line this says** —
/// read from the walk's own source, `crates/alo-walking/src/the_first_walk.rs`,
/// because the walk does not build this crate and a copy kept in either
/// would drift.
#[test]
fn the_walk_waits_for_the_words_the_panel_says() {
    let walk = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../alo-walking/src/the_first_walk.rs"),
    )
    .expect("the first walk's source is beside this crate");
    let declared = walk
        .split("pub const THE_TERMINAL_MAXIMISED: &str =")
        .nth(1)
        .and_then(|rest| rest.split('"').nth(1))
        .expect("the walk declares THE_TERMINAL_MAXIMISED");
    let said =
        super::what_the_panel_says("the terminal", WhatARowDoes::Maximise, true, (1440, 906));
    assert!(
        said.starts_with(declared),
        "the walk waits for {declared:?} and the shell says {said:?}"
    );
    assert!(
        !super::what_the_panel_says("the terminal", WhatARowDoes::Left, false, (720, 906))
            .starts_with(declared),
        "the control: a placement that does not fill the work area is not read as one that does"
    );
}
