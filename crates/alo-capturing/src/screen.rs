//! The screen a capture is taken from, as a size.
//!
//! A screenshot of *the whole screen* has to know what the whole screen is, and
//! a selected region has to be somewhere on it. This is that, and nothing else:
//! two numbers, and the one question worth asking of them — whether a rectangle
//! is on the screen or partly off it.
//!
//! # Why there is no display here
//!
//! Which physical display a screen is, how several of them are arranged and
//! which one a window is on belong to the plan's `alo-displays` and
//! `alo-dividing`, which do not exist yet. A capture does not need them: it
//! needs the extent it is cropping against. When those crates arrive, a
//! [`Screen`] is what one of them hands over, and nothing in this crate changes.
//!
//! # And no pixels
//!
//! A [`Screen`] is a size, not a buffer. Nothing in this crate ever holds what
//! is on the screen except as bytes the rented mechanism handed back
//! ([`crate::Picture`]), and nothing here reads a display.

use crate::refusing::NotTaken;

/// How big the screen a capture is taken from is, in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Screen {
    /// How wide it is.
    width: u32,
    /// How tall it is.
    height: u32,
}

impl Screen {
    /// A screen this many pixels across and this many down.
    ///
    /// # Errors
    /// [`NotTaken::NotAnAreaOfTheScreen`] for a screen with no width or no
    /// height. A machine whose screen is nothing is one where every region is
    /// off it, and saying so once here is better than every caller deciding
    /// what a zero-sized screen means.
    pub const fn measuring(width: u32, height: u32) -> Result<Self, NotTaken> {
        if width == 0 || height == 0 {
            return Err(NotTaken::NotAnAreaOfTheScreen);
        }
        Ok(Self { width, height })
    }

    /// How wide it is.
    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    /// How tall it is.
    #[must_use]
    pub const fn height(self) -> u32 {
        self.height
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A screen is its two numbers**, and they come back as they went in.
    #[test]
    fn a_screen_is_how_wide_and_how_tall_it_is() {
        let screen = Screen::measuring(2560, 1440).unwrap();
        assert_eq!(screen.width(), 2560);
        assert_eq!(screen.height(), 1440);
    }

    /// **A screen with no width or no height is not a screen.** Every region is
    /// off such a screen, and a caller that had to work that out for itself is
    /// a caller that eventually does not.
    #[test]
    fn a_screen_with_no_size_is_refused() {
        assert_eq!(
            Screen::measuring(0, 1440),
            Err(NotTaken::NotAnAreaOfTheScreen)
        );
        assert_eq!(
            Screen::measuring(2560, 0),
            Err(NotTaken::NotAnAreaOfTheScreen)
        );
        assert_eq!(Screen::measuring(0, 0), Err(NotTaken::NotAnAreaOfTheScreen));
    }
}
