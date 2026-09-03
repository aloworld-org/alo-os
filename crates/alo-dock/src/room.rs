//! How much room something takes, and the arithmetic that says so.
//!
//! One unit — the logical pixel — and every measurement this crate makes,
//! derived from [`crate::measures`] and from how big the person has made their
//! text. Nothing here reads a screen, opens a font or knows what a name is: it
//! answers *how much room would this need*, which is the question a threshold
//! has to be made of if it is to be a test rather than an opinion.
//!
//! **Two dock thicknesses, not one rotated.** A dock that runs across the
//! screen puts a name **under** its icon, so what it needs across its short edge
//! is a line of text. A dock that runs down the screen puts a name **beside**
//! its icon, so what it needs is a width — and a width is not a line height,
//! which is the whole of why `docs/features.md` says a dock is not a horizontal
//! bar somebody turned sideways. The two are worked out separately here and are
//! different sizes at every text size.

use alo_appearance::TextScale;

use crate::measures::{
    A_DOCK_MAY_TAKE_ONE_PART_IN, GAP, ICON, LABEL_EMS, LINE_IN_FIFTHS, MARGIN, TEXT_AT_ORDINARY,
};

/// How much room something takes, in logical pixels.
///
/// Logical rather than physical: a dense screen draws the same dock out of more
/// pixels rather than a smaller one, so nothing in this crate has to know how
/// dense a screen is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Room {
    /// The measurement itself.
    pixels: u32,
}

impl Room {
    /// This many logical pixels.
    #[must_use]
    pub const fn pixels(pixels: u32) -> Self {
        Self { pixels }
    }

    /// The measurement, for whoever is drawing.
    #[must_use]
    pub const fn as_pixels(self) -> u32 {
        self.pixels
    }

    /// This much and that much together.
    #[must_use]
    pub const fn and(self, other: Self) -> Self {
        Self {
            pixels: self.pixels.saturating_add(other.pixels),
        }
    }

    /// Whether this much fits inside that much.
    #[must_use]
    pub const fn fits_in(self, room: Self) -> bool {
        self.pixels <= room.pixels
    }

    /// How big the shell's text is at this size.
    #[must_use]
    pub fn text_at(text: TextScale) -> Self {
        Self::pixels(
            TEXT_AT_ORDINARY
                .saturating_mul(u32::from(text.as_percent()))
                .saturating_div(100),
        )
    }

    /// How tall one line of that text is, which is what a name under an icon
    /// takes.
    #[must_use]
    pub fn a_line_at(text: TextScale) -> Self {
        Self::pixels(
            Self::text_at(text)
                .pixels
                .saturating_mul(LINE_IN_FIFTHS)
                .saturating_div(5),
        )
    }

    /// How wide a name needs to be beside an icon before it is worth showing at
    /// all, which is [`LABEL_EMS`] of that text.
    #[must_use]
    pub fn a_name_beside_an_icon_at(text: TextScale) -> Self {
        Self::pixels(Self::text_at(text).pixels.saturating_mul(LABEL_EMS))
    }

    /// The side of one icon.
    #[must_use]
    pub const fn an_icon() -> Self {
        Self::pixels(ICON)
    }

    /// How thick a dock of icons alone is: the icon, and the dock's two faces.
    #[must_use]
    pub const fn a_dock_of_icons() -> Self {
        Self::pixels(MARGIN.saturating_add(ICON).saturating_add(MARGIN))
    }

    /// How thick a dock that runs across the screen is, with a name under each
    /// icon.
    #[must_use]
    pub fn a_dock_with_names_under(text: TextScale) -> Self {
        Self::a_dock_of_icons()
            .and(Self::pixels(GAP))
            .and(Self::a_line_at(text))
    }

    /// How thick a dock that runs down the screen is, with a name beside each
    /// icon.
    #[must_use]
    pub fn a_dock_with_names_beside(text: TextScale) -> Self {
        Self::a_dock_of_icons()
            .and(Self::pixels(GAP))
            .and(Self::a_name_beside_an_icon_at(text))
    }

    /// The most a dock may take out of the side of the screen it sits on.
    ///
    /// This is the ceiling the whole *labels give way* decision turns on:
    /// [`crate::Layout`] shows names when a dock that has them fits under it,
    /// and gives way to icons when it does not.
    #[must_use]
    pub const fn the_most_a_dock_may_take(side: Self) -> Self {
        Self::pixels(side.pixels.saturating_div(A_DOCK_MAY_TAKE_ONE_PART_IN))
    }

    /// The shortest a screen's side may be and still hold a dock at all.
    ///
    /// Worked out rather than picked: a dock of icons alone must fit under the
    /// ceiling, so the side must be [`A_DOCK_MAY_TAKE_ONE_PART_IN`] times it.
    /// [`crate::Screen`] refuses anything below this, which is what lets
    /// [`crate::Layout`] be a question with an answer rather than a `Result`.
    #[must_use]
    pub const fn the_least_a_side_can_be() -> Self {
        Self::pixels(
            Self::a_dock_of_icons()
                .pixels
                .saturating_mul(A_DOCK_MAY_TAKE_ONE_PART_IN),
        )
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::measures::SMALLEST_TARGET;
    use crate::screen::Screen;

    /// A size the tests name often enough to be worth a word.
    fn text(percent: u16) -> TextScale {
        TextScale::percent(percent).unwrap()
    }

    /// **A standard is a test, not a sentence.** EN 301 549 carries WCAG 2.5.8's
    /// minimum target size, and an icon in the dock is a target: it is what a
    /// person presses to open an application. An icon shrunk in some later
    /// change fails here.
    ///
    /// The assertion is about the icon this crate *hands out* rather than about
    /// the constant behind it, because an assertion written directly about a
    /// constant is folded away before it can ever fail.
    #[test]
    fn an_icon_is_at_least_the_smallest_target_the_standard_allows() {
        assert!(
            Room::an_icon().as_pixels() >= SMALLEST_TARGET,
            "an icon is {} and the standard's floor is {SMALLEST_TARGET}",
            Room::an_icon().as_pixels()
        );
    }

    /// **The dock leaves most of the screen to the person's work.** A dock
    /// entitled to half a screen is not a dock, and this holds on every side of
    /// every screen rather than only on the one the thresholds were chosen
    /// against.
    #[test]
    fn a_dock_may_never_take_half_of_anything() {
        for side in [
            Screen::the_smallest().width(),
            Screen::the_smallest().height(),
            Room::the_least_a_side_can_be(),
            Room::pixels(3840),
        ] {
            let ceiling = Room::the_most_a_dock_may_take(side);
            assert!(
                ceiling.as_pixels().saturating_mul(2) < side.as_pixels(),
                "a dock could take {} of a side of {}",
                ceiling.as_pixels(),
                side.as_pixels()
            );
        }
    }

    /// Text grows with the setting, and 100% is the size the shell was drawn
    /// at. The arithmetic is whole numbers throughout, so two machines reading
    /// the same settings file lay out identically.
    #[test]
    fn text_grows_with_the_size_a_person_set() {
        assert_eq!(Room::text_at(text(100)).as_pixels(), TEXT_AT_ORDINARY);
        assert_eq!(Room::text_at(text(200)).as_pixels(), 30);
        assert_eq!(Room::text_at(text(300)).as_pixels(), 45);
        assert!(Room::text_at(text(75)) < Room::text_at(text(100)));
    }

    /// A line is taller than the text in it, and a name beside an icon is wider
    /// than a line is tall — which is why the two orientations are worked out
    /// separately rather than one being the other rotated.
    #[test]
    fn a_name_beside_an_icon_takes_more_room_than_a_name_under_one() {
        for percent in [75, 100, 125, 200, 300] {
            let size = text(percent);
            assert!(
                Room::a_line_at(size) > Room::text_at(size),
                "at {percent}% a line is not taller than its text"
            );
            assert!(
                Room::a_dock_with_names_beside(size) > Room::a_dock_with_names_under(size),
                "at {percent}% the two orientations want the same room"
            );
        }
    }

    /// **Every measurement grows with the text, and none of them shrinks.** A
    /// person who makes the text bigger and gets a thinner dock has found a
    /// layout that will surprise them somewhere else too.
    #[test]
    fn nothing_gets_smaller_as_the_text_gets_bigger() {
        let (smallest, largest) = TextScale::range();
        let mut previous = (Room::pixels(0), Room::pixels(0));
        for percent in smallest..=largest {
            let size = text(percent);
            let now = (
                Room::a_dock_with_names_under(size),
                Room::a_dock_with_names_beside(size),
            );
            assert!(now.0 >= previous.0 && now.1 >= previous.1, "at {percent}%");
            previous = now;
        }
    }

    /// A dock of icons alone does not depend on the text at all, which is what
    /// makes it the thing a dock falls back to.
    #[test]
    fn a_dock_of_icons_is_the_same_at_every_text_size() {
        assert_eq!(Room::a_dock_of_icons().as_pixels(), MARGIN + ICON + MARGIN);
        assert!(Room::a_dock_of_icons() > Room::an_icon());
    }

    /// **The floor under a screen is worked out, not chosen.** A side exactly at
    /// the floor has room for a dock of icons and nothing more, which is the
    /// guarantee [`crate::Layout`] leans on when it answers without a `Result`.
    #[test]
    fn the_shortest_side_is_exactly_enough_for_a_dock_of_icons() {
        let least = Room::the_least_a_side_can_be();
        assert!(Room::a_dock_of_icons().fits_in(Room::the_most_a_dock_may_take(least)));
        let one_less = Room::pixels(least.as_pixels().saturating_sub(1));
        assert!(
            !Room::a_dock_of_icons().fits_in(Room::the_most_a_dock_may_take(one_less)),
            "the floor is the tightest one that works, not a round number above it"
        );
    }

    /// Room adds without wrapping, because a screen reported wrongly by a driver
    /// should give a silly layout rather than a tiny one.
    #[test]
    fn room_saturates_rather_than_wrapping() {
        let vast = Room::pixels(u32::MAX);
        assert_eq!(vast.and(Room::pixels(10)), vast);
        assert!(Room::pixels(1).fits_in(vast));
    }
}
