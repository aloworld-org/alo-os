//! Which way a share is cut, and which side of the cut a window goes on.

/// The direction a split runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Axis {
    /// Two shares next to each other, the boundary between them running top to
    /// bottom.
    SideBySide,
    /// One share above the other, the boundary between them running left to
    /// right.
    OneAboveTheOther,
}

impl Axis {
    /// The other one.
    #[must_use]
    pub const fn across(self) -> Self {
        match self {
            Self::SideBySide => Self::OneAboveTheOther,
            Self::OneAboveTheOther => Self::SideBySide,
        }
    }
}

/// A side of a share, and so also an edge of one.
///
/// Used both for *where the new window goes* when a share is split and for
/// *which edge of a window's share* a boundary is dragged by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    /// The left.
    Left,
    /// The right.
    Right,
    /// The top.
    Top,
    /// The bottom.
    Bottom,
}

impl Side {
    /// Every side, in the order a person reads around a rectangle.
    pub const ALL: [Self; 4] = [Self::Left, Self::Right, Self::Top, Self::Bottom];

    /// The axis a split onto this side runs along.
    #[must_use]
    pub const fn axis(self) -> Axis {
        match self {
            Self::Left | Self::Right => Axis::SideBySide,
            Self::Top | Self::Bottom => Axis::OneAboveTheOther,
        }
    }

    /// Whether this side is the first piece of a cut — the left or the top.
    #[must_use]
    pub const fn is_first(self) -> bool {
        matches!(self, Self::Left | Self::Top)
    }

    /// The side facing this one.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A side and its opposite are on one axis and on different pieces of the
    /// cut — the thing a split relies on to put two windows either side of it.
    #[test]
    fn a_side_and_its_opposite_share_an_axis_and_not_a_piece() {
        for side in Side::ALL {
            assert_eq!(side.axis(), side.opposite().axis());
            assert_ne!(side.is_first(), side.opposite().is_first());
            assert_eq!(side.opposite().opposite(), side);
            assert_ne!(side.axis(), side.axis().across());
        }
    }
}
