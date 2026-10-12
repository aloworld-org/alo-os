//! Act on a press that landed in an open window menu.
//!
//! The third of three things a menu needs to be a menu: it is laid out
//! (`crate::window_menu`), it is drawn (`crate::window_menu_paint`), and a
//! press on a row has to do what the row says. Without this the panel appears
//! and clicking it does nothing, which is a worse state than not drawing it.
//!
//! # Asked before the edge, because it is in front of it
//!
//! The menu opens eight units below the edge and overlaps the application, so
//! a press inside it was never a press on the edge or on the window. The same
//! rule `crate::pointer` already applies to the resize bands and the name
//! band: **the more specific surface answers first**, and the menu is the most
//! specific thing on the screen while it is open.
//!
//! # A press outside it closes it
//!
//! Which is what every menu a person has used does, and the only way out by
//! pointer that does not require knowing a key. The press that closes it is
//! **consumed** rather than passed on: a person dismissing a menu did not also
//! mean to click what was behind it.
//!
//! # The rows come from the same layout the draw used
//!
//! `the_menu_of` is called here with the same arguments the frame draws from.
//! Two layouts would mean a press landing where no row was drawn, which is the
//! fault `crate::the_edge_takes_the_keyboard` records for the edge and is why
//! that file takes the laid-out edge from its caller rather than building one.

use smithay::backend::input::ButtonState;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::window_edge::OnTheEdge;
use crate::window_edge_keys::TheKeyboardOnTheEdge;
use crate::window_menu::{THE_TITLE_BLOCK_IS_TALL, WhatARowDoes, WhatTheWindowAllows, the_menu_of};

impl crate::Server {
    /// Act on a press in whichever window menu is open.
    ///
    /// Answers whether the press was the menu's, in which case **nothing is
    /// delivered to the client**: the menu is the shell's own surface and no
    /// client was ever told about it.
    ///
    /// A press on a row that this window allows acts and closes the menu. A
    /// press on a row it does not allow is consumed and the menu stays open,
    /// so a person sees that the row they pressed is the muted one rather
    /// than having the menu vanish as though something happened. A press
    /// anywhere else closes it.
    pub(crate) fn a_press_in_the_menu(&mut self, state: ButtonState) -> bool {
        if state != ButtonState::Pressed {
            return false;
        }
        let Some(at) = self.where_the_pointer_is_in_logical_pixels() else {
            // **No pointer is not the origin**, the rule every other reader of
            // the pointer in this crate carries.
            return false;
        };
        let Some((surface, row)) = self.what_an_open_menu_has_under(at) else {
            return false;
        };

        match row {
            Some(row) => {
                if let Some(does) = self.a_row_of_the_menu_was_pressed(&surface, row) {
                    self.carry_out_the_row(&surface, does);
                }
            }
            // Outside the panel: dismissed, and the keyboard goes back to the
            // control that opened it, exactly as Escape does.
            None => self.remember_the_menu_closed(&surface),
        }
        true
    }

    /// Which window's menu is open, and what is under this position in it.
    ///
    /// The outer [`Option`] is *is a menu open at all*; the inner is *is this
    /// position on one of its rows*, where [`None`] means inside no row —
    /// either the title block or outside the panel entirely.
    fn what_an_open_menu_has_under(
        &self,
        at: smithay::utils::Point<i32, smithay::utils::Logical>,
    ) -> Option<(WlSurface, Option<crate::window_menu::MenuRow>)> {
        let open: Vec<WlSurface> = self
            .keyboard_on_edges
            .iter()
            .filter(|(_, where_it_is)| matches!(where_it_is, TheKeyboardOnTheEdge::InTheMenu(_)))
            .map(|(surface, _)| surface.clone())
            .collect();
        for surface in open {
            let edge = crate::window_edge::edge_of(
                crate::where_a_window_is(&surface),
                crate::window_edge_who_draws::who_draws_a_frame(),
                true,
                self.the_room_an_edge_is_clamped_in(),
            );
            let Some(menu) = the_menu_of(
                &edge,
                WhatTheWindowAllows::everything(),
                self.surfaces.has_window_mode(&surface),
                THE_TITLE_BLOCK_IS_TALL,
                self.the_room_an_edge_is_clamped_in(),
            ) else {
                continue;
            };
            return Some((surface, menu.what_is_under(at)));
        }
        None
    }

    /// The menu closed with nothing run.
    fn remember_the_menu_closed(&mut self, surface: &WlSurface) {
        self.keyboard_on_edges
            .iter_mut()
            .filter(|(it, _)| it == surface)
            .for_each(|(_, where_it_is)| {
                *where_it_is = TheKeyboardOnTheEdge::OnAControl(OnTheEdge::Menu);
            });
    }

    /// Do what a row says, through the mechanism that already does its job.
    ///
    /// **The same mechanisms an edge press reaches**, which is
    /// `crate::a_press_on_an_edge`'s rule and the reason this is four lines
    /// rather than four implementations: a window put aside from the menu and
    /// one put aside from the edge must arrive at the same queue having been
    /// through the same rules.
    ///
    /// Move and Resize answer nothing here. They begin an interaction the
    /// pointer must then carry, and no such gesture is begun from a menu row
    /// yet — recorded rather than faked, which is what
    /// `WhatARowDoes::what_it_does` says of the same two.
    fn carry_out_the_row(&mut self, surface: &WlSurface, does: WhatARowDoes) {
        match does {
            WhatARowDoes::Minimise => self.asked_to_put_aside.push(surface.clone()),
            WhatARowDoes::Maximise | WhatARowDoes::Restore => {
                let already = self.surfaces.has_window_mode(surface);
                // A client that refuses to be maximised is not an input error,
                // and what a person sees is the window not moving.
                let _ = self.set_window_maximized(surface, !already);
            }
            WhatARowDoes::Close => {
                let _ = self.request_window_close(surface);
            }
            WhatARowDoes::Move | WhatARowDoes::Resize => {}
        }
    }
}

#[cfg(test)]
#[path = "a_press_in_the_menu_tests.rs"]
mod tests;
