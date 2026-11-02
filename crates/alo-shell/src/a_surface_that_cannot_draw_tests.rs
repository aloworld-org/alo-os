//! **A record window that cannot be drawn leaves the desktop drawing**, and is
//! closed rather than left holding the keyboard.
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt;

use alo_appearance::TextScale;
use alo_egress::Indicator;
use alo_indicator::Indicating;
use alo_strings::Direction;

use crate::desktop_testing::{an_appearance, an_undivided_display, noon_look, nothing_offered};
use crate::{
    EgressStatus, FillingWindow, RecordFrame, RecordLook, RenderError, RunningWindow, Server,
    WindowControlLabels,
};

/// The output: one a real machine has.
const SIZE: (i32, i32) = (1280, 720);

/// A record look that cannot fit `SIZE`: the largest text this machine offers,
/// which asks for a window taller than the display has room for.
fn too_large_for_the_display() -> RecordLook {
    let (_, largest) = TextScale::range();
    RecordLook {
        scale: TextScale::percent(largest).unwrap(),
        ..crate::record_testing::light()
    }
}

/// **The frame is drawn with the record left out, and the refusal is handed
/// back** — where until 2026-11-02 it ended the whole desktop.
#[test]
fn a_record_that_cannot_draw_leaves_the_rest_of_the_frame() {
    let afternoon = crate::record_testing::an_afternoon_kept();
    let mut window = crate::RecordWindow::on_an_output();
    window.opened_by_hand(&afternoon.recounting());
    let words = crate::record_testing::words();
    let look = too_large_for_the_display();
    let clock = crate::record_testing::a_clock();

    // The control: this record, at this size, really is refused, so what
    // follows is about a refusal and not about a record that drew.
    let mut labels = WindowControlLabels::new().unwrap();
    assert!(
        matches!(
            crate::record_raster::picture(
                window.shows(),
                &words,
                &mut labels,
                SIZE,
                look,
                clock,
                crate::display_scale::DisplayScale::ONE,
            ),
            Err(RenderError::RecordScene)
        ),
        "the record drew at this size, so this test proves nothing"
    );

    let dock = alo_dock::Dock::shipped();
    let strings = crate::desktop_testing::words();
    let mut egress = EgressStatus::on_an_output();
    Indicating::nowhere().show(Some(&mut egress), &Indicator::default());
    let (running, filling) = (RunningWindow::closed(), FillingWindow::closed());
    let pictures = crate::nested_desktop::frame_pictures(
        crate::DesktopFrame {
            tasks: &[],
            market: None,
            what_each_application_draws: &alo_dock::artwork::NOTHING,
            the_dock_has_open: &crate::what_the_dock_has_open::NOTHING,
            the_docks_words: crate::desktop_testing::no_words(),
            dock: &dock,
            look: noon_look(&an_appearance(), Direction::LeftToRight),
            strings: &strings,
            egress: &egress,
            running: &running,
            filling: &filling,
            in_use: &[],
            notifications: &[],
            capturing: None,
            division: an_undivided_display(),
            offer: nothing_offered(),
            windows: &[],
            put_aside: crate::desktop_testing::nothing_put_aside(),
            panel_is_revealed: true,
            filling_the_screen: false,
            display_scale: 100,
            on_the_dock: &[],
            in_more_apps: &[],
            the_keyboard_is_on_the_bar: None,
            the_docks_name_is_for: None,
            the_bar: None,
        },
        Some(RecordFrame {
            window: &window,
            strings: &words,
            look,
            clock,
        }),
        None,
        &mut labels,
        SIZE,
        None,
    )
    .expect("one window that cannot be drawn refused the whole desktop");
    assert!(pictures.record.is_none());
    assert!(matches!(
        pictures.record_refused,
        Some(RenderError::RecordScene)
    ));
    assert!(
        pictures.desktop.dock.is_some(),
        "the Dock was not drawn beside the record that could not be"
    );
}

/// **The window that could not be drawn is closed**, so it does not keep the
/// keyboard while nothing of it is on the screen.
#[test]
fn a_record_that_cannot_draw_is_closed_and_takes_no_keys() {
    let afternoon = crate::record_testing::an_afternoon_kept();
    let runtime = tempfile::tempdir().unwrap();
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut server = Server::bind(runtime.path(), "cannot-draw").unwrap();
    server.the_record_is_kept(Ok(afternoon.recounting()));
    assert_eq!(
        server.the_record_is_opened(),
        Some(crate::RecordOpened::Shown)
    );
    assert!(server.the_record_is_taking_the_keys());

    server.the_record_could_not_be_drawn("Virtual-1", &RenderError::RecordScene);
    assert!(!server.the_record_is_taking_the_keys());
    assert!(!server.record.is_open());
}

/// **Settings that cannot be drawn are closed too.**
#[test]
fn settings_that_cannot_draw_are_closed() {
    let runtime = tempfile::tempdir().unwrap();
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut server = Server::bind(runtime.path(), "cannot-draw-settings").unwrap();
    let home = tempfile::tempdir().unwrap();
    let nowhere = home.path().join("nothing-here");
    let places = crate::SettingsPlaces::of(
        None,
        Some(home.path().as_os_str()),
        None,
        &nowhere,
        &nowhere,
        &nowhere,
        &nowhere,
        None,
    );
    let _ = server
        .settings
        .opened_by_hand(&places, std::time::SystemTime::now());
    assert!(server.settings.is_open());
    server.settings_could_not_be_drawn("Virtual-1", &RenderError::RecordScene);
    assert!(!server.settings.is_open());
}
