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
//! adopting it as an accent, and ADR 0010 is where that was settled: the accents
//! a person chooses from are [`crate::accent`]'s five, none of which is in this
//! list, and asking for one of these as an accent is refused in words.
//!
//! **In the language they read.** A [`Token`] has no name in English here: what
//! it has is [`Token::word`], the declaration in [`crate::words`], and
//! [`Token::said`], which answers in the reader's own language and says whether
//! anybody translated it. A colour name is the hardest kind of string to
//! translate and the easiest to get silently wrong — several languages have no
//! ordinary word for terracotta — so each of the six carries a note describing
//! the colour rather than assuming the word travels.

use alo_strings::{Filling, Said, Strings, Word};
use serde::{Deserialize, Serialize};

use crate::colour::Colour;
use crate::words;

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

    /// The string this crate declares for it: the key a translator's file is
    /// sorted by, and the English beside it.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Navy => words::NAVY,
            Self::Terracotta => words::TERRACOTTA,
            Self::Cream => words::CREAM,
            Self::Porcelain => words::PORCELAIN,
            Self::Charcoal => words::CHARCOAL,
            Self::WarmStone => words::WARM_STONE,
        }
    }

    /// What this colour is called where a person picks it, in the language they
    /// read.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not. A
    /// `Strings` that was never given [`crate::appearance_words`] answers with
    /// the key, marked, and `Said::is_a_bug` — which is the honest answer to
    /// *the shell forgot to declare what this crate can say*, and is not
    /// something this crate can paper over with a word of its own.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

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
        let strings = in_english();
        for (at, token) in Token::ALL.iter().enumerate() {
            let said = token.said(&strings);
            assert!(!said.text().is_empty(), "{token:?}");
            assert!(!said.is_a_bug(), "{token:?} is not declared");
            for other in Token::ALL.iter().skip(at.saturating_add(1)) {
                assert_ne!(token.colour(), other.colour());
                assert_ne!(said.text(), other.said(&strings).text());
            }
        }
    }

    /// **A colour is named in the language of whoever is picking it.** German
    /// has an ordinary word for one of these and a borrowed one for the other,
    /// which is the pair the notes in [`crate::words`] were written for.
    #[test]
    fn a_colour_is_named_in_the_readers_language() {
        let strings = translated(&[
            (words::NAVY, "Marineblau"),
            (words::WARM_STONE, "Warmer Stein"),
        ]);
        assert_eq!(Token::Navy.said(&strings).text(), "Marineblau");
        assert!(Token::Navy.said(&strings).is_translated());

        // And the one nobody translated is still English, and says it is.
        let untranslated = Token::Terracotta.said(&strings);
        assert_eq!(untranslated.text(), "Terracotta");
        assert!(!untranslated.is_translated());
        assert!(!untranslated.is_a_bug());
    }
}
