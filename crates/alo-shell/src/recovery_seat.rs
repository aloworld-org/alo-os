//! Keystrokes for the recovery screen, through the seat this compositor
//! already has.
//!
//! Every key is **intercepted** at the seat and never forwarded. There is
//! nothing behind this screen to forward to — it is what a person reaches when
//! the desktop will not start — and a key that reached a half-started session
//! would be a key nobody could predict the effect of.

use smithay::{backend::input::KeyState, input::keyboard::FilterResult, utils::SERIAL_COUNTER};

use crate::{InputError, RecoveryKey, Server};

impl Server {
    /// One key from a trusted backend, as the recovery screen understands it.
    ///
    /// Takes a Linux evdev code, like [`Server::keyboard_key`], and updates the
    /// seat's XKB state with it. Returns [`None`] for a release, a repeated
    /// press of a key already down — so an Enter held down over from whatever
    /// failed a moment ago cannot choose twice, or choose at all — and an
    /// unmatched release. Nothing is forwarded to any client.
    ///
    /// # Errors
    /// [`InputError::InvalidKey`] for a code outside evdev's range, and
    /// [`InputError::Unavailable`] for a display bound without a keyboard.
    pub fn recovery_key(
        &mut self,
        code: u32,
        state: KeyState,
        time: u32,
    ) -> Result<Option<RecoveryKey>, InputError> {
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
        let meant = handle.input::<Option<RecoveryKey>, _>(
            &mut self.surfaces,
            code,
            state,
            SERIAL_COUNTER.next_serial(),
            time,
            |_, held, symbol| {
                FilterResult::Intercept(
                    (state == KeyState::Pressed)
                        .then(|| RecoveryKey::of(symbol.modified_sym(), held)),
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

    /// Left Shift.
    const KEY_LEFTSHIFT: u32 = 42;
    /// Tab.
    const KEY_TAB: u32 = 15;
    /// Enter.
    const KEY_ENTER: u32 = 28;
    /// Escape.
    const KEY_ESC: u32 = 1;
    /// The letter r, which is not a shortcut for anything here.
    const KEY_R: u32 = 19;

    /// A display with a keyboard on a US layout, in a private directory.
    fn seat() -> (tempfile::TempDir, Server) {
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let server = Server::bind_keyboard(
            runtime.path(),
            "recovery",
            XkbConfig {
                layout: "us",
                ..XkbConfig::default()
            },
        )
        .unwrap();
        (runtime, server)
    }

    /// A press and its release, returning what the press meant.
    fn tapped(server: &mut Server, code: u32) -> Option<RecoveryKey> {
        let meant = server.recovery_key(code, KeyState::Pressed, 1).unwrap();
        assert_eq!(
            server.recovery_key(code, KeyState::Released, 2).unwrap(),
            None
        );
        meant
    }

    /// **The whole screen is reachable through the seat with Tab and Enter**,
    /// and no other key on it does anything — not Escape, and not a letter.
    #[test]
    fn the_whole_screen_is_reachable_with_tab_and_enter() {
        let (_runtime, mut server) = seat();
        assert_eq!(tapped(&mut server, KEY_TAB), Some(RecoveryKey::Next));
        server
            .recovery_key(KEY_LEFTSHIFT, KeyState::Pressed, 3)
            .unwrap();
        assert_eq!(tapped(&mut server, KEY_TAB), Some(RecoveryKey::Previous));
        server
            .recovery_key(KEY_LEFTSHIFT, KeyState::Released, 4)
            .unwrap();
        assert_eq!(tapped(&mut server, KEY_ENTER), Some(RecoveryKey::Choose));
        assert_eq!(tapped(&mut server, KEY_ESC), Some(RecoveryKey::Nothing));
        assert_eq!(tapped(&mut server, KEY_R), Some(RecoveryKey::Nothing));
    }

    /// **An Enter held down chooses once.** A person whose session has just
    /// died may well be holding one.
    #[test]
    fn an_enter_held_down_chooses_once() {
        let (_runtime, mut server) = seat();
        assert_eq!(
            server
                .recovery_key(KEY_ENTER, KeyState::Pressed, 1)
                .unwrap(),
            Some(RecoveryKey::Choose)
        );
        for time in 2..10 {
            assert_eq!(
                server
                    .recovery_key(KEY_ENTER, KeyState::Pressed, time)
                    .unwrap(),
                None
            );
        }
    }

    /// **A code outside evdev's range is refused**, and so is a display with no
    /// keyboard, rather than a screen that silently takes no key.
    #[test]
    fn a_strange_code_and_a_missing_keyboard_are_refused() {
        let (_runtime, mut server) = seat();
        for code in [0, 0x300, u32::MAX - 8] {
            assert!(matches!(
                server.recovery_key(code, KeyState::Pressed, 1),
                Err(InputError::InvalidKey)
            ));
        }
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut bare = Server::bind(runtime.path(), "no-keyboard").unwrap();
        assert!(matches!(
            bare.recovery_key(KEY_ENTER, KeyState::Pressed, 1),
            Err(InputError::Unavailable)
        ));
    }
}
