//! **The keyboard on a window's band and in its panel.**
//!
//! `docs/design/the-window-controls.md` §9: *Escape dismisses the deepest layer
//! first; focus returns to the opener; arrows and Enter or Space; visible
//! focus.* `crate::window_edge_keys` has been that machine since 2026-10-11 and
//! `crate::the_edge_takes_the_keyboard` held one per window — and **no key ever
//! reached either**: `Alt+F3` opened the menu and every key after it went to the
//! application (`crates/alo-reached/what-nothing-calls.txt` listed
//! `this_windows_edge_was_told_a_key`). This file is that road.
//!
//! # The road in, and the layers out
//!
//! `Alt+F3` (*Show the window's options*) opens the panel of the window the
//! keyboard is in, **with the keyboard on its first choice** (Left), so it is
//! somewhere a person can see — a pointer opens it on none. Then:
//!
//! | key | in the panel | on the band |
//! |---|---|---|
//! | Right, Down | the next choice | the next control |
//! | Left, Up | the one before | the one before |
//! | Home, End | the first, the last | the first, the last |
//! | Enter, Space | do it; the panel closes and focus goes back to Window actions | press it |
//! | Escape | close the panel; focus goes back to Window actions | leave the band |
//!
//! **Tab is not taken**: it is the shell's key between surfaces, and taking
//! it inside the panel is a decision about the whole focus order that
//! `crate::window_menu_keys` already declined to make here.
//!
//! **Any other key is the application's**, so a person who opened the panel by
//! mistake and starts typing loses nothing. A key's release always goes on to
//! the application as it would have, which is the arrangement every surface
//! that takes keys here makes (`crate::a_key_reaches_the_docks_surfaces`).

use smithay::backend::input::KeyState;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::window_edge::OnTheEdge;
use crate::window_edge_keys::{AnEdgeKey, TheKeyboardOnTheEdge, WhatTheEdgeKeyDid};
use crate::{InputError, InputUpdate, Server};

/// The evdev codes this road reads.
mod codes {
    /// Escape.
    pub(super) const ESCAPE: u32 = 1;
    /// Enter.
    pub(super) const ENTER: u32 = 28;
    /// Space.
    pub(super) const SPACE: u32 = 57;
    /// Enter on the number pad.
    pub(super) const PAD_ENTER: u32 = 96;
    /// Home.
    pub(super) const HOME: u32 = 102;
    /// Up.
    pub(super) const UP: u32 = 103;
    /// Left.
    pub(super) const LEFT: u32 = 105;
    /// Right.
    pub(super) const RIGHT: u32 = 106;
    /// End.
    pub(super) const END: u32 = 107;
    /// Down.
    pub(super) const DOWN: u32 = 108;
}

/// What a key means on a band or in its panel, or [`None`] for a key that is
/// the application's.
#[must_use]
pub(crate) const fn which_key(code: u32) -> Option<AnEdgeKey> {
    match code {
        codes::ESCAPE => Some(AnEdgeKey::Leave),
        codes::RIGHT | codes::DOWN => Some(AnEdgeKey::Next),
        codes::LEFT | codes::UP => Some(AnEdgeKey::Previous),
        codes::HOME => Some(AnEdgeKey::First),
        codes::END => Some(AnEdgeKey::Last),
        codes::ENTER | codes::PAD_ENTER | codes::SPACE => Some(AnEdgeKey::Press),
        _ => None,
    }
}

impl Server {
    /// Whether some window's band or panel has the keyboard.
    #[must_use]
    pub fn a_windows_band_is_taking_the_keys(&self) -> bool {
        self.keyboard_on_edges
            .iter()
            .any(|(_, where_it_is)| *where_it_is != TheKeyboardOnTheEdge::Nowhere)
    }

    /// The window whose band has the keyboard: its panel's first, if one is
    /// open.
    fn whose_band_has_the_keys(&self) -> Option<WlSurface> {
        self.whose_panel_is_open().or_else(|| {
            self.keyboard_on_edges
                .iter()
                .find(|(_, where_it_is)| *where_it_is != TheKeyboardOnTheEdge::Nowhere)
                .map(|(surface, _)| surface.clone())
        })
    }

    /// A key, told to the band that has the keyboard: what it did.
    pub(crate) fn a_key_on_a_band(&mut self, key: AnEdgeKey) -> WhatTheEdgeKeyDid {
        let Some(surface) = self.whose_band_has_the_keys() else {
            return WhatTheEdgeKeyDid::Nothing;
        };
        let Some((edge, menu)) = self.the_band_and_its_panel(&surface) else {
            return WhatTheEdgeKeyDid::Nothing;
        };
        let did = self.this_windows_edge_was_told_a_key(&surface, key, &edge, menu.as_ref());
        match did {
            WhatTheEdgeKeyDid::ItRuns(OnTheEdge::Minimise) => {
                self.the_person_asked_to_put_aside(&surface);
            }
            WhatTheEdgeKeyDid::ItRuns(OnTheEdge::Close) => {
                let _ = self.request_window_close(&surface);
            }
            WhatTheEdgeKeyDid::ARowRuns(does) => self.carry_out_the_row(&surface, does),
            WhatTheEdgeKeyDid::ItRuns(OnTheEdge::WindowActions)
            | WhatTheEdgeKeyDid::TheKeyboardIsOn(_)
            | WhatTheEdgeKeyDid::TheMenuOpened
            | WhatTheEdgeKeyDid::TheKeyboardIsOnARow(_)
            | WhatTheEdgeKeyDid::TheMenuClosed
            | WhatTheEdgeKeyDid::TheKeyboardLeft
            | WhatTheEdgeKeyDid::Nothing => {}
        }
        did
    }

    /// This window's band, laid out with its controls shown — the keyboard
    /// holds them shown — and its panel, if it is open.
    fn the_band_and_its_panel(
        &self,
        surface: &WlSurface,
    ) -> Option<(
        crate::window_edge::WindowEdge,
        Option<crate::window_menu::WindowMenu>,
    )> {
        if let Some((edge, menu)) = self.the_open_panel(surface)
            && self.whose_panel_is_open().as_ref() == Some(surface)
        {
            return Some((edge, Some(menu)));
        }
        let scale = self.the_scale_an_edge_is_measured_at();
        let edge = crate::window_edge::edge_of(
            self.the_window_a_band_hangs_from(surface, scale),
            crate::window_edge_who_draws::who_draws_a_frame(),
            true,
            self.the_room_an_edge_is_clamped_in(),
        );
        Some((edge, None))
    }
}

/// One input event, while a window's band or panel has the keyboard.
pub(crate) fn a_key_reaches_a_windows_band(
    server: &mut Server,
    update: InputUpdate<'_>,
    extent: (i32, i32),
) -> Result<(), InputError> {
    let InputUpdate::Event(event) = update else {
        return server.libinput_update(update, extent);
    };
    let Some(crate::DirectSeatEvent::Key(key, _)) = crate::libinput_routing::translate(event)?
    else {
        return server.libinput_update(InputUpdate::Event(event), extent);
    };
    if key.state != KeyState::Pressed {
        return server.libinput_update(InputUpdate::Event(event), extent);
    }
    let Some(asked) = which_key(key.code) else {
        return server.libinput_update(InputUpdate::Event(event), extent);
    };
    match server.a_key_on_a_band(asked) {
        // **A key that did nothing here goes on**, so the band never swallows
        // a key it had no use for.
        WhatTheEdgeKeyDid::Nothing => server.libinput_update(InputUpdate::Event(event), extent),
        _ => Ok(()),
    }
}

#[cfg(test)]
#[path = "a_key_reaches_a_windows_band_tests.rs"]
mod tests;
