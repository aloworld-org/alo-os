//! What a screen reader's press on the shell's own tree comes to.
//!
//! A press reaches here from `crate::access_action` by way of the desktop's
//! loop, and is handed to **the same door a pointer's press uses** — nothing
//! is decided here that a click does not also decide. So a reader pressing a
//! task window's Close gets exactly what clicking it gets
//! (`crate::closing_a_task_window`): a finished task closes, a task waiting
//! for the person stays open with its question unanswered, and a task alo is
//! still working on stays open until Stop exists. A reader pressing a control
//! on the Dock gets what clicking it or pressing Enter on it gets
//! (`Server::carry_out_what_the_bar_was_asked_for`).

use alo_dock::places::WhatIsHere;

use crate::Server;
use crate::a_press_on_the_dock::what_this_slot_does;
use crate::access_action::Pressed;
use crate::closing_a_task_window::WhatClosingCameTo;

/// What a reader's press came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatAReaderPressCameTo {
    /// A band's control did what a pointer's press on it does.
    ABandDidIt,
    /// Nothing: that band is not drawn now, or has no such control.
    NothingOnABand,
    /// A task window's Close was pressed, and this is what closing it came to.
    ATaskWindow(WhatClosingCameTo),
    /// A task window's Minimise was pressed: it goes into the right-hand
    /// panel with the next frame, still running.
    ATaskGoesAside,
    /// The Dock did what a click on that control does.
    TheDockDidIt,
    /// Nothing: no Dock drawn now holds that control, or a click on it does
    /// nothing either.
    NothingOnTheDock,
    /// Nothing here: a press on the *Open with* sheet is the desktop's, which
    /// holds that sheet and hands such a press to it before anything reaches
    /// the server (`crate::the_open_with_sheet`).
    NotTheServers,
    /// The GitHub sign-in sheet did what a click on that button does.
    TheGitHubSheetDidIt,
    /// Nothing: that sheet is no longer up, or does not draw that button now.
    NothingOnTheGitHubSheet,
    /// The sheet alo asks for an application on did what a click there does.
    TheApplicationSheetDidIt,
    /// Nothing: that sheet is no longer up.
    NothingOnTheApplicationSheet,
    /// Settings did what a click on that does: the page was shown, or the
    /// keyboard went to the row and it is carried out as Enter is.
    SettingsDidIt,
    /// Nothing: Settings is not open here, the page is the form a provider is
    /// added in, or what was pressed is not on the page drawn now.
    NothingInSettings,
}

impl Server {
    /// A screen reader pressed `pressed` on the tree this machine published.
    pub fn a_reader_pressed(&mut self, pressed: Pressed) -> WhatAReaderPressCameTo {
        match pressed {
            Pressed::OnABand { band, control } => self.a_reader_pressed_on_a_band(&band, control),
            Pressed::OnTheDock(what) => self.a_reader_pressed_on_the_dock(&what),
            Pressed::OnTheOpenWithSheet { .. } | Pressed::OnTheFileChooser { .. } => {
                WhatAReaderPressCameTo::NotTheServers
            }
            // **The GitHub sign-in sheet is the server's**, so a press on it
            // is answered here, held to the sheet it was heard on.
            Pressed::OnTheGitHubSheet { sheet, on } => {
                if self.a_reader_pressed_on_the_github_sheet(sheet, on) {
                    WhatAReaderPressCameTo::TheGitHubSheetDidIt
                } else {
                    WhatAReaderPressCameTo::NothingOnTheGitHubSheet
                }
            }
            // **The sheet alo asks for an application on is the server's**,
            // so a press on it is answered here, held to the sheet it was
            // heard on.
            Pressed::OnTheApplicationSheet { sheet, on } => {
                if self.a_reader_pressed_on_the_application_sheet(sheet, on) {
                    WhatAReaderPressCameTo::TheApplicationSheetDidIt
                } else {
                    WhatAReaderPressCameTo::NothingOnTheApplicationSheet
                }
            }
            Pressed::InSettings(what) => self.a_reader_pressed_in_settings(&what),
        }
    }

    /// A reader pressed `control` on the band over `band`'s window: **the one
    /// road a pointer's press takes** (`crate::a_band_is_pressed`).
    ///
    /// **Held to a band drawn now**, as a press on the Dock is: a tree is a
    /// moment old by the time it is pressed, and a window that closed in that
    /// moment is not closed again, nor another in its place.
    fn a_reader_pressed_on_a_band(
        &mut self,
        band: &crate::which_band::ABand,
        control: crate::window_edge::OnTheEdge,
    ) -> WhatAReaderPressCameTo {
        let drawn_now = match band {
            crate::which_band::ABand::App(surface) => {
                self.mapped_surfaces().any(|it| it == surface)
            }
            crate::which_band::ABand::Alos(which) => self
                .alos_windows_with_a_band()
                .iter()
                .any(|(it, _)| it == which),
        };
        if !drawn_now {
            return WhatAReaderPressCameTo::NothingOnABand;
        }
        match self.a_bands_control_is_pressed(
            band,
            control,
            crate::a_band_is_pressed::PressedBy::AReader,
        ) {
            crate::a_band_is_pressed::WhatThePressDid::NoSuchControl => {
                WhatAReaderPressCameTo::NothingOnABand
            }
            crate::a_band_is_pressed::WhatThePressDid::Closing(
                crate::a_band_is_pressed::ClosingCameTo::ATask(came_to),
            ) => WhatAReaderPressCameTo::ATaskWindow(came_to),
            crate::a_band_is_pressed::WhatThePressDid::WentAside
                if matches!(
                    band,
                    crate::which_band::ABand::Alos(
                        crate::the_bands_over_alos_windows::AlosBand::ATask(_)
                    )
                ) =>
            {
                WhatAReaderPressCameTo::ATaskGoesAside
            }
            crate::a_band_is_pressed::WhatThePressDid::Closing(_)
            | crate::a_band_is_pressed::WhatThePressDid::WentAside
            | crate::a_band_is_pressed::WhatThePressDid::ThePanel => {
                WhatAReaderPressCameTo::ABandDidIt
            }
        }
    }

    /// A reader pressed `what` on the Dock.
    ///
    /// **Held to a Dock that is drawn now**, as a click is held to landing on
    /// a slot: a tree is a moment old by the time it is pressed, and an
    /// application closed and unpinned in that moment must not be opened again
    /// by a press heard before it went.
    fn a_reader_pressed_on_the_dock(&mut self, what: &WhatIsHere) -> WhatAReaderPressCameTo {
        if !self.a_dock_drawn_now_holds(what) {
            return WhatAReaderPressCameTo::NothingOnTheDock;
        }
        if self.carry_out_what_the_bar_was_asked_for(Some(what_this_slot_does(what))) {
            WhatAReaderPressCameTo::TheDockDidIt
        } else {
            WhatAReaderPressCameTo::NothingOnTheDock
        }
    }
}

#[cfg(test)]
#[path = "a_reader_presses_tests.rs"]
mod tests;
