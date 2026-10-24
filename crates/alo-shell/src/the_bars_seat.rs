//! Keystrokes for the alo Bar, through the seat this compositor already has.
//!
//! **The Bar could not be typed into on a running machine.** Measured
//! 2026-10-23 on a booted machine: `Super`+`A` was pressed, the chord fired,
//! `TheComposer::it_took_the_keyboard` ran — and then twenty-nine characters
//! were typed and the field stayed empty, because
//! `Server::a_key_reached_the_composer` had **no production caller at all**.
//! Eighteen mentions of it across the workspace, every one of them in a test.
//!
//! This is the half that was missing, and `crate::a_key_reaches_settings`
//! names the identical fault being fixed once before: *every link below this
//! one was finished and tested, and nothing on a running machine called
//! either.* Settings got its entrance then; the Bar gets one here.
//!
//! # Intercepted, exactly as Settings is
//!
//! While the Bar holds the keyboard every key is **intercepted** at the seat
//! and never forwarded, so an application behind it hears nothing of what a
//! person is typing into the Bar. Keyboard focus is left where it is, for
//! `crate::settings_seat`'s reason: interception is what stops the client
//! hearing, so clearing focus would buy nothing and cost two events.

use smithay::{backend::input::KeyState, input::keyboard::FilterResult, utils::SERIAL_COUNTER};

use crate::composer_keys::{ComposerKey, Composing};
use crate::{InputError, Server};

impl Server {
    /// What one key means to the Bar, or [`None`] where it means nothing.
    ///
    /// **Built from `Server::settings_key`, line for line**, because the two
    /// answer the same question about the same seat and a second reading of a
    /// keyboard is a second thing to get wrong. What differs is only the
    /// vocabulary the symbol is turned into.
    ///
    /// # Errors
    /// [`InputError::Unavailable`] for a display bound without a keyboard, and
    /// [`InputError::InvalidKey`] for a code outside evdev's range — which the
    /// caller treats as *nothing happened* rather than as a reason to take a
    /// person's desktop away.
    pub fn the_bars_key(
        &mut self,
        code: u32,
        state: KeyState,
        time: u32,
    ) -> Result<Option<ComposerKey>, InputError> {
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
        // A release, and a repeat of a key already down: neither is a press,
        // and the Bar acts on presses.
        if handle.pressed_keys().contains(&code) == (state == KeyState::Pressed) {
            return Ok(None);
        }
        let meant = handle.input::<Option<ComposerKey>, _>(
            &mut self.surfaces,
            code,
            state,
            SERIAL_COUNTER.next_serial(),
            time,
            |_, held, symbol| {
                FilterResult::Intercept((state == KeyState::Pressed).then(|| {
                    // **`Composing::No`, and said rather than assumed.** This
                    // compositor runs no input method, so nothing is ever
                    // mid-composition here. The day one is added this is the
                    // line that has to change, and it is one line because
                    // `ComposerKey::of` already takes the question.
                    ComposerKey::of(symbol.modified_sym(), held, Composing::No)
                }))
            },
        );
        if let Some(keyboard) = self.surfaces.keyboard.as_mut() {
            keyboard.time = time;
        }
        Ok(meant.flatten())
    }

    /// Whether the Bar is the thing taking keys right now.
    ///
    /// **One flag and nothing else** — no window, no clock, no places. The
    /// same question `Self::settings_is_taking_the_keys` answers for Settings,
    /// and asked in the same place for the same reason.
    pub fn the_bar_is_taking_the_keys(&self) -> bool {
        self.the_composer.has_the_keyboard()
    }
}
