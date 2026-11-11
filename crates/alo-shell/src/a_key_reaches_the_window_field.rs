//! **Keys typed into a window panel's field**, *alo · Ask about this window…*.
//!
//! `crate::a_key_reaches_a_windows_band` walks the panel with the arrows, by
//! evdev code. A field needs letters in the person's own layout, so while the
//! keyboard is in it (`Server::whose_window_field_has_the_keyboard`) every key
//! goes through the seat's XKB state instead — `Server::the_bars_key`'s road,
//! line for line — and is **intercepted**: the application behind the panel
//! hears nothing of what a person types about it.
//!
//! | key | in the field |
//! |---|---|
//! | letters, Space | typed at the caret |
//! | Left, Right, Home, End | move the caret |
//! | Backspace, Delete | take a letter back |
//! | Enter | send to alo, with this window, read now |
//! | Escape | clear what is typed; with nothing typed, close the panel |
//! | Up, Down | leave the field for the choice above or below |
//!
//! **Sending** is `Server::a_person_asked_alo_about_the_window`: the window's
//! title and application read at Enter and never before, the request recorded
//! once. Then the panel closes and the field empties. **A window that has gone
//! sends nothing and keeps the words**, so no request leaves without the
//! window it was about. Where the answer is shown is the task window the
//! request opens, until the answer card under the panel is drawn
//! (`docs/design/the-window-controls.md`, stage 3b).

use smithay::input::keyboard::{FilterResult, Keysym, ModifiersState};
use smithay::{backend::input::KeyState, utils::SERIAL_COUNTER};

use crate::the_edge_takes_the_keyboard::ByWhat;
use crate::the_window_panels_field::{AFieldKey, WhatTheFieldDid};
use crate::window_edge_keys::AnEdgeKey;
use crate::{InputError, Server};

impl AFieldKey {
    /// What a symbol means in the field, with the modifiers held.
    ///
    /// **Enter with Shift is nothing**, not a line: the field is one line, and
    /// a person who meant a new line has not meant to send. A letter held with
    /// Control, Alt or the logo key is a chord and types nothing.
    #[must_use]
    pub fn of(symbol: Keysym, held: &ModifiersState) -> Self {
        match symbol {
            Keysym::BackSpace => return Self::Erase,
            Keysym::Delete | Keysym::KP_Delete => return Self::EraseForwards,
            Keysym::Escape => return Self::Escape,
            Keysym::Left | Keysym::KP_Left => return Self::Left,
            Keysym::Right | Keysym::KP_Right => return Self::Right,
            Keysym::Home | Keysym::KP_Home => return Self::Start,
            Keysym::End | Keysym::KP_End => return Self::End,
            Keysym::Up | Keysym::KP_Up => return Self::Up,
            Keysym::Down | Keysym::KP_Down => return Self::Down,
            Keysym::Return | Keysym::KP_Enter => {
                return if held.shift { Self::Nothing } else { Self::Send };
            }
            _ => {}
        }
        if held.ctrl || held.alt || held.logo {
            return Self::Nothing;
        }
        match symbol.key_char() {
            Some(letter) if !letter.is_control() => Self::Letter(letter),
            _ => Self::Nothing,
        }
    }
}

impl Server {
    /// What one key means to the window field, or [`None`] where it means
    /// nothing — a release, or a repeat of a key already down.
    ///
    /// # Errors
    /// [`InputError::Unavailable`] for a display bound without a keyboard, and
    /// [`InputError::InvalidKey`] for a code outside evdev's range.
    pub fn the_window_fields_key(
        &mut self,
        code: u32,
        state: KeyState,
        time: u32,
    ) -> Result<Option<AFieldKey>, InputError> {
        if !(1..=0x2ff).contains(&code) {
            return Err(InputError::InvalidKey);
        }
        let handle = self
            .surfaces
            .keyboard
            .as_ref()
            .ok_or(InputError::Unavailable)?
            .handle
            .clone();
        let code = (code + 8).into();
        if handle.pressed_keys().contains(&code) == (state == KeyState::Pressed) {
            return Ok(None);
        }
        let meant = handle.input::<Option<AFieldKey>, _>(
            &mut self.surfaces,
            code,
            state,
            SERIAL_COUNTER.next_serial(),
            time,
            |_, held, symbol| {
                FilterResult::Intercept(
                    (state == KeyState::Pressed).then(|| AFieldKey::of(symbol.modified_sym(), held)),
                )
            },
        );
        if let Some(keyboard) = self.surfaces.keyboard.as_mut() {
            keyboard.time = time;
        }
        Ok(meant.flatten())
    }

    /// **One key in the window field that has the keyboard**, and what it
    /// did. Nothing at all when no field has it.
    pub fn a_key_reached_the_window_field(&mut self, key: AFieldKey) -> WhatTheFieldDid {
        let Some(surface) = self.whose_window_field_has_the_keyboard() else {
            return WhatTheFieldDid::Nothing;
        };
        let did = self
            .alo_in_a_window
            .the_field_to_type_in(&surface)
            .pressed(key);
        match &did {
            WhatTheFieldDid::SendThis(request) => {
                if !self.a_person_asked_alo_about_the_window(&surface, request) {
                    // **Nothing left, and the words stay**, to send again or
                    // not once the person sees why.
                    return WhatTheFieldDid::Nothing;
                }
                self.alo_in_a_window.the_field_to_type_in(&surface).emptied();
                self.the_panel_closes_and_the_band_lets_go(&surface);
            }
            WhatTheFieldDid::CloseThePanel => {
                self.the_menu_was_dismissed_on(&surface, ByWhat::TheKeyboard);
            }
            WhatTheFieldDid::Leave(AFieldKey::Up) => {
                let _ = self.a_key_on_a_band(AnEdgeKey::Previous);
            }
            WhatTheFieldDid::Leave(_) => {
                let _ = self.a_key_on_a_band(AnEdgeKey::Next);
            }
            WhatTheFieldDid::Nothing | WhatTheFieldDid::ItChanged => {}
        }
        did
    }
}

/// One input event, while a window panel's field has the keyboard: every key
/// is the field's, and none reaches the application.
pub(crate) fn a_key_reaches_the_window_field(
    server: &mut Server,
    key: &crate::DirectKeyEvent,
) -> Result<(), InputError> {
    match server.the_window_fields_key(key.code, key.state, key.time) {
        Ok(Some(meant)) => {
            let _ = server.a_key_reached_the_window_field(meant);
            Ok(())
        }
        Ok(None) | Err(InputError::InvalidKey) => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
#[path = "a_key_reaches_the_window_field_tests.rs"]
mod tests;
