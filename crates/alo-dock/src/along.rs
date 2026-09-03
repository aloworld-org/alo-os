//! Which way the dock runs, and the one thing that never turns with it.
//!
//! A dock has a long edge and a short one. The long edge is the one it runs
//! along — across the screen for a dock at the bottom or the top, down the
//! screen for one at the left or the right — and the short edge is its
//! thickness, which is the edge `docs/features.md` says the labels answer to.
//!
//! **Text never turns.** A dock that runs down the screen does not rotate its
//! names ninety degrees: rotated text is unreadable at a glance, a magnifier
//! shows it sideways, and no screen reader or text-selection behaviour anywhere
//! else in the system expects it. So a name in a vertical dock sits *beside* its
//! icon and still reads left to right — which is why the two orientations need
//! different amounts of room ([`crate::Room`]) rather than the same amount
//! turned on its side.

/// Which way the dock runs along the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Along {
    /// Across the screen: a dock at the bottom or the top.
    Across,
    /// Down the screen: a dock at the left or the right.
    Down,
}

impl Along {
    /// Which way text runs, whichever way the dock does.
    ///
    /// Always [`Along::Across`], and this is a method rather than a comment so
    /// that a later change that wants to rotate a label has to delete a rule
    /// instead of adding one.
    #[must_use]
    pub const fn text_runs() -> Self {
        Self::Across
    }

    /// The other one, which is what the dock's thickness is measured across.
    #[must_use]
    pub const fn across_the_short_edge(self) -> Self {
        match self {
            Self::Across => Self::Down,
            Self::Down => Self::Across,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A name is never rotated**, in either orientation. The reason is in this
    /// file's own documentation, and this is what stops it becoming advice.
    #[test]
    fn text_runs_the_same_way_whichever_way_the_dock_does() {
        assert_eq!(Along::text_runs(), Along::Across);
        for along in [Along::Across, Along::Down] {
            assert_eq!(Along::text_runs(), Along::Across, "{along:?}");
        }
    }

    /// The short edge is the other direction from the long one, and asking twice
    /// is asking for the same thing back.
    #[test]
    fn the_short_edge_is_across_the_long_one() {
        assert_eq!(Along::Across.across_the_short_edge(), Along::Down);
        assert_eq!(Along::Down.across_the_short_edge(), Along::Across);
        for along in [Along::Across, Along::Down] {
            assert_eq!(
                along.across_the_short_edge().across_the_short_edge(),
                along,
                "{along:?}"
            );
        }
    }
}
