//! What one screen wears: the background behind its windows, the edge its dock
//! sits on, and how warm it is drawn.
//!
//! **Two of the three are not this crate's.** A background is
//! `alo-appearance`'s and a dock's edge is `alo-dock`'s; what is decided here
//! is only *which screen each belongs to*, which is the one question a crate
//! about screens is allowed to answer. Nothing here draws, and nothing here
//! writes to either crate's file.
//!
//! **The third is, and this is where night light reaches a screen.** The warmth
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
//! # The dock's edge is not per screen yet, and that is `alo-dock`'s to decide
//!
//! `alo_dock::Dock` holds **one** edge for the machine. The v0.5 plan this
//! crate belongs to says *per-display placement is `alo-dock`'s decision*, and
//! this plan reads that crate and never edits it — so what happens today is
//! what `alo-dock` has decided today: the edge the person chose, on every
//! screen. When `alo-dock` gains an edge per screen, [`Wearing::of`] is the one
//! function that changes, and every caller of it keeps working.
//!
//! This is a finding rather than a silence, and it is written down in the task
//! report as one.

use alo_appearance::{Appearance, Background};
use alo_dock::{Dock, Edge};

use crate::night_light::Tonight;
use crate::reported::Reported;
use crate::warmth::{Warming, Warmth};

/// What one screen wears.
#[derive(Debug, Clone, PartialEq)]
pub struct Wearing {
    /// What is behind its windows.
    background: Background,
    /// Which edge of it the dock sits on.
    edge: Edge,
    /// How warm it is drawn.
    warmth: Warmth,
    /// What that warmth does to each of its three channels, worked out once
    /// here so that every caller warms a colour the same way.
    warming: Warming,
}

impl Wearing {
    /// What this screen wears, asked of the two crates that own the values and
    /// of night light for the third.
    #[must_use]
    pub fn of(screen: &Reported, appearance: &Appearance, dock: &Dock, tonight: &Tonight) -> Self {
        Self {
            background: appearance.background_on(screen.named_for_the_shell()),
            edge: dock.edge(),
            warmth: tonight.warmth(),
            warming: Warming::at(tonight.warmth()),
        }
    }

    /// What is behind its windows.
    #[must_use]
    pub const fn background(&self) -> &Background {
        &self.background
    }

    /// Which edge of it the dock sits on.
    #[must_use]
    pub const fn edge(&self) -> Edge {
        self.edge
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

    use alo_appearance::{DisplayId, Token};

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
        appearance.set_background(everywhere.clone());
        let only_here = Background::from(Token::Cream.colour());
        appearance.set_background_on(
            DisplayId::named(office.named_for_the_shell().name()).unwrap(),
            only_here.clone(),
        );

        let dock = Dock::shipped();
        assert_eq!(
            Wearing::of(&office, &appearance, &dock, &a_cold_evening()).background(),
            &only_here
        );
        assert_eq!(
            Wearing::of(&laptop, &appearance, &dock, &a_cold_evening()).background(),
            &everywhere
        );
    }

    /// **The edge a screen's dock sits on is the edge `alo-dock` decided**, and
    /// it follows the person's choice rather than what the release shipped.
    #[test]
    fn the_edge_is_the_one_alo_dock_decided() {
        let laptop = a_reported_laptop();
        let appearance = Appearance::shipped();

        let shipped = Dock::shipped();
        assert_eq!(
            Wearing::of(&laptop, &appearance, &shipped, &a_cold_evening()).edge(),
            shipped.edge()
        );

        let mut moved = Dock::shipped();
        moved.set_edge(Edge::Left);
        assert_eq!(
            Wearing::of(&laptop, &appearance, &moved, &a_cold_evening()).edge(),
            Edge::Left
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
        appearance.set_background_on(
            DisplayId::named(office.named_for_the_shell().name()).unwrap(),
            Background::from(Token::Cream.colour()),
        );
        let dock = Dock::shipped();

        let warm = a_warm_evening();
        let on_the_laptop = Wearing::of(&laptop, &appearance, &dock, &warm);
        let on_the_monitor = Wearing::of(&office, &appearance, &dock, &warm);
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
        let by_day = Wearing::of(&laptop, &appearance, &dock, &cold);
        assert!(by_day.warmth().changes_nothing());
        assert_eq!(
            by_day.warming().applied_to(Token::Terracotta.colour()),
            Token::Terracotta.colour(),
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
        let only_here = Background::from(Token::Cream.colour());
        appearance.set_background_on(
            DisplayId::named(one.named_for_the_shell().name()).unwrap(),
            only_here.clone(),
        );
        let dock = Dock::shipped();
        assert_eq!(
            Wearing::of(&one, &appearance, &dock, &a_cold_evening()).background(),
            &only_here
        );
        assert_ne!(
            Wearing::of(&two, &appearance, &dock, &a_cold_evening()).background(),
            &only_here
        );
    }
}
