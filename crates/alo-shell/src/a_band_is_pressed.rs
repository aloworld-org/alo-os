//! **What a press on any window's band does: one road for every window.**
//!
//! `docs/design/the-window-controls.md` gives every window the same band:
//! Window actions opens the window panel, Minimise puts the window aside,
//! Close asks it to close. Until 2026-11-21 that sentence was carried out six
//! times, once per kind of window, each with its own `match` from what is under
//! the pointer to an action (`a_press_on_the_terminals_edge`,
//! `a_press_on_the_settings_edge`, `a_press_on_the_records_edge`,
//! `a_press_on_a_task_windows_edge`, the applications' `a_press_on_an_edge`,
//! and a screen reader's press, which reached task windows only). A fix
//! landed on the road in front of whoever found the fault, and the other kinds
//! kept it: Alt+F4 was fixed for task windows on 10-28 and found again on the
//! terminal, Settings and the record on 11-08.
//!
//! So there is one function for what a control does,
//! [`crate::Server::a_bands_control_is_pressed`], and every road reaches it: a
//! pointer's press on any band, a screen reader's press on any band
//! (`crate::a_reader_presses`), and Alt+F4 on any window
//! (`crate::a_chord_closes_alos_own_window`,
//! `crate::a_task_window_hands_on_a_chord`, `crate::shortcut_dispatch`). What
//! differs between kinds of window is said once, in the exhaustive matches
//! below: how each kind closes, goes aside, and is brought forward. A new kind
//! of window does not compile until it says all three.
//!
//! **Which controls a band has is still not decided here.** It is
//! `crate::window_edge_reading::the_controls_of`, the one list the band is
//! drawn from and a reader is told; a press on a control a band does not have
//! does nothing.

use alo_put_aside::AlosOwnWindow;
use smithay::utils::{Logical, Point};

use crate::Server;
use crate::an_edge_in_its_displays_units::{
    a_point_in_the_edges_units, a_window_in_the_edges_units,
};
use crate::closing_a_task_window::WhatClosingCameTo;
use crate::closing_the_terminal::WhatClosingDid;
use crate::the_bands_over_alos_windows::AlosBand;
use crate::the_edge_takes_the_keyboard::ByWhat;
use crate::which_band::ABand;
use crate::which_window_is_in_front::AKindOfWindow;
use crate::window_edge::{Decorations, OnTheEdge};
use crate::window_edge_pointing::{UnderThePointer, what_is_under};

/// Who pressed a band's control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PressedBy {
    /// A pointer, on the control where it is drawn.
    ThePointer,
    /// A screen reader, on the control's node in the tree.
    AReader,
    /// A key: Alt+F4 for Close.
    TheKeys,
}

impl PressedBy {
    /// Where focus goes in a panel this press opens: a pointer leaves no ring,
    /// a reader and the keys are put somewhere they can be seen and heard.
    const fn as_the_panel_takes_it(self) -> ByWhat {
        match self {
            Self::ThePointer => ByWhat::ThePointer,
            Self::AReader | Self::TheKeys => ByWhat::TheKeyboard,
        }
    }
}

/// What closing a window came to, in each kind's own words.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ClosingCameTo {
    /// An application was asked to close; it handles unsaved work itself.
    /// An error where it could not be asked: it had already gone.
    TheApplicationWasAsked(Result<(), crate::WindowCloseError>),
    /// The terminal closed, or asked first because a command is running.
    TheTerminal(WhatClosingDid),
    /// Settings or the record closed.
    Closed,
    /// A task window: closed, or what kept it open.
    ATask(WhatClosingCameTo),
}

/// What a press on a band's control did.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum WhatThePressDid {
    /// Close was pressed.
    Closing(ClosingCameTo),
    /// Minimise was pressed: the window goes aside with the next frame.
    WentAside,
    /// Window actions was pressed: its panel opened, or closed if it was open.
    ThePanel,
    /// This band has no such control, so nothing was done.
    NoSuchControl,
}

impl AlosBand {
    /// The kind of window this band hangs over, in the order windows are
    /// drawn (`crate::which_window_is_in_front`).
    #[must_use]
    pub(crate) const fn the_kind(self) -> AKindOfWindow {
        match self {
            Self::Its(AlosOwnWindow::TheTerminal) => AKindOfWindow::TheTerminal,
            Self::Its(AlosOwnWindow::Settings) => AKindOfWindow::Settings,
            Self::Its(AlosOwnWindow::TheRecord) => AKindOfWindow::TheRecord,
            Self::ATask(_) => AKindOfWindow::TaskWindows,
        }
    }
}

impl Server {
    /// What this band carries, for any window.
    #[must_use]
    pub(crate) fn the_decorations_of(&self, band: &ABand) -> Decorations {
        match band {
            ABand::App(surface) => self.who_draws_the_header_of(surface),
            ABand::Alos(which) => which.decorations(),
        }
    }

    /// **The one road: `control` pressed on the band over `band`'s window.**
    ///
    /// A control the band does not have does nothing, whoever pressed it: a
    /// reader's tree a moment old, or a key, cannot reach a control the band
    /// would not draw.
    pub(crate) fn a_bands_control_is_pressed(
        &mut self,
        band: &ABand,
        control: OnTheEdge,
        by: PressedBy,
    ) -> WhatThePressDid {
        let has = crate::window_edge_reading::the_controls_of(self.the_decorations_of(band));
        if !has.contains(&control) {
            return WhatThePressDid::NoSuchControl;
        }
        match control {
            OnTheEdge::Close => WhatThePressDid::Closing(self.this_bands_window_closes(band)),
            OnTheEdge::Minimise => {
                self.this_bands_window_goes_aside(band);
                WhatThePressDid::WentAside
            }
            OnTheEdge::WindowActions => {
                self.the_menu_was_opened_on_the_band(band, by.as_the_panel_takes_it());
                WhatThePressDid::ThePanel
            }
        }
    }

    /// **Close, for every kind of window**: the door its own Close and
    /// Alt+F4 both take, so the two cannot disagree about what closing gives
    /// up.
    pub(crate) fn this_bands_window_closes(&mut self, band: &ABand) -> ClosingCameTo {
        match band {
            ABand::App(surface) => {
                ClosingCameTo::TheApplicationWasAsked(self.request_window_close(surface))
            }
            ABand::Alos(AlosBand::Its(AlosOwnWindow::TheTerminal)) => {
                ClosingCameTo::TheTerminal(self.the_person_closes_the_terminal())
            }
            ABand::Alos(AlosBand::Its(AlosOwnWindow::Settings)) => {
                self.the_person_closes_settings();
                ClosingCameTo::Closed
            }
            ABand::Alos(AlosBand::Its(AlosOwnWindow::TheRecord)) => {
                self.the_person_closes_the_record();
                ClosingCameTo::Closed
            }
            ABand::Alos(AlosBand::ATask(which)) => {
                ClosingCameTo::ATask(self.the_person_closes_a_task(*which))
            }
        }
    }

    /// **Minimise, for every kind of window**: into the right-hand panel, still
    /// running, with the next frame.
    fn this_bands_window_goes_aside(&mut self, band: &ABand) {
        match band {
            ABand::App(surface) => self.the_person_asked_to_put_aside(surface),
            ABand::Alos(AlosBand::Its(which)) => self.this_alo_window_is_asked_to_go_aside(*which),
            ABand::Alos(AlosBand::ATask(which)) => self.this_task_is_asked_to_go_aside(*which),
        }
    }

    /// **A press on alo's band anywhere but a control**: the window comes in
    /// front with the keyboard, as a press on the window itself does. A task
    /// window's band does no more than take the press, because a task window
    /// takes the keyboard from a press on its body.
    fn the_rest_of_alos_band_is_pressed(&mut self, which: AlosBand) {
        match which {
            AlosBand::Its(AlosOwnWindow::TheTerminal) => {
                self.the_composer.it_gave_the_keyboard_up();
                self.the_keyboard_is_now_in(None);
                self.the_person_went_to(AKindOfWindow::TheTerminal);
            }
            AlosBand::Its(AlosOwnWindow::Settings) => {
                self.the_composer.it_gave_the_keyboard_up();
                self.the_person_went_to(AKindOfWindow::Settings);
            }
            AlosBand::Its(AlosOwnWindow::TheRecord) => {
                self.the_composer.it_gave_the_keyboard_up();
                self.the_person_went_to(AKindOfWindow::TheRecord);
            }
            AlosBand::ATask(_) => {}
        }
    }

    /// The terminal, Settings and the record, each with a band on the display
    /// now, and its title as its band says it: what a reader is told is open.
    #[must_use]
    pub(crate) fn alos_own_windows_as_read(
        &self,
        strings: &alo_strings::Strings,
    ) -> Vec<(AlosOwnWindow, String)> {
        let say = |word: alo_strings::Word| {
            strings
                .say(&word.key(), &alo_strings::Filling::nothing())
                .text()
                .to_owned()
        };
        self.alos_windows_with_a_band()
            .into_iter()
            .filter_map(|(which, _)| match which {
                AlosBand::Its(AlosOwnWindow::TheTerminal) => Some((
                    AlosOwnWindow::TheTerminal,
                    say(alo_terminal::words::TERMINAL),
                )),
                AlosBand::Its(AlosOwnWindow::Settings) => Some((
                    AlosOwnWindow::Settings,
                    say(crate::the_settings_words::SETTINGS),
                )),
                AlosBand::Its(AlosOwnWindow::TheRecord) => Some((
                    AlosOwnWindow::TheRecord,
                    crate::the_records_words::TheRecordsSentences::of(strings).title,
                )),
                AlosBand::ATask(_) => None,
            })
            .collect()
    }

    /// The band over one of alo's windows of `kind` under `at`, in the
    /// display's pixels, newest first, and what on it is under the pointer: a
    /// control only while the band shows it.
    fn alos_band_under(
        &self,
        kind: AKindOfWindow,
        at: Point<i32, Logical>,
    ) -> Option<(AlosBand, UnderThePointer)> {
        let scale = self.the_scale_an_edge_is_measured_at();
        let room = self.the_room_an_edge_is_clamped_in();
        let at = a_point_in_the_edges_units(at, scale);
        self.alos_windows_with_a_band()
            .into_iter()
            .rev()
            .filter(|(which, _)| which.the_kind() == kind)
            .find_map(|(which, window)| {
                let edge = self.alos_band_as_pressed(
                    which,
                    a_window_in_the_edges_units(window, scale),
                    room,
                );
                what_is_under(&edge, at).map(|under| (which, under))
            })
    }

    /// Whether a band over one of alo's windows of `kind` is under `at`, in
    /// the display's pixels, without pressing it.
    #[must_use]
    pub(crate) fn alos_band_is_under(&self, kind: AKindOfWindow, at: Point<i32, Logical>) -> bool {
        self.alos_band_under(kind, at).is_some()
    }

    /// **A pointer's press at `at`, in the display's pixels, on the band over
    /// one of alo's windows of `kind`.** Answers whether a band took it:
    /// anywhere on a band takes the press, because the band is drawn over
    /// whatever is behind it.
    pub(crate) fn a_press_on_the_band_over_alos(
        &mut self,
        kind: AKindOfWindow,
        at: Point<i32, Logical>,
    ) -> bool {
        match self.alos_band_under(kind, at) {
            None => false,
            Some((which, UnderThePointer::Control(control))) => {
                let _ = self.a_bands_control_is_pressed(
                    &ABand::Alos(which),
                    control,
                    PressedBy::ThePointer,
                );
                true
            }
            Some((which, UnderThePointer::Drag | UnderThePointer::TheEdgeAndNothingOnIt)) => {
                self.the_rest_of_alos_band_is_pressed(which);
                true
            }
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_band_is_pressed_tests.rs"]
mod tests;
