//! A rectangle of the screen: the *selected region* of the promise.
//!
//! Four numbers, with the screen's own origin at the top left, and one rule:
//! **a region is a part of one screen, and it is entirely on it.** A rectangle
//! that ran off the edge would be a capture whose contents depend on what the
//! rented mechanism does about the overhang, which is a question nobody should
//! have to answer twice.
//!
//! # No negative corner
//!
//! Where several displays are arranged around each other, a machine may put one
//! of them at a negative coordinate. That arrangement is `alo-dividing`'s, and
//! the crate does not exist; what reaches here is a rectangle **on one screen**,
//! whose origin is that screen's own top left. A capture that spans two displays
//! is not something this task promises, and modelling it with a signed corner
//! before anything can arrange displays would be modelling a guess.
//!
//! # Nothing here knows what is inside it
//!
//! A [`Region`] is where to cut, not what was cut. The pixels come back from
//! the rented mechanism and go straight to a file or the clipboard; nothing in
//! this crate looks at them, which is also why the file's name can promise to
//! say nothing about what was on the screen ([`crate::naming`]).

use crate::refusing::NotTaken;
use crate::screen::Screen;

/// A rectangle of one screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Region {
    /// How far from the screen's left edge it starts.
    from_the_left: u32,
    /// How far from the screen's top edge it starts.
    from_the_top: u32,
    /// How wide it is.
    width: u32,
    /// How tall it is.
    height: u32,
}

impl Region {
    /// The rectangle starting this far in and this far down, this big.
    ///
    /// # Errors
    /// [`NotTaken::NotAnAreaOfTheScreen`] for a rectangle with no width or no
    /// height. A person who dragged a selection and let go without moving has
    /// selected nothing, and a picture of nothing is not what they meant.
    pub const fn of(
        from_the_left: u32,
        from_the_top: u32,
        width: u32,
        height: u32,
    ) -> Result<Self, NotTaken> {
        if width == 0 || height == 0 {
            return Err(NotTaken::NotAnAreaOfTheScreen);
        }
        Ok(Self {
            from_the_left,
            from_the_top,
            width,
            height,
        })
    }

    /// The rectangle that is all of that screen.
    ///
    /// Cannot fail, and that is a property of [`Screen`] rather than a check
    /// here: a screen cannot be built with no width or no height, so the
    /// rectangle over all of one always has both.
    #[must_use]
    pub const fn over(screen: Screen) -> Self {
        Self {
            from_the_left: 0,
            from_the_top: 0,
            width: screen.width(),
            height: screen.height(),
        }
    }

    /// How far from the screen's left edge it starts.
    #[must_use]
    pub const fn from_the_left(self) -> u32 {
        self.from_the_left
    }

    /// How far from the screen's top edge it starts.
    #[must_use]
    pub const fn from_the_top(self) -> u32 {
        self.from_the_top
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

    /// Whether the whole of this rectangle is on that screen.
    ///
    /// Saturating throughout, because a rectangle far enough out to overflow a
    /// number is off the screen, and an addition that wrapped would say it was
    /// on it.
    #[must_use]
    pub const fn is_on(self, screen: Screen) -> bool {
        let right = self.from_the_left.saturating_add(self.width);
        let bottom = self.from_the_top.saturating_add(self.height);
        right <= screen.width() && bottom <= screen.height()
    }

    /// How much of that screen is to the right of this rectangle.
    ///
    /// What the rented mechanism is told to cut away, alongside
    /// [`Region::from_the_left`]. Zero when the rectangle is not on the screen
    /// at all, which cannot reach the mechanism: [`Region::is_on`] is asked
    /// first, and a rectangle that is off the screen is refused before anything
    /// is taken.
    #[must_use]
    pub const fn to_the_right_on(self, screen: Screen) -> u32 {
        screen
            .width()
            .saturating_sub(self.from_the_left.saturating_add(self.width))
    }

    /// How much of that screen is below this rectangle.
    #[must_use]
    pub const fn below_on(self, screen: Screen) -> u32 {
        screen
            .height()
            .saturating_sub(self.from_the_top.saturating_add(self.height))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A screen these tests are about.
    fn a_screen() -> Screen {
        Screen::measuring(1920, 1080).unwrap()
    }

    /// **A region is its four numbers**, and they come back as they went in.
    #[test]
    fn a_region_is_where_it_starts_and_how_big_it_is() {
        let region = Region::of(100, 50, 400, 300).unwrap();
        assert_eq!(region.from_the_left(), 100);
        assert_eq!(region.from_the_top(), 50);
        assert_eq!(region.width(), 400);
        assert_eq!(region.height(), 300);
    }

    /// **A selection nobody dragged is not a region.** Letting go of the mouse
    /// without moving it selects nothing, and a picture of nothing is not what
    /// anybody meant.
    #[test]
    fn a_region_with_no_width_or_no_height_is_refused() {
        assert_eq!(
            Region::of(100, 50, 0, 300),
            Err(NotTaken::NotAnAreaOfTheScreen)
        );
        assert_eq!(
            Region::of(100, 50, 400, 0),
            Err(NotTaken::NotAnAreaOfTheScreen)
        );
    }

    /// **A region entirely on the screen is on it, and one hanging over an edge
    /// is not.** Exactly filling the screen counts as on it.
    #[test]
    fn a_region_is_on_the_screen_or_it_is_not() {
        let screen = a_screen();
        assert!(Region::of(0, 0, 1920, 1080).unwrap().is_on(screen));
        assert!(Region::of(1919, 1079, 1, 1).unwrap().is_on(screen));
        assert!(!Region::of(1, 0, 1920, 1080).unwrap().is_on(screen));
        assert!(!Region::of(0, 1, 1920, 1080).unwrap().is_on(screen));
        assert!(!Region::of(2000, 0, 10, 10).unwrap().is_on(screen));
    }

    /// **A rectangle far enough out to overflow a number is off the screen.**
    /// An addition that wrapped would report it as on the screen, and the crop
    /// handed to the rented mechanism would be somebody else's pixels.
    #[test]
    fn a_region_that_would_overflow_is_off_the_screen() {
        let screen = a_screen();
        let far = Region::of(u32::MAX, u32::MAX, 10, 10).unwrap();
        assert!(!far.is_on(screen));
        assert_eq!(far.to_the_right_on(screen), 0);
        assert_eq!(far.below_on(screen), 0);
    }

    /// **What is cut away is what is left over**, on each of the four sides,
    /// and the four add back up to the screen.
    #[test]
    fn what_is_cut_away_adds_back_up_to_the_screen() {
        let screen = a_screen();
        let region = Region::of(100, 50, 400, 300).unwrap();
        assert_eq!(region.to_the_right_on(screen), 1920 - 100 - 400);
        assert_eq!(region.below_on(screen), 1080 - 50 - 300);
        assert_eq!(
            region.from_the_left() + region.width() + region.to_the_right_on(screen),
            screen.width()
        );
        assert_eq!(
            region.from_the_top() + region.height() + region.below_on(screen),
            screen.height()
        );
    }
}
