//! Every string this crate can say, and the English beside each one.
//!
//! Two sentences, and the first is the point of the file: the refusal a
//! person reads when a sign-in fails is **one sentence for two facts** — a
//! wrong password and an unknown name — because a sentence that told them
//! apart would let anybody at the sign-in surface enumerate which names have
//! accounts. `crate::store` keeps the same promise in time; this file keeps
//! it in words.
//!
//! # Neither sentence has a gap in it
//!
//! A gap is the one road text somebody typed — a name tried at a sign-in
//! prompt — could take into a sentence a person reads. The numbers behind
//! [`NOT_THIS_MACHINES_PERSON`] stay in the refusal value for whoever fixes
//! the machine; the sentence itself carries none of them.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Re-exported as every other declaring crate does, so this crate's own
/// files name it as `crate::words::Word`.
pub use alo_strings::Word;

/// A name and password that did not sign anybody in.
///
/// Said for a wrong password **and** for a name with no account,
/// deliberately as one sentence — see this file's header.
pub const NOT_SIGNED_IN: Word = Word::saying(
    "accounts.not-signed-in",
    "that name and password do not sign anyone in on this machine — check both and try again",
)
.noting(
    "Said at the sign-in screen when the name or the password is wrong. It is deliberately one \
     sentence for both cases: saying which was wrong would tell an attacker which names have \
     accounts on this machine. Keep the translation equally silent about which of the two it was.",
);

/// An account that authenticated and is not the person this machine is
/// described as.
pub const NOT_THIS_MACHINES_PERSON: Word = Word::saying(
    "accounts.not-this-machines-person",
    "this machine is set up for a different account, so you were not signed in — its \
     description and its accounts disagree, and whoever installed it has to put that right",
)
.noting(
    "Said when the name and password were correct but the machine's own description file names \
     a different login as its person, so no session was started. The person in front of the \
     screen usually cannot fix this themselves; the sentence sends them to whoever installed or \
     manages the machine. \"Description\" refers to the machine's configuration, not to prose.",
);

/// Everything this crate can say.
pub const EVERY_WORD: [Word; 2] = [NOT_SIGNED_IN, NOT_THIS_MACHINES_PERSON];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads: it keeps its English and its `Display` for
/// the reason every other crate's does, which is that whoever reads it is
/// whoever is fixing the list.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note
    /// that could not be attached.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// This crate's words, as a vocabulary of their own.
///
/// For a test, and for anything holding only this list. A machine loads one
/// vocabulary that every crate declares into, which is [`declare_into`].
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn accounts_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put this crate's words into a vocabulary something else is building.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared
/// it first said what that string is.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alo_strings::Key;
    use std::collections::BTreeSet;

    /// Every key here is a key, which is what `Word::key` going through
    /// `Key::unchecked` owes each declaring crate.
    #[test]
    fn every_key_this_crate_writes_is_a_key() {
        for word in EVERY_WORD {
            assert!(Key::named(word.named()).is_ok(), "{}", word.named());
        }
    }

    /// A key means one string, so the list may not name one twice.
    #[test]
    fn no_key_is_declared_twice() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// **Every word tells a translator what it is about** — both of these
    /// turn on facts no literal translation carries: one is deliberately
    /// vague, and the other names a file as if it were a person's choice.
    #[test]
    fn every_word_tells_a_translator_what_it_is_about() {
        for word in EVERY_WORD {
            assert!(
                word.note().is_some_and(|note| !note.trim().is_empty()),
                "{} has nothing to say to a translator",
                word.named()
            );
        }
    }

    /// **Nothing here has a gap in it.** A gap is the only road a name tried
    /// at a sign-in prompt could take into a sentence a person reads.
    #[test]
    fn no_sentence_here_has_a_gap_a_typed_name_could_enter_by() {
        for word in EVERY_WORD {
            assert!(!word.says().contains('{'), "{} carries a gap", word.named());
        }
    }

    /// The list declares into an empty vocabulary and refuses a second
    /// declaration, which is what one key meaning one string looks like.
    #[test]
    fn the_list_declares_once_and_only_once() {
        let mut vocabulary = match accounts_words() {
            Ok(vocabulary) => vocabulary,
            Err(why) => unreachable!("this crate's own list would not declare: {why}"),
        };
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }
}
