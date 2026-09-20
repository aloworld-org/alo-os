//! **Which palette every surface is drawn in: the design's, or the one high
//! contrast decides.**
//!
//! Task 12 of `docs/autonomy/v0-5-the-shell-plan.md`: *the magnifier and high
//! contrast apply as decided*. What high contrast is, is
//! `alo_access::HighContrast` — a second palette held to WCAG 2.2 AAA over
//! every pair this crate draws — and nothing about it is decided here. This is
//! the one door between that palette and the colours this crate paints with, so
//! that a surface cannot be drawn in the design's cream while its neighbour is
//! drawn in the contrast palette's white.
//!
//! # One value, carried by the look
//!
//! Every surface this crate draws already carries a *look* — light or dark, the
//! text size, which way the person reads — and every one of those answers
//! belongs to another crate. This is one more of them: [`Contrast::of`] reads
//! `alo_access::TurnedOn`, which is what the person turned on, and the look
//! hands it to whichever palette is being built. There is no *accessibility
//! mode* here and no second drawing path: the same layout is drawn, in other
//! colours.
//!
//! # The four roles are the shell's own, read back by that crate
//!
//! `alo_access::THE_PAIRS_THE_SHELL_DRAWS` names four roles — a window's
//! ground, the dock's ground, words and edges, and the accent — and says
//! outright that they were read off this crate's palettes. [`Role`] is that
//! list, used here to pick a colour, so a role added to a surface with no
//! colour in that crate's palette cannot be drawn in high contrast without
//! that crate deciding one.
//!
//! # The accent, in high contrast, is not the person's
//!
//! On the ordinary desktop the accent is the person's own, and terracotta is
//! refused there because it means the agent (ADR 0010). In high contrast the
//! accent is `alo_access::HighContrast`'s, which that crate argues at length:
//! at AAA the reserved colour cannot be drawn as text at all, so it draws
//! terracotta's own hue taken deep enough to read. A person's chosen accent is
//! still *checked* here — an accent nobody may choose is refused before any
//! palette is built — and then not drawn, because a colour a person cannot read
//! is not a preference this palette can honour.

use alo_access::high_contrast::Role;
use alo_access::{HighContrast, Setting, TurnedOn};
use alo_appearance::{Colour, Scheme, Token};

/// **Whether a surface is drawn in the design's colours or in high contrast.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Contrast {
    /// The design brief's palette: cream, porcelain, navy, charcoal.
    #[default]
    AsDesigned,
    /// `alo_access::HighContrast`'s palette, at WCAG 2.2 AAA.
    High,
}

impl Contrast {
    /// **What the person turned on**, which is the only road to drawing in high
    /// contrast.
    #[must_use]
    pub fn of(turned_on: &TurnedOn) -> Self {
        if turned_on.has(Setting::HighContrast) {
            Self::High
        } else {
            Self::AsDesigned
        }
    }

    /// The colour of one role, in this scheme.
    ///
    /// [`Role::Accent`] is not here: the accent is a person's own colour in one
    /// palette and this crate's in the other, so it is asked for with the
    /// colour that was chosen ([`Contrast::accent`]).
    pub(crate) fn colour(self, role: Role, scheme: Scheme) -> [u8; 3] {
        match self {
            Self::High => rgb(HighContrast::of(scheme).role(role)),
            Self::AsDesigned => rgb(match (role, scheme) {
                (Role::Ground, Scheme::Light) => Token::Cream,
                (Role::Dock, Scheme::Light) => Token::Porcelain,
                (Role::Ink, Scheme::Light) => Token::Navy,
                (Role::Ground | Role::Dock, Scheme::Dark) => Token::Charcoal,
                (Role::Ink, Scheme::Dark) => Token::Cream,
                // The accent has no token: it is the person's own colour,
                // and `accent` is where it is asked for.
                (Role::Accent, _) => Token::Terracotta,
            }
            .colour()),
        }
    }

    /// A window's or a panel's ground.
    pub(crate) fn ground(self, scheme: Scheme) -> [u8; 3] {
        self.colour(Role::Ground, scheme)
    }

    /// The dock's ground, a field's inside, and every other ground that is not
    /// the window's own.
    pub(crate) fn dock(self, scheme: Scheme) -> [u8; 3] {
        self.colour(Role::Dock, scheme)
    }

    /// Words, edges and rules.
    pub(crate) fn ink(self, scheme: Scheme) -> [u8; 3] {
        self.colour(Role::Ink, scheme)
    }

    /// The accent: the person's own where the design is drawn, and this
    /// palette's where it is not.
    pub(crate) fn accent(self, scheme: Scheme, chosen: Colour) -> [u8; 3] {
        match self {
            Self::High => rgb(HighContrast::of(scheme).accent),
            Self::AsDesigned => rgb(chosen),
        }
    }
}

/// **Every colour this palette hands out**, for a surface's own test asking
/// whether it drew anything that is not in it.
#[cfg(test)]
pub(crate) fn every_colour_of(contrast: Contrast, scheme: Scheme, chosen: Colour) -> [[u8; 3]; 4] {
    [
        contrast.ground(scheme),
        contrast.dock(scheme),
        contrast.ink(scheme),
        contrast.accent(scheme, chosen),
    ]
}

/// A colour as the painter takes it.
const fn rgb(colour: Colour) -> [u8; 3] {
    [colour.red(), colour.green(), colour.blue()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use alo_access::THE_PAIRS_THE_SHELL_DRAWS;

    /// Contrast between two colours as the painter holds them, through
    /// `alo-appearance`'s own measure so that nothing is measured twice.
    fn between(one: [u8; 3], other: [u8; 3]) -> f64 {
        Colour::of(one[0], one[1], one[2]).contrast_with(Colour::of(other[0], other[1], other[2]))
    }

    /// **Nothing is turned on until somebody turns it on.**
    #[test]
    fn a_machine_nobody_has_changed_is_drawn_as_designed() {
        assert_eq!(Contrast::of(&TurnedOn::nothing()), Contrast::AsDesigned);
    }

    /// **Turning high contrast on is the whole road to it**, and turning it off
    /// puts the design back.
    #[test]
    fn the_setting_is_the_only_road_to_the_other_palette() {
        let mut turned_on = TurnedOn::nothing();
        turned_on.turn_on(Setting::HighContrast);
        assert_eq!(Contrast::of(&turned_on), Contrast::High);
        turned_on.turn_off(Setting::HighContrast);
        assert_eq!(Contrast::of(&turned_on), Contrast::AsDesigned);
    }

    /// **The colours handed out are the other crate's, unchanged** — every role
    /// in both schemes, so nothing can be adjusted on the way through.
    #[test]
    fn high_contrast_hands_out_the_palette_that_crate_decided() {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let theirs = HighContrast::of(scheme);
            assert_eq!(Contrast::High.ground(scheme), rgb(theirs.ground));
            assert_eq!(Contrast::High.dock(scheme), rgb(theirs.dock));
            assert_eq!(Contrast::High.ink(scheme), rgb(theirs.ink));
            assert_eq!(
                Contrast::High.accent(scheme, Token::Navy.colour()),
                rgb(theirs.accent),
                "{scheme:?}: a chosen accent was drawn where the contrast palette's belongs"
            );
        }
    }

    /// **The design is drawn as it always was**: the same four tokens, so that
    /// adding this door changed no pixel for the person who turned nothing on.
    #[test]
    fn as_designed_is_the_palette_this_crate_already_drew() {
        assert_eq!(
            Contrast::AsDesigned.ground(Scheme::Light),
            rgb(Token::Cream.colour())
        );
        assert_eq!(
            Contrast::AsDesigned.dock(Scheme::Light),
            rgb(Token::Porcelain.colour())
        );
        assert_eq!(
            Contrast::AsDesigned.ink(Scheme::Light),
            rgb(Token::Navy.colour())
        );
        assert_eq!(
            Contrast::AsDesigned.ground(Scheme::Dark),
            rgb(Token::Charcoal.colour())
        );
        assert_eq!(
            Contrast::AsDesigned.ink(Scheme::Dark),
            rgb(Token::Cream.colour())
        );
        let chosen = Token::Navy.colour();
        assert_eq!(
            Contrast::AsDesigned.accent(Scheme::Light, chosen),
            rgb(chosen)
        );
    }

    /// **Every pair this crate draws clears AAA once the door is used**, in
    /// both schemes — the same pairs `alo-access` measured, measured again
    /// through the colours this crate would actually paint.
    #[test]
    fn every_pair_drawn_through_this_door_is_readable_at_aaa() {
        for scheme in [Scheme::Light, Scheme::Dark] {
            for (drawn, on) in THE_PAIRS_THE_SHELL_DRAWS {
                let pair = between(
                    Contrast::High.colour(drawn, scheme),
                    Contrast::High.colour(on, scheme),
                );
                assert!(
                    pair >= alo_access::high_contrast::ENOUGH_FOR_TEXT_AT_AAA,
                    "{scheme:?}: {drawn:?} on {on:?} is {pair:.2} to 1 as this crate paints it"
                );
            }
        }
    }
}
