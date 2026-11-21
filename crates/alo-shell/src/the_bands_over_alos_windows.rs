//! **The bands over alo's own windows reveal as every window's band does** —
//! the terminal's, Settings', the record's and each task window's.
//!
//! # The fault this closes
//!
//! `docs/design/the-window-controls.md`: *No controls are drawn at rest.* The
//! controls appear as the pointer enters the band, stay while it is there, and
//! fade as it leaves; *Always show window controls* keeps them shown. Every
//! application's band has done that since #1138, through
//! `crate::the_window_edge_reveals`.
//!
//! alo's own windows did not. Each of the four places that drew one of their
//! bands — `crate::the_terminals_edge`, `crate::an_alo_windows_edge` (Settings
//! and the record) and `crate::a_task_windows_edge` — laid its band out
//! revealed and drew its controls **all the way shown**, whatever the pointer
//! did, under a reading of 2026-10-28 and 2026-11-01 (*it carries the only
//! Close this window has*) that predates the specification. A booted machine at
//! `3959dd65` showed it: the terminal and the task windows with Minimise and
//! Close drawn while the pointer was in the far corner of the screen, beside
//! applications whose bands drew none. The owner read that as the old controls,
//! which overrules the reading.
//!
//! # One rule, so a band cannot have a reveal of its own
//!
//! Each band here is told where the pointer is by
//! `crate::the_window_edge_reveals::a_band_is_told`, the function every
//! application's band is told by, in the same walk and from the same pointer.
//! What is kept here is only *which* windows those are and their state — a
//! `WlSurface` is an application's, and none of alo's windows has one.
//!
//! **The reason for the reading still holds and is answered the way every
//! window's is**: a Close that appears under the pointer is found by pointing at
//! the band, as on any window; Alt+F4 closes each of these; a screen reader
//! reaches every control without hovering (`crate::window_edge_reading`); and a
//! person who wants them always there turns on *Always show window controls*,
//! which these bands now obey as well.

use std::time::Instant;

use alo_dock::revealing::{Revealing, ThePointer};
use alo_put_aside::AlosOwnWindow;
use smithay::utils::{Logical, Physical, Point, Rectangle, Size};

use crate::the_controls_fade::TheFade;
use crate::the_tasks_a_person_opened::WhichTask;
use crate::the_window_edge_reveals::{a_band_is_told, how_shown, what_the_pointer_is};
use crate::which_band::ABand;
use crate::window_edge::{Decorations, WindowEdge, edge_of};

/// Which of alo's windows a band hangs over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlosBand {
    /// The terminal, Settings or the record.
    Its(AlosOwnWindow),
    /// One task window: the alo chat window a request opens.
    ATask(WhichTask),
}

impl AlosBand {
    /// What this band carries, as `crate::window_edge_reading` answers it.
    #[must_use]
    pub(crate) const fn decorations(self) -> Decorations {
        match self {
            Self::Its(AlosOwnWindow::TheTerminal) => Decorations::AlosOwnWindowThatIsPlaced,
            Self::Its(AlosOwnWindow::Settings | AlosOwnWindow::TheRecord) => {
                Decorations::AlosOwnWindow
            }
            Self::ATask(_) => Decorations::ATaskWindow,
        }
    }
}

/// One band's reveal and fade, as the frame before left them.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ItsBand {
    /// Whose band.
    which: AlosBand,
    /// Whether the pointer has revealed it.
    revealed: Revealing,
    /// How far its controls have faded in or out.
    fade: TheFade,
}

impl crate::Server {
    /// Every one of alo's windows with a band on this display now, with where
    /// its window is drawn, in the display's pixels.
    pub(crate) fn alos_windows_with_a_band(&self) -> Vec<(AlosBand, Rectangle<i32, Physical>)> {
        let mut every = Vec::new();
        if let Some(window) = self.where_the_terminal_is_seen() {
            every.push((AlosBand::Its(AlosOwnWindow::TheTerminal), window));
        }
        if let Some(window) = self.where_settings_is_seen() {
            every.push((AlosBand::Its(AlosOwnWindow::Settings), window));
        }
        if let Some(window) = self.where_the_records_body_is_seen() {
            every.push((AlosBand::Its(AlosOwnWindow::TheRecord), window));
        }
        every.extend(
            self.the_task_windows_on_the_screen()
                .into_iter()
                .map(|(which, window)| (AlosBand::ATask(which), window)),
        );
        every
    }

    /// Tell every band over alo's own windows where the pointer is, `at` in
    /// the bands' units, by the rule every band follows.
    ///
    /// Called by `Server::the_edges_were_told_where_the_pointer_is`, once a
    /// frame, so these bands and the applications' are told by one walk.
    pub(crate) fn alos_bands_were_told(
        &mut self,
        at: Option<Point<i32, Logical>>,
        (room, scale): (Size<i32, Logical>, u32),
        (now, less_motion): (Instant, bool),
    ) {
        let present = self.alos_windows_with_a_band();
        // **Pruned against what is open**, so a window that closes takes its
        // band's state with it and a task numbered the same later starts at
        // rest.
        self.alos_bands
            .retain(|band| present.iter().any(|(which, _)| *which == band.which));
        let open: Vec<AlosBand> = present.iter().map(|(which, _)| *which).collect();
        self.only_alos_bands_still_open(&open);
        self.only_alos_windows_still_open_are_placed();
        let always = self.window_controls.are_always_shown();
        for (which, window) in present {
            let was = self.alos_bands.iter().find(|band| band.which == which);
            let revealed = was.map_or_else(Revealing::covered, |band| band.revealed);
            let edge = edge_of(
                crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(window, scale),
                which.decorations(),
                revealed.is_revealed(),
                room,
            );
            let pointer = at.map_or(ThePointer::Elsewhere, |at| what_the_pointer_is(&edge, at));
            // **The keyboard and the panel hold it shown**, as on any band: the
            // one table `crate::the_edge_takes_the_keyboard` keeps for every
            // band says where they are.
            let keyboard = self.where_the_keyboard_is_on_this_band(&ABand::Alos(which));
            let (revealed, fade) = a_band_is_told(
                (revealed, was.map(|band| band.fade)),
                (
                    pointer,
                    keyboard.whether_the_keyboard_holds_it(),
                    keyboard.whether_a_menu_holds_it(),
                ),
                always,
                (now, less_motion),
            );
            let told = ItsBand {
                which,
                revealed,
                fade,
            };
            match self.alos_bands.iter_mut().find(|band| band.which == which) {
                Some(band) => *band = told,
                None => self.alos_bands.push(told),
            }
        }
    }

    /// Whether this band's controls are shown — which is also whether a
    /// press reaches them. `false` for a band nothing has told about a
    /// pointer yet, unless the person keeps the controls always shown.
    #[must_use]
    pub(crate) fn is_alos_band_revealed(&self, which: AlosBand) -> bool {
        self.window_controls.are_always_shown()
            || self
                .alos_bands
                .iter()
                .find(|band| band.which == which)
                .is_some_and(|band| band.revealed.is_revealed())
    }

    /// How shown this band's controls are drawn at `now`, in thousandths.
    #[must_use]
    pub(crate) fn how_shown_alos_band_is(&self, which: AlosBand, now: Instant) -> u16 {
        how_shown(
            self.window_controls.are_always_shown(),
            self.is_alos_band_revealed(which),
            self.alos_bands
                .iter()
                .find(|band| band.which == which)
                .map(|band| &band.fade),
            (now, self.motion_is_reduced()),
        )
    }

    /// The band over one of alo's windows, `window` in the bands' units, as a
    /// press finds it: its controls laid out only while they are revealed.
    #[must_use]
    pub(crate) fn alos_band_as_pressed(
        &self,
        which: AlosBand,
        window: Rectangle<i32, Logical>,
        room: Size<i32, Logical>,
    ) -> WindowEdge {
        edge_of(
            window,
            which.decorations(),
            self.is_alos_band_revealed(which),
            room,
        )
    }

    /// The band over one of alo's windows as it is drawn at `now`, and how
    /// shown its controls are: laid out with them **while any of them is still
    /// to be seen**, so a fade out has something to draw — the rule
    /// `crate::every_windows_edge` draws an application's band by.
    #[must_use]
    pub(crate) fn alos_band_as_drawn(
        &self,
        which: AlosBand,
        window: Rectangle<i32, Logical>,
        room: Size<i32, Logical>,
        now: Instant,
    ) -> (WindowEdge, u16) {
        let shown = self.how_shown_alos_band_is(which, now);
        let edge = edge_of(
            window,
            which.decorations(),
            shown > 0 || self.is_alos_band_revealed(which),
            room,
        );
        (edge, shown)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_bands_over_alos_windows_tests.rs"]
mod tests;
