//! Keystrokes for the sign-in screen, through the seat this compositor already
//! has.
//!
//! The seat's keyboard carries the person's XKB layout (`Server::bind_keyboard`)
//! and keeps its modifier state. The sign-in screen needs exactly that — the
//! symbol a key makes on this layout with Shift or AltGr held — and needs no
//! client to receive anything: nobody is signed in, so no application is
//! running to be typed at. So every key is **intercepted** at the seat and
//! never forwarded, and the keyboard is focused on nothing before each one, so
//! that a client that was somehow mapped still cannot hear a password typed.

use smithay::{backend::input::KeyState, input::keyboard::FilterResult, utils::SERIAL_COUNTER};

use crate::{InputError, Server, SignInKey};

impl Server {
    /// One key from a trusted backend, as the sign-in screen understands it.
    ///
    /// Takes a Linux evdev code, like [`Server::keyboard_key`], and updates the
    /// seat's XKB state with it. Returns [`None`] for a release, a repeated
    /// press of a key already down and an unmatched release, and the meaning of
    /// the press otherwise. Nothing is forwarded to any client, whatever this
    /// returns.
    ///
    /// # Errors
    /// [`InputError::InvalidKey`] for a code outside evdev's range, and
    /// [`InputError::Unavailable`] for a display bound without a keyboard.
    pub fn sign_in_key(
        &mut self,
        code: u32,
        state: KeyState,
        time: u32,
    ) -> Result<Option<SignInKey>, InputError> {
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
        // Nobody hears a key the sign-in screen takes.
        self.surfaces.set_keyboard_focus(None)?;
        let code = (code + 8).into();
        if handle.pressed_keys().contains(&code) == (state == KeyState::Pressed) {
            return Ok(None);
        }
        let meant = handle.input::<Option<SignInKey>, _>(
            &mut self.surfaces,
            code,
            state,
            SERIAL_COUNTER.next_serial(),
            time,
            |_, held, symbol| {
                FilterResult::Intercept(
                    (state == KeyState::Pressed)
                        .then(|| SignInKey::of(symbol.modified_sym(), held)),
                )
            },
        );
        if let Some(keyboard) = self.surfaces.keyboard.as_mut() {
            keyboard.time = time;
        }
        Ok(meant.flatten())
    }

    /// Let go of every key the seat believes is held, without telling anyone.
    ///
    /// For the moment the screen loses the keyboard — the parent window loses
    /// focus — so that a Shift held as it left is not still held when it
    /// comes back and every letter typed afterwards is not a capital.
    ///
    /// # Errors
    /// [`InputError::Unavailable`] for a display bound without a keyboard.
    pub fn sign_in_keys_released(&mut self) -> Result<(), InputError> {
        let keyboard = self
            .surfaces
            .keyboard
            .as_ref()
            .ok_or(InputError::Unavailable)?;
        let handle = keyboard.handle.clone();
        let time = keyboard.time;
        for code in handle.pressed_keys() {
            handle.input::<(), _>(
                &mut self.surfaces,
                code,
                KeyState::Released,
                SERIAL_COUNTER.next_serial(),
                time,
                |_, _, _| FilterResult::Intercept(()),
            );
        }
        Ok(())
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
    const KEY_A: u32 = 30;
    /// Left Shift.
    const KEY_LEFTSHIFT: u32 = 42;
    /// Left Control.
    const KEY_LEFTCTRL: u32 = 29;
    /// Backspace.
    const KEY_BACKSPACE: u32 = 14;
    /// Tab.
    const KEY_TAB: u32 = 15;
    /// Enter.
    const KEY_ENTER: u32 = 28;

    /// A display with a keyboard on a US layout, in a private directory.
    fn seat() -> (tempfile::TempDir, Server) {
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let server = Server::bind_keyboard(
            runtime.path(),
            "sign-in",
            XkbConfig {
                layout: "us",
                ..XkbConfig::default()
            },
        )
        .unwrap();
        (runtime, server)
    }

    /// A press and its release, returning what the press meant.
    fn tapped(server: &mut Server, code: u32) -> Option<SignInKey> {
        let meant = server.sign_in_key(code, KeyState::Pressed, 1).unwrap();
        assert_eq!(
            server.sign_in_key(code, KeyState::Released, 2).unwrap(),
            None
        );
        meant
    }

    /// **Keys arrive through the seat's own layout and modifiers**: a letter,
    /// the same letter with Shift held, and the three keys that move things.
    #[test]
    fn keys_arrive_through_the_seats_layout_and_modifiers() {
        let (_runtime, mut server) = seat();
        assert_eq!(tapped(&mut server, KEY_A), Some(SignInKey::Letter('a')));

        assert_eq!(
            server
                .sign_in_key(KEY_LEFTSHIFT, KeyState::Pressed, 3)
                .unwrap(),
            Some(SignInKey::Nothing)
        );
        assert_eq!(tapped(&mut server, KEY_A), Some(SignInKey::Letter('A')));
        server
            .sign_in_key(KEY_LEFTSHIFT, KeyState::Released, 4)
            .unwrap();
        assert_eq!(tapped(&mut server, KEY_A), Some(SignInKey::Letter('a')));

        assert_eq!(tapped(&mut server, KEY_BACKSPACE), Some(SignInKey::Erase));
        assert_eq!(tapped(&mut server, KEY_TAB), Some(SignInKey::OtherField));
        assert_eq!(tapped(&mut server, KEY_ENTER), Some(SignInKey::Enter));
    }

    /// **A chord types nothing**, through the real seat rather than a
    /// constructed modifier state.
    #[test]
    fn a_chord_through_the_seat_types_nothing() {
        let (_runtime, mut server) = seat();
        server
            .sign_in_key(KEY_LEFTCTRL, KeyState::Pressed, 1)
            .unwrap();
        assert_eq!(tapped(&mut server, KEY_A), Some(SignInKey::Nothing));
    }

    /// **A press of a key already down, and a release of one that is not, mean
    /// nothing** — a stuck or replayed event types no second letter.
    #[test]
    fn a_repeated_press_and_an_unmatched_release_mean_nothing() {
        let (_runtime, mut server) = seat();
        assert_eq!(
            server.sign_in_key(KEY_A, KeyState::Released, 1).unwrap(),
            None
        );
        assert_eq!(
            server.sign_in_key(KEY_A, KeyState::Pressed, 2).unwrap(),
            Some(SignInKey::Letter('a'))
        );
        assert_eq!(
            server.sign_in_key(KEY_A, KeyState::Pressed, 3).unwrap(),
            None
        );
    }

    /// **A code outside evdev's range is refused**, not typed.
    #[test]
    fn a_code_outside_the_range_is_refused() {
        let (_runtime, mut server) = seat();
        for code in [0, 0x300, u32::MAX - 8] {
            assert!(matches!(
                server.sign_in_key(code, KeyState::Pressed, 1),
                Err(InputError::InvalidKey)
            ));
        }
    }

    /// **A display with no keyboard is refused**, rather than a screen that
    /// silently takes no keys.
    #[test]
    fn a_display_without_a_keyboard_is_refused() {
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut server = Server::bind(runtime.path(), "no-keyboard").unwrap();
        assert!(matches!(
            server.sign_in_key(KEY_A, KeyState::Pressed, 1),
            Err(InputError::Unavailable)
        ));
        assert!(matches!(
            server.sign_in_keys_released(),
            Err(InputError::Unavailable)
        ));
    }

    /// **A Shift held as the screen lost the keyboard is let go**, so the
    /// letters typed when it comes back are not capitals.
    #[test]
    fn keys_held_when_the_screen_loses_the_keyboard_are_let_go() {
        let (_runtime, mut server) = seat();
        server
            .sign_in_key(KEY_LEFTSHIFT, KeyState::Pressed, 1)
            .unwrap();
        server.sign_in_keys_released().unwrap();
        assert_eq!(tapped(&mut server, KEY_A), Some(SignInKey::Letter('a')));
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod the_keyboard_can_change {
    use super::*;
    use smithay::input::keyboard::XkbConfig;
    use std::os::unix::fs::PermissionsExt;

    /// The key beside the home row that four layouts disagree about.
    ///
    /// evdev 16 is `q` on a British keyboard, `a` on a French one, and `q`
    /// again on a German — the German pair that differs is 44, `y` against
    /// `z`. Two keys, so a change is read rather than assumed.
    const KEY_Q: u32 = 16;
    /// evdev 44: `z` on gb and us, `y` on de, `w` on fr.
    const KEY_Z: u32 = 44;

    fn seat(layout: &str) -> (tempfile::TempDir, Server) {
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let server = Server::bind_keyboard(
            runtime.path(),
            "changing",
            XkbConfig {
                layout,
                ..XkbConfig::default()
            },
        )
        .unwrap();
        (runtime, server)
    }

    /// What this seat makes of a key, right now.
    fn letter(server: &mut Server, code: u32) -> Option<char> {
        let meant = server.sign_in_key(code, KeyState::Pressed, 1).unwrap();
        server.sign_in_key(code, KeyState::Released, 2).unwrap();
        match meant {
            Some(SignInKey::Letter(letter)) => Some(letter),
            _ => None,
        }
    }

    /// **The keyboard really changes**, read from what a key produces rather
    /// than from the call returning `Ok`.
    ///
    /// This is the whole point: `ALO_KEYBOARD=gb` was named in a unit file and
    /// nothing on a running machine could change it, so a person on a French
    /// keyboard typed their password through a British one.
    #[test]
    fn the_layout_a_key_means_really_changes() {
        let (_runtime, mut server) = seat("gb");
        assert_eq!(letter(&mut server, KEY_Q), Some('q'), "gb: evdev 16 is q");
        assert_eq!(letter(&mut server, KEY_Z), Some('z'), "gb: evdev 44 is z");

        server
            .the_keyboard_is_now(XkbConfig {
                layout: "fr",
                ..XkbConfig::default()
            })
            .unwrap();
        assert_eq!(
            letter(&mut server, KEY_Q),
            Some('a'),
            "fr: evdev 16 is a, so the layout did not change"
        );

        server
            .the_keyboard_is_now(XkbConfig {
                layout: "de",
                ..XkbConfig::default()
            })
            .unwrap();
        assert_eq!(
            letter(&mut server, KEY_Z),
            Some('y'),
            "de: evdev 44 is y, so the layout did not change again"
        );
    }

    /// **A key held when the keyboard changes is let go first.**
    ///
    /// A key held under one layout is a different symbol under the next, so a
    /// client told it was released would be told the wrong one — or never
    /// told, and would repeat a letter for ever.
    #[test]
    fn a_key_held_when_the_keyboard_changes_is_let_go() {
        let (_runtime, mut server) = seat("gb");
        // Shift down, and left down.
        server.sign_in_key(42, KeyState::Pressed, 1).unwrap();
        assert_eq!(
            server.sign_in_key(KEY_Q, KeyState::Pressed, 2).unwrap(),
            Some(SignInKey::Letter('Q')),
            "with Shift held, gb's evdev 16 is a capital"
        );
        server
            .the_keyboard_is_now(XkbConfig {
                layout: "fr",
                ..XkbConfig::default()
            })
            .unwrap();
        // Shift was let go by the change, so the next letter is not a capital.
        assert_eq!(
            letter(&mut server, KEY_Q),
            Some('a'),
            "a Shift held across the change was still held afterwards"
        );
    }

    /// **A layout this machine does not have leaves the keyboard as it was.**
    ///
    /// A machine whose layout could not be changed must still be typeable.
    #[test]
    fn a_layout_this_machine_does_not_have_leaves_the_keyboard_alone() {
        let (_runtime, mut server) = seat("gb");
        let refused = server.the_keyboard_is_now(XkbConfig {
            layout: "not-a-layout-any-machine-has",
            ..XkbConfig::default()
        });
        assert!(
            refused.is_err(),
            "a layout that does not exist was accepted"
        );
        assert_eq!(
            letter(&mut server, KEY_Q),
            Some('q'),
            "a refused change left the keyboard broken"
        );
    }

    /// **A display with no keyboard is refused**, not silently ignored.
    #[test]
    fn a_display_without_a_keyboard_is_refused() {
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut server = Server::bind(runtime.path(), "no-keyboard").unwrap();
        assert!(matches!(
            server.the_keyboard_is_now(XkbConfig::default()),
            Err(InputError::Unavailable)
        ));
    }
}
