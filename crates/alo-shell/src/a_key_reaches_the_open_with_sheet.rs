//! **Every key and every press reaches the *Open with* sheet while it is up**
//! (frame `25:52`, `docs/design/default-apps-and-open-with.md`).
//!
//! The password sheet's rule (`crate::a_key_reaches_the_password_sheet`), for
//! the same reason: while the desktop is asking the person something, every key
//! is **intercepted** at the seat and never forwarded, so nothing typed reaches
//! the window behind it. Keys are read as Settings reads them
//! (`crate::Server::settings_key`): Up and Down — and Left and Right — choose,
//! Enter or Space opens, Escape cancels.
//!
//! **A press of the pointer is the sheet's too**: on a row it chooses that
//! row, on *Open* it opens, on *Cancel* it cancels, and anywhere else it does
//! nothing — a click meant for the sheet never lands on a window behind it.
//! The pointer still moves, by the road it always took.

use smithay::backend::input::ButtonState;

use crate::the_open_with_sheet::{SheetKey, TheOpenWithSheet};
use crate::{InputError, InputUpdate, Server};

/// Where the sheet was last drawn: each row, *Cancel* and *Open*.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WhereTheSheetIs {
    /// Each offered application's row, in order.
    pub(crate) rows: Vec<crate::settings_canvas::Area>,
    /// *Cancel*.
    pub(crate) cancel: Option<crate::settings_canvas::Area>,
    /// *Open*.
    pub(crate) open: Option<crate::settings_canvas::Area>,
}

/// What a press at a point on the display does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ThePressIsOn {
    /// A row: the keyboard goes to it.
    ARow(usize),
    /// *Open*.
    Open,
    /// *Cancel*.
    Cancel,
    /// Anywhere else, which does nothing.
    Nothing,
}

impl WhereTheSheetIs {
    /// What a press at (`x`, `y`) is on.
    pub(crate) fn pressed_at(&self, x: i32, y: i32) -> ThePressIsOn {
        let point = smithay::utils::Point::from((x, y));
        if self.open.is_some_and(|open| open.contains(point)) {
            return ThePressIsOn::Open;
        }
        if self.cancel.is_some_and(|cancel| cancel.contains(point)) {
            return ThePressIsOn::Cancel;
        }
        self.rows
            .iter()
            .position(|row| row.contains(point))
            .map_or(ThePressIsOn::Nothing, ThePressIsOn::ARow)
    }
}

/// Route one seat update while the sheet is up.
///
/// # Errors
/// Whatever the seat refuses about the key, and whatever the ordinary road
/// refuses about everything else.
pub(crate) fn a_key_reaches_the_open_with_sheet(
    server: &mut Server,
    sheet: &mut TheOpenWithSheet,
    drawn: &WhereTheSheetIs,
    update: InputUpdate<'_>,
    extent: (i32, i32),
) -> Result<(), InputError> {
    let InputUpdate::Event(event) = update else {
        return server.libinput_update(update, extent);
    };
    let now = std::time::Instant::now();
    match crate::libinput_routing::translate(event)? {
        Some(crate::DirectSeatEvent::Key(key, _)) => {
            match server.settings_key(key.code, key.state, key.time) {
                Ok(Some(pressed)) => {
                    sheet.pressed(SheetKey::of(pressed.key), now);
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
            {
                match drawn.pressed_at(at.x, at.y) {
                    ThePressIsOn::ARow(row) => sheet.pointed_at(row),
                    ThePressIsOn::Open => sheet.open_in(None, now),
                    ThePressIsOn::Cancel => sheet.cancelled(now),
                    ThePressIsOn::Nothing => {}
                }
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

    /// **A press lands on what is under it**, and anywhere else on nothing.
    #[test]
    fn a_press_lands_on_what_is_under_it() {
        let drawn = WhereTheSheetIs {
            rows: vec![
                Rectangle::new((100, 100).into(), (400, 72).into()),
                Rectangle::new((100, 173).into(), (400, 72).into()),
            ],
            cancel: Some(Rectangle::new((300, 260).into(), (80, 48).into())),
            open: Some(Rectangle::new((390, 260).into(), (70, 48).into())),
        };
        assert_eq!(drawn.pressed_at(120, 110), ThePressIsOn::ARow(0));
        assert_eq!(drawn.pressed_at(120, 200), ThePressIsOn::ARow(1));
        assert_eq!(drawn.pressed_at(310, 270), ThePressIsOn::Cancel);
        assert_eq!(drawn.pressed_at(400, 270), ThePressIsOn::Open);
        assert_eq!(drawn.pressed_at(10, 10), ThePressIsOn::Nothing);
        assert_eq!(
            WhereTheSheetIs::default().pressed_at(120, 110),
            ThePressIsOn::Nothing
        );
    }
}
