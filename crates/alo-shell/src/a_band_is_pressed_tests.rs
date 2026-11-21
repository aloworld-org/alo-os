//! **Every kind of alo's own window: each control on its band does what it
//! says, pressed by the pointer, by a screen reader and, for Close, by
//! Alt+F4.**
//!
//! A test of the rule, not of one window. `docs/design/the-window-controls.md`
//! gives every window the same band, and `CLAUDE.md` says a rule applies
//! everywhere it is true. Until 2026-11-21 each kind of window had its own
//! press road. The fixes landed on one road at a time: Alt+F4 was fixed for
//! task windows on 10-28 and found broken on the terminal, Settings and the
//! record on 11-08, and a screen reader could press only a task window's
//! controls. So this walks **every** kind, through [`every_kind`], whose
//! match on `AlosOwnWindow` is exhaustive: a new kind of alo window does not
//! compile until it is listed here. An application's window is the same rule
//! over a real Wayland client, in `tests/every_band_is_pressed`.
//!
//! For each kind and each control `window_edge_reading::the_controls_of` gives
//! it, on a fresh session:
//! - **the pointer**: point at the band, press the control where it is drawn,
//!   down `Server::pointer_button`, and see its effect;
//! - **a screen reader**: the tree has a node for that control that can be
//!   pressed, and pressing it has the same effect;
//! - **Alt+F4**, with the window holding the keyboard: what its Close does.
//!
//! **What would have to be true for this to pass while the product is
//! broken:** the effect checks read the server's state, not the screen. A
//! window that is asked to go aside and is then never drawn aside, or a
//! panel that opens and is not drawn, passes here. The walk's band section
//! (PR #1440) presses the same controls on a booted machine and looks.

use std::os::unix::fs::PermissionsExt as _;
use std::time::Duration;

use alo_access::{Surface, TurnedOn};
use alo_appearance::TextScale;
use alo_models::InferenceSource;
use alo_put_aside::AlosOwnWindow;
use alo_strings::Direction;
use alo_tasks::ATask;
use smithay::backend::input::{ButtonState, KeyState};
use smithay::input::keyboard::XkbConfig;
use smithay::utils::{Physical, Point, Rectangle, Size};

use crate::Server;
use crate::access_action::Pressed;
use crate::display_scale::DisplayScale;
use crate::the_bands_over_alos_windows::AlosBand;
use crate::the_terminal_a_person_opened::A_SENSIBLE_FIRST_GRID;
use crate::which_band::ABand;
use crate::which_window_is_in_front::AKindOfWindow;
use crate::window_edge::OnTheEdge;

/// The display the design's frames are the size of.
const A_DISPLAY: (i32, i32) = (1440, 960);

/// evdev's codes for Alt and F4.
const LEFT_ALT: u32 = 56;
/// F4.
const F4: u32 = 62;

/// One kind of alo's own window.
#[derive(Debug, Clone, Copy)]
enum Kind {
    /// The terminal, Settings or the record.
    Its(AlosOwnWindow),
    /// A task window.
    ATask,
}

/// Every kind of alo's own window that wears a band. **Exhaustive**: a new
/// `AlosOwnWindow` fails to compile here until it is listed.
fn every_kind() -> Vec<Kind> {
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
fn a_session(named: &str) -> (tempfile::TempDir, Server) {
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
struct Opened {
    /// Its runtime directory, and the record's folder, kept while it is.
    _kept: (tempfile::TempDir, Option<crate::record_testing::Kept>),
    /// The server it is open on.
    server: Server,
    /// Its band.
    band: AlosBand,
    /// Its window, on the display.
    window: Rectangle<i32, Physical>,
}

/// `kind`, opened on a session of its own.
fn opened(kind: Kind, named: &str) -> Opened {
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
fn settings_is_drawn(server: &mut Server) {
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
fn the_pointer_at(server: &mut Server, at: Point<i32, Physical>) {
    server
        .pointer_motion(f64::from(at.x), f64::from(at.y), 1)
        .expect("the pointer moved");
    server.the_edges_were_told_where_the_pointer_is();
    std::thread::sleep(crate::the_controls_fade::THEY_GO_IN + Duration::from_millis(20));
}

/// Where `control` is drawn on the band over `window`, at 100 per cent,
/// revealed: the middle of its target.
fn where_it_is(
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
fn on_the_band(window: Rectangle<i32, Physical>) -> Point<i32, Physical> {
    Point::from((
        window.loc.x + 30,
        window.loc.y - crate::window_edge::THE_REGION_IS_TALL / 2,
    ))
}

/// **What pressing `control` on `band` should have done**, read off the
/// server: the window closing or asking, going aside, or its panel open.
fn it_did(server: &Server, band: AlosBand, control: OnTheEdge) -> bool {
    match control {
        OnTheEdge::Close => match band {
            AlosBand::Its(AlosOwnWindow::TheTerminal) => {
                server.alos_own_terminal().is_none() || server.the_question_closing_asks().is_some()
            }
            AlosBand::Its(AlosOwnWindow::Settings) => !server.the_settings_window().is_open(),
            AlosBand::Its(AlosOwnWindow::TheRecord) => !server.the_record_window().is_open(),
            AlosBand::ATask(which) => !server.tasks().is_open(which),
        },
        OnTheEdge::Minimise => match band {
            AlosBand::Its(which) => server.alos_windows_asked_to_go_aside.contains(&which),
            AlosBand::ATask(which) => server.tasks_asked_to_go_aside.contains(&which),
        },
        OnTheEdge::WindowActions => server
            .where_the_keyboard_is_on_this_band(&ABand::Alos(band))
            .the_menu()
            .is_some(),
    }
}

/// The controls `band` has, which is the list every road is held to.
fn its_controls(band: AlosBand) -> Vec<OnTheEdge> {
    crate::window_edge_reading::the_controls_of(band.decorations())
}

/// **The pointer**: every control on every kind does what it says.
#[test]
fn every_control_on_every_kind_of_alo_window_works_by_the_pointer() {
    for kind in every_kind() {
        for control in its_controls(kind) {
            let mut it = opened(kind, &format!("band-pointer-{kind:?}-{control:?}"));
            the_pointer_at(&mut it.server, on_the_band(it.window));
            let at = where_it_is(it.band, it.window, control);
            the_pointer_at(&mut it.server, at);
            let _ = it
                .server
                .pointer_button(0x110, ButtonState::Pressed, 2)
                .expect("the press was handled");
            let _ = it
                .server
                .pointer_button(0x110, ButtonState::Released, 3)
                .expect("the release was handled");
            assert!(
                it_did(&it.server, it.band, control),
                "{control:?} pressed by the pointer on {kind:?}'s band did nothing"
            );
        }
    }
}

/// **A screen reader**: every control on every kind is in the tree, can be
/// pressed there, and pressing it does what the pointer's press does.
#[test]
fn every_control_on_every_kind_of_alo_window_works_by_a_screen_reader() {
    for kind in every_kind() {
        for control in its_controls(kind) {
            let mut it = opened(kind, &format!("band-reader-{kind:?}-{control:?}"));
            let pressed = Pressed::OnABand {
                band: ABand::Alos(it.band),
                control,
            };
            let tree = it.server.read_aloud_with_the_frames_open(
                &crate::approval_testing::words(),
                &Surface::ALL,
                &TurnedOn::nothing(),
            );
            assert!(
                tree.what_can_be_pressed().contains(&pressed),
                "a screen reader cannot find {control:?} on {kind:?}'s band to press it"
            );
            let _ = it.server.a_reader_pressed(pressed);
            assert!(
                it_did(&it.server, it.band, control),
                "{control:?} pressed by a screen reader on {kind:?}'s band did nothing"
            );
        }
    }
}

/// **Alt+F4 does what Close does**, on every kind holding the keyboard, down
/// the road that kind's keys take on a running machine.
#[test]
fn alt_f4_does_what_close_does_on_every_kind_of_alo_window() {
    for kind in every_kind() {
        let mut it = opened(kind, &format!("band-alt-f4-{kind:?}"));
        for (code, state) in [
            (LEFT_ALT, KeyState::Pressed),
            (F4, KeyState::Pressed),
            (F4, KeyState::Released),
            (LEFT_ALT, KeyState::Released),
        ] {
            let event = crate::DirectKeyEvent {
                code,
                state,
                time: 1,
            };
            match it.band {
                AlosBand::Its(AlosOwnWindow::Settings) => {
                    crate::a_key_reaches_settings::the_key_settings_took(
                        &mut it.server,
                        event,
                        &crate::settings_testing::settings_words(),
                    )
                    .expect("Settings took the key");
                }
                AlosBand::Its(AlosOwnWindow::TheRecord) => {
                    crate::a_key_reaches_the_record::the_key_the_record_took(&mut it.server, event)
                        .expect("the record took the key");
                }
                AlosBand::ATask(_) => {
                    crate::a_key_reaches_the_task_in_front::the_key_the_task_took(
                        &mut it.server,
                        event,
                        None,
                        &crate::approval_testing::words(),
                    )
                    .expect("the task took the key");
                }
                AlosBand::Its(AlosOwnWindow::TheTerminal) => {
                    let _ = it
                        .server
                        .keyboard_key(code, state, 1)
                        .expect("the seat took the key");
                }
            }
        }
        assert!(
            it_did(&it.server, it.band, OnTheEdge::Close),
            "Alt+F4 on {kind:?} did not do what its Close does"
        );
    }
}
