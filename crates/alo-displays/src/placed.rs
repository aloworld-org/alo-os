//! Where one screen sits in a person's arrangement, how large it draws, and
//! whether it is the main one.
//!
//! A position is the top left corner of the screen, in the pixels the whole
//! arrangement is laid out in — so a screen to the left of the main one has a
//! negative first number, which is exactly the case the laptop that forgets
//! gets wrong every morning.
//!
//! **A screen takes less room the larger it draws.** A 4K panel at 200% is
//! 1920 wide in an arrangement, not 3840, because everything on it is drawn
//! twice the size. That is why [`Placed::overlaps`] is asked with the pixels
//! the screen reports now rather than with a size written in the file: a screen
//! whose resolution changed has moved, whatever the file says.

use serde::{Deserialize, Serialize};

use crate::reported::Resolution;
use crate::scale::Scale;

/// Where the top left corner of a screen is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(from = "(i32, i32)", into = "(i32, i32)")]
pub struct Position {
    /// How far across, which is negative to the left of the main screen.
    across: i32,
    /// How far down, which is negative above it.
    down: i32,
}

impl Position {
    /// This far across and this far down.
    #[must_use]
    pub const fn at(across: i32, down: i32) -> Self {
        Self { across, down }
    }

    /// The corner every arrangement is measured from.
    #[must_use]
    pub const fn the_origin() -> Self {
        Self { across: 0, down: 0 }
    }

    /// How far across.
    #[must_use]
    pub const fn across(self) -> i32 {
        self.across
    }

    /// How far down.
    #[must_use]
    pub const fn down(self) -> i32 {
        self.down
    }
}

impl From<(i32, i32)> for Position {
    fn from((across, down): (i32, i32)) -> Self {
        Self { across, down }
    }
}

impl From<Position> for (i32, i32) {
    fn from(position: Position) -> Self {
        (position.across, position.down)
    }
}

/// One screen's place in an arrangement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placed {
    /// Where its top left corner is.
    at: Position,
    /// How large everything on it is drawn.
    scale: Scale,
    /// Whether it is the one a new window opens on.
    main: bool,
}

impl Placed {
    /// A screen here, drawn at this size, that is not the main one.
    #[must_use]
    pub const fn at(at: Position, scale: Scale) -> Self {
        Self {
            at,
            scale,
            main: false,
        }
    }

    /// The same place, as the main screen.
    #[must_use]
    pub const fn as_the_main_screen(self) -> Self {
        Self { main: true, ..self }
    }

    /// The same place, at another size.
    #[must_use]
    pub const fn at_this_size(self, scale: Scale) -> Self {
        Self { scale, ..self }
    }

    /// Where its top left corner is.
    #[must_use]
    pub const fn position(self) -> Position {
        self.at
    }

    /// How large everything on it is drawn.
    #[must_use]
    pub const fn scale(self) -> Scale {
        self.scale
    }

    /// Whether it is the one a new window opens on.
    #[must_use]
    pub const fn is_the_main_screen(self) -> bool {
        self.main
    }

    /// How much room this screen takes in the arrangement, given the pixels it
    /// reports now: across, then down.
    #[must_use]
    pub const fn room(self, pixels: Resolution) -> (u32, u32) {
        (
            self.scale.laid_out(pixels.width()),
            self.scale.laid_out(pixels.height()),
        )
    }

    /// Whether these two screens would be drawn over each other.
    ///
    /// Touching is not overlapping: two screens side by side share an edge, and
    /// an arrangement in which they did not would have a seam of nothing down
    /// the middle of somebody's desk.
    #[must_use]
    pub const fn overlaps(self, mine: Resolution, other: Self, theirs: Resolution) -> bool {
        let (my_width, my_height) = self.room(mine);
        let (their_width, their_height) = other.room(theirs);
        let my_right = self.at.across.saturating_add_unsigned(my_width);
        let my_bottom = self.at.down.saturating_add_unsigned(my_height);
        let their_right = other.at.across.saturating_add_unsigned(their_width);
        let their_bottom = other.at.down.saturating_add_unsigned(their_height);
        self.at.across < their_right
            && other.at.across < my_right
            && self.at.down < their_bottom
            && other.at.down < my_bottom
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::a_screen;

    /// A screen to the left of the main one is at a negative number, and comes
    /// back out of a settings file as the pair it went in as.
    #[test]
    fn a_screen_to_the_left_is_at_a_negative_number_and_survives_a_file() {
        let left = Position::at(-1920, 0);
        assert_eq!(left.across(), -1920);
        assert_eq!(left.down(), 0);
        assert_eq!(
            serde_json::from_str::<Position>("[-1920, 0]").unwrap(),
            left
        );
        assert_eq!(serde_json::to_string(&left).unwrap(), "[-1920,0]");
        assert_eq!(Position::the_origin(), Position::at(0, 0));
    }

    /// **A screen takes less room the larger it draws.** A 4K panel at 200% is
    /// half as wide in an arrangement as its pixels are.
    #[test]
    fn a_screen_takes_less_room_the_larger_it_draws() {
        let (pixels, _) = a_screen((3840, 2160), None);
        let hundred = Placed::at(Position::the_origin(), Scale::a_hundred());
        assert_eq!(hundred.room(pixels), (3840, 2160));
        let doubled = hundred.at_this_size(Scale::per_cent(200).unwrap());
        assert_eq!(doubled.room(pixels), (1920, 1080));
    }

    /// **Two screens side by side touch and do not overlap**, and one moved a
    /// pixel back over the other does.
    #[test]
    fn side_by_side_is_not_overlapping_and_one_pixel_over_is() {
        let (pixels, _) = a_screen((1920, 1080), None);
        let left = Placed::at(Position::the_origin(), Scale::a_hundred());
        let right = Placed::at(Position::at(1920, 0), Scale::a_hundred());
        assert!(!left.overlaps(pixels, right, pixels));
        assert!(!right.overlaps(pixels, left, pixels));

        let over = Placed::at(Position::at(1919, 0), Scale::a_hundred());
        assert!(left.overlaps(pixels, over, pixels));
        assert!(over.overlaps(pixels, left, pixels));

        let above = Placed::at(Position::at(0, -1080), Scale::a_hundred());
        assert!(!left.overlaps(pixels, above, pixels));
    }

    /// The main screen is a property of a place, and there is one road to it.
    #[test]
    fn the_main_screen_is_a_property_of_a_place() {
        let ordinary = Placed::at(Position::the_origin(), Scale::a_hundred());
        assert!(!ordinary.is_the_main_screen());
        assert!(ordinary.as_the_main_screen().is_the_main_screen());
        assert_eq!(
            ordinary.as_the_main_screen().position(),
            ordinary.position()
        );
    }
}
