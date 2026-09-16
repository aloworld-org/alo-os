//! What is being captured: the whole screen, one window, or a selected region.
//!
//! Three, and the list is closed — the plan's own sentence as a type. A fourth
//! thing to capture is a fourth thing a person has to be told about on the
//! indicator and a fourth shape of refusal to keep honest, and the promise is
//! three.
//!
//! # Every one of them is a rectangle by the time the machine is asked
//!
//! A window is a rectangle on the screen, a region is a rectangle, and the
//! whole screen is the rectangle that is all of it. [`What::across`] is that,
//! and it is what the rented mechanism is told to cut to — so the question
//! *which pixels* is answered here, in a value with tests around it, and not
//! inside the thing we rent.
//!
//! # A window is not reached by a handle
//!
//! Nothing here asks the compositor to find a window and photograph it. The
//! window arrives as a [`crate::Window`], already carrying whose session it is
//! in and whether it is the lock screen, because those two facts are what the
//! capture is refused on and a handle carries neither.

use crate::region::Region;
use crate::screen::Screen;
use crate::window::Window;

/// What a screenshot is of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum What {
    /// The whole screen.
    TheWholeScreen,
    /// One window.
    OneWindow(Window),
    /// A rectangle somebody selected.
    APartOfIt(Region),
}

impl What {
    /// The rectangle of the screen this covers.
    ///
    /// The whole screen is the rectangle that is all of it, which cannot fail:
    /// a [`Screen`] cannot be built with no width or no height, so the
    /// rectangle over all of it always has both.
    #[must_use]
    pub fn across(&self, screen: Screen) -> Region {
        match self {
            Self::TheWholeScreen => Region::over(screen),
            Self::OneWindow(window) => window.where_it_is(),
            Self::APartOfIt(region) => *region,
        }
    }

    /// The window this is of, or [`None`].
    #[must_use]
    pub const fn window(&self) -> Option<&Window> {
        match self {
            Self::OneWindow(window) => Some(window),
            Self::TheWholeScreen | Self::APartOfIt(_) => None,
        }
    }

    /// The name this is written down by, where something has to be written
    /// down.
    ///
    /// Never shown to a person and never put in a file name: what a person is
    /// told is [`crate::Taken::said`], and a file's name says nothing about
    /// what was on the screen ([`crate::naming`]).
    #[must_use]
    pub const fn named(&self) -> &'static str {
        match self {
            Self::TheWholeScreen => "the whole screen",
            Self::OneWindow(_) => "one window",
            Self::APartOfIt(_) => "a part of the screen",
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::session::WhoseSession;
    use crate::window::WindowId;
    use std::collections::BTreeSet;

    /// The screen these tests are about.
    fn a_screen() -> Screen {
        Screen::measuring(1920, 1080).unwrap()
    }

    /// **The whole screen is the rectangle that is all of it**, with nothing
    /// left over on any side.
    #[test]
    fn the_whole_screen_is_the_whole_screen() {
        let screen = a_screen();
        let across = What::TheWholeScreen.across(screen);
        assert_eq!(across, Region::of(0, 0, 1920, 1080).unwrap());
        assert!(across.is_on(screen));
        assert_eq!(across.to_the_right_on(screen), 0);
        assert_eq!(across.below_on(screen), 0);
    }

    /// **A window's rectangle is the window's own**, which is the one the
    /// compositor handed over and not one worked out again here.
    #[test]
    fn a_windows_rectangle_is_the_windows_own() {
        let where_it_is = Region::of(100, 50, 400, 300).unwrap();
        let window = Window::of(
            WindowId::recorded(7),
            &WhoseSession::of("anna"),
            where_it_is,
        );
        let what = What::OneWindow(window.clone());
        assert_eq!(what.across(a_screen()), where_it_is);
        assert_eq!(what.window(), Some(&window));
    }

    /// **A selected region is itself**, unchanged by being asked for.
    #[test]
    fn a_selected_region_is_itself() {
        let region = Region::of(5, 6, 7, 8).unwrap();
        let what = What::APartOfIt(region);
        assert_eq!(what.across(a_screen()), region);
        assert_eq!(what.window(), None);
    }

    /// **The three are told apart, and there are three.** A fourth thing to
    /// capture is a fourth thing a person has to be told about, and this is
    /// where somebody adding one meets the promise.
    #[test]
    fn there_are_three_and_each_is_written_down_as_itself() {
        let region = Region::of(5, 6, 7, 8).unwrap();
        let every = [
            What::TheWholeScreen,
            What::OneWindow(Window::of(
                WindowId::recorded(7),
                &WhoseSession::of("anna"),
                region,
            )),
            What::APartOfIt(region),
        ];
        let named: BTreeSet<&str> = every.iter().map(What::named).collect();
        assert_eq!(named.len(), 3);
        assert_eq!(
            every.iter().filter(|what| what.window().is_some()).count(),
            1
        );
    }
}
