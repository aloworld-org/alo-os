//! **A palette of its own, held to WCAG AAA over every pair the shell draws.**
//!
//! High contrast is not the shell's palette with the contrast turned up: the
//! shell's six tokens are a design — cream, porcelain, warm stone — and their
//! quietness is the point of them. A person who cannot read them is not helped
//! by a slightly darker version, so this is a second palette, decided here,
//! that meets **WCAG 2.2 AAA** — a contrast of 7 to 1 for text — on every pair
//! the shell actually draws.
//!
//! # Terracotta keeps its meaning, and stops carrying it alone
//!
//! [ADR 0010](../../../docs/decisions/0010-terracotta-is-reserved-and-never-alone.md)
//! reserves terracotta for the agent and says a colour alone cannot carry a
//! signal anybody must rely on, so **the agent always appears with a mark and a
//! word beside it**. Both halves matter here. At AAA the shell's terracotta
//! cannot be drawn as text — on cream it reaches 3.09 to 1, under half of what
//! text needs — so this palette draws the agent in terracotta's **own hue, 12°**,
//! taken down in lightness to `#862A13` on a light ground and up to `#F7CFC5` on
//! a dark one, and no further: it is the reserved colour, still meaning the
//! agent and nothing else, not a personal accent, which that ADR forbids anybody
//! to select. And the person this palette exists for is exactly the person the
//! mark and the word were written for.
//!
//! # The pairs are the shell's, not a list invented here
//!
//! [`THE_PAIRS_THE_SHELL_DRAWS`] names the four roles `alo-shell`'s desktop
//! palette has — a window's ground, the dock's ground, words and edges, and the
//! person's accent — and the pairs among them that carry text. A role added to
//! the shell with no pair here is a role nobody measured, which is why the pairs
//! are written as data and read by a test rather than checked by eye.

use alo_appearance::{Colour, Scheme};

/// **The palette drawn when high contrast is on.**
///
/// The same four roles the shell's own palette has, so that turning this on
/// changes the colours and nothing else about how anything is laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighContrast {
    /// A window's ground.
    pub ground: Colour,
    /// The dock's ground.
    pub dock: Colour,
    /// Words, edges and rules.
    pub ink: Colour,
    /// The agent's colour, deep enough to read as text.
    pub accent: Colour,
}

impl HighContrast {
    /// The palette for this scheme.
    #[must_use]
    pub const fn of(scheme: Scheme) -> Self {
        match scheme {
            Scheme::Light => Self {
                ground: Colour::of(0xFF, 0xFF, 0xFF),
                dock: Colour::of(0xED, 0xED, 0xED),
                ink: Colour::of(0x00, 0x00, 0x00),
                accent: Colour::of(0x86, 0x2A, 0x13),
            },
            Scheme::Dark => Self {
                ground: Colour::of(0x00, 0x00, 0x00),
                dock: Colour::of(0x14, 0x14, 0x14),
                ink: Colour::of(0xFF, 0xFF, 0xFF),
                accent: Colour::of(0xF7, 0xCF, 0xC5),
            },
        }
    }

    /// The colour of one role in this palette.
    #[must_use]
    pub const fn role(&self, role: Role) -> Colour {
        match role {
            Role::Ground => self.ground,
            Role::Dock => self.dock,
            Role::Ink => self.ink,
            Role::Accent => self.accent,
        }
    }
}

/// One of the four things the shell draws with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// A window's ground.
    Ground,
    /// The dock's ground.
    Dock,
    /// Words, edges and rules.
    Ink,
    /// The person's accent — the agent's colour.
    Accent,
}

/// **Every pair of roles the shell draws one on the other**, as text.
///
/// Read off `alo-shell`'s `DesktopPalette`, which has these four and no others:
/// ink and accent are what is drawn, ground and dock are what they are drawn on.
pub const THE_PAIRS_THE_SHELL_DRAWS: [(Role, Role); 4] = [
    (Role::Ink, Role::Ground),
    (Role::Ink, Role::Dock),
    (Role::Accent, Role::Ground),
    (Role::Accent, Role::Dock),
];

/// **What text must reach to be read by somebody who needs this palette** —
/// WCAG 2.2 AAA for text at ordinary size, 7 to 1.
///
/// `alo_appearance::contrast::ENOUGH_FOR_TEXT` is 4.5, which is AA, and is the
/// bar for the shell's ordinary palette. This palette exists for the person that
/// bar does not reach.
pub const ENOUGH_FOR_TEXT_AT_AAA: f64 = 7.0;

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every pair the shell draws clears AAA, in both schemes.**
    #[test]
    fn every_pair_the_shell_draws_is_readable_at_aaa() {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let palette = HighContrast::of(scheme);
            for (drawn, on) in THE_PAIRS_THE_SHELL_DRAWS {
                let contrast = palette.role(drawn).contrast_with(palette.role(on));
                assert!(
                    contrast >= ENOUGH_FOR_TEXT_AT_AAA,
                    "{scheme:?}: {drawn:?} on {on:?} is {contrast:.2} to 1, under {ENOUGH_FOR_TEXT_AT_AAA}"
                );
            }
        }
    }

    /// **The shell's own palette does not clear it**, which is why this one
    /// exists rather than a switch that raises the other.
    #[test]
    fn the_shells_own_terracotta_is_why_this_palette_exists() {
        let theirs = alo_appearance::Token::Terracotta
            .colour()
            .contrast_with(alo_appearance::Token::Cream.colour());
        assert!(
            theirs < ENOUGH_FOR_TEXT_AT_AAA,
            "the shell's terracotta on cream is {theirs:.2} to 1; if it now clears AAA this \
             palette should be re-argued rather than kept"
        );
    }

    /// **The accent is still terracotta**: the same hue, taken deep enough to
    /// read, rather than another colour wearing the agent's meaning.
    #[test]
    fn the_accent_is_the_agents_colour_taken_deep_enough_to_read() {
        let terracotta = alo_appearance::Token::Terracotta.colour();
        for scheme in [Scheme::Light, Scheme::Dark] {
            let accent = HighContrast::of(scheme).accent;
            assert_eq!(
                hue_of(accent).round(),
                hue_of(terracotta).round(),
                "{scheme:?}: the accent is no longer terracotta's hue"
            );
        }
    }

    /// A colour's hue in degrees, for saying *the same colour, darker*.
    fn hue_of(colour: Colour) -> f64 {
        let [red, green, blue] =
            [colour.red(), colour.green(), colour.blue()].map(|part| f64::from(part) / 255.0);
        let most = red.max(green).max(blue);
        let least = red.min(green).min(blue);
        let span = most - least;
        if span == 0.0 {
            return 0.0;
        }
        let degrees = if most == red {
            60.0 * (((green - blue) / span) % 6.0)
        } else if most == green {
            60.0 * ((blue - red) / span + 2.0)
        } else {
            60.0 * ((red - green) / span + 4.0)
        };
        if degrees < 0.0 {
            degrees + 360.0
        } else {
            degrees
        }
    }
}
