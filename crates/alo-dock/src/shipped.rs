//! Where the dock is before anybody moves it.
//!
//! **The bottom**, because that is where somebody arriving from Windows or from
//! macOS will look for it, and a system that started somewhere else in order to
//! be different would be a system deciding something on their behalf on the
//! first morning. `docs/features.md` promises the person decides where it goes;
//! deciding is easier from a place they recognise.
//!
//! As in `alo-appearance` and `alo-shortcuts`, what ships lives in the running
//! release rather than in the settings file, so a release can move this and
//! reach every machine that never touched it.

use crate::edge::Edge;

/// What the dock is before anybody changes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shipped {
    /// Which edge it is on.
    edge: Edge,
}

impl Shipped {
    /// The dock the image ships.
    #[must_use]
    pub const fn of_the_image() -> Self {
        Self { edge: Edge::Bottom }
    }

    /// A different default — a release being tried out against a person's
    /// changes, or a test of what a new default would do to them.
    #[must_use]
    pub const fn of(edge: Edge) -> Self {
        Self { edge }
    }

    /// Which edge the dock is on.
    #[must_use]
    pub const fn edge(self) -> Edge {
        self.edge
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

    /// **What ships is an edge a person could also have chosen.** A default
    /// outside the offered set would be a position somebody could lose by
    /// touching the setting and never get back.
    #[test]
    fn what_ships_is_one_of_the_four_a_person_is_offered() {
        let shipped = Shipped::of_the_image();
        assert_eq!(shipped.edge(), Edge::Bottom);
        assert!(Edge::ALL.contains(&shipped.edge()));
        assert_eq!(shipped, Shipped::default());
    }

    /// A release trying out a different default is an ordinary thing to build.
    #[test]
    fn a_different_release_can_ship_a_different_edge() {
        let other = Shipped::of(Edge::Left);
        assert_ne!(other, Shipped::of_the_image());
        assert_eq!(other.edge(), Edge::Left);
    }
}
