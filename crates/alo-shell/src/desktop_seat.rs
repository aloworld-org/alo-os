//! Keystrokes for the two desktop windows, through the seat this compositor
//! already has.
//!
//! While either window is open every key routed to it is **intercepted** at the
//! seat and never forwarded: an application behind the window does not hear
//! the arrows that move through it or the Escape that closes it. Keyboard focus
//! is left where it is, so the window a person was working in is theirs again
//! the moment the desktop window closes.

use alo_strings::Direction;
use smithay::{
    backend::input::KeyState,
    input::keyboard::{FilterResult, Keysym, ModifiersState},
    utils::SERIAL_COUNTER,
};

use crate::{FillingKey, InputError, RunningKey, Server};

impl Server {
    /// One key from a trusted backend, as the window of what is running
    /// understands it.
    ///
    /// Takes a Linux evdev code, like [`Server::keyboard_key`]. Returns
    /// [`None`] for a release, a repeated press of a key already down, and an
    /// unmatched release. Nothing is forwarded to any client.
    ///
    /// # Errors
    /// [`InputError::InvalidKey`] for a code outside evdev's range, and
    /// [`InputError::Unavailable`] for a display bound without a keyboard.
    pub fn running_key(
        &mut self,
        code: u32,
        state: KeyState,
        time: u32,
    ) -> Result<Option<RunningKey>, InputError> {
        self.desktop_key(code, state, time, RunningKey::of)
    }

    /// One key from a trusted backend, as the window of what is filling the
    /// disk understands it for somebody reading `reading`.
    ///
    /// # Errors
    /// As [`Server::running_key`].
    pub fn filling_key(
        &mut self,
        code: u32,
        state: KeyState,
        time: u32,
        reading: Direction,
    ) -> Result<Option<FillingKey>, InputError> {
        self.desktop_key(code, state, time, |symbol, held| {
            FillingKey::of(symbol, held, reading)
        })
    }

    /// One key, intercepted, and what `meaning` makes of its symbol.
    fn desktop_key<K>(
        &mut self,
        code: u32,
        state: KeyState,
        time: u32,
        meaning: impl FnOnce(Keysym, &ModifiersState) -> K,
    ) -> Result<Option<K>, InputError> {
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
        let meant = handle.input::<Option<K>, _>(
            &mut self.surfaces,
            code,
            state,
            SERIAL_COUNTER.next_serial(),
            time,
            |_, held, symbol| {
                FilterResult::Intercept(
                    (state == KeyState::Pressed).then(|| meaning(symbol.modified_sym(), held)),
                )
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
    use smithay::input::keyboard::XkbConfig;
    use std::os::unix::fs::PermissionsExt;

    /// Down, as evdev numbers it.
    const KEY_DOWN: u32 = 108;
    /// Right.
    const KEY_RIGHT: u32 = 106;
    /// Escape.
    const KEY_ESC: u32 = 1;
    /// `k`.
    const KEY_K: u32 = 37;

    /// A display with a keyboard on a US layout, in a private directory.
    fn seat() -> (tempfile::TempDir, Server) {
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let server = Server::bind_keyboard(
            runtime.path(),
            "desktop",
            XkbConfig {
                layout: "us",
                ..XkbConfig::default()
            },
        )
        .unwrap();
        (runtime, server)
    }

    /// **Keys arrive through the seat**: Down moves in both windows, Right opens
    /// a folder for somebody reading left to right and closes one for somebody
    /// reading right to left, and `k` does nothing to a process. A held key
    /// means something once.
    #[test]
    fn keys_arrive_through_the_seat_and_mean_something_once() {
        let (_runtime, mut server) = seat();
        assert_eq!(
            server.running_key(KEY_DOWN, KeyState::Pressed, 1).unwrap(),
            Some(RunningKey::Down)
        );
        assert_eq!(
            server.running_key(KEY_DOWN, KeyState::Pressed, 2).unwrap(),
            None
        );
        assert_eq!(
            server.running_key(KEY_DOWN, KeyState::Released, 3).unwrap(),
            None
        );
        assert_eq!(
            server.running_key(KEY_K, KeyState::Pressed, 4).unwrap(),
            Some(RunningKey::Nothing)
        );
        server.running_key(KEY_K, KeyState::Released, 5).unwrap();
        for (reading, meant) in [
            (Direction::LeftToRight, FillingKey::Open),
            (Direction::RightToLeft, FillingKey::Shut),
        ] {
            assert_eq!(
                server
                    .filling_key(KEY_RIGHT, KeyState::Pressed, 6, reading)
                    .unwrap(),
                Some(meant)
            );
            server
                .filling_key(KEY_RIGHT, KeyState::Released, 7, reading)
                .unwrap();
        }
        assert_eq!(
            server
                .filling_key(KEY_ESC, KeyState::Pressed, 8, Direction::LeftToRight)
                .unwrap(),
            Some(FillingKey::Close)
        );
    }

    /// **A strange code and a missing keyboard are refused**, rather than a
    /// window that silently takes no keys.
    #[test]
    fn a_strange_code_and_a_missing_keyboard_are_refused() {
        let (_runtime, mut server) = seat();
        for code in [0, 0x300, u32::MAX - 8] {
            assert!(matches!(
                server.running_key(code, KeyState::Pressed, 1),
                Err(InputError::InvalidKey)
            ));
            assert!(matches!(
                server.filling_key(code, KeyState::Pressed, 1, Direction::LeftToRight),
                Err(InputError::InvalidKey)
            ));
        }
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut bare = Server::bind(runtime.path(), "no-keyboard").unwrap();
        assert!(matches!(
            bare.running_key(KEY_ESC, KeyState::Pressed, 1),
            Err(InputError::Unavailable)
        ));
        assert!(matches!(
            bare.filling_key(KEY_ESC, KeyState::Pressed, 1, Direction::LeftToRight),
            Err(InputError::Unavailable)
        ));
    }
}
