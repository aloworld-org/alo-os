//! Act on a press while a window panel is open.
//!
//! The panel is laid out (`crate::window_menu`) and drawn
//! (`crate::window_menu_paint`); a press on a choice has to do what it says.
//!
//! # Asked before the band, because it is in front of it
//!
//! The panel opens below the band and overlaps the application, so a press
//! inside it was never a press on the band or on the window: **presses never
//! pass through to the application** (`docs/design/the-window-controls.md`
//! §9). A press inside the panel and on no choice is the panel's, and does
//! nothing.
//!
//! # A press outside it closes it, and applies nothing
//!
//! The press that closes it is **consumed** rather than passed on: a person
//! dismissing a panel did not also mean to click what was behind it. **The one
//! exception is Window actions itself**, which is left to
//! `crate::a_press_on_an_edge` so that pressing it again closes the panel
//! rather than closing and reopening it.
//!
//! # The choices come from the same layout the draw used
//!
//! [`crate::Server::the_open_panel`] is the one place the panel is laid out
//! for a frame and for a press alike.

use smithay::backend::input::ButtonState;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point};

use crate::the_edge_takes_the_keyboard::ByWhat;
use crate::window_edge::OnTheEdge;
use crate::window_menu::{HowTheWindowIs, WhatARowDoes, WindowMenu, the_menu_of};

/// What a press did while a panel was open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ThePress {
    /// It was on a choice, which was carried out.
    Chose(WhatARowDoes),
    /// It was inside the panel and on no choice: the panel's, and nothing.
    InsideOnNothing,
    /// It was outside: the panel closed and nothing was applied.
    Dismissed,
    /// It was on Window actions, which answers it itself.
    OnWindowActions,
}

/// Where a press at `at` landed, against this open panel and its band's
/// Window actions.
#[must_use]
pub(crate) fn where_it_landed(
    menu: &WindowMenu,
    window_actions: Option<smithay::utils::Rectangle<i32, Logical>>,
    at: Point<i32, Logical>,
) -> ThePress {
    if let Some(row) = menu.what_is_under(at) {
        return ThePress::Chose(row.does);
    }
    if menu.panel.contains(at) {
        return ThePress::InsideOnNothing;
    }
    if window_actions.is_some_and(|it| it.contains(at)) {
        return ThePress::OnWindowActions;
    }
    ThePress::Dismissed
}

impl crate::Server {
    /// A press while a window panel may be open. Answers whether the panel
    /// took it, so nothing else does.
    pub(crate) fn a_press_in_the_menu(&mut self, state: ButtonState) -> bool {
        if state != ButtonState::Pressed {
            return false;
        }
        let Some(surface) = self.whose_panel_is_open() else {
            return false;
        };
        let Some(at) = self.where_the_pointer_is_in_logical_pixels() else {
            return false;
        };
        let scale = self.the_scale_an_edge_is_measured_at();
        let at = crate::an_edge_in_its_displays_units::a_point_in_the_edges_units(at, scale);
        let Some((edge, menu)) = self.the_open_panel(&surface) else {
            // **A panel whose window has gone has nothing to answer**; the
            // keyboard's state is pruned with the window.
            return false;
        };
        let window_actions = edge
            .the_control(OnTheEdge::WindowActions)
            .map(|it| it.target);
        match where_it_landed(&menu, window_actions, at) {
            ThePress::Chose(does) => {
                if let Some(row) = menu.rows.iter().copied().find(|it| it.does == does) {
                    let does = self.a_row_of_the_menu_was_pressed(&surface, row);
                    self.carry_out_the_row(&surface, does);
                }
                true
            }
            ThePress::InsideOnNothing => true,
            ThePress::Dismissed => {
                self.the_menu_was_dismissed_on(&surface, ByWhat::ThePointer);
                true
            }
            ThePress::OnWindowActions => false,
        }
    }

    /// The open panel of this window, laid out exactly as the frame draws it,
    /// with the band it hangs from.
    #[must_use]
    pub(crate) fn the_open_panel(
        &self,
        surface: &WlSurface,
    ) -> Option<(crate::window_edge::WindowEdge, WindowMenu)> {
        let scale = self.the_scale_an_edge_is_measured_at();
        let room = self.the_room_an_edge_is_clamped_in();
        let edge = crate::window_edge::edge_of(
            self.the_window_a_band_hangs_from(surface, scale),
            crate::window_edge_who_draws::who_draws_a_frame(),
            true,
            room,
        );
        let menu = the_menu_of(&edge, self.how_this_window_is(surface), room)?;
        Some((edge, menu))
    }

    /// Whether this window is maximised or full screen now, which decides the
    /// words its panel reads.
    #[must_use]
    pub(crate) fn how_this_window_is(&self, surface: &WlSurface) -> HowTheWindowIs {
        let mode = self.surfaces.requested_window_mode(surface);
        HowTheWindowIs {
            maximised: mode == crate::window_mode::Mode::Maximized,
            full_screen: mode == crate::window_mode::Mode::FillingTheScreen,
        }
    }

    /// Do what a choice in the panel says, to this window.
    ///
    /// **A refusal changes nothing and is counted, never swallowed**: the
    /// window-mode refusals are the same the chords meet, and a window that
    /// is busy being dragged is left as it is.
    pub(crate) fn carry_out_the_row(&mut self, surface: &WlSurface, does: WhatARowDoes) {
        let done = match does {
            WhatARowDoes::Maximise => self.set_window_maximized(surface, true).is_ok(),
            WhatARowDoes::Restore => self.set_window_maximized(surface, false).is_ok(),
            WhatARowDoes::FullScreen => self.set_window_full_screen(surface, true).is_ok(),
            WhatARowDoes::ExitFullScreen => self.set_window_full_screen(surface, false).is_ok(),
        };
        if !done {
            self.a_window_panel_choice_was_refused();
        }
    }
}

impl crate::Server {
    /// A choice from a window panel, or a double-click on a band, that the
    /// window refused: counted.
    pub(crate) fn a_window_panel_choice_was_refused(&mut self) {
        self.window_panel_refusals = self.window_panel_refusals.saturating_add(1);
    }

    /// How many such choices were refused, since this seat stood up.
    #[must_use]
    pub fn how_many_window_panel_choices_were_refused(&self) -> usize {
        self.window_panel_refusals
    }
}

#[cfg(test)]
#[path = "a_press_in_the_menu_tests.rs"]
mod tests;
