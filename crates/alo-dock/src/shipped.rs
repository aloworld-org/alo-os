//! What the dock is before anybody changes anything.
//!
//! **Where it is, is not here any more.** The dock is along the bottom and that
//! is not a default — it is [ADR
//! 0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md),
//! and a fixed position belongs in the layout rather than in a table of things a
//! release can move. This file held an edge until then, and the reason the bottom
//! was chosen is now the record's.
//!
//! What is left is **whether it gives way when a window needs the room**, which
//! is a real choice with a real default: it does not. A dock that vanished on the
//! first morning would be a dock somebody has to go looking for.
//!
//! As in `alo-appearance` and `alo-shortcuts`, what ships lives in the running
//! release rather than in the settings file, so a release can move this and
//! reach every machine that never touched it.

use crate::hiding::Hiding;

/// What the dock is before anybody changes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shipped {
    /// Whether it gives way when a window needs the room.
    hiding: Hiding,
}

impl Shipped {
    /// The dock the image ships.
    #[must_use]
    pub const fn of_the_image() -> Self {
        Self {
            hiding: Hiding::Never,
        }
    }

    /// A different default — a release being tried out against a person's
    /// changes, or a test of what a new default would do to them.
    #[must_use]
    pub const fn of(hiding: Hiding) -> Self {
        Self { hiding }
    }

    /// Whether the dock gives way when a window needs the room.
    #[must_use]
    pub const fn hiding(self) -> Hiding {
        self.hiding
    }
}

impl Default for Shipped {
    fn default() -> Self {
        Self::of_the_image()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A dock somebody can find on the first morning.** The shipped answer is
    /// that it never gives way, which is also [`Hiding`]'s own default — two
    /// files agreeing rather than one of them deciding quietly.
    #[test]
    fn the_shipped_dock_does_not_give_way() {
        let shipped = Shipped::of_the_image();
        assert_eq!(shipped.hiding(), Hiding::Never);
        assert_eq!(shipped.hiding(), Hiding::default());
        assert_eq!(shipped, Shipped::default());
    }

    /// A release trying out a different default is an ordinary thing to build.
    #[test]
    fn a_different_release_can_ship_a_different_answer() {
        let other = Shipped::of(Hiding::WhenAWindowNeedsTheRoom);
        assert_ne!(other, Shipped::of_the_image());
        assert_eq!(other.hiding(), Hiding::WhenAWindowNeedsTheRoom);
    }
}
