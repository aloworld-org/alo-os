//! **Every key and every press reaches the file chooser while it is up**
//! (`docs/design/the-file-chooser.md`).
//!
//! The password sheet's rule (`crate::a_key_reaches_the_password_sheet`), for
//! the same reason: while the desktop is asking the person something, every key
//! is **intercepted** at the seat and never forwarded, so nothing typed reaches
//! the window behind it. Keys are read as the password sheet reads them,
//! because a name is typed here.
//!
//! **A press of the pointer is the sheet's too**: on a row, the name, *Cancel*
//! or the other button it does what `crate::the_file_chooser` says, and
//! anywhere else nothing — a click meant for the sheet never lands on a window
//! behind it.

use smithay::backend::input::ButtonState;

use crate::the_file_chooser::{OnTheFileChooser, THE_DISK, TheFileChooser, the_key};
use crate::{InputError, InputUpdate, Server};

/// Where the sheet was last drawn.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WhereTheChooserIs {
    /// Each row shown, with its place in the list.
    pub(crate) rows: Vec<(usize, crate::settings_canvas::Area)>,
    /// The name, when saving.
    pub(crate) name: Option<crate::settings_canvas::Area>,
    /// *Cancel*.
    pub(crate) cancel: Option<crate::settings_canvas::Area>,
    /// *Open* or *Save*.
    pub(crate) button: Option<crate::settings_canvas::Area>,
}

impl WhereTheChooserIs {
    /// What a press at (`x`, `y`) is on, or [`None`] for nothing.
    pub(crate) fn pressed_at(&self, x: i32, y: i32) -> Option<OnTheFileChooser> {
        let point = smithay::utils::Point::from((x, y));
        if self.button.is_some_and(|area| area.contains(point)) {
            return Some(OnTheFileChooser::TheButton);
        }
        if self.cancel.is_some_and(|area| area.contains(point)) {
            return Some(OnTheFileChooser::Cancel);
        }
        if self.name.is_some_and(|area| area.contains(point)) {
            return Some(OnTheFileChooser::TheName);
        }
        self.rows
            .iter()
            .find(|(_, area)| area.contains(point))
            .map(|(row, _)| OnTheFileChooser::TheRow(*row))
    }
}

/// Route one seat update while the sheet is up.
///
/// # Errors
/// Whatever the seat refuses about the key, and whatever the ordinary road
/// refuses about everything else.
pub(crate) fn a_key_reaches_the_file_chooser(
    server: &mut Server,
    sheet: &mut TheFileChooser,
    drawn: &WhereTheChooserIs,
    update: InputUpdate<'_>,
    extent: (i32, i32),
) -> Result<(), InputError> {
    let InputUpdate::Event(event) = update else {
        return server.libinput_update(update, extent);
    };
    let now = std::time::Instant::now();
    match crate::libinput_routing::translate(event)? {
        Some(crate::DirectSeatEvent::Key(key, _)) => {
            match server.password_sheet_key(key.code, key.state, key.time) {
                Ok(Some(pressed)) => {
                    if let Some(key) = the_key(pressed) {
                        sheet.pressed(key, now, &THE_DISK);
                    }
                    Ok(())
                }
                Ok(None) | Err(InputError::InvalidKey) => Ok(()),
                Err(error) => Err(error),
            }
        }
        // **A press is the sheet's**, and its release with it.
        Some(crate::DirectSeatEvent::Button { state, .. }) => {
            if state == ButtonState::Pressed
                && let Some(at) = server.where_the_pointer_is_in_pixels()
                && let Some(on) = drawn.pressed_at(at.x, at.y)
            {
                sheet.clicked(on, now, &THE_DISK);
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
        let drawn = WhereTheChooserIs {
            rows: vec![
                (4, Rectangle::new((100, 100).into(), (400, 72).into())),
                (5, Rectangle::new((100, 173).into(), (400, 72).into())),
            ],
            name: Some(Rectangle::new((100, 260).into(), (400, 48).into())),
            cancel: Some(Rectangle::new((300, 320).into(), (80, 48).into())),
            button: Some(Rectangle::new((390, 320).into(), (70, 48).into())),
        };
        assert_eq!(
            drawn.pressed_at(120, 110),
            Some(OnTheFileChooser::TheRow(4))
        );
        assert_eq!(
            drawn.pressed_at(120, 200),
            Some(OnTheFileChooser::TheRow(5))
        );
        assert_eq!(drawn.pressed_at(120, 270), Some(OnTheFileChooser::TheName));
        assert_eq!(drawn.pressed_at(310, 330), Some(OnTheFileChooser::Cancel));
        assert_eq!(
            drawn.pressed_at(400, 330),
            Some(OnTheFileChooser::TheButton)
        );
        assert_eq!(drawn.pressed_at(10, 10), None);
        assert_eq!(WhereTheChooserIs::default().pressed_at(120, 110), None);
    }
}
