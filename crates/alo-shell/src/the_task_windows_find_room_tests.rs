//! **A task window opens in the room the Dock and the put-aside panel leave**,
//! as every other window alo places does — with the edge above it, which
//! carries its Close, below the top of the view.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_appearance::TextScale;
use alo_models::InferenceSource;
use alo_strings::Direction;
use alo_tasks::ATask;
use smithay::utils::Rectangle;

use crate::Server;

/// The display, at life size: the owner's screenshot of 2026-11-20 was about
/// this shape.
const DISPLAY: (i32, i32) = (1280, 800);

/// Where the Dock's band begins, from the top.
const DOCK_TOP: i32 = 700;

/// Where the put-aside panel's column begins, from the left.
const PANEL_AT: i32 = 1100;

/// A seat with the Dock and the panel drawn on its one display.
fn a_seat_with_the_dock_and_the_panel(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind(directory.path(), named).expect("a bound socket");
    server.the_fixed_controls_were_drawn(
        "the-one-display",
        crate::FixedControlsDrawn {
            dock_band: Some(Rectangle::new(
                (400, DOCK_TOP).into(),
                (480, DISPLAY.1 - DOCK_TOP).into(),
            )),
            panel_reserved: Rectangle::new(
                (PANEL_AT, 0).into(),
                (DISPLAY.0 - PANEL_AT, DISPLAY.1).into(),
            ),
            what_is_leaving: None,
            top_controls: None,
        },
        TextScale::ordinary(),
    );
    (directory, server)
}

/// **The task window and its edge lie wholly in the room**: its Close on the
/// screen, its foot above the Dock's top, and its side clear of the panel's
/// column. Before 2026-11-20 it was sized for the whole view, and its reply
/// field reached under the Dock.
#[test]
fn a_task_window_opens_above_the_dock_and_clear_of_the_panel() {
    let (_directory, mut server) = a_seat_with_the_dock_and_the_panel("task-above-the-dock");
    let _which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "Open the file hello.txt in my home folder in VSCodium",
        InferenceSource::ThisMachine,
    ));
    let here = server.the_place_now();
    server.tasks_mut().new_windows_open_on(here);
    let camera = server.the_camera().at();
    let view = Rectangle::new((camera.x, camera.y).into(), DISPLAY.into());

    assert_eq!(
        server.the_task_windows_find_room(view, Direction::LeftToRight, 100),
        1,
        "the premise: one task window was placed"
    );
    let (_, window) = server
        .the_task_windows_on_the_plane()
        .first()
        .copied()
        .expect("placed on this Place");
    let edge = crate::window_edge::THE_REGION_IS_TALL;
    assert!(
        window.loc.y - edge >= camera.y,
        "its Close is off the top: {window:?}"
    );
    assert!(
        window.loc.y + window.size.h <= camera.y + DOCK_TOP,
        "the task window reaches behind the Dock: {window:?}"
    );
    assert!(
        window.loc.x + window.size.w <= camera.x + PANEL_AT,
        "the task window opened under the panel: {window:?}"
    );
}
