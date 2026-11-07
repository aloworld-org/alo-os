//! Keystrokes for the record window, through the seat this compositor already
//! has.
//!
//! While the window is open every key is **intercepted** at the seat and never
//! forwarded: an application behind the window does not hear the arrows that
//! move through the account or the Escape that closes it. Keyboard focus is
//! left where it is, as it is for the approval surface — the window a person
//! was working in is theirs again the moment the record window closes.

use smithay::{backend::input::KeyState, input::keyboard::FilterResult, utils::SERIAL_COUNTER};

use crate::{InputError, RecordKey, Server};

impl Server {
    /// One key from a trusted backend, as the record window understands it.
    ///
    /// Takes a Linux evdev code, like [`Server::keyboard_key`], and updates the
    /// seat's XKB state with it. Returns [`None`] for a release, a repeated
    /// press of a key already down, and an unmatched release. Nothing is
    /// forwarded to any client, whatever this returns.
    ///
    /// # Errors
    /// [`InputError::InvalidKey`] for a code outside evdev's range, and
    /// [`InputError::Unavailable`] for a display bound without a keyboard.
    pub fn record_key(
        &mut self,
        code: u32,
        state: KeyState,
        time: u32,
    ) -> Result<Option<RecordKey>, InputError> {
        Ok(self
            .what_a_key_is_to_the_record(code, state, time)?
            .map(|meant| match meant {
                ToTheRecord::AKey(key) => key,
                ToTheRecord::AChord(_) => RecordKey::Nothing,
            }))
    }

    /// One key, as the record window understands it **or as a chord the person
    /// has bound** (ADR 0145, stage 4): a chord with Control, Alt or the logo
    /// key held means nothing to the record, and on the canvas it reaches its
    /// action — so Super+Tab leaves the record as it leaves any window.
    ///
    /// # Errors
    /// As [`Self::record_key`].
    pub(crate) fn what_a_key_is_to_the_record(
        &mut self,
        code: u32,
        state: KeyState,
        time: u32,
    ) -> Result<Option<ToTheRecord>, InputError> {
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
        let shortcuts = self.shortcuts.take();
        let meant = handle.input::<Option<ToTheRecord>, _>(
            &mut self.surfaces,
            code,
            state,
            SERIAL_COUNTER.next_serial(),
            time,
            |_, held, symbol| {
                FilterResult::Intercept((state == KeyState::Pressed).then(|| {
                    let key = RecordKey::of(symbol.modified_sym(), held);
                    match (
                        key,
                        Server::the_chord_a_key_makes(shortcuts.as_ref(), &symbol, held),
                    ) {
                        (RecordKey::Nothing, Some(chord)) => ToTheRecord::AChord(chord),
                        _ => ToTheRecord::AKey(key),
                    }
                }))
            },
        );
        self.shortcuts = shortcuts;
        if let Some(keyboard) = self.surfaces.keyboard.as_mut() {
            keyboard.time = time;
        }
        Ok(meant.flatten())
    }
}

/// What one key is to the record window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToTheRecord {
    /// A key the window understands, or [`RecordKey::Nothing`].
    AKey(RecordKey),
    /// A chord the person bound, which means nothing to the window.
    AChord(alo_shortcuts::Chord),
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use smithay::input::keyboard::XkbConfig;
    use std::os::unix::fs::PermissionsExt;

    /// evdev codes, as a backend hands them over.
    const KEY_F: u32 = 33;
    /// Left Control.
    const KEY_LEFTCTRL: u32 = 29;
    /// Down.
    const KEY_DOWN: u32 = 108;
    /// Escape.
    const KEY_ESC: u32 = 1;

    /// A display with a keyboard on a US layout, in a private directory.
    fn seat() -> (tempfile::TempDir, Server) {
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let server = Server::bind_keyboard(
            runtime.path(),
            "record",
            XkbConfig {
                layout: "us",
                ..XkbConfig::default()
            },
        )
        .unwrap();
        (runtime, server)
    }

    /// A press and its release, returning what the press meant.
    fn tapped(server: &mut Server, code: u32) -> Option<RecordKey> {
        let meant = server.record_key(code, KeyState::Pressed, 1).unwrap();
        assert_eq!(
            server.record_key(code, KeyState::Released, 2).unwrap(),
            None
        );
        meant
    }

    /// **Keys arrive through the seat's own modifiers**: Down moves and Escape
    /// closes, a letter is nothing, and Control+F — a search elsewhere — is
    /// nothing here.
    #[test]
    fn keys_arrive_through_the_seats_modifiers() {
        let (_runtime, mut server) = seat();
        assert_eq!(tapped(&mut server, KEY_DOWN), Some(RecordKey::Older));
        assert_eq!(tapped(&mut server, KEY_ESC), Some(RecordKey::Close));
        assert_eq!(tapped(&mut server, KEY_F), Some(RecordKey::Nothing));
        server
            .record_key(KEY_LEFTCTRL, KeyState::Pressed, 3)
            .unwrap();
        assert_eq!(tapped(&mut server, KEY_F), Some(RecordKey::Nothing));
        server
            .record_key(KEY_LEFTCTRL, KeyState::Released, 4)
            .unwrap();
    }

    /// **A key held down moves once.** The repeated press of a key already down
    /// means nothing.
    #[test]
    fn a_key_held_down_moves_once() {
        let (_runtime, mut server) = seat();
        assert_eq!(
            server.record_key(KEY_DOWN, KeyState::Pressed, 1).unwrap(),
            Some(RecordKey::Older)
        );
        for time in 2..6 {
            assert_eq!(
                server
                    .record_key(KEY_DOWN, KeyState::Pressed, time)
                    .unwrap(),
                None
            );
        }
    }

    /// **A code outside evdev's range is refused**, and so is a display with no
    /// keyboard, rather than a window that silently takes no keys.
    #[test]
    fn a_strange_code_and_a_missing_keyboard_are_refused() {
        let (_runtime, mut server) = seat();
        for code in [0, 0x300, u32::MAX - 8] {
            assert!(matches!(
                server.record_key(code, KeyState::Pressed, 1),
                Err(InputError::InvalidKey)
            ));
        }
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut bare = Server::bind(runtime.path(), "no-keyboard").unwrap();
        assert!(matches!(
            bare.record_key(KEY_ESC, KeyState::Pressed, 1),
            Err(InputError::Unavailable)
        ));
    }
}
