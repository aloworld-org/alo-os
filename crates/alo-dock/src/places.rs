//! Where each application sits on the bar.
//!
//! **The one piece both drawing and clicking stand on.** A drawing that put an
//! icon somewhere and a click that looked for it somewhere else would be two
//! answers to one question, and the second would be wrong in a way nobody could
//! see — a person pressing a picture and reaching the application beside it.
//! So there is one answer, computed once, and whoever draws and whoever routes
//! both read it.
//!
//! # The arithmetic
//!
//! A bar is [`crate::Room::a_bar_holding`] wide. Inside it: the room at the
//! start, then each icon, then a gap before the next. Nothing is centred within
//! its own cell and nothing is stretched to fill — an icon is
//! [`crate::measures::ICON`] across, always, because it is a thing a person
//! presses and its size is held to a floor by
//! [`alo_appearance::targets::ENHANCED_TARGET`].
//!
//! **`always` is doing two jobs in that sentence and the owner has separated
//! them.** A place is one number, so the glyph drawn and the area pressed are
//! the same value and cannot move apart. The ruling of 2026-09-30 is that the
//! Dock's glyph shrinks from 32 to 28 as the bar fills while the target stays
//! at least 44 — which this file cannot say. Recorded here rather than in a
//! commit message, because the next person to widen a place will read this.
//!
//! # What is a place and what is not
//!
//! A place is where an application's icon **is**. It is not where its name is
//! drawn, or where a preview would open, or how big a hover target should be —
//! those are the compositor's, and a crate that cannot measure a font has no
//! business deciding them.
//!
//! **Nothing here knows about a screen.** The places are relative to the bar's
//! own corner, so whoever draws it can put the bar wherever the layout says and
//! the places move with it. That is what lets the same arithmetic serve two
//! displays of different sizes without being asked twice.

use crate::holding::OnTheDock;
use crate::measures::{GAP, ICON, MARGIN};
use crate::window::AppId;

/// Where one application's icon sits, relative to the bar's own corner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct APlace {
    /// Which application.
    app: AppId,
    /// How far from the bar's left edge the icon starts.
    from_the_start: u32,
    /// How far across it is, which is always [`ICON`].
    across: u32,
}

impl APlace {
    /// Which application.
    #[must_use]
    pub const fn app(&self) -> &AppId {
        &self.app
    }

    /// How far from the bar's left edge the icon starts.
    #[must_use]
    pub const fn from_the_start(&self) -> u32 {
        self.from_the_start
    }

    /// How far across it is.
    #[must_use]
    pub const fn across(&self) -> u32 {
        self.across
    }

    /// The first position past its right edge.
    #[must_use]
    pub const fn past_its_end(&self) -> u32 {
        self.from_the_start.saturating_add(self.across)
    }

    /// Whether this position along the bar is inside this icon.
    ///
    /// **Half-open**: the start is inside and the end is the next one's. Two
    /// icons cannot both claim a pixel, which is what stops a press on the seam
    /// reaching whichever was asked first.
    #[must_use]
    pub const fn holds(&self, along: u32) -> bool {
        along >= self.from_the_start && along < self.past_its_end()
    }
}

/// Every application's place on one bar, in the order they are drawn.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Places {
    /// In drawing order, left to right.
    places: Vec<APlace>,
}

impl Places {
    /// Where each of these sits, in the order given.
    ///
    /// The order is [`crate::Holding::showing`]'s — pinned first, then what is
    /// open, in the order it opened — so where an icon sits follows the same
    /// rule as whether it is there at all.
    #[must_use]
    pub fn of(showing: &[OnTheDock]) -> Self {
        let mut from_the_start = MARGIN;
        let mut places = Vec::with_capacity(showing.len());
        for one in showing {
            places.push(APlace {
                app: one.app().clone(),
                from_the_start,
                across: ICON,
            });
            from_the_start = from_the_start.saturating_add(ICON).saturating_add(GAP);
        }
        Self { places }
    }

    /// Every place, in drawing order.
    #[must_use]
    pub fn each(&self) -> &[APlace] {
        &self.places
    }

    /// How many.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.places.len()
    }

    /// Whether there are none.
    #[must_use]
    pub fn are_none(&self) -> bool {
        self.places.is_empty()
    }

    /// Which application is at this position along the bar, if any.
    ///
    /// **This is the click.** A press lands somewhere along the bar and this
    /// says whose icon that was — or nothing, for the room at the ends and the
    /// gaps between. Pressing a gap does nothing, deliberately: a gap that
    /// reached its nearest neighbour would mean a person aiming at the space
    /// between two applications opening one of them.
    #[must_use]
    pub fn at(&self, along: u32) -> Option<&APlace> {
        self.places.iter().find(|place| place.holds(along))
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::holding::Holding;
    use crate::windows::Windows;
    use crate::{Room, measures};

    fn app(name: &str) -> AppId {
        AppId::named(name).unwrap()
    }

    /// A Dock with these pinned and nothing open.
    fn showing(names: &[&str]) -> Vec<OnTheDock> {
        let mut holding = Holding::nothing();
        for name in names {
            holding.pin(app(name));
        }
        holding.showing(&Windows::none())
    }

    fn place(places: &Places, at: usize) -> &APlace {
        places
            .each()
            .get(at)
            .unwrap_or_else(|| panic!("no place {at} of {}", places.how_many()))
    }

    /// **The first icon starts at the bar's own margin**, and each one after it
    /// is an icon and a gap further along.
    #[test]
    fn the_icons_are_laid_out_from_the_margin_with_a_gap_between() {
        let places = Places::of(&showing(&["Docs", "Browser", "Blender"]));
        assert_eq!(places.how_many(), 3);

        assert_eq!(place(&places, 0).from_the_start(), measures::MARGIN);
        assert_eq!(
            place(&places, 1).from_the_start(),
            measures::MARGIN + measures::ICON + measures::GAP
        );
        assert_eq!(
            place(&places, 2).from_the_start(),
            measures::MARGIN + 2 * (measures::ICON + measures::GAP)
        );
    }

    /// **Every icon is the same size**, because it is a thing a person presses
    /// and pressing is not easier for the application that happens to be first.
    #[test]
    fn every_icon_is_the_same_size_and_big_enough_to_press() {
        let places = Places::of(&showing(&["Docs", "Browser", "Blender", "Ptyxis"]));
        for one in places.each() {
            assert_eq!(one.across(), measures::ICON);
            assert!(
                one.across() >= alo_appearance::targets::ENHANCED_TARGET,
                "an icon is below what a control is built to, WCAG 2.5.5 enhanced"
            );
        }
    }

    /// **The places fit inside the bar they were laid out for.** The last icon's
    /// end plus the room at the end is exactly the bar's width — so the
    /// arithmetic here and `Room::a_bar_holding` cannot drift apart.
    #[test]
    fn the_places_fill_exactly_the_bar_that_holds_them() {
        for how_many in [1_usize, 2, 5, 9] {
            let names: Vec<String> = (0..how_many).map(|n| format!("App {n}")).collect();
            let borrowed: Vec<&str> = names.iter().map(String::as_str).collect();
            let places = Places::of(&showing(&borrowed));

            let last = place(&places, how_many - 1);
            let wanted = Room::a_bar_holding(how_many).as_pixels();
            assert_eq!(
                last.past_its_end() + measures::MARGIN,
                wanted,
                "holding {how_many}"
            );
        }
    }

    /// **A press lands on the application whose icon it is.** The middle of each
    /// icon finds that icon and no other.
    #[test]
    fn a_press_in_the_middle_of_an_icon_finds_that_application() {
        let places = Places::of(&showing(&["Docs", "Browser", "Blender"]));
        for one in places.each() {
            let middle = one.from_the_start() + one.across() / 2;
            assert_eq!(
                places.at(middle).map(APlace::app),
                Some(one.app()),
                "the middle of {} found something else",
                one.app().name()
            );
        }
    }

    /// **The seam belongs to exactly one icon.** The last pixel of one and the
    /// first of the next are different applications, and neither is claimed
    /// twice — so a press on the boundary is not a race.
    #[test]
    fn two_icons_never_both_claim_a_position() {
        let places = Places::of(&showing(&["Docs", "Browser", "Blender"]));
        let first = place(&places, 0);
        let second = place(&places, 1);

        let last_of_first = first.past_its_end() - 1;
        assert_eq!(places.at(last_of_first).map(APlace::app), Some(first.app()));
        assert!(!second.holds(last_of_first));

        let first_of_second = second.from_the_start();
        assert_eq!(
            places.at(first_of_second).map(APlace::app),
            Some(second.app())
        );
        assert!(!first.holds(first_of_second));
    }

    /// **Pressing a gap does nothing.** A gap that reached its nearest icon
    /// would mean a person aiming at the space between two applications opening
    /// one of them.
    #[test]
    fn pressing_the_gap_between_two_icons_finds_nothing() {
        let places = Places::of(&showing(&["Docs", "Browser"]));
        let first = place(&places, 0);
        for along in first.past_its_end()..place(&places, 1).from_the_start() {
            assert!(places.at(along).is_none(), "the gap at {along} was claimed");
        }
    }

    /// **The room at either end belongs to nobody**, for the same reason.
    #[test]
    fn the_room_at_the_ends_belongs_to_nobody() {
        let places = Places::of(&showing(&["Docs", "Browser"]));
        for along in 0..measures::MARGIN {
            assert!(places.at(along).is_none(), "the start at {along}");
        }
        let past_the_last = place(&places, 1).past_its_end();
        for along in past_the_last..past_the_last + measures::MARGIN {
            assert!(places.at(along).is_none(), "the end at {along}");
        }
    }

    /// A Dock holding nothing has no places, and nothing can be pressed on it.
    #[test]
    fn a_dock_holding_nothing_has_nowhere_to_press() {
        let places = Places::of(&[]);
        assert!(places.are_none());
        assert_eq!(places.how_many(), 0);
        for along in 0..64 {
            assert!(places.at(along).is_none());
        }
    }

    /// **The order is the Dock's order**, not an order this file invents:
    /// pinned first, in the order they were pinned.
    #[test]
    fn the_order_is_the_order_the_dock_shows() {
        let names = ["Docs", "Browser", "Blender"];
        let places = Places::of(&showing(&names));
        let laid_out: Vec<&str> = places.each().iter().map(|p| p.app().name()).collect();
        assert_eq!(laid_out, names);
    }
}
