//! Keystrokes for Settings, through the seat this compositor already has.
//!
//! While Settings is open every key is **intercepted** at the seat and never
//! forwarded: an application behind the window hears neither the arrows that
//! move through Settings nor the chord a person is binding. Keyboard focus is
//! left where it is, as it is for the record window.

use smithay::{backend::input::KeyState, input::keyboard::FilterResult, utils::SERIAL_COUNTER};

use crate::settings_chord::chord_of;
use crate::{InputError, Server, SettingsKey, SettingsPress};

impl Server {
    /// One key from a trusted backend, as Settings understands it.
    ///
    /// Takes a Linux evdev code, like [`Server::keyboard_key`], and updates the
    /// seat's XKB state with it. Returns [`None`] for a release, a repeated
    /// press of a key already down, and an unmatched release. Nothing is
    /// forwarded to any client, whatever this returns.
    ///
    /// # Errors
    /// [`InputError::InvalidKey`] for a code outside evdev's range, and
    /// [`InputError::Unavailable`] for a display bound without a keyboard.
    pub fn settings_key(
        &mut self,
        code: u32,
        state: KeyState,
        time: u32,
    ) -> Result<Option<SettingsPress>, InputError> {
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
        let meant = handle.input::<Option<SettingsPress>, _>(
            &mut self.surfaces,
            code,
            state,
            SERIAL_COUNTER.next_serial(),
            time,
            |_, held, symbol| {
                FilterResult::Intercept((state == KeyState::Pressed).then(|| {
                    let pressed = symbol.modified_sym();
                    let unshifted = symbol.raw_latin_sym_or_raw_current_sym().unwrap_or(pressed);
                    SettingsPress {
                        key: SettingsKey::of(pressed, held),
                        chord: chord_of(unshifted, held),
                    }
                }))
            },
        );
        if let Some(keyboard) = self.surfaces.keyboard.as_mut() {
            keyboard.time = time;
        }
        Ok(meant.flatten())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_shortcuts::{Key, Modifier, Modifiers};
    use smithay::input::keyboard::XkbConfig;
    use std::os::unix::fs::PermissionsExt;

    /// evdev codes, as a backend hands them over.
    const KEY_T: u32 = 20;
    /// Left Control.
    const KEY_LEFTCTRL: u32 = 29;
    /// Left Alt.
    const KEY_LEFTALT: u32 = 56;
    /// Down.
    const KEY_DOWN: u32 = 108;
    /// Enter.
    const KEY_ENTER: u32 = 28;

    /// A display with a keyboard on a US layout, in a private directory.
    fn seat() -> (tempfile::TempDir, Server) {
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let server = Server::bind_keyboard(
            runtime.path(),
            "settings",
            XkbConfig {
                layout: "us",
                ..XkbConfig::default()
            },
        )
        .unwrap();
        (runtime, server)
    }

    /// A press and its release, returning what the press meant.
    fn tapped(server: &mut Server, code: u32) -> Option<SettingsPress> {
        let meant = server.settings_key(code, KeyState::Pressed, 1).unwrap();
        assert_eq!(
            server.settings_key(code, KeyState::Released, 2).unwrap(),
            None
        );
        meant
    }

    /// **Keys arrive through the seat's own modifiers**: Down moves, Enter
    /// chooses, and Control+Alt+T is nothing to the list and the chord it is
    /// to a shortcut.
    #[test]
    fn keys_arrive_through_the_seats_modifiers() {
        let (_runtime, mut server) = seat();
        let down = tapped(&mut server, KEY_DOWN).unwrap();
        assert_eq!(down.key, SettingsKey::Next);
        assert_eq!(down.chord, Some((Modifiers::none(), Key::Down)));
        assert_eq!(
            tapped(&mut server, KEY_ENTER).unwrap().key,
            SettingsKey::Choose
        );

        server
            .settings_key(KEY_LEFTCTRL, KeyState::Pressed, 3)
            .unwrap();
        server
            .settings_key(KEY_LEFTALT, KeyState::Pressed, 4)
            .unwrap();
        let chord = tapped(&mut server, KEY_T).unwrap();
        assert_eq!(chord.key, SettingsKey::Nothing);
        assert_eq!(
            chord.chord,
            Some((Modifiers::just(Modifier::Ctrl).and(Modifier::Alt), Key::T))
        );
    }

    /// **A code outside evdev's range is refused**, and so is a display with no
    /// keyboard, rather than a window that silently takes no keys.
    #[test]
    fn a_strange_code_and_a_missing_keyboard_are_refused() {
        let (_runtime, mut server) = seat();
        for code in [0, 0x300] {
            assert!(matches!(
                server.settings_key(code, KeyState::Pressed, 1),
                Err(InputError::InvalidKey)
            ));
        }
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut bare = Server::bind(runtime.path(), "no-keyboard").unwrap();
        assert!(matches!(
            bare.settings_key(KEY_ENTER, KeyState::Pressed, 1),
            Err(InputError::Unavailable)
        ));
    }
}
