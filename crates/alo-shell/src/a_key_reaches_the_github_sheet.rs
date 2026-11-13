//! **Every key and every press reaches the GitHub sign-in sheet while it is
//! up** (`docs/design/github-sign-in.md`).
//!
//! The *Open with* sheet's rule (`crate::a_key_reaches_the_open_with_sheet`),
//! for the same reason: while the desktop is asking the person something,
//! every key is **intercepted** at the seat and never forwarded, so nothing
//! typed reaches the window behind it — Settings, which the sheet was opened
//! from, included. Keys are read as Settings reads them
//! (`crate::Server::settings_key`): Tab and the arrows move between the
//! buttons, Enter or Space presses one, Escape cancels.
//!
//! **A press of the pointer is the sheet's too**: on a button it presses it,
//! and anywhere else it does nothing — a click meant for the sheet never lands
//! on a window behind it. The pointer still moves, by the road it always took.

use smithay::backend::input::ButtonState;

use crate::the_github_sheet::{GitHubSheetKey, OnTheGitHubSheet};
use crate::{InputError, InputUpdate, Server};

/// Where the sheet was last drawn: each button.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WhereTheGitHubSheetIs {
    /// Each button, where it is on the display.
    pub(crate) buttons: Vec<(OnTheGitHubSheet, crate::settings_canvas::Area)>,
}

impl WhereTheGitHubSheetIs {
    /// The button a press at (`x`, `y`) lands on, or [`None`] for anywhere
    /// else.
    pub(crate) fn pressed_at(&self, x: i32, y: i32) -> Option<OnTheGitHubSheet> {
        let point = smithay::utils::Point::from((x, y));
        self.buttons
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
pub(crate) fn a_key_reaches_the_github_sheet(
    server: &mut Server,
    drawn: &WhereTheGitHubSheetIs,
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
                    server.the_github_sheet_key(GitHubSheetKey::of(pressed.key));
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
                server.the_github_sheet_pressed(on);
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

    /// **A press lands on the button under it**, and anywhere else on
    /// nothing — never on a window behind the sheet.
    #[test]
    fn a_press_lands_on_the_button_under_it() {
        let drawn = WhereTheGitHubSheetIs {
            buttons: vec![
                (
                    OnTheGitHubSheet::Copy,
                    Rectangle::new((400, 200).into(), (80, 48).into()),
                ),
                (
                    OnTheGitHubSheet::OpenThePage,
                    Rectangle::new((390, 460).into(), (220, 48).into()),
                ),
                (
                    OnTheGitHubSheet::Cancel,
                    Rectangle::new((300, 460).into(), (82, 48).into()),
                ),
            ],
        };
        assert_eq!(drawn.pressed_at(410, 210), Some(OnTheGitHubSheet::Copy));
        assert_eq!(
            drawn.pressed_at(500, 470),
            Some(OnTheGitHubSheet::OpenThePage)
        );
        assert_eq!(drawn.pressed_at(310, 470), Some(OnTheGitHubSheet::Cancel));
        assert_eq!(drawn.pressed_at(10, 10), None);
        assert_eq!(WhereTheGitHubSheetIs::default().pressed_at(410, 210), None);
    }
}
