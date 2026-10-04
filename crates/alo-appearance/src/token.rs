//! The colours a person can **pick**, by the names they are called.
//!
//! Six names, offered to somebody choosing a plain colour to put behind their
//! windows — so they are offered the colours their machine is already made of
//! rather than a colour wheel with no anchor in it.
//!
//! # This is a name, and [`crate::role::Role`] is a role
//!
//! The two were one type until 2026-10-04 and ADR 0092 separated them. A
//! compositor drawing a frame is not choosing anything: it needs to know that
//! this rectangle is a canvas and that one is a card, which is what a [`Role`]
//! says and what a colour name cannot. `porcelain` is the proof — four
//! consumers used it and meant four different things.
//!
//! **Each of the six delegates its value to the role it equals**, so every hex
//! in this crate is written in exactly one place:
//!
//! ```text
//! Navy      -> Role::TextPrimary     #102A43   unchanged
//! DeepTeal  -> Role::AccentDefault   #0F6B72   unchanged, ADR 0067
//! Cream     -> Role::BgCanvas        #FAF7F2   was #F8F6F2
//! Porcelain -> Role::BgSurface       #FFFFFF   was #F4F1EC
//! WarmStone -> Role::TextMuted       #596B78   was #7A6F62
//! Charcoal     the one literal       #1F2529   not a design variable
//! ```
//!
//! # Why the names were kept when three values moved
//!
//! A `Token` is serialised into a person's settings and
//! `docs/contracts/person-settings.md` is a public surface. **Renaming or
//! removing a variant would change the meaning of a value somebody has already
//! stored**, so the six names are retained rather than deprecated: choosing
//! "cream" for a background is a legitimate thing to call a colour. What was
//! wrong was a *drawing* authority organised by name, and that is what moved to
//! [`Role`].
//!
//! Three values moved because the design file says so, which is a correction
//! and not a renaming — the entry a person stored still means the colour it
//! named. `charcoal` is the one value the design file does not define, and
//! [`Token::Charcoal`] is retained because every choosable accent is measured
//! against it: a hex that reads on cream is illegible on charcoal.
//!
//! **Deep teal is in the list and is not an ordinary colour.** The design brief
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
//! ordinary word for deep teal — so each of the six carries a note describing
//! the colour rather than assuming the word travels.

use alo_strings::{Filling, Said, Strings, Word};
use serde::{Deserialize, Serialize};

use crate::colour::Colour;
use crate::role::Role;
use crate::words;

/// One of the colours alo OS is built out of.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Token {
    /// Structure and text — [`Role::TextPrimary`].
    Navy,
    /// alo — present, acting, proposing, or awaiting an approval, and nothing
    /// else. Never drawn without the mark or the word beside it.
    DeepTeal,
    /// The workspace canvas — [`Role::BgCanvas`].
    ///
    /// Called the *reading ground* until 2026-10-04, and its value moved from
    /// `#F8F6F2` to the design file's `#FAF7F2` with it.
    Cream,
    /// A document, card or panel surface — [`Role::BgSurface`].
    ///
    /// **This is the name the separation was made for.** It was *the workspace
    /// canvas* and four consumers meant four things by it; each was audited and
    /// given the role it actually had (ADR 0092). As a colour somebody picks it
    /// is white, and `shipped::THE_SURFACE` — which really is the canvas —
    /// moved to [`Self::Cream`] rather than following the name.
    Porcelain,
    /// The dark ground, and the one value here the design file does not define.
    ///
    /// Retained deliberately: [`crate::accent`] measures every choosable accent
    /// against it, so removing it removes the dark half of every contrast
    /// check. The light rail being [`Role::BgSurface`] does **not** make white
    /// the dark-theme rail colour.
    Charcoal,
    /// Metadata — [`Role::TextMuted`].
    ///
    /// No longer a warm grey: the design file's value is `#596B78`, where this
    /// was `#7A6F62`. The old value measured 4.36:1 on [`Role::BgCool`], below
    /// the 4.5:1 EN 301 549 asks of text; the new one measures 4.91:1.
    WarmStone,
}

impl Token {
    /// All six, in the order the design brief lists them.
    pub const ALL: [Self; 6] = [
        Self::Navy,
        Self::DeepTeal,
        Self::Cream,
        Self::Porcelain,
        Self::Charcoal,
        Self::WarmStone,
    ];

    /// The role this name is, or `None` for the one that is not a role.
    ///
    /// [`Self::Charcoal`] is the dark ground and the design file does not define
    /// it, so it has no role and answers `None` rather than being mapped to a
    /// near neighbour. The dark mode does carry candidates — `bg/subtle
    /// #183B56` and `navy/950 #07131F` were measured — and neither is adopted,
    /// because dark-theme parity needs its own measured evidence.
    #[must_use]
    pub const fn role(self) -> Option<Role> {
        match self {
            Self::Navy => Some(Role::TextPrimary),
            Self::DeepTeal => Some(Role::AccentDefault),
            Self::Cream => Some(Role::BgCanvas),
            Self::Porcelain => Some(Role::BgSurface),
            Self::WarmStone => Some(Role::TextMuted),
            Self::Charcoal => None,
        }
    }

    /// The colour itself.
    ///
    /// Taken from [`Self::role`] wherever there is one, so that a value exists
    /// in one place and cannot drift between a name and a role.
    #[must_use]
    pub const fn colour(self) -> Colour {
        match self.role() {
            Some(role) => role.colour(),
            // The dark ground, which is not a design variable. ADR 0092.
            None => Colour::of(0x1F, 0x25, 0x29),
        }
    }

    /// The string this crate declares for it: the key a translator's file is
    /// sorted by, and the English beside it.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Navy => words::NAVY,
            Self::DeepTeal => words::DEEP_TEAL,
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

    /// **The six are the values the design file measures**, written out here
    /// rather than derived, so that a change to a role's value has to be typed
    /// twice before it reaches a person's background picker.
    ///
    /// `tests/a_palette_with_one_source.rs` is the half of this that reads
    /// `docs/design/palette.toml`; this half is what a reader of this file can
    /// check without leaving it.
    #[test]
    fn the_six_names_carry_the_design_files_values() {
        let written = [
            (Token::Navy, "#102A43"),
            (Token::DeepTeal, "#0F6B72"),
            (Token::Cream, "#FAF7F2"),
            (Token::Porcelain, "#FFFFFF"),
            (Token::Charcoal, "#1F2529"),
            (Token::WarmStone, "#596B78"),
        ];
        for (token, hex) in written {
            assert_eq!(token.colour(), Colour::written(hex).unwrap());
            assert_eq!(token.colour().to_string(), hex);
        }
        assert_eq!(written.len(), Token::ALL.len(), "all six, and only six");
    }

    /// **A name that has a role takes its value from that role**, which is what
    /// makes one hex live in one place.
    ///
    /// Without this, `colour()` could be rewritten back to a literal per
    /// variant and every other test here would still pass — the values would
    /// simply be free to drift again.
    #[test]
    fn a_name_with_a_role_has_the_roles_colour() {
        let mut had_a_role: usize = 0;
        for token in Token::ALL {
            match token.role() {
                Some(role) => {
                    assert_eq!(
                        token.colour(),
                        role.colour(),
                        "{:?} does not carry {}",
                        token,
                        role.key()
                    );
                    had_a_role = had_a_role.saturating_add(1);
                }
                // The dark ground is the only one without a role.
                None => assert_eq!(token, Token::Charcoal),
            }
        }
        assert_eq!(had_a_role, 5, "five of the six are a design role");
    }

    /// **The twelve roles are not all reachable through a name, and that is the
    /// point.** Seven of them — secondary text, the cool surface, the border
    /// and the three statuses, and the soft accent — are drawing roles with no
    /// business in a background picker.
    ///
    /// Asserted so that somebody extending `Token` to cover the palette is
    /// stopped here and reads ADR 0092 instead.
    #[test]
    fn the_names_do_not_cover_the_roles() {
        let reachable: Vec<&str> = Token::ALL
            .iter()
            .filter_map(|token| token.role())
            .map(Role::key)
            .collect();
        assert_eq!(reachable.len(), 5);
        for role in Role::ALL {
            let offered = reachable.contains(&role.key());
            let should_be = matches!(
                role,
                Role::TextPrimary
                    | Role::AccentDefault
                    | Role::BgCanvas
                    | Role::BgSurface
                    | Role::TextMuted
            );
            assert_eq!(
                offered,
                should_be,
                "{} is offered as a name: {offered}, and should be: {should_be}",
                role.key()
            );
        }
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
        let untranslated = Token::DeepTeal.said(&strings);
        assert_eq!(untranslated.text(), "Deep teal");
        assert!(!untranslated.is_translated());
        assert!(!untranslated.is_a_bug());
    }
}
