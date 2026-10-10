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
    A_SIDE_DOCKS_LANE, ABOVE_AND_BELOW_AN_ICON, GAP, ICON, LINE_IN_FIFTHS, MARGIN,
    TEXT_AT_ORDINARY, THE_LEAST_A_SIDE_CAN_BE,
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

    /// The side of one icon.
    #[must_use]
    pub const fn an_icon() -> Self {
        Self::pixels(ICON)
    }

    /// How wide the lane a dock down either side of the screen occupies.
    ///
    /// [`crate::measures::A_SIDE_DOCKS_LANE`], measured off the design file's
    /// frames — and deliberately **not** [`Self::a_dock_of_icons`], which is six
    /// pixels narrower. That constant's own note carries why the two are kept apart
    /// rather than reconciled.
    #[must_use]
    pub const fn a_side_docks_lane() -> Self {
        Self::pixels(A_SIDE_DOCKS_LANE)
    }

    /// How thick a dock of icons is: the icon, and the dock's two faces.
    ///
    /// **Seventy-six, measured off the design** — `Dock + alo Bar` is 76 tall
    /// with a 48 hit area inside it, 14 above and 14 below. It was
    /// `MARGIN + ICON + MARGIN` = 64 until 2026-10-10, which read as a
    /// measurement and was a proposal; the owner superseded it with the
    /// measured number on that day.
    ///
    /// **It does not vary with the text size, and that is the same ruling.**
    /// There is no name in the bar to leave room for: a name is shown on hover
    /// and on keyboard focus, outside the bar, and never changes its height. So
    /// this is one number for every text size, which is what
    /// `a_dock_with_names_under` existed to deny.
    #[must_use]
    pub const fn a_dock_of_icons() -> Self {
        Self::pixels(
            ABOVE_AND_BELOW_AN_ICON
                .saturating_add(ICON)
                .saturating_add(ABOVE_AND_BELOW_AN_ICON),
        )
    }

    /// How wide a bar holding this many icons is: the room inside it, the
    /// icons, and a gap between each pair of them.
    ///
    /// **A bar's width is what it holds.** It was the whole width of the screen
    /// while the Dock was a band flush to the edge; the designs show a bar
    /// centred with room either side, so the width is arithmetic over its
    /// contents rather than a property of the screen.
    ///
    /// None at all is still a bar — the room inside it, and nothing between —
    /// because a Dock with nothing pinned and nothing open is a thing a person
    /// can still reach.
    #[must_use]
    pub fn a_bar_holding(how_many: usize) -> Self {
        let icons = u32::try_from(how_many).unwrap_or(u32::MAX);
        let gaps = icons.saturating_sub(1);
        Self::pixels(
            MARGIN
                .saturating_add(icons.saturating_mul(ICON))
                .saturating_add(gaps.saturating_mul(GAP))
                .saturating_add(MARGIN),
        )
    }

    /// How many slots fit in a bar this long: the inverse of
    /// [`Self::a_bar_holding`].
    ///
    /// **The question `alo_dock::fit` has needed a caller for.** `fit` takes
    /// *room for `at_most` icons* and nothing worked that number out, so nothing
    /// called it and nothing ever went into an overflow. This is where the
    /// number comes from: the edge the bar runs along, less the room the layout
    /// keeps at its ends.
    ///
    /// A **slot**, not an icon, because the overflow control takes one — the
    /// owner's ruling of 2026-10-10: *overflow occupies one application slot:
    /// 48 × 48 logical pixels … use the same slot spacing as neighbouring
    /// applications on all four edges.* So this counts places and `fit` decides
    /// what goes in them.
    ///
    /// Exact rather than approximate, and asserted against `a_bar_holding` over
    /// the whole range in this file's tests: `how_many_fit(a_bar_holding(n))` is
    /// `n` for every `n`, and one pixel less is one slot less at every step.
    #[must_use]
    pub const fn how_many_fit(self) -> usize {
        // `MARGIN + n*ICON + (n-1)*GAP <= along`, solved for the largest whole
        // `n`. Written with the `+ GAP` on the left so the division is the
        // whole of the rounding: `n*(ICON + GAP) <= along - 2*MARGIN + GAP`.
        let ends = MARGIN.saturating_mul(2);
        let usable = self.pixels.saturating_sub(ends).saturating_add(GAP);
        (usable / ICON.saturating_add(GAP)) as usize
    }

    /// The shortest a screen's side may be and still be laid out for.
    ///
    /// **Stated, not worked out.** It was the dock's thickness times a share of
    /// the screen, which made the bar's height decide which displays this
    /// product supports — so measuring the bar at 76 moved the floor and
    /// dropped displays nobody had decided to drop.
    /// [`crate::measures::THE_LEAST_A_SIDE_CAN_BE`] carries the owner's ruling
    /// and why 384 is the number.
    ///
    /// [`crate::Screen`] refuses anything below this, which is what lets
    /// [`crate::Layout`] be a question with an answer rather than a `Result`.
    #[must_use]
    pub const fn the_least_a_side_can_be() -> Self {
        Self::pixels(THE_LEAST_A_SIDE_CAN_BE)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_appearance::targets::{ENHANCED_TARGET, SMALLEST_TARGET};

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
        // **And above what a control is built to**, which is the stronger claim
        // and the one that would fail first. The floor is what a layout may not
        // go below; the enhanced figure is what the owner decided a control is
        // on 2026-09-30. An icon clearing the first and missing this would be
        // compliant and still too small to press.
        assert!(
            Room::an_icon().as_pixels() >= ENHANCED_TARGET,
            "an icon is {} and a control is built to {ENHANCED_TARGET}",
            Room::an_icon().as_pixels()
        );
    }

    /// **Every accepted screen can hold the bar**, which is what the floor is
    /// for now that it is stated rather than derived.
    ///
    /// This walked the ceiling — a share of each side — which the owner removed
    /// on 2026-10-10 along with the coupling that let it decide which displays
    /// are supported.
    #[test]
    fn the_bar_fits_on_the_shortest_side_this_crate_accepts() {
        let least = Room::the_least_a_side_can_be();
        assert!(
            Room::a_dock_of_icons().fits_in(least),
            "the bar does not fit on the shortest side this crate accepts, so a display it \
             says yes to cannot hold a dock"
        );
        assert!(
            Room::a_side_docks_lane().fits_in(least),
            "and the same for a dock down a side"
        );
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

    /// A line is taller than the text in it.
    ///
    /// **The second half of this went with `a_dock_with_names_under` on
    /// 2026-10-10.** It asserted that a dock with names was thicker than one
    /// without — true, and the cost is exactly what the owner removed: the
    /// verified design has no names in the bar, so there is no thicker dock to
    /// compare against. `a_line_at` is still used for text elsewhere, so the
    /// half that is about a line stays.
    #[test]
    fn a_line_is_taller_than_its_text() {
        for percent in [75, 100, 125, 200, 300] {
            let size = text(percent);
            assert!(
                Room::a_line_at(size) > Room::text_at(size),
                "at {percent}% a line is not taller than its text"
            );
        }
    }

    /// **Every measurement that depends on the text grows with it, and none of
    /// them shrinks.** A person who makes the text bigger and gets a smaller
    /// measurement has found a layout that will surprise them somewhere else
    /// too.
    ///
    /// This walked `a_dock_with_names_under` until 2026-10-10. The dock's
    /// thickness does not depend on the text any more — which is a stronger
    /// statement than *it grows* and is held in `crate::layout` — so what is
    /// walked here is the measurement that still does.
    #[test]
    fn nothing_gets_smaller_as_the_text_gets_bigger() {
        let (smallest, largest) = TextScale::range();
        let mut previous = Room::pixels(0);
        for percent in smallest..=largest {
            let now = Room::a_line_at(text(percent));
            assert!(now >= previous, "at {percent}%");
            previous = now;
        }
        assert!(
            previous > Room::a_line_at(text(smallest)),
            "the premise: a line really does grow across this range, so the walk above is not \
             comparing a constant with itself"
        );
    }

    /// A dock of icons alone does not depend on the text at all, which is what
    /// makes it the thing a dock falls back to.
    #[test]
    fn a_dock_of_icons_is_the_same_at_every_text_size() {
        assert_eq!(
            Room::a_dock_of_icons().as_pixels(),
            ABOVE_AND_BELOW_AN_ICON + ICON + ABOVE_AND_BELOW_AN_ICON
        );
        assert_eq!(
            Room::a_dock_of_icons().as_pixels(),
            76,
            "the measured height of `Dock + alo Bar`, which superseded the proposed 64 on \
             2026-10-10"
        );
        assert!(Room::a_dock_of_icons() > Room::an_icon());
    }

    /// **The floor under a screen is chosen, not worked out** — which is the
    /// reverse of what this test said, and the reversal is the point.
    ///
    /// It asserted the floor was *the tightest one that works*: a side exactly
    /// at it had room for a dock of icons and not a pixel more. That made the
    /// floor an arithmetic result, so measuring the bar at 76 instead of 64 on
    /// 2026-10-10 moved it from 384 to 456 and silently dropped every display
    /// between. The owner's ruling that day separated the two.
    ///
    /// So the floor is now **384 because that is what this product accepted**,
    /// and the bar fits inside it with room to spare. *With room to spare* is
    /// the thing to hold: it is what tightness used to guarantee and what
    /// nothing else now does.
    #[test]
    fn the_shortest_side_has_room_for_a_dock_and_more() {
        let least = Room::the_least_a_side_can_be();
        assert_eq!(least.as_pixels(), 384);
        assert!(
            Room::a_dock_of_icons().fits_in(least),
            "a side at the floor cannot hold the bar"
        );
        assert!(
            Room::a_dock_of_icons().as_pixels() * 2 < least.as_pixels(),
            "the bar takes more than half the shortest side this crate accepts, which leaves a \
             person less of their screen than the dock"
        );
    }

    /// **`how_many_fit` is exactly the inverse of `a_bar_holding`**, over the
    /// whole range rather than at a size somebody thought to try.
    ///
    /// Two claims, and the second is the one that would catch an off-by-one that
    /// the first alone would let through: a bar exactly wide enough for `n`
    /// holds `n`, and **one pixel narrower holds `n - 1`**. A rounding that went
    /// the other way would satisfy the first at every `n` and overflow the bar
    /// by a gap.
    #[test]
    fn how_many_fit_is_the_inverse_of_how_wide_a_bar_is() {
        for how_many in 0..60_usize {
            let exactly = Room::a_bar_holding(how_many);
            assert_eq!(
                exactly.how_many_fit(),
                how_many,
                "a bar {} wide was built to hold {how_many}",
                exactly.as_pixels()
            );
            // One pixel short of holding `how_many` holds one fewer. At zero
            // there is nothing to take away: `a_bar_holding(0)` is the two
            // margins and a bar narrower than those still holds none.
            let a_pixel_short = Room::pixels(exactly.as_pixels().saturating_sub(1));
            assert_eq!(
                a_pixel_short.how_many_fit(),
                how_many.saturating_sub(1),
                "a bar one pixel short of holding {how_many} does not hold one fewer"
            );
        }
    }

    /// **A bar too small for one slot holds none**, rather than one it cannot
    /// draw or a count that wrapped.
    #[test]
    fn a_bar_with_no_room_holds_nothing_and_does_not_wrap() {
        for pixels in 0..=Room::a_bar_holding(1).as_pixels() {
            let fits = Room::pixels(pixels).how_many_fit();
            let expected = usize::from(pixels >= Room::a_bar_holding(1).as_pixels());
            assert_eq!(fits, expected, "a bar {pixels} wide said it holds {fits}");
        }
        // And a silly size from a driver gives a silly answer rather than zero
        // or a panic, which is this file's rule everywhere else.
        assert!(Room::pixels(u32::MAX).how_many_fit() > 1000);
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
