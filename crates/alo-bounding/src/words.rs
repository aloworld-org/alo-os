//! Everything this crate says to a person, and the English beside it.
//!
//! [`crate::NotBounded`] has eleven reasons and keeps its English, for the
//! reason `failing.rs` gives: every one of them is a fact about the *kernel
//! underneath the daemon*, read by whoever is standing a machine up. None of
//! them is read by somebody who signed in.
//!
//! But somebody does have to be told, and until this file existed nobody had
//! decided what they are told. ADR 0013 and ADR 0015 both end on *a turn whose
//! boundary cannot be applied does not run*, and a person whose agent does
//! nothing and says nothing is a person with a broken computer.
//!
//! # Eleven reasons, one sentence
//!
//! That is the decision here, and it is deliberate rather than lazy. A person
//! reading *this kernel has no `file.f_path`* learns nothing they can act on and
//! is handed a fact about their machine's internals in a language that is a
//! second language for most of the people alo OS is for. What they need is what
//! is true in all eleven: nothing was done, nothing was allowed either, and the
//! machine — not their agent, and not what they asked for — is what has to be
//! looked at.
//!
//! So the eleven keep their English for the administrator and the person reads
//! one sentence. [`crate::NotBounded::said`] is the join, and it deliberately
//! has no gap in it: putting the reason inside would put an untranslated English
//! clause in the middle of a translated sentence, which is the failure the
//! 9-series spent seven items removing.
//!
//! # This list is not in `alo-saying`
//!
//! `alo-saying` reaches every crate that says anything and deliberately does not
//! reach `alo-agentd`, because that crate is Linux and a vocabulary three strings
//! shorter on one host would be a translation refused on that host and accepted
//! on another. This crate is Linux for the same reason and is left out for the
//! same reason: whatever assembles a process declares [`declare_into`] on top of
//! the machine's vocabulary, which is the rule `docs/contracts/translations.md`
//! states.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Re-exported as every other declaring crate does, so this crate's own files
/// name it as `crate::words::Word`.
pub use alo_strings::Word;

/// The boundary a turn runs inside could not be imposed, so nothing ran.
pub const NOTHING_CAN_BE_BOUNDED: Word = Word::saying(
    "bounding.nothing-can-be-bounded",
    "nothing was done: this machine cannot hold an agent inside what you granted it, so it will \
     not let one act at all — ask whoever set this machine up to look at it",
)
.noting(
    "Shown to a person whose agent did nothing because alo OS could not put the boundary around \
     it that keeps it inside the folders and files they granted. Nothing was attempted and \
     nothing was refused: this is a fault in the machine rather than an answer to anything the \
     person or the agent asked for. The reason is a separate, technical sentence kept in English \
     for whoever administers the machine, and must not be worked into this one.",
);

/// Everything this crate can say, in the order this file declares them.
///
/// The array is what a test reads down and what [`declare_into`] walks, so a
/// word declared above and left out here is a string nothing can look up.
pub const EVERY_WORD: [Word; 1] = [NOTHING_CAN_BE_BOUNDED];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads: it keeps its English and its `Display` for the
/// reason every other declaring crate's does, which is that whoever reads it is
/// whoever is fixing the list.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
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
pub fn bounding_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put this crate's words into a vocabulary something else is building.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds this key — nothing is
/// replaced, because a key means one string and whoever declared it first said
/// what that string is.
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
    use alo_strings::Key;

    /// Every key here is a key, which is what `Word::key` going through
    /// `Key::unchecked` owes each declaring crate.
    #[test]
    fn every_key_this_crate_writes_is_a_key() {
        for word in EVERY_WORD {
            assert!(Key::named(word.named()).is_ok(), "{}", word.named());
        }
    }

    /// **The one sentence carries a note**, and it has to: a person is being
    /// told that something they cannot see failed, and a translator who thought
    /// this was a refusal would write it as one.
    #[test]
    fn the_sentence_tells_a_translator_what_it_is_about() {
        for word in EVERY_WORD {
            assert!(
                word.note().is_some_and(|note| !note.trim().is_empty()),
                "{} has nothing to say to a translator",
                word.named()
            );
        }
    }

    /// **Nothing here has a gap in it**, which is this file's own decision: the
    /// eleven reasons are English for an administrator, and one of them dropped
    /// into a translated sentence would be half a sentence nobody can read.
    #[test]
    fn nothing_here_can_be_handed_a_reason() {
        for word in EVERY_WORD {
            assert!(
                !word.says().contains('{'),
                "{} has a gap in it",
                word.named()
            );
        }
    }

    /// The list can be declared, which is what anything loading it does.
    #[test]
    fn this_crate_can_declare_its_own_words() {
        assert_eq!(bounding_words().unwrap().how_many(), EVERY_WORD.len());
    }

    /// **The sentence names nothing alo OS rents.** `alo-saying` asks this of
    /// the machine's whole vocabulary, and this list is deliberately not in
    /// that vocabulary — so asking it here is what stops the exemption being a
    /// hole. It matters more here than anywhere: everything else this crate can
    /// say names `aya`, BPF, a control group or a kernel structure, and the one
    /// string that reaches a person may not.
    #[test]
    fn nothing_this_crate_says_names_anything_we_rent() {
        let overheard = alo_saying::what_a_person_would_have_to_learn(&bounding_words().unwrap());
        assert!(
            overheard.is_empty(),
            "{}",
            overheard
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
