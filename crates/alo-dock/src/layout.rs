//! The whole answer: a screen and a text size, laid out along the bottom.
//!
//! This is where `docs/features.md`'s *labels give way to icons where the short
//! edge demands it* becomes arithmetic, and the arithmetic is short enough to
//! read in one sitting:
//!
//! 1. The dock is along the bottom ([ADR
//!    0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)),
//!    so it takes its thickness out of the screen's **height** and runs the whole
//!    of its **width**.
//! 2. The height has a ceiling: the most a dock may take of it
//!    ([`crate::Room::the_most_a_dock_may_take`]).
//! 3. A dock with names on it wants a thickness that depends on the text size —
//!    a line of text under each icon.
//! 4. If what it wants fits under the ceiling, the names are drawn. If it does
//!    not, they give way and the dock is icons alone — which always fits,
//!    because [`crate::Screen`] refuses a screen where it would not.
//!
//! **It takes from the side it sits on, not from the screen's short side.** A
//! dock along the bottom takes from the height even on a screen that is taller
//! than it is wide, because the height is the side it is sitting on. Measuring
//! against `min(width, height)` instead would squeeze the dock on a portrait
//! screen for a reason that has nothing to do with where it is.
//!
//! **Nothing here is a judgement.** There is no *feels cramped*, no breakpoint
//! list and no eye. Every number comes from [`crate::measures`], and the
//! thresholds those numbers produce are held to EN 301 549's requirement that
//! text reach 200% without losing content — on the smallest screen alo OS lays
//! out for. The tests at the bottom of this file are that requirement, and they
//! are also what fixes the two numbers nobody could have picked honestly: the
//! share of the height a dock may take, and that a name under an icon needs a
//! line of text.
//!
//! **Nothing here reads anything either.** The screen is passed in and the text
//! size is passed in — the rule `alo-capability` set in item 1 and
//! `alo-appearance` kept, so a settings panel previewing *what would this look
//! like at 200%* asks exactly the question the compositor asks at 200%.
//!
//! **Which way the person reads is no longer asked.** It was only ever used to
//! put the status area at the far end of a row, and the status area is not the
//! Dock's any more.

use alo_appearance::TextScale;

use crate::labels::Labels;
use crate::room::Room;
use crate::screen::Screen;

/// A dock, laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    /// Its short edge: how much of the screen's height it takes.
    thickness: Room,
    /// Its long edge: how far it runs, which is the whole width.
    length: Room,
    /// What became of the names.
    labels: Labels,
}

impl Layout {
    /// A dock along the bottom of this screen, with the text at this size.
    ///
    /// Answers rather than refuses: a [`Screen`] that exists is a screen a dock
    /// fits on, because that is what [`Screen::of`] checks.
    #[must_use]
    pub fn of(screen: Screen, text: TextScale) -> Self {
        let ceiling = Room::the_most_a_dock_may_take(screen.height());
        let with_names = Room::a_dock_with_names_under(text);
        let (thickness, labels) = if with_names.fits_in(ceiling) {
            (with_names, Labels::Under)
        } else {
            (Room::a_dock_of_icons(), Labels::GaveWay(text.as_percent()))
        };
        Self {
            thickness,
            length: screen.width(),
            labels,
        }
    }

    /// How much of the screen's height it takes, across its short edge.
    #[must_use]
    pub const fn thickness(self) -> Room {
        self.thickness
    }

    /// How far it runs along the bottom, which is the whole width.
    ///
    /// *Whether it hides when a window needs the room* is [`crate::hiding`],
    /// and it is deliberately not here: a layout says how much room the dock
    /// takes when it is shown, and whether it is shown at all is a person's
    /// choice answered against what the windows want. A layout that also hid
    /// itself would have to know about windows, which this crate does not.
    #[must_use]
    pub const fn length(self) -> Room {
        self.length
    }

    /// What became of the names.
    #[must_use]
    pub const fn labels(self) -> Labels {
        self.labels
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::measures::{A_DOCK_MAY_TAKE_ONE_PART_IN, THE_STANDARDS_TEXT};

    /// A size the tests name often enough to be worth a word.
    fn text(percent: u16) -> TextScale {
        TextScale::percent(percent).unwrap()
    }

    /// A dock on the smallest screen alo OS lays out for.
    fn on_the_smallest(percent: u16) -> Layout {
        Layout::of(Screen::the_smallest(), text(percent))
    }

    /// **EN 301 549 is a test, not a sentence.** The standard an EU
    /// public-sector desktop is procured against requires text to reach 200%
    /// without loss of content, so a dock on the smallest screen alo OS lays out
    /// for still has its names at that size.
    ///
    /// This is the test that fixes the two numbers nobody could have picked by
    /// eye. Loosen either and the dock takes more of somebody's screen than it
    /// has any claim to; tighten either and this fails.
    #[test]
    fn names_survive_the_text_size_the_standard_requires() {
        let layout = on_the_smallest(THE_STANDARDS_TEXT);
        assert!(
            layout.labels().are_shown(),
            "the names went at {THE_STANDARDS_TEXT}% on the smallest screen"
        );
        assert_eq!(layout.labels(), Labels::Under);
    }

    /// **And the rule is not vacuous.** Above what the standard requires, on the
    /// smallest screen, the names do give way — so *labels give way to icons
    /// where the short edge demands it* is a thing that happens rather than a
    /// branch nothing reaches.
    #[test]
    fn names_give_way_when_the_short_edge_finally_demands_it() {
        let (_, largest) = TextScale::range();
        let layout = on_the_smallest(largest);
        assert_eq!(layout.labels(), Labels::GaveWay(largest));
        assert_eq!(layout.thickness(), Room::a_dock_of_icons());
    }

    /// **A bigger screen keeps its names longer**, because the ceiling is a
    /// share of the height rather than a fixed number of pixels. The same person
    /// with the same text size gets names on the desk and icons on the laptop,
    /// which is the behaviour a share buys.
    #[test]
    fn a_bigger_screen_keeps_its_names_at_a_size_a_small_one_cannot() {
        let (_, largest) = TextScale::range();
        let desk = Screen::of(3840, 2160).unwrap();
        assert!(Layout::of(desk, text(largest)).labels().are_shown());
        assert!(!on_the_smallest(largest).labels().are_shown());
    }

    /// **The dock never takes more than its share**, at any text size, on any
    /// screen — which is the promise the ceiling exists to keep and the reason
    /// [`Layout::of`] can answer without a `Result`.
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
            for percent in smallest..=largest {
                let layout = Layout::of(screen, text(percent));
                let ceiling = Room::the_most_a_dock_may_take(screen.height());
                assert!(
                    layout.thickness().fits_in(ceiling),
                    "at {percent}% it took {} of a ceiling of {}",
                    layout.thickness().as_pixels(),
                    ceiling.as_pixels()
                );
            }
        }
    }

    /// **Names never come back once they have gone.** A person turning their
    /// text up one step at a time meets the change once; a dock whose labels
    /// flickered back at a larger size would be a layout nobody could describe.
    #[test]
    fn once_the_names_have_given_way_a_larger_size_never_brings_them_back() {
        let (smallest, largest) = TextScale::range();
        let mut gone = false;
        for percent in smallest..=largest {
            let shown = on_the_smallest(percent).labels().are_shown();
            if gone {
                assert!(!shown, "the names came back at {percent}%");
            }
            gone |= !shown;
        }
        assert!(gone, "they never give way at all, so nothing was tested");
    }

    /// **The thickness comes out of the height and the length out of the
    /// width**, on a portrait screen as much as a landscape one — because the
    /// dock takes from the side it sits on rather than from whichever side is
    /// shorter.
    #[test]
    fn a_dock_takes_from_the_height_and_spans_the_width_on_either_shape() {
        for screen in [
            Screen::of(1366, 768).unwrap(),
            Screen::of(768, 1366).unwrap(),
        ] {
            let layout = Layout::of(screen, text(100));
            assert_eq!(layout.length(), screen.width());
            assert!(
                layout
                    .thickness()
                    .fits_in(Room::the_most_a_dock_may_take(screen.height()))
            );
        }
    }

    /// **The share a dock may take is the tightest one that keeps the
    /// standard.** One part more and the names would go at exactly the size
    /// EN 301 549 requires them to survive — so the number in
    /// [`crate::measures`] is fixed by the requirement rather than chosen, and
    /// this is the test that says which way it is fixed.
    ///
    /// **It still is, with one orientation instead of two.** The share used to
    /// be justified against a dock down the side as well, and the honest worry
    /// on removing that was that the surviving case might leave the number
    /// loose. It does not: on the smallest screen's height, one part in seven
    /// would already lose the names at 200%.
    #[test]
    fn the_share_is_as_tight_as_the_standard_allows() {
        let screen = Screen::the_smallest();
        let standard = text(THE_STANDARDS_TEXT);
        let side = screen.height().as_pixels();
        let tighter = Room::pixels(side / (A_DOCK_MAY_TAKE_ONE_PART_IN + 1));
        assert!(
            !Room::a_dock_with_names_under(standard).fits_in(tighter),
            "a share of one part in {} would still fit, so the chosen share is loose",
            A_DOCK_MAY_TAKE_ONE_PART_IN + 1
        );
    }
}
