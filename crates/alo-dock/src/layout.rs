//! The whole answer: an edge, a screen and a text size, laid out.
//!
//! This is where `docs/features.md`'s *labels give way to icons where the short
//! edge demands it* becomes arithmetic, and the arithmetic is short enough to
//! read in one sitting:
//!
//! 1. The edge says which way the dock runs, and which side of the screen it
//!    takes its thickness from ([`crate::Screen`]).
//! 2. That side has a ceiling: the most a dock may take of it
//!    ([`crate::Room::the_most_a_dock_may_take`]).
//! 3. A dock with names on it wants a thickness that depends on the text size —
//!    a line of text under each icon when it runs across the screen, a name's
//!    worth of width beside each icon when it runs down.
//! 4. If what it wants fits under the ceiling, the names are drawn. If it does
//!    not, they give way and the dock is icons alone — which always fits,
//!    because [`crate::Screen`] refuses a screen where it would not.
//!
//! **Nothing here is a judgement.** There is no *feels cramped*, no breakpoint
//! list and no eye. Every number comes from [`crate::measures`], and the
//! thresholds those numbers produce are held to EN 301 549's requirement that
//! text reach 200% without losing content — on the smallest screen alo OS lays
//! out for, on all four edges. The tests at the bottom of this file are that
//! requirement, and they are also what fixes the two numbers nobody could have
//! picked honestly: the share of a side a dock may take, and how much room a
//! name needs beside an icon.
//!
//! **Nothing here reads anything either.** The screen is passed in, the text
//! size is passed in, and which way the person reads is passed in — the rule
//! `alo-capability` set in item 1 and `alo-appearance` kept, so a settings panel
//! previewing *what would this look like at 200%* asks exactly the question the
//! compositor asks at 200%.

use alo_appearance::TextScale;
use alo_strings::Direction;

use crate::along::Along;
use crate::edge::Edge;
use crate::labels::Labels;
use crate::room::Room;
use crate::screen::Screen;
use crate::status::StatusArea;

/// A dock, laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    /// Which edge it is on.
    edge: Edge,
    /// Which way it runs.
    along: Along,
    /// Its short edge: how much of the screen it takes.
    thickness: Room,
    /// Its long edge: how far it runs.
    length: Room,
    /// What became of the names.
    labels: Labels,
    /// Where the status area is.
    status: StatusArea,
}

impl Layout {
    /// A dock on this edge of this screen, with the text at this size, read by
    /// somebody reading this way.
    ///
    /// Answers rather than refuses: a [`Screen`] that exists is a screen a dock
    /// fits on, because that is what [`Screen::of`] checks.
    #[must_use]
    pub fn of(edge: Edge, screen: Screen, text: TextScale, reading: Direction) -> Self {
        let along = edge.along();
        let ceiling = Room::the_most_a_dock_may_take(screen.the_side_a_dock_takes_from(along));
        let (with_names, where_they_go) = match along {
            Along::Across => (Room::a_dock_with_names_under(text), Labels::Under),
            Along::Down => (Room::a_dock_with_names_beside(text), Labels::Beside),
        };
        let (thickness, labels) = if with_names.fits_in(ceiling) {
            (with_names, where_they_go)
        } else {
            (Room::a_dock_of_icons(), Labels::GaveWay(text.as_percent()))
        };
        Self {
            edge,
            along,
            thickness,
            length: screen.the_side_a_dock_runs_along(along),
            labels,
            status: StatusArea::of(along, reading),
        }
    }

    /// Which edge it is on.
    #[must_use]
    pub const fn edge(self) -> Edge {
        self.edge
    }

    /// Which way it runs.
    #[must_use]
    pub const fn along(self) -> Along {
        self.along
    }

    /// How much of the screen it takes, across its short edge.
    #[must_use]
    pub const fn thickness(self) -> Room {
        self.thickness
    }

    /// How far it runs along the edge it is on, which at v0.01 is the whole of
    /// that edge. *Whether it hides when a window needs the room* is v0.5.
    #[must_use]
    pub const fn length(self) -> Room {
        self.length
    }

    /// What became of the names.
    #[must_use]
    pub const fn labels(self) -> Labels {
        self.labels
    }

    /// Where the status area is, and which way it runs.
    #[must_use]
    pub const fn status(self) -> StatusArea {
        self.status
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::measures::{A_DOCK_MAY_TAKE_ONE_PART_IN, LABEL_EMS, THE_STANDARDS_TEXT};
    use crate::status::End;

    /// A size the tests name often enough to be worth a word.
    fn text(percent: u16) -> TextScale {
        TextScale::percent(percent).unwrap()
    }

    /// A dock on this edge of the smallest screen alo OS lays out for.
    fn on_the_smallest(edge: Edge, percent: u16) -> Layout {
        Layout::of(
            edge,
            Screen::the_smallest(),
            text(percent),
            Direction::LeftToRight,
        )
    }

    /// **EN 301 549 is a test, not a sentence.** The standard an EU
    /// public-sector desktop is procured against requires text to reach 200%
    /// without loss of content, so a dock on the smallest screen alo OS lays out
    /// for still has its names at that size — on all four edges, not only on the
    /// two easy ones.
    ///
    /// This is the test that fixes the two numbers nobody could have picked by
    /// eye. Loosen either and the dock takes more of somebody's screen than it
    /// has any claim to; tighten either and this fails.
    #[test]
    fn names_survive_the_text_size_the_standard_requires() {
        for edge in Edge::ALL {
            let layout = on_the_smallest(edge, THE_STANDARDS_TEXT);
            assert!(
                layout.labels().are_shown(),
                "{edge:?} lost its names at {THE_STANDARDS_TEXT}% on the smallest screen"
            );
        }
        for edge in [Edge::Bottom, Edge::Top] {
            assert_eq!(
                on_the_smallest(edge, THE_STANDARDS_TEXT).labels(),
                Labels::Under
            );
        }
        for edge in [Edge::Left, Edge::Right] {
            assert_eq!(
                on_the_smallest(edge, THE_STANDARDS_TEXT).labels(),
                Labels::Beside
            );
        }
    }

    /// **And the rule is not vacuous.** Above what the standard requires, on the
    /// smallest screen, the names do give way — so *labels give way to icons
    /// where the short edge demands it* is a thing that happens rather than a
    /// branch nothing reaches.
    #[test]
    fn names_give_way_when_the_short_edge_finally_demands_it() {
        let (_, largest) = TextScale::range();
        for edge in Edge::ALL {
            let layout = on_the_smallest(edge, largest);
            assert_eq!(
                layout.labels(),
                Labels::GaveWay(largest),
                "{edge:?} kept its names at {largest}%"
            );
            assert_eq!(layout.thickness(), Room::a_dock_of_icons());
        }
    }

    /// **A bigger screen keeps its names longer**, because the ceiling is a
    /// share of the side rather than a fixed number of pixels. The same person
    /// with the same text size gets names on the desk and icons on the laptop,
    /// which is the behaviour a share buys.
    #[test]
    fn a_bigger_screen_keeps_its_names_at_a_size_a_small_one_cannot() {
        let (_, largest) = TextScale::range();
        let desk = Screen::of(3840, 2160).unwrap();
        for edge in Edge::ALL {
            let big = Layout::of(edge, desk, text(largest), Direction::LeftToRight);
            assert!(big.labels().are_shown(), "{edge:?} on a large screen");
            assert!(!on_the_smallest(edge, largest).labels().are_shown());
        }
    }

    /// **The dock never takes more than its share**, at any text size, on any
    /// edge, on any screen — which is the promise the ceiling exists to keep and
    /// the reason [`Layout::of`] can answer without a `Result`.
    #[test]
    fn the_dock_never_takes_more_of_a_screen_than_it_may() {
        let (smallest, largest) = TextScale::range();
        let screens = [
            Screen::the_smallest(),
            Screen::of(1920, 1080).unwrap(),
            Screen::of(3840, 2160).unwrap(),
            Screen::of(1080, 1920).unwrap(),
            Screen::of(384, 384).unwrap(),
        ];
        for screen in screens {
            for edge in Edge::ALL {
                for percent in smallest..=largest {
                    let layout = Layout::of(edge, screen, text(percent), Direction::LeftToRight);
                    let ceiling = Room::the_most_a_dock_may_take(
                        screen.the_side_a_dock_takes_from(edge.along()),
                    );
                    assert!(
                        layout.thickness().fits_in(ceiling),
                        "{edge:?} at {percent}% took {} of a ceiling of {}",
                        layout.thickness().as_pixels(),
                        ceiling.as_pixels()
                    );
                }
            }
        }
    }

    /// **Names never come back once they have gone.** A person turning their
    /// text up one step at a time meets the change once; a dock whose labels
    /// flickered back at a larger size would be a layout nobody could describe.
    #[test]
    fn once_the_names_have_given_way_a_larger_size_never_brings_them_back() {
        let (smallest, largest) = TextScale::range();
        for edge in Edge::ALL {
            let mut gone = false;
            for percent in smallest..=largest {
                let shown = on_the_smallest(edge, percent).labels().are_shown();
                if gone {
                    assert!(!shown, "{edge:?} got its names back at {percent}%");
                }
                gone |= !shown;
            }
            assert!(
                gone,
                "{edge:?} never gives way at all, so nothing was tested"
            );
        }
    }

    /// **Two edges that run the same way lay out the same.** The bottom and the
    /// top are one layout on two edges, and so are the left and the right —
    /// which is what says the orientation is doing the work rather than four
    /// separate cases somebody wrote out.
    #[test]
    fn edges_that_run_the_same_way_are_one_layout() {
        for percent in [75, 100, 200, 300] {
            for (one, other) in [(Edge::Bottom, Edge::Top), (Edge::Left, Edge::Right)] {
                let first = on_the_smallest(one, percent);
                let second = on_the_smallest(other, percent);
                assert_eq!(first.thickness(), second.thickness(), "at {percent}%");
                assert_eq!(first.labels(), second.labels(), "at {percent}%");
                assert_eq!(first.length(), second.length(), "at {percent}%");
                assert_eq!(first.status(), second.status(), "at {percent}%");
                assert_ne!(first.edge(), second.edge());
            }
        }
    }

    /// The dock spans the edge it is on, and the status area is at the far end
    /// of it — which for a dock down the side of the screen is the bottom,
    /// whichever way the person reads.
    #[test]
    fn the_dock_spans_its_edge_and_the_status_area_is_at_the_far_end() {
        let screen = Screen::the_smallest();
        let across = Layout::of(Edge::Bottom, screen, text(100), Direction::LeftToRight);
        assert_eq!(across.length(), screen.width());
        assert_eq!(across.status().at(), End::Right);
        assert_eq!(across.status().runs(), Along::Across);

        let down = Layout::of(Edge::Left, screen, text(100), Direction::RightToLeft);
        assert_eq!(down.length(), screen.height());
        assert_eq!(down.status().at(), End::Bottom);
        assert_eq!(down.status().runs(), Along::Down);
    }

    /// **The share a dock may take is the tightest one that keeps the
    /// standard.** One part more and the names would go at exactly the size
    /// EN 301 549 requires them to survive — so the number in
    /// [`crate::measures`] is fixed by the requirement rather than chosen, and
    /// this is the test that says which way it is fixed.
    #[test]
    fn the_share_and_the_name_floor_are_as_tight_as_the_standard_allows() {
        let screen = Screen::the_smallest();
        let standard = text(THE_STANDARDS_TEXT);

        for (along, wanted) in [
            (Along::Across, Room::a_dock_with_names_under(standard)),
            (Along::Down, Room::a_dock_with_names_beside(standard)),
        ] {
            let side = screen.the_side_a_dock_takes_from(along).as_pixels();
            let tighter = Room::pixels(side / (A_DOCK_MAY_TAKE_ONE_PART_IN + 1));
            assert!(
                !wanted.fits_in(tighter),
                "a share of one part in {} would still fit, so the chosen share is loose",
                A_DOCK_MAY_TAKE_ONE_PART_IN + 1
            );
        }

        // And a name asked for one em more would not fit down the side of that
        // screen at the size the standard requires.
        let ceiling =
            Room::the_most_a_dock_may_take(screen.the_side_a_dock_takes_from(Along::Down));
        let one_em_more = Room::a_dock_with_names_beside(standard).and(Room::text_at(standard));
        assert!(
            !one_em_more.fits_in(ceiling),
            "a floor of {} ems would still fit, so the chosen floor is loose",
            LABEL_EMS + 1
        );
    }
}
