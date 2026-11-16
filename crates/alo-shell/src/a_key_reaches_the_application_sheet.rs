//! **Every key and every press reaches the sheet alo asks for an application
//! on, while it is up** (`docs/design/when-alo-asks-for-an-application.md`).
//!
//! The password sheet's rule, for its reason: while the desktop is asking the
//! person something, every key is **intercepted** at the seat and never
//! forwarded, so nothing typed reaches the window behind it — and nothing an
//! application or an agent sends can press anything here. Keys are read as
//! Settings reads them (`crate::Server::settings_key`).
//!
//! **A press of the pointer is the sheet's too**: on a length it moves the
//! keyboard there, on a button it answers, and anywhere else it does nothing.

use smithay::backend::input::ButtonState;

use crate::the_application_sheet::{ApplicationSheetKey, OnTheApplicationSheet};
use crate::{InputError, InputUpdate, Server};

/// Where the sheet was last drawn: each length and button.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WhereTheApplicationSheetIs {
    /// Each length and button, where it is on the display.
    pub(crate) targets: Vec<(OnTheApplicationSheet, crate::settings_canvas::Area)>,
}

impl WhereTheApplicationSheetIs {
    /// What a press at (`x`, `y`) is on, if anything.
    pub(crate) fn pressed_at(&self, x: i32, y: i32) -> Option<OnTheApplicationSheet> {
        let point = smithay::utils::Point::from((x, y));
        self.targets
            .iter()
            .find(|(_, area)| area.contains(point))
            .map(|(on, _)| *on)
    }
}

/// Route one seat update while the sheet is up.
///
/// # Errors
/// Whatever the seat refuses about the key, and whatever the ordinary road
/// refuses about everything else.
pub(crate) fn a_key_reaches_the_application_sheet(
    server: &mut Server,
    drawn: &WhereTheApplicationSheetIs,
    update: InputUpdate<'_>,
    extent: (i32, i32),
) -> Result<(), InputError> {
    let InputUpdate::Event(event) = update else {
        return server.libinput_update(update, extent);
    };
    match crate::libinput_routing::translate(event)? {
        Some(crate::DirectSeatEvent::Key(key, _)) => {
            match server.settings_key(key.code, key.state, key.time) {
                Ok(Some(pressed)) => {
                    server.the_application_sheet_key(ApplicationSheetKey::of(pressed.key));
                    Ok(())
                }
                Ok(None) | Err(InputError::InvalidKey) => Ok(()),
                Err(error) => Err(error),
            }
        }
        // **A press is the sheet's**, and its release with it, so no window
        // behind it is clicked.
        Some(crate::DirectSeatEvent::Button { state, .. }) => {
            if state == ButtonState::Pressed
                && let Some(at) = server.where_the_pointer_is_in_pixels()
                && let Some(on) = drawn.pressed_at(at.x, at.y)
            {
                server.the_application_sheet_pressed(on);
            }
            Ok(())
        }
        _ => server.libinput_update(InputUpdate::Event(event), extent),
    }
}

#[cfg(test)]
mod tests {
    use smithay::utils::Rectangle;

    use super::*;
    use crate::the_application_sheet::HowLongForAnApplication;

    /// **A press lands on what is under it**, and anywhere else on nothing.
    #[test]
    fn a_press_lands_on_what_is_under_it() {
        let session = OnTheApplicationSheet::For(HowLongForAnApplication::ThisSession);
        let drawn = WhereTheApplicationSheetIs {
            targets: vec![
                (session, Rectangle::new((400, 200).into(), (630, 70).into())),
                (
                    OnTheApplicationSheet::NotNow,
                    Rectangle::new((400, 460).into(), (160, 48).into()),
                ),
                (
                    OnTheApplicationSheet::LetAloUseIt,
                    Rectangle::new((572, 460).into(), (280, 48).into()),
                ),
            ],
        };
        assert_eq!(drawn.pressed_at(410, 210), Some(session));
        assert_eq!(
            drawn.pressed_at(410, 470),
            Some(OnTheApplicationSheet::NotNow)
        );
        assert_eq!(
            drawn.pressed_at(600, 470),
            Some(OnTheApplicationSheet::LetAloUseIt)
        );
        assert_eq!(drawn.pressed_at(10, 10), None);
        assert_eq!(
            WhereTheApplicationSheetIs::default().pressed_at(410, 210),
            None
        );
    }
}
