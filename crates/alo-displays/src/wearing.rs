//! What one screen wears: the background behind its windows, and how warm it is
//! drawn.
//!
//! **One of the two is not this crate's.** A background is `alo-appearance`'s;
//! what is decided here is only *which screen it belongs to*, which is the one
//! question a crate about screens is allowed to answer. Nothing here draws, and
//! nothing here writes to that crate's file.
//!
//! **The other is, and this is where night light reaches a screen.** The warmth
//! is one decision for the machine ([`crate::NightLight`]) — the clock and the
//! sun are the same in every direction a person can turn their head — but it is
//! **applied per screen**, here, beside that screen's background, so the shell
//! warms each output as it draws it. A single tinted sheet laid over the whole
//! desk would have to decide what to do about screens at different sizes, in
//! different places, and plugged in halfway through the evening.
//!
//! # The background is already per screen, and is asked for by name
//!
//! `alo_appearance::Appearance::background_on` takes the name the shell knows a
//! screen by, and answers with the exception the person made for that screen,
//! or their choice for everywhere, or the wallpaper the image shipped — in that
//! order. The name is [`crate::Reported::named_for_the_shell`]: the socket and
//! the screen's own description together, which is exactly what that crate's
//! own documentation describes. Two identical screens are therefore two names,
//! because their sockets differ, and a background chosen for one of them does
//! not appear on the other.
//!
//! # The dock is not something a screen wears
//!
//! This held a third value: **which edge of this screen the dock sat on**. It was
//! one edge for the machine, then for two days an exception a person could make
//! per screen, and [ADR
//! 0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
//! then fixed the dock along the bottom edge of every screen.
//!
//! **So there is nothing per screen left to answer.** A value that is the same on
//! every output is not something an output wears, and keeping it here would be
//! this crate reporting a constant as though a screen had a say in it. `Wearing`
//! no longer takes a `Dock` at all.
//!
//! What a screen still decides about the dock is its **size**, because the
//! thickness comes out of the screen's height — and that is asked of
//! `alo_dock::Dock::layout_on`, which takes a screen and is the right door for it.
//! It is not routed through here, because a layout is not worn: it is worked out
//! at the moment of drawing, from a screen this crate already reports.

use alo_appearance::{Appearance, Background};

use crate::night_light::Tonight;
use crate::reported::Reported;
use crate::warmth::{Warming, Warmth};

/// What one screen wears.
#[derive(Debug, Clone, PartialEq)]
pub struct Wearing {
    /// What is behind its windows.
    background: Background,
    /// How warm it is drawn.
    warmth: Warmth,
    /// What that warmth does to each of its three channels, worked out once
    /// here so that every caller warms a colour the same way.
    warming: Warming,
}

impl Wearing {
    /// What this screen wears, asked of the crate that owns the background and
    /// of night light for the warmth.
    #[must_use]
    pub fn of(screen: &Reported, appearance: &Appearance, tonight: &Tonight) -> Self {
        Self {
            background: appearance.background_on(screen.named_for_the_shell()),
            warmth: tonight.warmth(),
            warming: Warming::at(tonight.warmth()),
        }
    }

    /// What is behind its windows.
    #[must_use]
    pub const fn background(&self) -> &Background {
        &self.background
    }

    /// How warm it is drawn, which is [`Warmth::neutral`] when night light is
    /// not on.
    #[must_use]
    pub const fn warmth(&self) -> Warmth {
        self.warmth
    }

    /// What that warmth does to each of its three channels.
    #[must_use]
    pub const fn warming(&self) -> Warming {
        self.warming
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use alo_appearance::{DisplayId, THE_SURFACE, Token};

    use super::*;
    use crate::between::Between;
    use crate::moment::Moment;
    use crate::night_light::NightLight;
    use crate::nightly::Nightly;
    use crate::testing::{a_reported_laptop, a_reported_office_screen};
    use crate::time_of_day::TimeOfDay;

    /// Night light off, which is what a machine ships with.
    fn a_cold_evening() -> Tonight {
        NightLight::as_shipped().at(Moment::at(UNIX_EPOCH, 0))
    }

    /// Night light on at 2700 K, at eleven in the evening.
    fn a_warm_evening() -> Tonight {
        let night = NightLight::of(
            Nightly::Between(
                Between::these_two_times(
                    TimeOfDay::written("22:00").unwrap(),
                    TimeOfDay::written("07:00").unwrap(),
                )
                .unwrap(),
            ),
            Warmth::kelvin(2700).unwrap(),
        );
        night.at(Moment::at(
            UNIX_EPOCH + Duration::from_secs(23 * 60 * 60),
            0,
        ))
    }

    /// **A background chosen for one screen is worn by that screen and not by
    /// the other**, and a screen nobody singled out wears the person's choice
    /// for everywhere.
    #[test]
    fn a_background_chosen_for_one_screen_is_worn_by_that_screen_alone() {
        let laptop = a_reported_laptop();
        let office = a_reported_office_screen();

        let mut appearance = Appearance::shipped();
        let everywhere = Background::from(Token::Navy.colour());
        appearance.set_background(everywhere);
        let only_here = Background::from(Token::Cream.colour());
        appearance.set_background_on(
            DisplayId::named(office.named_for_the_shell().name()).unwrap(),
            only_here,
        );

        assert_eq!(
            Wearing::of(&office, &appearance, &a_cold_evening()).background(),
            &only_here
        );
        assert_eq!(
            Wearing::of(&laptop, &appearance, &a_cold_evening()).background(),
            &everywhere
        );
    }

    /// **Two screens on one desk wear the same dock**, because there is nothing
    /// about it left to differ.
    ///
    /// This is what is left of three tests that asked which edge each screen's
    /// dock sat on — one for the machine's edge, one for a screen singled out.
    /// ADR 0076 fixed the dock along the bottom of every screen, so the question
    /// they asked has one answer and `Wearing` no longer carries it. What is kept
    /// is the statement that nothing about a screen changes it, since that is the
    /// part a later change could break.
    #[test]
    fn nothing_about_a_screen_changes_where_the_dock_is() {
        let laptop = a_reported_laptop();
        let office = a_reported_office_screen();
        let appearance = Appearance::shipped();
        let evening = a_cold_evening();

        let dock = alo_dock::Dock::shipped();
        let on_the_laptop = dock.layout_on(
            alo_dock::Screen::of(1366, 768).unwrap(),
            alo_appearance::TextScale::ordinary(),
        );
        let on_the_monitor = dock.layout_on(
            alo_dock::Screen::of(1366, 768).unwrap(),
            alo_appearance::TextScale::ordinary(),
        );
        assert_eq!(
            on_the_laptop, on_the_monitor,
            "two screens of a size lay the dock out identically"
        );

        // And what a screen does wear is still its own.
        assert_ne!(
            laptop.named_for_the_shell(),
            office.named_for_the_shell(),
            "two screens are two names"
        );
        assert_eq!(
            Wearing::of(&laptop, &appearance, &evening),
            Wearing::of(&laptop, &appearance, &evening)
        );
    }

    /// **Night light is applied per screen.** Every screen on the desk is
    /// warmed, each beside its own background, and when night light is off
    /// every screen is drawn exactly as it was — which is the clause the shell
    /// reads when it warms one output at a time.
    #[test]
    fn every_screen_wears_the_warmth_beside_its_own_background() {
        let laptop = a_reported_laptop();
        let office = a_reported_office_screen();
        let mut appearance = Appearance::shipped();

        // **Not the shipped surface**, and said rather than assumed. This was
        // `Token::Cream` and worked only while the shipped surface was
        // `Token::Porcelain`; ADR 0092 moved the surface to the canvas, which
        // *is* cream, and the assertion below then compared a colour with
        // itself. The premise is asserted so it cannot quietly go false again.
        let only_there = Token::Navy;
        assert_ne!(
            only_there.colour(),
            THE_SURFACE.colour(),
            "this test needs a background the other screen does not already have"
        );
        appearance.set_background_on(
            DisplayId::named(office.named_for_the_shell().name()).unwrap(),
            Background::from(only_there.colour()),
        );

        let warm = a_warm_evening();
        let on_the_laptop = Wearing::of(&laptop, &appearance, &warm);
        let on_the_monitor = Wearing::of(&office, &appearance, &warm);
        assert_eq!(on_the_laptop.warmth().as_kelvin(), 2700);
        assert_eq!(on_the_monitor.warmth(), on_the_laptop.warmth());
        assert_ne!(
            on_the_monitor.background(),
            on_the_laptop.background(),
            "the same warmth, and still each screen's own background"
        );
        assert!(
            on_the_laptop.warming().blue() < on_the_laptop.warming().green(),
            "a warmed screen has less blue in it than green"
        );

        let cold = a_cold_evening();
        let by_day = Wearing::of(&laptop, &appearance, &cold);
        assert!(by_day.warmth().changes_nothing());
        assert_eq!(
            by_day.warming().applied_to(Token::DeepTeal.colour()),
            Token::DeepTeal.colour(),
            "with night light off a screen is drawn exactly as it was"
        );
    }

    /// **Two identical screens wear their own backgrounds**, because the name
    /// each is known by holds the socket it is plugged into.
    #[test]
    fn two_identical_screens_wear_their_own_backgrounds() {
        let one = a_reported_office_screen();
        let two = crate::testing::a_second_office_screen();
        assert_ne!(one.named_for_the_shell(), two.named_for_the_shell());

        let mut appearance = Appearance::shipped();

        // Not the shipped surface, for the reason given in
        // `every_screen_wears_the_warmth_beside_its_own_background`: the second
        // screen falls back to it, so a background equal to it would make the
        // last assertion compare a colour with itself and pass for the wrong
        // reason — or, as here, fail for the right one.
        let only_here = Background::from(Token::Navy.colour());
        assert_ne!(
            only_here,
            Background::from(THE_SURFACE.colour()),
            "this test needs a background the other screen does not already have"
        );
        appearance.set_background_on(
            DisplayId::named(one.named_for_the_shell().name()).unwrap(),
            only_here,
        );
        assert_eq!(
            Wearing::of(&one, &appearance, &a_cold_evening()).background(),
            &only_here
        );
        assert_ne!(
            Wearing::of(&two, &appearance, &a_cold_evening()).background(),
            &only_here
        );
    }
}
