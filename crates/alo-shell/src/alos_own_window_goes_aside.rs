//! **alo's own windows go into the right-hand panel, and come back** — the
//! terminal, Settings and the record, minimised the way an application's
//! window is.
//!
//! The owner's approved design `10 / Minimised windows, Mac-style` (`27:2` in
//! `XbSmsBYix0TrcQ0Qzaup64`, 2026-11-12) puts every window a person minimises
//! in the panel at the right-hand edge, and ADR 0145 made these three windows
//! canvas windows like any application's. Until this file their edge carried
//! Close alone (`crate::window_edge::Decorations::AlosOwnWindow`), because
//! nothing could put one of them aside and a Minimise that reached nothing
//! would have looked live.
//!
//! *This makes an existing promise true, because `docs/features.md` promises
//! that minimising puts a window in the panel and these are windows.*
//!
//! # Aside is a fourth answer to *where is it*, not a closed window
//!
//! A window put aside is **still open**: the terminal's shell keeps running and
//! printing, a Settings page keeps what was typed into it, the record keeps its
//! place in the list. What changes is that it is not on the canvas: it is not
//! drawn, a press does not reach it, the keys do not go to it, and *Show all*
//! does not count it. Each window already had exactly that answer for being on
//! another Place — `Server::settings_is_open_here`,
//! `Server::where_the_terminal_is_on_the_plane` and their neighbours — so being
//! aside is asked in those same few places, and every reader of them follows.
//!
//! # Two holders, kept agreeing
//!
//! The panel belongs to the desktop and the server cannot see it, so the server
//! keeps its own list of which of these windows are aside — the one the drawing
//! and the keyboard read — and the panel keeps the order and the square. They
//! are put together, in [`crate::Server::put_alos_windows_aside_that_were_asked`],
//! and kept agreeing once a frame by
//! [`crate::Server::the_panel_lets_go_of_alos_windows_back`]: a window brought
//! back by any road leaves the panel, and a window closed while aside leaves
//! both. **And a square with no window behind it, or a window aside with no
//! square, is never left standing** — the second would be a window a person
//! cannot see and cannot get back.
//!
//! # Any road to the window brings it back
//!
//! Its square in the panel, the chord or the Dock that opens it, Super+Tab
//! reaching it: each of them ends at `Server::the_person_went_to`, and going to
//! a window that is aside brings it back first. One door rather than one per
//! road, so a road added later cannot forget it.

use alo_canvas::Place;
use alo_put_aside::{AlosOwnWindow, Panel};

use crate::which_window_is_in_front::AKindOfWindow;

/// The kind of window in front that `which` is.
#[must_use]
pub(crate) const fn its_kind(which: AlosOwnWindow) -> AKindOfWindow {
    match which {
        AlosOwnWindow::TheTerminal => AKindOfWindow::TheTerminal,
        AlosOwnWindow::Settings => AKindOfWindow::Settings,
        AlosOwnWindow::TheRecord => AKindOfWindow::TheRecord,
    }
}

/// Which of alo's own windows a kind is, where it is one.
#[must_use]
pub(crate) const fn which_of_alos(kind: AKindOfWindow) -> Option<AlosOwnWindow> {
    match kind {
        AKindOfWindow::TheTerminal => Some(AlosOwnWindow::TheTerminal),
        AKindOfWindow::Settings => Some(AlosOwnWindow::Settings),
        AKindOfWindow::TheRecord => Some(AlosOwnWindow::TheRecord),
        AKindOfWindow::Applications | AKindOfWindow::TaskWindows => None,
    }
}

impl crate::Server {
    /// **A person pressed Minimise on one of alo's windows**, or used the
    /// chord: it goes into the panel on the next frame, which lends it.
    pub(crate) fn this_alo_window_is_asked_to_go_aside(&mut self, which: AlosOwnWindow) {
        if !self.alos_windows_asked_to_go_aside.contains(&which) {
            self.alos_windows_asked_to_go_aside.push(which);
        }
    }

    /// Whether this one of alo's windows is in the panel now.
    #[must_use]
    pub fn this_alo_window_is_aside(&self, which: AlosOwnWindow) -> bool {
        self.alos_windows_aside.contains(&which)
    }

    /// Whether this one of alo's windows is open at all, on any Place, aside
    /// or not.
    #[must_use]
    pub(crate) fn alos_window_is_open(&self, which: AlosOwnWindow) -> bool {
        match which {
            AlosOwnWindow::TheTerminal => self.alos_own_terminal.is_some(),
            AlosOwnWindow::Settings => self.settings.is_open(),
            AlosOwnWindow::TheRecord => self.record.is_open(),
        }
    }

    /// The Place this one of alo's windows is on: where it was placed on the
    /// canvas, or the Place being looked at for one not placed yet — which is
    /// where it is about to be drawn.
    fn the_place_of_alos_window(&self, which: AlosOwnWindow) -> Place {
        let placed = match which {
            AlosOwnWindow::TheTerminal => self.the_terminal_on_the_plane.map(|(_, place)| place),
            AlosOwnWindow::Settings => self.settings_on_the_plane.map(|(_, place)| place),
            AlosOwnWindow::TheRecord => self.the_record_on_the_plane.map(|(_, place)| place),
        };
        placed.unwrap_or_else(|| self.the_place_now())
    }

    /// **Put every one of alo's windows that was asked for into the panel.**
    /// Answers how many went.
    ///
    /// The asks are taken before anything is tried, so a refusal is not
    /// retried for ever; a desktop with no panel puts nothing aside and leaves
    /// the windows where they are, rather than hiding a window with nowhere to
    /// be.
    pub fn put_alos_windows_aside_that_were_asked(&mut self, panel: Option<&mut Panel>) -> usize {
        let asked = std::mem::take(&mut self.alos_windows_asked_to_go_aside);
        let Some(panel) = panel else {
            return 0;
        };
        asked
            .into_iter()
            .filter(|which| self.put_alos_window_aside(panel, *which))
            .count()
    }

    /// **Put one of alo's windows into the panel**, still open. Answers whether
    /// it went: a window that is not open, or is aside already, cannot.
    pub(crate) fn put_alos_window_aside(&mut self, panel: &mut Panel, which: AlosOwnWindow) -> bool {
        if !self.alos_window_is_open(which) || self.this_alo_window_is_aside(which) {
            return false;
        }
        let place = self.the_place_of_alos_window(which);
        // **The panel first, then the window leaves the canvas**, as an
        // application's window is hidden only after the panel took it
        // (`crate::putting_a_window_aside`): the other order could leave a
        // window nobody can see and nothing can bring back.
        if panel.put_alos_window_aside(which, place).is_err() {
            return false;
        }
        self.alos_windows_aside.push(which);
        // **And the keyboard leaves it** with nothing more said here:
        // `Server::the_kind_in_front` no longer counts a window that is aside,
        // so the keys go to whatever the person had behind it, and no key
        // reaches a window nobody can see.
        true
    }

    /// **Bring one of alo's windows back from the panel**: it leaves the panel,
    /// comes in front with the keys, and the view travels to it where it is out
    /// of sight. Answers whether it was aside.
    pub(crate) fn bring_alos_window_back(&mut self, panel: &mut Panel, which: AlosOwnWindow) -> bool {
        let was_aside = self.this_alo_window_is_aside(which);
        // Taken out of the panel whatever the server thought, so a square can
        // never outlive the window it stood for.
        let _ = panel.bring_alos_window_back(which);
        if !was_aside {
            return false;
        }
        match which {
            AlosOwnWindow::Settings => self.the_person_went_to_settings(),
            AlosOwnWindow::TheRecord => self.the_person_went_to_the_record(),
            AlosOwnWindow::TheTerminal => {
                self.the_composer.it_gave_the_keyboard_up();
                self.the_person_went_to(AKindOfWindow::TheTerminal);
                let _ = self.the_view_goes_to_the_terminal();
            }
        }
        true
    }

    /// **Going to a window of this kind brings it back if it is aside** —
    /// the one door every road to one of alo's windows passes through
    /// (`Server::the_person_went_to`). Answers whether it was aside.
    ///
    /// The server's own list only: the panel lets go of the square on the next
    /// frame, in [`Self::the_panel_lets_go_of_alos_windows_back`].
    pub(crate) fn a_window_of_this_kind_comes_back_from_aside(&mut self, kind: AKindOfWindow) -> bool {
        let Some(which) = which_of_alos(kind) else {
            return false;
        };
        let before = self.alos_windows_aside.len();
        self.alos_windows_aside.retain(|it| *it != which);
        self.alos_windows_aside.len() != before
    }

    /// **Keep the panel and the server agreeing**, once a frame: a window
    /// closed while it was aside leaves both, a window brought back by any
    /// road leaves the panel, and a window the server holds aside that the
    /// panel lost comes back onto the canvas rather than staying where nobody
    /// can reach it. Answers how many squares the panel let go of.
    pub fn the_panel_lets_go_of_alos_windows_back(&mut self, panel: Option<&mut Panel>) -> usize {
        // A window that closed is aside nowhere.
        let still_open: Vec<AlosOwnWindow> = [
            AlosOwnWindow::TheTerminal,
            AlosOwnWindow::Settings,
            AlosOwnWindow::TheRecord,
        ]
        .into_iter()
        .filter(|which| self.alos_window_is_open(*which))
        .collect();
        self.alos_windows_aside
            .retain(|which| still_open.contains(which));
        let Some(panel) = panel else {
            // **No panel, nothing aside**: a window can only be away where
            // there is a panel to bring it back from.
            self.alos_windows_aside.clear();
            return 0;
        };
        let squares: Vec<AlosOwnWindow> = panel.alos_windows().map(|it| it.which()).collect();
        let mut let_go = 0;
        for which in squares {
            if !self.this_alo_window_is_aside(which) && panel.bring_alos_window_back(which).is_ok() {
                let_go += 1;
            }
        }
        self.alos_windows_aside
            .retain(|which| panel.holds_alos_window(*which));
        let_go
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "alos_own_window_goes_aside_tests.rs"]
mod tests;
