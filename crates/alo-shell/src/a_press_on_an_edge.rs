//! What a press on a window's band reaches.
//!
//! `docs/design/the-window-controls.md`: the band is the window's only
//! free-move handle (ADR 0071); its three controls are Window actions,
//! Minimise and Close; **double-clicking the band maximises**, and again
//! restores; **right-clicking it opens the window panel**. Buttons never start a
//! drag, and a control that is not shown is never pressed — `crate::window_edge`
//! lays out none while concealed, so a press there is a press on the band.
//!
//! The decision — *which control, or the band, or nothing* — is
//! [`crate::window_edge_pointing::what_is_under`], the one reading of a point
//! on a band, which is also what the hover look is drawn from.
//!
//! # Moving acts now; the rest wait, and that is not a preference
//!
//! A drag has to take hold on the press that started it, so it is begun here.
//! The rest are **recorded and met in `crate::direct_desktop`**: putting a
//! window aside needs the live `Panel`, which `alo-desktop` owns and input has
//! no road to, and the others join it so that a refused close or maximise is
//! answered in its own error type rather than relabelled as an input fault.
//!
//! # Pressing on a band does not bring its window forward
//!
//! Neither revealing the controls nor pressing one raises the window
//! (`docs/design/the-window-controls.md`, §4): what a press asks for is done to
//! the window it was pressed on, wherever it is in the order.
//!
//! # Which window, when two bands overlap
//!
//! The first mapped surface whose band claims the point, in
//! `Server::mapped_surfaces` order — a rule rather than a measurement, because
//! nothing here orders mapped surfaces by what is in front.

use std::time::{Duration, Instant};

use smithay::backend::input::ButtonState;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::{Logical, Point};

use crate::the_edge_takes_the_keyboard::ByWhat;
use crate::window_edge::{OnTheEdge, edge_of};
use crate::window_edge_pointing::{UnderThePointer, what_is_under};

/// The primary button, `BTN_LEFT`.
const THE_PRIMARY_BUTTON: u32 = 0x110;

/// The secondary button, `BTN_RIGHT`.
const THE_SECONDARY_BUTTON: u32 = 0x111;

/// How soon a second press must follow the first to be a double-click.
///
/// **500 ms, chosen by this lane on 2026-11-09, because the design is silent**:
/// the longest of the common desktop defaults, so the person who presses
/// slowest is the one it fits. Recorded in `docs/design/the-window-controls.md`
/// as the owner's to change. The double-click is never the only road — the
/// panel's Maximise and `Super+Up` do the same — so nobody who cannot press
/// twice that fast loses anything.
pub(crate) const A_DOUBLE_CLICK_IS_WITHIN: Duration = Duration::from_millis(500);

/// How far the pointer may move between the two presses of a double-click, in
/// logical units. Chosen with [`A_DOUBLE_CLICK_IS_WITHIN`], for the same
/// reason: a hand that shakes a little still double-clicks.
pub(crate) const A_DOUBLE_CLICK_STAYS_WITHIN: i32 = 4;

/// What a press on a band asks for, met once the input batch is done.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AskedOnTheBand {
    /// One of the three controls was pressed.
    Control(OnTheEdge),
    /// The band was double-clicked: maximise, or restore.
    MaximiseOrRestore,
    /// The band was right-clicked: open the window panel.
    OpenThePanel,
}

/// The last primary press on a band, for telling a double-click.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ThePressBefore {
    /// Whose band it was on.
    surface: WlSurface,
    /// When.
    at: Instant,
    /// Where, in the band's units.
    where_: Point<i32, Logical>,
    /// Where the window was: a second press on a window that has since been
    /// maximised, restored or moved is a new first press, not a double-click.
    window: smithay::utils::Rectangle<i32, Logical>,
    /// How the window was — maximised, full screen — for the same reason: a
    /// window can change mode and keep the rectangle it is drawn at until the
    /// application answers.
    how: crate::window_menu::HowTheWindowIs,
}

impl ThePressBefore {
    /// Whether a press on `surface` at `where_`, `now`, is the second of a
    /// double-click after this one.
    #[must_use]
    pub(crate) fn makes_a_double_click(
        &self,
        surface: &WlSurface,
        now: Instant,
        where_: Point<i32, Logical>,
        window: smithay::utils::Rectangle<i32, Logical>,
        how: crate::window_menu::HowTheWindowIs,
    ) -> bool {
        is_a_double_click(
            &self.surface == surface && self.window == window && self.how == how,
            now.saturating_duration_since(self.at),
            (where_.x - self.where_.x, where_.y - self.where_.y),
        )
    }
}

/// The rule, apart from the types that carry it: the same window, soon enough,
/// and near enough.
#[must_use]
pub(crate) fn is_a_double_click(same_window: bool, apart: Duration, moved: (i32, i32)) -> bool {
    same_window
        && apart <= A_DOUBLE_CLICK_IS_WITHIN
        && moved.0.abs() <= A_DOUBLE_CLICK_STAYS_WITHIN
        && moved.1.abs() <= A_DOUBLE_CLICK_STAYS_WITHIN
}

impl crate::Server {
    /// A press on some window's band. Answers whether the band took it.
    pub(crate) fn a_press_on_an_edge(&mut self, button: u32, state: ButtonState) -> bool {
        if state != ButtonState::Pressed {
            return false;
        }
        let Some(at) = self.where_the_pointer_is_in_logical_pixels() else {
            return false;
        };
        let decorations = crate::window_edge_who_draws::who_draws_a_frame();
        let mapped: Vec<WlSurface> = self.mapped_surfaces().cloned().collect();
        let scale = self.the_scale_an_edge_is_measured_at();
        let at = crate::an_edge_in_its_displays_units::a_point_in_the_edges_units(at, scale);
        let now = Instant::now();
        for surface in mapped {
            let window = crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(
                self.where_a_window_is_drawn(&surface),
                scale,
            );
            let edge = edge_of(
                window,
                decorations,
                self.is_this_windows_edge_revealed(&surface),
                self.the_room_an_edge_is_clamped_in(),
            );
            let Some(under) = what_is_under(&edge, at) else {
                continue;
            };
            if button == THE_SECONDARY_BUTTON {
                // **Right-click anywhere on the band opens the same panel**,
                // over a control too: a right-click never presses one.
                self.asked_on_an_edge
                    .push((surface, AskedOnTheBand::OpenThePanel));
                return true;
            }
            match under {
                UnderThePointer::Control(does) => {
                    self.the_press_before = None;
                    self.asked_on_an_edge
                        .push((surface, AskedOnTheBand::Control(does)));
                    return true;
                }
                UnderThePointer::Drag | UnderThePointer::TheEdgeAndNothingOnIt => {
                    if button != THE_PRIMARY_BUTTON {
                        return self.surfaces.start_move_from_the_name(&surface, button);
                    }
                    let twice = self.the_press_before.as_ref().is_some_and(|before| {
                        before.makes_a_double_click(
                            &surface,
                            now,
                            at,
                            window,
                            self.how_this_window_is(&surface),
                        )
                    });
                    if twice {
                        self.the_press_before = None;
                        self.asked_on_an_edge
                            .push((surface, AskedOnTheBand::MaximiseOrRestore));
                        return true;
                    }
                    self.the_press_before = Some(ThePressBefore {
                        surface: surface.clone(),
                        at: now,
                        where_: at,
                        window,
                        how: self.how_this_window_is(&surface),
                    });
                    return self.surfaces.start_move_from_the_name(&surface, button);
                }
            }
        }
        false
    }

    /// Carry out what presses on bands asked for, once the input batch is
    /// done.
    pub(crate) fn meet_what_was_pressed_on_an_edge(&mut self) {
        let asked = std::mem::take(&mut self.asked_on_an_edge);
        for (surface, what) in asked {
            match what {
                AskedOnTheBand::Control(OnTheEdge::Minimise) => {
                    self.the_person_asked_to_put_aside(&surface);
                }
                AskedOnTheBand::Control(OnTheEdge::Close) => {
                    let _ = self.request_window_close(&surface);
                }
                AskedOnTheBand::Control(OnTheEdge::WindowActions) => {
                    self.the_menu_was_opened_on(&surface, ByWhat::ThePointer);
                }
                AskedOnTheBand::OpenThePanel => {
                    if self.whose_panel_is_open().as_ref() != Some(&surface) {
                        self.the_menu_was_opened_on(&surface, ByWhat::ThePointer);
                    }
                }
                AskedOnTheBand::MaximiseOrRestore => {
                    let maximised = self.surfaces.requested_window_mode(&surface)
                        == crate::window_mode::Mode::Maximized;
                    if self.set_window_maximized(&surface, !maximised).is_err() {
                        self.a_window_panel_choice_was_refused();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "a_press_on_an_edge_tests.rs"]
mod tests;
