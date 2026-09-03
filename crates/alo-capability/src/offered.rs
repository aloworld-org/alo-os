//! One option a verb offers, and the two things it is at once.
//!
//! A [`crate::Takes::Choice`] is how a verb offers a decision without accepting
//! free text: the options exist before the model does. Each of them has to be
//! two things that cannot be one string, and that is the whole of this file.
//!
//! - **A name**, matched exactly. It is what a model sends, what the record
//!   keeps, and what a shell writes into a script — an identity, in the sense
//!   [`crate::grant`] means it, so it never changes meaning and is never
//!   translated. `left_half`.
//! - **A word**, which is what a person reads. It goes into the sentence they
//!   are asked to approve, so it is a string somebody translates, declared the
//!   way a verb's own words have been since item 9g. *on the left half of the
//!   screen*.
//!
//! # Why they are two things
//!
//! Before item 11a a choice was a list of plain strings, and the name was used
//! for both jobs — so the sentence a person approved read *put Blender on the
//! `left_half`*: untranslated English, in the one string the whole capability
//! model is built around, in every language on earth. That is item 9g's
//! guarantee — the string a translator is handed is the string the declaration
//! was checked against — failing for the one argument kind 9g did not reach.
//!
//! Going the other way round would be worse. If the *word* were the identity, a
//! model would have to send `on the left half of the screen`, a translator
//! could change what a verb can be called by editing a sentence, and the record
//! would keep a different value on a German machine than on a Greek one. So the
//! name is the identity and the word is the reading of it, exactly as
//! `alo-applications` splits an application's identifier from its name, and for
//! the same reason: what is approved and recorded must not be something anybody
//! downstream can rewrite.
//!
//! **Everything that has to be true of an option is checked where the verb is
//! declared** ([`crate::verb::Verb::checked`]), because that is where the other
//! declaration rules live and an option is part of a declaration.

use alo_strings::{Filling, Key, Said, Strings, Word};

/// One option a verb offers.
///
/// There is no `Deserialize` and no way to build one from a bare string: an
/// option carries a [`Word`], so a verb cannot express a choice whose options
/// are not translatable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offered {
    /// What names it: the identity a model sends and the record keeps.
    name: String,
    /// What a person reads for it.
    word: Word,
}

impl Offered {
    /// An option of this name, which a person reads as this.
    #[must_use]
    pub fn called(name: &str, word: Word) -> Self {
        Self {
            name: name.trim().to_owned(),
            word,
        }
    }

    /// What names it. An identity, matched exactly.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What names the words a person reads for it.
    #[must_use]
    pub fn key(&self) -> Key {
        self.word.key()
    }

    /// What a person reads for it, in the language they read.
    ///
    /// This is what goes into the approval sentence, and what a shell showing
    /// somebody the options would put beside each one. The answer says whether
    /// anybody translated it.
    #[must_use]
    pub fn shown(&self, strings: &Strings) -> Said {
        strings.say(&self.key(), &Filling::nothing())
    }

    /// The word as the verb declared it, for the checks made where a verb is
    /// declared.
    pub(crate) fn as_written(&self) -> &'static str {
        self.word.says()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{speaking, translating};

    /// The arrangement `alo-applications` declares, which is what this type was
    /// built for.
    const LEFT_HALF: Word =
        Word::saying("testing.where.left-half", "on the left half of the screen");

    /// An option is a name and a word, and neither stands in for the other.
    #[test]
    fn an_option_is_a_name_and_a_word() {
        let offered = Offered::called("  left_half  ", LEFT_HALF);
        assert_eq!(offered.name(), "left_half");
        assert_eq!(offered.key(), LEFT_HALF.key());
        assert_eq!(offered.as_written(), "on the left half of the screen");
        assert_eq!(
            offered.shown(&speaking(&[LEFT_HALF])).text(),
            "on the left half of the screen"
        );
    }

    /// **The name never moves and the word does.** A translation changes what
    /// somebody reads and nothing about what a model may send or what the
    /// record will hold.
    #[test]
    fn the_word_is_translated_and_the_name_is_not() {
        let offered = Offered::called("left_half", LEFT_HALF);
        let strings = translating(
            &[LEFT_HALF],
            &[(LEFT_HALF, "auf der linken Bildschirmhälfte")],
        );
        let said = offered.shown(&strings);
        assert!(said.is_translated());
        assert_eq!(said.text(), "auf der linken Bildschirmhälfte");
        assert_eq!(offered.name(), "left_half");
    }

    /// An option nobody has translated says so, like everything else here,
    /// rather than looking like a phrase somebody wrote in the reader's
    /// language.
    #[test]
    fn an_untranslated_option_says_where_it_came_from() {
        let said = Offered::called("left_half", LEFT_HALF).shown(&speaking(&[LEFT_HALF]));
        assert!(!said.is_translated());
        assert!(!said.is_a_bug());
    }

    /// A word this crate's vocabulary has never heard of shows the key and says
    /// it is a bug here — the same answer as any other undeclared string.
    #[test]
    fn an_option_declared_from_a_word_nobody_declares_is_a_bug_here() {
        const NOWHERE: Word = Word::saying("testing.where.nowhere", "nowhere at all");
        let said = Offered::called("nowhere", NOWHERE).shown(&speaking(&[LEFT_HALF]));
        assert!(said.is_a_bug());
        assert_eq!(said.text(), "«testing.where.nowhere»");
    }
}
