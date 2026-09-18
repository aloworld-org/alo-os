//! What one screen wears: the background behind its windows and the edge its
//! dock sits on.
//!
//! **Neither value is this crate's.** A background is `alo-appearance`'s and a
//! dock's edge is `alo-dock`'s; what is decided here is only *which screen each
//! belongs to*, which is the one question a crate about screens is allowed to
//! answer. Nothing here draws, and nothing here writes to either crate's file.
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

use crate::reported::Reported;

/// What one screen wears.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wearing {
    /// What is behind its windows.
    background: Background,
    /// Which edge of it the dock sits on.
    edge: Edge,
}

impl Wearing {
    /// What this screen wears, asked of the two crates that own the values.
    #[must_use]
    pub fn of(screen: &Reported, appearance: &Appearance, dock: &Dock) -> Self {
        Self {
            background: appearance.background_on(screen.named_for_the_shell()),
            edge: dock.edge(),
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
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_appearance::{DisplayId, Token};

    use super::*;
    use crate::testing::{a_reported_laptop, a_reported_office_screen};

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
            Wearing::of(&office, &appearance, &dock).background(),
            &only_here
        );
        assert_eq!(
            Wearing::of(&laptop, &appearance, &dock).background(),
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
            Wearing::of(&laptop, &appearance, &shipped).edge(),
            shipped.edge()
        );

        let mut moved = Dock::shipped();
        moved.set_edge(Edge::Left);
        assert_eq!(Wearing::of(&laptop, &appearance, &moved).edge(), Edge::Left);
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
            Wearing::of(&one, &appearance, &dock).background(),
            &only_here
        );
        assert_ne!(
            Wearing::of(&two, &appearance, &dock).background(),
            &only_here
        );
    }
}
