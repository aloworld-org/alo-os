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

use alo_access::{Surface, TurnedOn};
use alo_put_aside::AlosOwnWindow;
use smithay::backend::input::{ButtonState, KeyState};

use crate::Server;
use crate::access_action::Pressed;
use crate::every_kind_of_band_testing::{
    Kind, every_kind, on_the_band, opened, the_pointer_at, where_it_is,
};
use crate::the_bands_over_alos_windows::AlosBand;
use crate::which_band::ABand;
use crate::window_edge::OnTheEdge;

/// evdev's codes for Alt and F4.
const LEFT_ALT: u32 = 56;
/// F4.
const F4: u32 = 62;

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
fn its_controls(kind: Kind) -> Vec<OnTheEdge> {
    crate::window_edge_reading::the_controls_of(match kind {
        Kind::Its(which) => AlosBand::Its(which).decorations(),
        Kind::ATask => crate::window_edge::Decorations::ATaskWindow,
    })
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
