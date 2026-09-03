//! Which edge of the screen the dock is on.
//!
//! The whole of what a person chooses about the dock at v0.01: four edges, one
//! of them at a time, picked in Settings. *The dock's size*, *whether it hides
//! when a window needs the room* and *one dock per display* are all v0.5 in
//! `docs/features.md` and are deliberately not choices here — a setting built
//! early is a setting somebody has to keep working for a release it was not
//! designed for.
//!
//! **An edge is a side of the screen, not a side of the reading.** *Left* means
//! the physical left, and it stays there for somebody reading Arabic. What does
//! follow the reading is where the status area sits along the dock, which is
//! [`crate::StatusArea`]'s and is a different question: the dock's *position* is
//! furniture a person put somewhere, and the *far end* of a row is a fact about
//! how they read it.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::along::Along;
use crate::words::{self, Word};

/// Which edge of the screen the dock sits on.
///
/// Written into the settings file by name, so a file written by an older release
/// still says what its owner chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Edge {
    /// Along the bottom of the screen, which is what a machine ships with.
    Bottom,
    /// Down the left of the screen.
    Left,
    /// Down the right of the screen.
    Right,
    /// Along the top of the screen.
    Top,
}

impl Edge {
    /// The four, in the order a settings panel offers them: the one a machine
    /// ships with first, then the two sides, then the top.
    pub const ALL: [Self; 4] = [Self::Bottom, Self::Left, Self::Right, Self::Top];

    /// Which way a dock on this edge runs.
    #[must_use]
    pub const fn along(self) -> Along {
        match self {
            Self::Bottom | Self::Top => Along::Across,
            Self::Left | Self::Right => Along::Down,
        }
    }

    /// The string this crate declares for this edge.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Bottom => words::BOTTOM,
            Self::Left => words::LEFT,
            Self::Right => words::RIGHT,
            Self::Top => words::TOP,
        }
    }

    /// What this edge is called, in the language the person reads. Never fails
    /// and never panics.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use std::collections::BTreeSet;

    /// The two edges along the top and the bottom run across the screen, and the
    /// two down the sides run down it. That is the whole of the derivation, and
    /// it is what makes the orientation something nobody sets separately.
    #[test]
    fn which_way_the_dock_runs_follows_from_the_edge_it_is_on() {
        assert_eq!(Edge::Bottom.along(), Along::Across);
        assert_eq!(Edge::Top.along(), Along::Across);
        assert_eq!(Edge::Left.along(), Along::Down);
        assert_eq!(Edge::Right.along(), Along::Down);
    }

    /// Four edges, each with a string of its own, and the list a panel shows
    /// starts with the one a fresh machine already has.
    #[test]
    fn there_are_four_edges_and_each_is_named() {
        assert_eq!(Edge::ALL.len(), 4);
        assert_eq!(Edge::ALL.first(), Some(&Edge::Bottom));
        let keys: BTreeSet<String> = Edge::ALL
            .iter()
            .map(|edge| edge.word().key().to_string())
            .collect();
        assert_eq!(keys.len(), 4, "one string each");

        let strings = in_english();
        let named: Vec<String> = Edge::ALL
            .iter()
            .map(|edge| edge.said(&strings).into_text())
            .collect();
        assert_eq!(named, ["Bottom", "Left", "Right", "Top"]);
    }

    /// The list a person picks from is read in their language, and an untranslated
    /// machine still answers with a word rather than with a key.
    #[test]
    fn the_list_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[
            (words::BOTTOM, "Unten"),
            (words::LEFT, "Links"),
            (words::RIGHT, "Rechts"),
            (words::TOP, "Oben"),
        ]);
        let named: Vec<String> = Edge::ALL
            .iter()
            .map(|edge| edge.said(&strings).into_text())
            .collect();
        assert_eq!(named, ["Unten", "Links", "Rechts", "Oben"]);
        for edge in Edge::ALL {
            assert!(edge.said(&strings).is_translated(), "{edge:?}");
        }
    }

    /// **An edge is written down by name.** A number would mean a settings file
    /// that stops meaning what it said the moment somebody reorders the list.
    #[test]
    fn an_edge_is_written_by_name() {
        assert_eq!(serde_json::to_string(&Edge::Left).unwrap(), "\"Left\"");
        assert_eq!(
            serde_json::from_str::<Edge>("\"Top\"").unwrap(),
            Edge::Top,
            "and reads back"
        );
        assert!(
            serde_json::from_str::<Edge>("\"Middle\"").is_err(),
            "an edge a person invented is refused where the file is read"
        );
    }
}
