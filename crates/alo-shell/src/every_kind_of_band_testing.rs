//! **Every kind of alo's own window that wears a band, opened as a frame
//! leaves it**: what the rule-shaped band tests open, one kind at a time
//! (`crate::a_band_is_pressed`, `crate::a_band_is_drawn`).
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;
use std::time::Duration;

use alo_appearance::TextScale;
use alo_models::InferenceSource;
use alo_put_aside::AlosOwnWindow;
use alo_strings::Direction;
use alo_tasks::ATask;
use smithay::input::keyboard::XkbConfig;
use smithay::utils::{Physical, Point, Rectangle, Size};

use crate::Server;
use crate::display_scale::DisplayScale;
use crate::the_bands_over_alos_windows::AlosBand;
use crate::the_terminal_a_person_opened::A_SENSIBLE_FIRST_GRID;
use crate::which_window_is_in_front::AKindOfWindow;
use crate::window_edge::OnTheEdge;

/// The display the design's frames are the size of.
pub(crate) const A_DISPLAY: (i32, i32) = (1440, 960);

/// One kind of alo's own window.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Kind {
    /// The terminal, Settings or the record.
    Its(AlosOwnWindow),
    /// A task window.
    ATask,
}

/// Every kind of alo's own window that wears a band. **Exhaustive**: a new
/// `AlosOwnWindow` fails to compile here until it is listed.
pub(crate) fn every_kind() -> Vec<Kind> {
    let listed = [
        AlosOwnWindow::TheTerminal,
        AlosOwnWindow::Settings,
        AlosOwnWindow::TheRecord,
    ];
    for which in listed {
        match which {
            AlosOwnWindow::TheTerminal | AlosOwnWindow::Settings | AlosOwnWindow::TheRecord => {}
        }
    }
    let mut every: Vec<Kind> = listed.into_iter().map(Kind::Its).collect();
    every.push(Kind::ATask);
    every
}

/// A session with a keyboard, a pointer and the shipped shortcuts.
pub(crate) fn a_session(named: &str) -> (tempfile::TempDir, Server) {
    // **A socket name of letters, digits and dashes**: a kind's `Debug` has
    // brackets in it, which a Wayland socket name refuses.
    let named: String = named
        .chars()
        .map(|it| if it.is_ascii_alphanumeric() { it } else { '-' })
        .collect();
    let named = named.as_str();
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
    server.the_shortcuts_are(alo_shortcuts::Shortcuts::shipped());
    (runtime, server)
}

/// One kind of window, open, placed and drawn as a frame leaves it, holding
/// the keyboard: its band, and where its window is on the display.
pub(crate) struct Opened {
    /// Its runtime directory, and the record's folder, kept while it is.
    pub(crate) _kept: (tempfile::TempDir, Option<crate::record_testing::Kept>),
    /// The server it is open on.
    pub(crate) server: Server,
    /// Its band.
    pub(crate) band: AlosBand,
    /// Its window, on the display.
    pub(crate) window: Rectangle<i32, Physical>,
}

/// `kind`, opened on a session of its own.
pub(crate) fn opened(kind: Kind, named: &str) -> Opened {
    let (runtime, mut server) = a_session(named);
    let mut record = None;
    let band = match kind {
        Kind::Its(AlosOwnWindow::TheTerminal) => {
            assert!(server.open_alos_own_terminal(A_SENSIBLE_FIRST_GRID));
            let mut fonts = crate::WindowControlLabels::new()
                .expect("the bundled faces load")
                .fonts;
            let _ = server.the_terminal_fits_its_window(
                &mut fonts,
                A_DISPLAY,
                1.0,
                Direction::LeftToRight,
            );
            server.the_person_went_to(AKindOfWindow::TheTerminal);
            AlosBand::Its(AlosOwnWindow::TheTerminal)
        }
        Kind::Its(AlosOwnWindow::Settings) => {
            let machine = crate::settings_testing::a_persons_machine(named);
            server.the_settings_places_are(machine.places.clone());
            server.settings = machine.opened();
            server.the_person_went_to_settings();
            settings_is_drawn(&mut server);
            AlosBand::Its(AlosOwnWindow::Settings)
        }
        Kind::Its(AlosOwnWindow::TheRecord) => {
            let kept = crate::record_testing::an_afternoon_kept();
            server.the_record_is_kept(Ok(kept.recounting()));
            let _ = server.the_record_is_opened();
            let _ = server.the_record_is_placed((880, 796), A_DISPLAY, Direction::LeftToRight);
            server.where_the_records_body_was_drawn =
                Some(Rectangle::new((280, 100).into(), (880, 752).into()));
            record = Some(kept);
            AlosBand::Its(AlosOwnWindow::TheRecord)
        }
        Kind::ATask => {
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
            // **Finished**, so Close closes it: a task alo is still working on
            // is not closed by Close (ADR 0139), which is its own test.
            server
                .tasks_mut()
                .the_mut(which)
                .expect("the task is open")
                .alo_says(alo_tasks::conversation::Turn::from_alo(
                    "Your invoices are tidy.",
                ));
            server.this_task_took_the_keyboard(which);
            AlosBand::ATask(which)
        }
    };
    let window = server
        .alos_windows_with_a_band()
        .into_iter()
        .find(|(which, _)| *which == band)
        .map(|(_, window)| window)
        .expect("the window is on the display with a band");
    Opened {
        _kept: (runtime, record),
        server,
        band,
        window,
    }
}

/// What a frame does with Settings: place it and keep where it drew it.
pub(crate) fn settings_is_drawn(server: &mut Server) {
    let sizes = crate::settings_on_the_plane::how_big_settings_is(
        A_DISPLAY,
        TextScale::ordinary(),
        DisplayScale::ONE,
    )
    .expect("room");
    let _ = server.settings_is_placed(sizes.window, A_DISPLAY, Direction::LeftToRight);
    let room = server
        .where_the_settings_room_is_drawn("", sizes)
        .expect("placed");
    let mut labels = crate::WindowControlLabels::new().expect("the bundled faces");
    let picture = crate::settings_raster::picture(
        server.the_settings_window(),
        &crate::settings_testing::settings_words(),
        &mut labels,
        sizes.room,
        crate::settings_testing::light(),
        std::time::SystemTime::now(),
        None,
        DisplayScale::ONE,
    )
    .expect("Settings is drawn");
    server.settings_was_drawn(crate::settings_as_drawn::SettingsAsDrawn::of(
        &picture, room,
    ));
}

/// The pointer at `at` on the display, and a frame told every band where it
/// is, past the fade in.
pub(crate) fn the_pointer_at(server: &mut Server, at: Point<i32, Physical>) {
    server
        .pointer_motion(f64::from(at.x), f64::from(at.y), 1)
        .expect("the pointer moved");
    server.the_edges_were_told_where_the_pointer_is();
    std::thread::sleep(crate::the_controls_fade::THEY_GO_IN + Duration::from_millis(20));
}

/// Where `control` is drawn on the band over `window`, at 100 per cent,
/// revealed: the middle of its target.
pub(crate) fn where_it_is(
    band: AlosBand,
    window: Rectangle<i32, Physical>,
    control: OnTheEdge,
) -> Point<i32, Physical> {
    let edge = crate::window_edge::edge_of(
        crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(window, 100),
        band.decorations(),
        true,
        Size::from(A_DISPLAY),
    );
    let target = edge
        .the_control(control)
        .expect("the control: this band has this control")
        .target;
    Point::from((
        target.loc.x + target.size.w / 2,
        target.loc.y + target.size.h / 2,
    ))
}

/// A point on the band over `window`, clear of every control.
pub(crate) fn on_the_band(window: Rectangle<i32, Physical>) -> Point<i32, Physical> {
    Point::from((
        window.loc.x + 30,
        window.loc.y - crate::window_edge::THE_REGION_IS_TALL / 2,
    ))
}
