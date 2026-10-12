//! What one key press means to *More apps*' search field.
//!
//! The seat's XKB state has already turned the press into a symbol with this
//! person's layout and modifiers applied; this is the short list of what a
//! search field does with one. The shape is `crate::sign_in_keys`' exactly,
//! and deliberately so — two fields on one machine that disagreed about what
//! Backspace does would be two fields a person has to learn separately.
//!
//! # Why this is not in `crate::a_key_reaches_the_docks_surfaces`
//!
//! That file's own rule: *these are the codes libinput reports, before any
//! layout is applied. Every one of them sits in the same place on every
//! layout, which is why a shell may act on the code — and why nothing that
//! moves between layouts may be added here. **A letter key is a layout's
//! business and never this file's.***
//!
//! A letter is exactly that. So the Dock's surfaces keep taking Escape, the
//! arrows and Enter by code, and anything a person *types* arrives here with
//! the layout already applied. A French keyboard's `a` is the same key as a
//! British keyboard's `q`, and only one of those two roads knows it.

use smithay::backend::input::KeyState;
use smithay::input::keyboard::{Keysym, ModifiersState};

/// One key press, as the search field understands it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ASearchKey {
    /// A letter to add to what is typed.
    Letter(char),
    /// Take the last letter back.
    Erase,
    /// Anything else, which the field does not take.
    ///
    /// **Including Escape, Enter and the arrows**, which never reach here:
    /// they are taken by code before the layout is applied, and they mean what
    /// they meant before this field could be typed into. A search field that
    /// swallowed Escape would be one a person cannot close.
    Nothing,
}

impl ASearchKey {
    /// What a symbol means to the field, with the modifiers that were held.
    ///
    /// **A letter typed with Control, Alt or the logo key held is nothing.**
    /// Those are chords — a person pressing `Ctrl`+`W` means *close* and not
    /// *type a w*, and a search field that took the letter would filter the
    /// grid under them while they waited for something else to happen.
    #[must_use]
    pub(crate) fn of(symbol: Keysym, held: &ModifiersState) -> Self {
        if symbol == Keysym::BackSpace {
            return Self::Erase;
        }
        if held.ctrl || held.alt || held.logo {
            return Self::Nothing;
        }
        match symbol.key_char() {
            // **A control character is refused here and again in the model.**
            // Two places on purpose: this one knows about keyboards and that
            // one is handed characters by whoever wires a seat next.
            Some(letter) if !letter.is_control() => Self::Letter(letter),
            _ => Self::Nothing,
        }
    }
}

/// Let this key reach the overview's search field, if it is a letter.
///
/// **Through the seat, not around it**, which is what applies this person's
/// layout: the same `Server::desktop_key` road the sign-in screen and the
/// chords take, so one machine cannot disagree with itself about which key a
/// person pressed depending on what is on the screen.
///
/// A key the field does not take goes on to the application exactly as it did
/// before this file existed. **That includes every key while nothing is typed
/// and nothing matches** — the field narrows a list, it does not capture a
/// keyboard.
///
/// # Errors
/// Whatever the seat refuses about the update it forwards.
pub(crate) fn a_key_types_into_the_search_field(
    server: &mut crate::Server,
    event: &smithay::reexports::input::Event,
    extent: (i32, i32),
    code: u32,
) -> Result<(), crate::InputError> {
    let meant = server.the_docks_search_key(code, KeyState::Pressed, 0)?;
    let changed = match meant {
        Some(ASearchKey::Letter(letter)) => server.what_was_typed_into_more_apps.typed(letter),
        Some(ASearchKey::Erase) => server.what_was_typed_into_more_apps.erased(),
        Some(ASearchKey::Nothing) | None => {
            return server.libinput_update(crate::InputUpdate::Event(event), extent);
        }
    };
    // **Taken whether or not it changed anything.** A letter past the field's
    // bound, and a Backspace on an empty field, are both keys a person aimed
    // at the search field: passing them on would send a letter to the
    // application behind a popover the person is looking at.
    let _redraw = changed;
    Ok(())
}

#[cfg(test)]
#[path = "a_key_types_into_more_apps_tests.rs"]
mod tests;
