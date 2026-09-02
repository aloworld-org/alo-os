//! The colours alo OS is built out of, as code rather than as a table in a
//! document.
//!
//! These are the six in `docs/design/figma-brief.md`, verbatim. They are here so
//! that a person choosing a plain colour for their background is offered the
//! colours their machine is already made of, rather than a colour wheel with no
//! anchor in it — and so that the shell and this crate cannot drift into
//! disagreeing about what navy is.
//!
//! **Terracotta is in the list and is not an ordinary colour.** The design brief
//! spends it in one place only: where the agent is present or acting, about five
//! percent of any screen, so that a person can tell at a glance whether the
//! machine is doing something on their behalf. A person may still put it behind
//! their windows if they want it there — a background is not a signal, and
//! nothing on top of it changes meaning. What may *not* happen is the shell
//! adopting it as its accent, which is the tension item 8a in
//! `docs/autonomy/QUEUE.md` exists to settle.

use serde::{Deserialize, Serialize};

use crate::colour::Colour;

/// One of the colours alo OS is built out of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Token {
    /// Structure and text.
    Navy,
    /// The agent — present or acting, and nothing else.
    Terracotta,
    /// The reading ground.
    Cream,
    /// The workspace canvas.
    Porcelain,
    /// The rail.
    Charcoal,
    /// Metadata.
    WarmStone,
}

impl Token {
    /// All six, in the order the design brief lists them.
    pub const ALL: [Self; 6] = [
        Self::Navy,
        Self::Terracotta,
        Self::Cream,
        Self::Porcelain,
        Self::Charcoal,
        Self::WarmStone,
    ];

    /// The colour itself.
    #[must_use]
    pub const fn colour(self) -> Colour {
        match self {
            Self::Navy => Colour::of(0x10, 0x2A, 0x43),
            Self::Terracotta => Colour::of(0xE7, 0x6F, 0x51),
            Self::Cream => Colour::of(0xF8, 0xF6, 0xF2),
            Self::Porcelain => Colour::of(0xF4, 0xF1, 0xEC),
            Self::Charcoal => Colour::of(0x1F, 0x25, 0x29),
            Self::WarmStone => Colour::of(0x7A, 0x6F, 0x62),
        }
    }

    /// What this colour is called where a person picks it.
    ///
    /// English, and one of the strings item 9 in `docs/autonomy/QUEUE.md`
    /// externalises. A colour name is a translator's judgement rather than a
    /// translator's typing: several languages have no ordinary word for
    /// terracotta, and the one they reach for may not be the colour.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Navy => "Navy",
            Self::Terracotta => "Terracotta",
            Self::Cream => "Cream",
            Self::Porcelain => "Porcelain",
            Self::Charcoal => "Charcoal",
            Self::WarmStone => "Warm stone",
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **The palette is the one in `docs/design/figma-brief.md`**, and this test
    /// is what stops the two from drifting apart: a colour changed here without
    /// the brief changing is caught by the person who wrote the brief.
    #[test]
    fn the_palette_is_the_one_in_the_design_brief() {
        let written = [
            (Token::Navy, "#102A43"),
            (Token::Terracotta, "#E76F51"),
            (Token::Cream, "#F8F6F2"),
            (Token::Porcelain, "#F4F1EC"),
            (Token::Charcoal, "#1F2529"),
            (Token::WarmStone, "#7A6F62"),
        ];
        for (token, hex) in written {
            assert_eq!(token.colour(), Colour::written(hex).unwrap());
            assert_eq!(token.colour().to_string(), hex);
        }
        assert_eq!(written.len(), Token::ALL.len(), "all six, and only six");
    }

    /// Every colour is named and no two share a name or a value, because a
    /// picker with two identical entries is a picker a person cannot use.
    #[test]
    fn every_colour_is_named_and_distinct() {
        for (at, token) in Token::ALL.iter().enumerate() {
            assert!(!token.name().is_empty());
            for other in Token::ALL.iter().skip(at.saturating_add(1)) {
                assert_ne!(token.colour(), other.colour());
                assert_ne!(token.name(), other.name());
            }
        }
    }
}
