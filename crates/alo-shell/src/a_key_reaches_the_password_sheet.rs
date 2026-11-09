//! **Every key reaches the password sheet while it is up** (ADR 0150).
//!
//! The approval surface's rule (`crate::approval_seat`), for the same reason
//! and with more at stake: while polkit is asking for the person's password,
//! every key is **intercepted** at the seat and never forwarded, so nothing
//! typed reaches the window behind it — a password typed into somebody else's
//! window is the one thing this sheet must never let happen. Keyboard focus is
//! left where it is, as the approval surface leaves it: the window the person
//! was working in is theirs again the moment the sheet comes down.
//!
//! A pointer, a device arriving or leaving, takes the road it always took.

use smithay::{backend::input::KeyState, input::keyboard::FilterResult, utils::SERIAL_COUNTER};

use crate::the_password_sheet::ThePasswordSheet;
use crate::{InputError, InputUpdate, Server, SignInKey};

impl Server {
    /// One key from a trusted backend, as the password sheet understands it.
    ///
    /// [`None`] for a release, a repeated press of a key already down — so an
    /// Enter held down sends nothing twice — and an unmatched release. Nothing
    /// is forwarded to any client, whatever this returns.
    ///
    /// # Errors
    /// [`InputError::InvalidKey`] for a code outside evdev's range, and
    /// [`InputError::Unavailable`] for a display bound without a keyboard.
    pub(crate) fn password_sheet_key(
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
                    (state == KeyState::Pressed).then(|| SignInKey::of(symbol.modified_sym(), held)),
                )
            },
        );
        if let Some(keyboard) = self.surfaces.keyboard.as_mut() {
            keyboard.time = time;
        }
        Ok(meant.flatten())
    }
}

/// Route one seat update while the password sheet is up.
///
/// # Errors
/// Whatever the seat refuses about the key, and whatever the ordinary road
/// refuses about everything else. A code outside evdev's range is not an
/// error here, for `crate::a_key_reaches_settings`' reason.
pub(crate) fn a_key_reaches_the_password_sheet(
    server: &mut Server,
    sheet: &mut ThePasswordSheet,
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
    match server.password_sheet_key(key.code, key.state, key.time) {
        Ok(Some(pressed)) => {
            sheet.pressed(pressed);
            Ok(())
        }
        Ok(None) | Err(InputError::InvalidKey) => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::os::unix::fs::PermissionsExt;
    use std::sync::mpsc::channel;

    use smithay::input::keyboard::XkbConfig;

    use super::*;
    use crate::the_password_sheet::{FromTheSheet, PasswordAsked, ToTheSheet, WhatItIsFor};

    /// evdev codes: 1, 2, Enter, Escape.
    const KEY_1: u32 = 2;
    /// Two.
    const KEY_2: u32 = 3;
    /// Enter.
    const KEY_ENTER: u32 = 28;
    /// Escape.
    const KEY_ESC: u32 = 1;

    /// A display with a keyboard on a US layout.
    fn seat() -> (tempfile::TempDir, Server) {
        let runtime = tempfile::tempdir().unwrap();
        std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let server = Server::bind_keyboard(
            runtime.path(),
            "password-sheet",
            XkbConfig {
                layout: "us",
                ..XkbConfig::default()
            },
        )
        .unwrap();
        (runtime, server)
    }

    /// A press and its release, handed to the sheet the way the desk hands it.
    fn tapped(server: &mut Server, sheet: &mut ThePasswordSheet, code: u32) {
        for (state, time) in [(KeyState::Pressed, 1), (KeyState::Released, 2)] {
            if let Some(pressed) = server.password_sheet_key(code, state, time).unwrap() {
                sheet.pressed(pressed);
            }
        }
    }

    /// **Keys typed at the seat reach the sheet as letters, Enter sends them
    /// once, and Escape cancels the next question.**
    #[test]
    fn keys_at_the_seat_reach_the_sheet() {
        let (_runtime, mut server) = seat();
        let mut sheet = ThePasswordSheet::none();
        let (answer, answered) = channel();
        sheet.told(ToTheSheet::Ask(PasswordAsked {
            cookie: "c1".to_owned(),
            what_for: WhatItIsFor::ACommandAloWasAsked("true".to_owned()),
            answer,
        }));
        for code in [KEY_1, KEY_2, KEY_ENTER] {
            tapped(&mut server, &mut sheet, code);
        }
        let came: Vec<String> = answered
            .try_iter()
            .map(|answer| match answer {
                FromTheSheet::Typed(typed) => typed.as_str().to_owned(),
                FromTheSheet::Cancelled => "cancelled".to_owned(),
            })
            .collect();
        assert_eq!(came, ["12"]);
        let (answer, answered) = channel();
        sheet.told(ToTheSheet::Over {
            cookie: "c1".to_owned(),
        });
        sheet.told(ToTheSheet::Ask(PasswordAsked {
            cookie: "c2".to_owned(),
            what_for: WhatItIsFor::SomethingElse("Authentication is required".to_owned()),
            answer,
        }));
        tapped(&mut server, &mut sheet, KEY_ESC);
        assert!(matches!(answered.try_recv(), Ok(FromTheSheet::Cancelled)));
        assert!(!sheet.is_up());
    }

    /// **An Enter held down sends once**: the repeated press of a key already
    /// down means nothing.
    #[test]
    fn an_enter_held_down_sends_once() {
        let (_runtime, mut server) = seat();
        assert_eq!(
            server.password_sheet_key(KEY_ENTER, KeyState::Pressed, 1).unwrap(),
            Some(SignInKey::Enter)
        );
        for time in 2..6 {
            assert_eq!(
                server.password_sheet_key(KEY_ENTER, KeyState::Pressed, time).unwrap(),
                None
            );
        }
    }
}
