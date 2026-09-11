//! Every string this crate can say, and the English beside each one.
//!
//! Two, and each is the sentence for one of the two moments this crate is
//! about.
//!
//! [`NOT_KEPT`] is the write that failed. `alo_remembering::kept` replaces the
//! file whole or not at all, so the one thing a person needs at that moment is
//! that **nothing moved**: the file is as it was, the daemon was not knocked,
//! and what is granted did not change in either direction. Which failure it
//! was keeps its English inside [`crate::NotChanged`], for whoever is looking
//! at the machine — the person is told the one thing that is theirs to know.
//!
//! [`AT_THE_NEXT_SIGN_IN`] is the change that stood with nobody at the door.
//! It exists because silence there would be ambiguous in the worst direction:
//! a person who revoked something worrying and was told nothing could not
//! tell *the machine handled it* from *nothing happened*. The sentence says
//! the change is safe, when it takes effect everywhere, and that there is
//! nothing more to do — because a machine that leaves a person wondering
//! whether to do it again is a machine that gets the same change made twice.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

/// The change was not saved, and nothing moved.
pub const NOT_KEPT: Word = Word::saying(
    "changing.not-kept",
    "The change was not saved: the file this machine keeps its grants in would not take it. \
     Everything is as it was — nothing was granted and nothing was revoked",
)
.noting(
    "Shown when writing the machine's own grants file fails — a disk problem, or a file somebody \
     has interfered with. The second sentence is the point and must survive translation: the \
     file is replaced whole or not at all, so a failed save changes nothing in either direction \
     — no new permission exists, and no permission the person tried to take away has been taken \
     away. What exactly went wrong is written to the machine's log in English for whoever \
     maintains it; this sentence is everything the person needs.",
);

/// The change is saved, and it takes effect at the next sign-in.
pub const AT_THE_NEXT_SIGN_IN: Word = Word::saying(
    "changing.at-the-next-sign-in",
    "The change is saved. Nothing is running to hear about it right now, so it takes effect at \
     the next sign-in — there is nothing more to do",
)
.noting(
    "Shown when a grant was made or revoked and no agent service was running to be told. The \
     change (a permission given or taken away) is safely on the machine's own disk and will \
     apply the moment anything starts that reads it. The last clause matters: the person must \
     not be left wondering whether to make the change again.",
);

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 2] = [NOT_KEPT, AT_THE_NEXT_SIGN_IN];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// are what say so. It is a `Result` rather than an unwrap because a library
/// that panics on its own string table takes the shell with it, and because
/// [`declare_into`] can genuinely fail against a vocabulary that already holds
/// one of these keys.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn changing_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// The machine has one vocabulary and every crate adds its own to it —
/// `alo-saying` is the one place that calls all of these.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared it
/// first said what that string is.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// **What we ship is held to the rule everybody else is held to.**
    /// `Word::key` does not check, because a key written in this file cannot
    /// arrive from anywhere; this is the test that makes that true.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
        }
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "changing", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        assert_eq!(changing_words().unwrap().how_many(), EVERY_WORD.len());
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = changing_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Both sentences are whole.** Each is read at a moment something
    /// already needs explaining, and a `{}` in front of a person at that
    /// moment is the worst available rendering of it.
    #[test]
    fn nothing_here_has_a_gap_in_it() {
        for word in EVERY_WORD {
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{} has a gap in it",
                word.named()
            );
        }
    }

    /// **The sentences stand alone**, so each begins one.
    #[test]
    fn every_sentence_begins_a_sentence() {
        for word in EVERY_WORD {
            let first = word.says().chars().next().unwrap();
            assert!(
                first.is_uppercase(),
                "{} does not begin a sentence",
                word.named()
            );
        }
    }

    /// **Every word carries a note.** Neither can be translated from its own
    /// words: each is read at a moment the translator has to be able to
    /// picture.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Nothing here counts anything.** A number written into an English
    /// sentence cannot be translated into a language with three plural forms.
    #[test]
    fn nothing_here_counts_anything() {
        for word in EVERY_WORD {
            assert!(
                !word.says().chars().any(|letter| letter.is_ascii_digit()),
                "{}",
                word.named()
            );
        }
    }

    /// **The failure says nothing moved and the success says what to do** —
    /// which is nothing. The two facts a person is owed at these two moments,
    /// held where the translator meets them.
    #[test]
    fn each_sentence_carries_the_fact_it_exists_for() {
        assert!(NOT_KEPT.says().contains("nothing was granted"));
        assert!(NOT_KEPT.says().contains("nothing was revoked"));
        assert!(AT_THE_NEXT_SIGN_IN.says().contains("next sign-in"));
        assert!(AT_THE_NEXT_SIGN_IN.says().contains("nothing more to do"));
    }
}
