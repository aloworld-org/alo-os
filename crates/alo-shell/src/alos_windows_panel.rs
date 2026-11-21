//! **The window panel on alo's own windows**: Window actions on the
//! terminal's band and on a task window's opens the same panel an
//! application's does, kept in the same table, walked by the same keys and
//! drawn by the same painter.
//!
//! What differs is only what is underneath: an application's window is found
//! by its surface, alo's by [`AlosBand`]; an application is placed by its
//! client's mode, alo's by `crate::alos_window_placed`. Everything a person
//! sees and presses — the panel's layout (`crate::window_menu`), its keys
//! (`crate::window_edge_keys`), the one-panel-at-a-time rule and the preview —
//! is the one implementation, reached through [`ABand`].

use smithay::utils::{Logical, Rectangle};

use crate::the_bands_over_alos_windows::AlosBand;
use crate::which_band::ABand;
use crate::window_edge::{WindowEdge, edge_of};
use crate::window_menu::{HowTheWindowIs, WhatARowDoes, WindowMenu, the_menu_of};

impl crate::Server {
    /// Where this one of alo's windows is drawn, in the band's units at the
    /// scale presses are measured at.
    fn alos_window_in_the_edges_units(&self, which: AlosBand) -> Option<Rectangle<i32, Logical>> {
        let scale = self.the_scale_an_edge_is_measured_at();
        self.alos_windows_with_a_band()
            .into_iter()
            .find(|(it, _)| *it == which)
            .map(|(_, window)| {
                crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(window, scale)
            })
    }

    /// How this window is now, which decides what its panel reads: Restore
    /// once the panel has placed it, and **no Full screen and no Help yet** —
    /// neither works on alo's windows, and a choice that does not work is not
    /// drawn.
    #[must_use]
    pub(crate) fn how_alos_window_is(&self, which: AlosBand) -> HowTheWindowIs {
        HowTheWindowIs {
            maximised: self.is_alos_window_placed(which),
            full_screen: false,
            alo_may_help: false,
            full_screen_is_not_offered: true,
        }
    }

    /// This window's band laid out with its controls — the keyboard or the
    /// panel holds them shown — and its panel, if it is open: what a key is
    /// told against.
    #[must_use]
    pub(crate) fn alos_band_and_its_panel(
        &self,
        which: AlosBand,
    ) -> Option<(WindowEdge, Option<WindowMenu>)> {
        let room = self.the_room_an_edge_is_clamped_in();
        let edge = edge_of(
            self.alos_window_in_the_edges_units(which)?,
            which.decorations(),
            true,
            room,
        );
        let open = self.whose_bands_panel_is_open() == Some(ABand::Alos(which));
        let menu = if open {
            the_menu_of(&edge, self.how_alos_window_is(which), room)
        } else {
            None
        };
        Some((edge, menu))
    }

    /// Do what a choice in the panel says, to whichever window's panel it is.
    ///
    /// **A refusal changes nothing and is counted, never swallowed**, on
    /// alo's windows as on an application's.
    pub(crate) fn carry_out_the_row_on(&mut self, band: &ABand, does: WhatARowDoes) {
        match band {
            ABand::App(surface) => self.carry_out_the_row(surface, does),
            ABand::Alos(which) => {
                if !self.place_alos_window(*which, does) {
                    self.a_window_panel_choice_was_refused();
                }
            }
        }
    }
}
