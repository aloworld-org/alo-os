//! What the dock is doing with the names of the things in it.
//!
//! `docs/features.md` promises that *labels give way to icons where the short
//! edge demands it*, and this is the answer to that clause: three states, one of
//! which is a name that is not being drawn. Which one applies is
//! [`crate::Layout`]'s to work out; what they mean, and what is said about them,
//! is here.
//!
//! # Giving way is not taking away
//!
//! A name that is not drawn is still the application's name. It is still what a
//! screen reader announces, still what appears when somebody rests on the icon,
//! and still what the application is called everywhere else in the system.
//! EN 301 549 carries WCAG's requirement that text resize to 200% **without loss
//! of content or function**, and a dock that dropped names as the text grew
//! would be losing content in the one setting a person turns up because they
//! cannot read the screen — unless the name is still there.
//!
//! So the reassurance is inside the string ([`crate::words::NAMES_GAVE_WAY`])
//! rather than beside it: a translator is handed it, a checked translation
//! cannot silently drop it, and a person who has just watched the names
//! disappear reads why and reads where they went, in one sentence, in their own
//! language.

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// Where the names in the dock are, if they are anywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Labels {
    /// Under each icon, with room for a line of text below the picture — which
    /// is where a name goes on a dock along the bottom, and ADR 0076 fixed the
    /// dock there.
    ///
    /// `Beside` was the other placement, for a dock down the side of the screen,
    /// and went with it. Two placements are why this is an enum rather than a
    /// `bool`, and one placement is not a reason to make it one: *drawn, and
    /// where* is still the question, and the third state is still an answer to it.
    Under,
    /// Nowhere: at this text size, as a percentage, there was not room. The
    /// name is still announced and still shown when somebody rests on the icon.
    GaveWay(u16),
}

impl Labels {
    /// Whether the names are being drawn.
    #[must_use]
    pub const fn are_shown(self) -> bool {
        matches!(self, Self::Under)
    }

    /// The string this crate declares for this state.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Under => words::NAMES_UNDER,
            Self::GaveWay(_) => words::NAMES_GAVE_WAY,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    ///
    /// The percentage goes in without a sign on it, and the sign is part of the
    /// sentence — so a language that writes *200 %* with a space, or puts the
    /// sign in front, can.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let filling = match self {
            Self::Under => Filling::nothing(),
            Self::GaveWay(percent) => Filling::of("percent", percent.to_string()),
        };
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// One of the two is a name being drawn and the other is not — which is the
    /// question a settings panel and a compositor both ask first.
    #[test]
    fn only_the_placement_is_names_being_drawn() {
        assert!(Labels::Under.are_shown());
        assert!(!Labels::GaveWay(300).are_shown());
    }

    /// Each state has a string of its own, and they are two sentences rather
    /// than one with a word swapped into it, because a language that inflects
    /// the placement needs the whole sentence in front of it — which is
    /// `alo-egress`' rule about its indicator line, met here.
    #[test]
    fn each_state_says_something_of_its_own() {
        let strings = in_english();
        assert_eq!(
            Labels::Under.said(&strings).text(),
            "each icon has its name under it"
        );
        assert_ne!(
            Labels::Under.word().key(),
            Labels::GaveWay(300).word().key()
        );
    }

    /// **The sentence about names disappearing says where they went.** It is
    /// read by somebody who turned their text up because they could not read the
    /// screen, so the half that matters is the second one.
    #[test]
    fn giving_way_says_the_size_and_says_the_name_is_still_there() {
        let strings = in_english();
        let said = Labels::GaveWay(300).said(&strings);
        assert_eq!(
            said.text(),
            "there is no room for names at 300% text, so the dock shows icons — resting on one \
             still gives its name, and a screen reader still reads it"
        );
        assert!(said.unfilled().is_empty());
    }

    /// The percent sign belongs to the sentence, so a language that writes a
    /// space before it can — the same rule `alo_appearance::TextScale` settled
    /// for its two refusals.
    #[test]
    fn the_percent_sign_is_the_translators_to_place() {
        let strings = translated(&[(
            words::NAMES_GAVE_WAY,
            "bei {percent} % Textgröße ist kein Platz für Namen — das Dock zeigt Symbole, und der \
             Name wird weiterhin vorgelesen",
        )]);
        let said = Labels::GaveWay(300).said(&strings);
        assert_eq!(
            said.text(),
            "bei 300 % Textgröße ist kein Platz für Namen — das Dock zeigt Symbole, und der Name \
             wird weiterhin vorgelesen"
        );
        assert!(said.is_translated());
        assert!(said.unfilled().is_empty());
    }
}
