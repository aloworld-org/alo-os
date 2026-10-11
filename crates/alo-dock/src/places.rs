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

use crate::holding::{Fitted, OnTheDock};
use crate::measures::{GAP, ICON, MARGIN};
use crate::window::AppId;

/// What sits in one of the bar's slots.
///
/// **The overflow control is a slot like any other**, by the owner's ruling of
/// 2026-10-10: *overflow occupies one application slot: 48 × 48 logical pixels.
/// Centre the overflow glyph inside it and use the same slot spacing as
/// neighbouring applications on all four edges. This supersedes the conflicting
/// 72 × 44 annotation and 44 × 44 component measurement.*
///
/// So it is held here rather than as a rectangle the draw works out beside the
/// places. One arithmetic lays out the bar and one hit test reads it, which is
/// the whole reason `crate::places` exists: *a drawing that put an icon
/// somewhere and a click that looked for it somewhere else would be two answers
/// to one question.* A control laid out separately would be exactly that, for
/// the one slot a person presses when they have the most open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatIsHere {
    /// An application.
    Application(AppId),
    /// The control that opens the list of applications there was no room for.
    ///
    /// **Always on the bar, and last**, which is a correction of 2026-10-11.
    /// This was placed right after the applications and only when something had
    /// overflowed — a control invented by this lane rather than measured.
    ///
    /// The design file settles both halves at once: `Edge=Top, State=Rest`
    /// (`348:23840`) has **no overflow at all** and still ends
    /// `… divider, composer, History, More apps and canvas options`, at x 634 of
    /// 691. `Edge=Top, State=Overflow` (`348:23874`) has the same trailing group
    /// after a longer row of applications. The bottom Dock is the same shape:
    /// `Canvas options menu` is the last child of `Dock + alo Bar`.
    ///
    /// **Its name says it has a second job**: *More apps **and canvas
    /// options***. What those options are is not specified anywhere this lane
    /// can find, so this opens the applications and nothing else, and the gap
    /// is named here rather than guessed at.
    TheOverflow,
    /// The rule between the applications and the commands after them.
    ///
    /// **A line, not a control**: nothing presses it, and `Places::at` answers
    /// it so that a press on it reaches nothing rather than the control beside
    /// it. Measured on the edge Docks as `Apps / commands divider`.
    TheDivider,
    /// The field: *Find, open, or ask…*
    ///
    /// **The *alo Bar* half of *Dock + alo Bar*.** Three hundred and forty-four
    /// across on both Dock families, which is the one measurement they agreed
    /// on before ADR 0101 reconciled them.
    TheComposer,
    /// The control that lays every open window out at once.
    ///
    /// **The bottom Dock's**, by ADR 0101: *preserve the bottom Dock's composer
    /// and system actions.* The edge Docks put History here instead, which is
    /// the *keep the distinct edge layouts* half of the same ruling.
    ShowAll,
    /// The control that opens what a person did before.
    ///
    /// The edge Docks', for the reason [`Self::ShowAll`] gives.
    History,
}

impl WhatIsHere {
    /// How far across the bar this takes.
    ///
    /// **Four widths and not one**, which is why [`APlace::across`] has always
    /// been a field rather than a constant: an application's target, the
    /// composer, the word *Show all* and a hairline are four different things
    /// and only one of them is a square.
    #[must_use]
    pub const fn across(&self) -> u32 {
        match self {
            Self::Application(_) | Self::TheOverflow | Self::History => ICON,
            Self::TheComposer => crate::measures::THE_COMPOSER_IS_WIDE,
            Self::ShowAll => crate::measures::SHOW_ALL_IS_WIDE,
            Self::TheDivider => crate::measures::THE_DOCKS_EDGE,
        }
    }

    /// Whether a press on this reaches anything.
    #[must_use]
    pub const fn can_be_pressed(&self) -> bool {
        !matches!(self, Self::TheDivider)
    }
}

/// Where one slot sits, relative to the bar's own corner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct APlace {
    /// What is here.
    what: WhatIsHere,
    /// How far from the bar's left edge the icon starts.
    from_the_start: u32,
    /// How far across it is, which is always [`ICON`].
    across: u32,
}

impl APlace {
    /// What is here.
    #[must_use]
    pub const fn what(&self) -> &WhatIsHere {
        &self.what
    }

    /// Which application, or [`None`] for the overflow control.
    ///
    /// **An `Option` rather than a default**, because the one caller that draws
    /// a letter must not draw one here: a control that showed the first letter
    /// of something would read as an application a person could open, and there
    /// is no application called *more*.
    #[must_use]
    pub const fn app(&self) -> Option<&AppId> {
        match &self.what {
            WhatIsHere::Application(app) => Some(app),
            WhatIsHere::TheOverflow
            | WhatIsHere::TheDivider
            | WhatIsHere::TheComposer
            | WhatIsHere::ShowAll
            | WhatIsHere::History => None,
        }
    }

    /// Whether this is the control that opens the overflow.
    #[must_use]
    pub const fn is_the_overflow(&self) -> bool {
        matches!(self.what, WhatIsHere::TheOverflow)
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
    /// Where everything on this bar sits, in the order it is drawn.
    ///
    /// The order is [`crate::Holding::showing`]'s — pinned first, then what is
    /// open, in the order it opened — so where an icon sits follows the same
    /// rule as whether it is there at all. **The overflow control is last**, in
    /// the slot after the final application, and only when [`Fitted`] has
    /// something in it.
    ///
    /// It takes a [`Fitted`] rather than a slice because the two questions —
    /// *what is drawn* and *is there a control* — have one answer, and a
    /// caller holding them apart could draw a control for an empty overflow or
    /// an overflow with no way into it. `Fitted` is what [`crate::fit`] returns
    /// and is the only thing that knows both.
    #[must_use]
    pub fn of(fitted: &Fitted, edge: crate::Edge) -> Self {
        let mut from_the_start = MARGIN;
        let showing = fitted.on_the_dock();
        let mut places = Vec::with_capacity(showing.len() + 5);
        let mut put = |what: WhatIsHere, at: &mut u32| {
            let across = what.across();
            places.push(APlace {
                what,
                from_the_start: *at,
                across,
            });
            *at = at.saturating_add(across).saturating_add(GAP);
        };
        for one in showing {
            put(
                WhatIsHere::Application(one.app().clone()),
                &mut from_the_start,
            );
        }
        // **Then the rule, the field and the commands, in the measured order.**
        // `348:23840` and `348:23874` both end: `Apps / commands divider`, the
        // composer, `Open History`, `More apps and canvas options`. The bottom
        // Dock has no rule and puts *Show all* where History is — ADR 0101's
        // *keep the distinct edge layouts*, with its *preserve the bottom
        // Dock's composer and system actions*.
        //
        // **`More` is last and always there**, in both states of both families.
        if edge == crate::Edge::Bottom {
            put(WhatIsHere::TheComposer, &mut from_the_start);
            put(WhatIsHere::ShowAll, &mut from_the_start);
        } else {
            put(WhatIsHere::TheDivider, &mut from_the_start);
            put(WhatIsHere::TheComposer, &mut from_the_start);
            put(WhatIsHere::History, &mut from_the_start);
        }
        put(WhatIsHere::TheOverflow, &mut from_the_start);
        Self { places }
    }

    /// How far across the bar everything on it reaches, including the room at
    /// both ends.
    ///
    /// **The bar's width is what it holds**, and what it holds is no longer only
    /// icons: `crate::Room::a_bar_holding` counts slots of one size and cannot
    /// answer this. Taken from the last place rather than summed, so a change to
    /// the order or the contents cannot make the two disagree.
    #[must_use]
    pub fn how_far_across(&self) -> crate::Room {
        let past = self
            .places
            .last()
            .map_or(MARGIN, |last| last.past_its_end());
        crate::Room::pixels(past.saturating_add(MARGIN))
    }

    /// Where these sit on a bar with nothing in the overflow.
    ///
    /// The shorthand for the case every test and every bar that fits is in, so
    /// that asking for places does not oblige a caller to build a [`Fitted`] it
    /// has no use for. It cannot produce an overflow slot, which is why it is
    /// separate rather than a default argument.
    #[must_use]
    pub fn of_all_of(showing: &[OnTheDock], edge: crate::Edge) -> Self {
        Self::of(&crate::fit(showing.to_vec(), showing.len()), edge)
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
        self.places
            .iter()
            .find(|place| place.holds(along) && place.what.can_be_pressed())
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
    use crate::edge::Edge;
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

    /// Only the application slots, which is what every test below 2026-10-11
    /// was written about.
    ///
    /// **The bar holds more than applications now** — a rule, the composer and
    /// one or two commands — so a test that walked every place and asserted it
    /// was an icon's size was asking about a bar that no longer exists. Each of
    /// them still has its question; this is how they ask it.
    fn applications(places: &Places) -> Vec<&APlace> {
        places
            .each()
            .iter()
            .filter(|place| place.app().is_some())
            .collect()
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
        let places = Places::of_all_of(&showing(&["Docs", "Browser", "Blender"]), Edge::Bottom);
        assert_eq!(applications(&places).len(), 3);

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
        let places = Places::of_all_of(
            &showing(&["Docs", "Browser", "Blender", "Ptyxis"]),
            Edge::Bottom,
        );
        for one in applications(&places) {
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
            let places = Places::of_all_of(&showing(&borrowed), Edge::Bottom);

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
        let places = Places::of_all_of(&showing(&["Docs", "Browser", "Blender"]), Edge::Bottom);
        for one in applications(&places) {
            let middle = one.from_the_start() + one.across() / 2;
            assert_eq!(
                places.at(middle).map(APlace::app),
                Some(one.app()),
                "the middle of {:?} found something else",
                one.what()
            );
        }
    }

    /// **The seam belongs to exactly one icon.** The last pixel of one and the
    /// first of the next are different applications, and neither is claimed
    /// twice — so a press on the boundary is not a race.
    #[test]
    fn two_icons_never_both_claim_a_position() {
        let places = Places::of_all_of(&showing(&["Docs", "Browser", "Blender"]), Edge::Bottom);
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
        let places = Places::of_all_of(&showing(&["Docs", "Browser"]), Edge::Bottom);
        let first = place(&places, 0);
        for along in first.past_its_end()..place(&places, 1).from_the_start() {
            assert!(places.at(along).is_none(), "the gap at {along} was claimed");
        }
    }

    /// **The room at either end belongs to nobody**, for the same reason.
    #[test]
    fn the_room_at_the_ends_belongs_to_nobody() {
        let places = Places::of_all_of(&showing(&["Docs", "Browser"]), Edge::Bottom);
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
        let places = Places::of_all_of(&[], Edge::Bottom);
        assert!(
            applications(&places).is_empty(),
            "a Dock holding nothing laid out an application"
        );
        // **The bar is not empty, and that is the design.** It still carries
        // the composer and the commands after it, which is what a person types
        // into on a machine with nothing open. What a Dock holding nothing has
        // none of is *applications*.
        assert!(
            places.each().iter().all(|place| place.app().is_none()),
            "something on an empty Dock claims to be an application"
        );
        for along in 0..MARGIN {
            assert!(places.at(along).is_none(), "the room at the start");
        }
    }

    /// **The overflow control is the last slot on the bar, laid out like any
    /// other.**
    ///
    /// The owner's ruling of 2026-10-10: *overflow occupies one application
    /// slot: 48 × 48 logical pixels … use the same slot spacing as neighbouring
    /// applications on all four edges.* So there is nothing to assert about a
    /// special size or a special gap — what is asserted is that there is
    /// **nothing special**: it is where the next application would have been,
    /// and it is the same width.
    #[test]
    fn the_overflow_control_is_the_last_slot_on_the_bar() {
        let everything = showing(&["Docs", "Browser", "Blender", "Mail"]);
        let fitted = crate::fit(everything, 3);
        assert!(fitted.anything_over(), "the premise: something overflowed");
        assert_eq!(
            fitted.on_the_dock().len(),
            2,
            "the premise: `fit` kept one slot back for the control"
        );

        let places = Places::of(&fitted, Edge::Bottom);
        assert_eq!(applications(&places).len(), 2);
        // **Last on the bar**, after the composer and *Show all* — measured in
        // both states of both Dock families on 2026-10-11.
        let control = place(&places, places.how_many() - 1);
        assert!(control.is_the_overflow());
        assert_eq!(control.app(), None, "the control is not an application");
        assert_eq!(
            control.across(),
            measures::ICON,
            "the control is not an application's width"
        );
        // **After everything else, not where the next application would have
        // been.** It was that until 2026-10-11, which put it between the
        // applications and the composer; the design has it last.
        let before_it = place(&places, places.how_many() - 2);
        assert_eq!(
            control.from_the_start(),
            before_it.past_its_end() + measures::GAP,
            "the control is not one gap after the slot before it"
        );

        // And the two before it really are the applications, so this cannot
        // pass by everything being a control.
        assert_eq!(place(&places, 0).app().map(AppId::name), Some("Docs"));
        assert_eq!(place(&places, 1).app().map(AppId::name), Some("Browser"));
    }

    /// **The control is there whether or not anything has overflowed.**
    ///
    /// **This asserted the opposite until 2026-10-11** — that a bar with room
    /// for everything had no control — and it was a rule this lane invented
    /// rather than measured. `Edge=Top, State=Rest` (`348:23840`) has no
    /// overflow at all and still ends with `More apps and canvas options`, and
    /// the bottom Dock's `Canvas options menu` is its last child in the resting
    /// frame too.
    ///
    /// So what is asserted now is permanence, and that a bar that fits is the
    /// same bar either way.
    #[test]
    fn the_control_is_on_the_bar_whether_or_not_anything_overflowed() {
        let everything = showing(&["Docs", "Browser", "Blender"]);
        let fitted = crate::fit(everything.clone(), 3);
        assert!(!fitted.anything_over(), "the premise: nothing overflowed");

        let places = Places::of(&fitted, Edge::Bottom);
        assert_eq!(applications(&places).len(), everything.len());
        assert!(
            places.each().iter().any(|place| place.is_the_overflow()),
            "a bar with room for everything has no control to open more with"
        );
        assert_eq!(
            places,
            Places::of_all_of(&everything, Edge::Bottom),
            "`of_all_of` is not the same answer as `fit` with room for everything"
        );
    }

    /// **Pressing the control finds the control**, and the application beside it
    /// finds the application.
    ///
    /// The hit test is the reason this file exists, and the control is the slot
    /// where getting it wrong costs most: a person reaches for it exactly when
    /// they have the most open and the thing they want is behind it.
    #[test]
    fn a_press_on_the_control_is_not_a_press_on_the_last_application() {
        let fitted = crate::fit(showing(&["Docs", "Browser", "Blender", "Mail"]), 3);
        let places = Places::of(&fitted, Edge::Bottom);

        let last_application = place(&places, 1);
        let control = place(&places, places.how_many() - 1);

        let on_the_control = control.from_the_start() + control.across() / 2;
        assert_eq!(
            places.at(on_the_control).map(APlace::what),
            Some(&WhatIsHere::TheOverflow),
            "the middle of the control did not find the control"
        );

        let on_the_application = last_application.from_the_start() + last_application.across() / 2;
        assert_eq!(
            places.at(on_the_application).and_then(APlace::app),
            Some(&app("Browser")),
            "the middle of the last application found something else"
        );

        // And the gap between them belongs to neither, like every other gap.
        assert!(
            places.at(last_application.past_its_end()).is_none(),
            "the gap before the control reaches one of them"
        );
    }

    /// **The order is the Dock's order**, not an order this file invents:
    /// pinned first, in the order they were pinned.
    #[test]
    fn the_order_is_the_order_the_dock_shows() {
        let names = ["Docs", "Browser", "Blender"];
        let places = Places::of_all_of(&showing(&names), Edge::Bottom);
        let laid_out: Vec<&str> = places
            .each()
            .iter()
            .filter_map(|p| p.app().map(AppId::name))
            .collect();
        assert_eq!(laid_out, names);
    }
}
