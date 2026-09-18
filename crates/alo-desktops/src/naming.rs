//! What a person calls a desktop, and why a name was not taken.
//!
//! A desktop need not have a name — most people never name one, and an unnamed
//! desktop is read as *Desktop 3* from its position
//! ([`crate::words::DESKTOP_NUMBERED`]). A [`Name`] is what a person types when
//! they do want one, and the three refusals below are the three ways what they
//! typed is not a name.
//!
//! **Surrounding space is trimmed rather than refused.** A person who typed a
//! trailing space meant the name without it, and a refusal there would be a
//! system correcting somebody about something they cannot see. A name that is
//! *only* space is refused, because there is nothing left of it.
//!
//! # Any script, any length a person can read at a glance
//!
//! There is no rule here about which letters a name may be made of: *Mail*,
//! *Πόστα* and *Ríomhphost* are all names, and a check that liked one of them
//! more than the others would be exactly the bug the i18n rule in `CLAUDE.md`
//! exists to stop. What is refused is a name with control characters in it —
//! which is not a script, it is a name that would draw as something else — and
//! one longer than [`MOST_CHARACTERS`], which is a label, not a document.

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// The most characters a desktop's name may have.
///
/// Counted in characters a person would count, not bytes: *Müller* is six here
/// and seven in UTF-8, and a limit in bytes would give a shorter name to the
/// languages that need the most of them.
pub const MOST_CHARACTERS: usize = 40;

/// What a person calls one desktop.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

impl Name {
    /// The name a person typed, with any surrounding space trimmed off.
    ///
    /// # Errors
    /// [`NameError`], in the language the person reads.
    pub fn given(typed: &str) -> Result<Self, NameError> {
        let trimmed = typed.trim();
        if trimmed.is_empty() {
            return Err(NameError::Nothing);
        }
        if trimmed.chars().any(char::is_control) {
            return Err(NameError::NotWords);
        }
        let how_many = trimmed.chars().count();
        if how_many > MOST_CHARACTERS {
            return Err(NameError::TooLong(how_many));
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// The name, as it is shown.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Why what a person typed is not a name for a desktop.
///
/// There is no `Display`: the only road to words is [`NameError::said`], in the
/// language the person reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameError {
    /// Nothing, or nothing but space.
    Nothing,
    /// Longer than [`MOST_CHARACTERS`], with how many characters it had.
    TooLong(usize),
    /// Characters that are not writing: a name that would draw as something
    /// other than what was typed.
    NotWords,
}

impl NameError {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Nothing => words::NAME_IS_NOTHING,
            Self::TooLong(_) => words::NAME_TOO_LONG,
            Self::NotWords => words::NAME_IS_NOT_WORDS,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let filling = match self {
            Self::TooLong(_) => Filling::of("most", MOST_CHARACTERS.to_string()),
            Self::Nothing | Self::NotWords => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// A name is what was typed, trimmed, in whatever script a person writes.
    #[test]
    fn a_name_is_what_was_typed_in_any_script() {
        for typed in ["Mail", "  Mail  ", "Πόστα", "Ríomhphost", "Müller's day"] {
            assert_eq!(Name::given(typed).unwrap().as_str(), typed.trim());
        }
    }

    /// The three ways what was typed is not a name, each with its own sentence.
    #[test]
    fn nothing_control_characters_and_a_document_are_not_names() {
        assert_eq!(Name::given(""), Err(NameError::Nothing));
        assert_eq!(Name::given("   \t "), Err(NameError::Nothing));
        assert_eq!(Name::given("Mail\nand notes"), Err(NameError::NotWords));
        let long = "ü".repeat(MOST_CHARACTERS + 1);
        assert_eq!(Name::given(&long), Err(NameError::TooLong(41)));
        // Exactly the limit is a name, counted in characters and not in bytes:
        // this one is 40 characters and 80 bytes.
        let limit = "ü".repeat(MOST_CHARACTERS);
        assert_eq!(limit.len(), MOST_CHARACTERS * 2);
        assert_eq!(Name::given(&limit).unwrap().as_str(), limit);

        let strings = in_english();
        let said = NameError::TooLong(41).said(&strings);
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(said.text().contains("40"), "{said}");
    }
}
