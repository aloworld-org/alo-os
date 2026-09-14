//! Keystrokes for the approval surface, through the seat this compositor
//! already has.
//!
//! While a question is in front of the person, every key is **intercepted** at
//! the seat and never forwarded: an application behind the question does not
//! hear the Tab and the Enter that answer it, and nothing typed at the surface
//! reaches a window. Unlike the sign-in screen, keyboard focus is left where it
//! is — a person signed in has a window they were working in, and it is theirs
//! again the moment the question is answered.

use smithay::{backend::input::KeyState, input::keyboard::FilterResult, utils::SERIAL_COUNTER};

use crate::{ApprovalKey, InputError, Server};

impl Server {
    /// One key from a trusted backend, as the approval surface understands it.
    ///
    /// Takes a Linux evdev code, like [`Server::keyboard_key`], and updates the
    /// seat's XKB state with it. Returns [`None`] for a release, a repeated
    /// press of a key already down — so an Enter held down answers nothing
    /// twice — and an unmatched release. Nothing is forwarded to any client,
    /// whatever this returns.
    ///
    /// # Errors
    /// [`InputError::InvalidKey`] for a code outside evdev's range, and
    /// [`InputError::Unavailable`] for a display bound without a keyboard.
    pub fn approval_key(
        &mut self,
        code: u32,
        state: KeyState,
        time: u32,
    ) -> Result<Option<ApprovalKey>, InputError> {
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
        let meant = handle.input::<Option<ApprovalKey>, _>(
            &mut self.surfaces,
            code,
            state,
            SERIAL_COUNTER.next_serial(),
            time,
            |_, held, symbol| {
                FilterResult::Intercept(
                    (state == KeyState::Pressed)
                        .then(|| ApprovalKey::of(symbol.modified_sym(), held)),
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

    /// evdev codes, as a backend hands them over.
    const KEY_Y: u32 = 21;
    /// Left Shift.
    const KEY_LEFTSHIFT: u32 = 42;
    /// Tab.
    const KEY_TAB: u32 = 15;
    /// Enter.
    const KEY_ENTER: u32 = 28;
    /// Escape.
    const KEY_ESC: u32 = 1;

    /// A display with a keyboard on a US layout, in a private directory.
    fn seat() -> (tempfile::TempDir, Server) {
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let server = Server::bind_keyboard(
            runtime.path(),
            "approval",
            XkbConfig {
                layout: "us",
                ..XkbConfig::default()
            },
        )
        .unwrap();
        (runtime, server)
    }

    /// A press and its release, returning what the press meant.
    fn tapped(server: &mut Server, code: u32) -> Option<ApprovalKey> {
        let meant = server.approval_key(code, KeyState::Pressed, 1).unwrap();
        assert_eq!(
            server.approval_key(code, KeyState::Released, 2).unwrap(),
            None
        );
        meant
    }

    /// **Keys arrive through the seat's own modifiers**: Tab, Shift+Tab and
    /// Enter move and choose, and a letter or Escape is nothing.
    #[test]
    fn keys_arrive_through_the_seats_modifiers() {
        let (_runtime, mut server) = seat();
        assert_eq!(tapped(&mut server, KEY_TAB), Some(ApprovalKey::NextAnswer));
        server
            .approval_key(KEY_LEFTSHIFT, KeyState::Pressed, 3)
            .unwrap();
        assert_eq!(
            tapped(&mut server, KEY_TAB),
            Some(ApprovalKey::PreviousAnswer)
        );
        server
            .approval_key(KEY_LEFTSHIFT, KeyState::Released, 4)
            .unwrap();
        assert_eq!(tapped(&mut server, KEY_ENTER), Some(ApprovalKey::Choose));
        assert_eq!(tapped(&mut server, KEY_Y), Some(ApprovalKey::Nothing));
        assert_eq!(tapped(&mut server, KEY_ESC), Some(ApprovalKey::Nothing));
    }

    /// **An Enter held down chooses once.** The repeated press of a key already
    /// down means nothing, so one press cannot answer a question and then the
    /// next one that goes up.
    #[test]
    fn an_enter_held_down_chooses_once() {
        let (_runtime, mut server) = seat();
        assert_eq!(
            server
                .approval_key(KEY_ENTER, KeyState::Pressed, 1)
                .unwrap(),
            Some(ApprovalKey::Choose)
        );
        for time in 2..10 {
            assert_eq!(
                server
                    .approval_key(KEY_ENTER, KeyState::Pressed, time)
                    .unwrap(),
                None
            );
        }
    }

    /// **A code outside evdev's range is refused**, and so is a display with no
    /// keyboard, rather than a surface that silently takes no answer.
    #[test]
    fn a_strange_code_and_a_missing_keyboard_are_refused() {
        let (_runtime, mut server) = seat();
        for code in [0, 0x300, u32::MAX - 8] {
            assert!(matches!(
                server.approval_key(code, KeyState::Pressed, 1),
                Err(InputError::InvalidKey)
            ));
        }
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut bare = Server::bind(runtime.path(), "no-keyboard").unwrap();
        assert!(matches!(
            bare.approval_key(KEY_ENTER, KeyState::Pressed, 1),
            Err(InputError::Unavailable)
        ));
    }
}
