//! Every string this crate can say, and the English beside each one.
//!
//! Eight of them, and all eight are about one thing: a file that is there and
//! is not settings. Nothing here says anything about a **choice**, and that
//! absence is deliberate — `crate::bound` has the argument. A person's choice
//! is honoured or refused by the rule an organisation set, the rule's own words
//! are `alo-models`', and no rule an organisation can set refuses anything a
//! person can currently choose. A sentence for a refusal that cannot happen is
//! a string a translator was handed for nothing.
//!
//! The last three arrived with the person's own weights (ADR 0019), and they
//! are still about the file: two are a `[[brought]]` entry that is not weights,
//! and the third is a file whose two halves disagree. `alo_models::WeightsError`
//! says two of those things about a **list** and is not reworded here, because
//! what a person acts on is the path and a list has no path in it.
//!
//! # They all name the file
//!
//! A path is data and is never translated, in the way a filename is not
//! translated in `alo-files` and an address is not in `alo-egress`. It is in
//! every one of them because it is the thing a person needs in order to act,
//! and *your settings* is not that thing on a machine with several logins.
//!
//! # Nothing here counts anything
//!
//! `alo-models`' rule from item 9f, kept: there is no
//! [`alo_strings::Plural`] in this list, and a sentence that would have had to
//! count would carry the number beside it instead.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Re-exported as every declaring crate does, so this crate's own files name it
/// as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// A file that is there and is not settings — [`crate::NotSet`].
//
// Five sentences, because they send a person to five different places. Each
// names the file: {path} is where it is, exactly as it is written on the disk,
// and is never translated.
// ---------------------------------------------------------------------------

/// The disk would not give the file up.
pub const SETTINGS_NOT_READ: Word = Word::saying(
    "choosing.settings.not-read",
    "your settings at {path} could not be read, so nothing has been chosen to answer questions",
)
.noting(
    "{path} is a file on this machine and is never translated. This is a disk or a permission \
     rather than anything a person typed — most often a file they cannot read. The second half \
     says what follows from it, which is the thing they are about to notice.",
);

/// The text is not settings.
pub const SETTINGS_NOT_UNDERSTOOD: Word = Word::saying(
    "choosing.settings.not-understood",
    "your settings at {path} are not settings alo OS can read, so nothing has been chosen to \
     answer questions — nothing in the file has been used",
)
.noting(
    "{path} is a file on this machine and is never translated. The last clause is the important \
     one: alo OS did not take the half it understood. Said of a file with a typo in it, a key \
     nobody declared, or a choice this machine has no list for.",
);

/// The file says it is a shape this alo OS does not read.
pub const SETTINGS_FROM_A_NEWER_ALO_OS: Word = Word::saying(
    "choosing.settings.another-format",
    "your settings at {path} were written by a newer alo OS than this one, so nothing in the file \
     has been used",
)
.noting(
    "{path} is a file on this machine and is never translated. Said rather than guessed at: a \
     newer alo OS may let somebody choose things this one cannot honour, and reading such a file \
     part-way would answer their questions somewhere they did not pick. \"alo OS\" is the \
     product's name and is never translated.",
);

/// A list was named and no model with it.
pub const SETTINGS_NAME_NO_MODEL: Word = Word::saying(
    "choosing.settings.no-model",
    "your settings at {path} name a list of models and no model in it, so nothing has been chosen \
     to answer questions",
)
.noting(
    "{path} is a file on this machine and is never translated. A \"list of models\" is either the \
     catalogue alo OS ships or the weights the person added themselves. This is what an empty \
     value looks like — somebody cleared a field rather than removing the setting.",
);

/// A language that is not one.
pub const SETTINGS_NAME_NO_LANGUAGE: Word = Word::saying(
    "choosing.settings.not-a-language",
    "your settings at {path} ask to be read in {language}, which is not written the way a language \
     is named, so nothing in the file has been used",
)
.noting(
    "{path} and {language} are both taken from the file exactly as they are written there and are \
     never translated — {language} is what somebody typed where a language tag belongs, and \
     quoting it back is what lets them find it. A tag is written the way the rest of the world \
     writes one: de, pt-BR, sr-Latn-RS.",
);

/// Weights on the person's own list with no name.
pub const SETTINGS_WEIGHTS_UNNAMED: Word = Word::saying(
    "choosing.settings.weights-unnamed",
    "your settings at {path} list weights with no name, and there would be nothing to ask for, so \
     nothing in the file has been used",
)
.noting(
    "{path} is a file on this machine and is never translated. \"Weights\" are a model somebody \
     put on this machine themselves — the file the model is; alo OS never offered it and does not \
     state its licence. A name here is what the model runtime answers to rather than a title \
     somebody picked.",
);

/// The same weights on that list twice.
pub const SETTINGS_WEIGHTS_TWICE: Word = Word::saying(
    "choosing.settings.weights-twice",
    "your settings at {path} list weights called {model} twice, so nothing said which of them \
     answered a question, and nothing in the file has been used",
)
.noting(
    "{path} and {model} are both taken from the file exactly as they are written there and are \
     never translated — {model} is the name the model runtime answers to. \"Weights\" are a model \
     somebody put on this machine themselves. The middle clause is why this is refused rather \
     than tidied: alo OS says which model answered, and it could not.",
);

/// A choice naming weights the person's own list does not have.
pub const SETTINGS_NOT_BROUGHT: Word = Word::saying(
    "choosing.settings.not-brought",
    "your settings at {path} say {model} answers your questions and do not list weights of that \
     name, so nothing in the file has been used",
)
.noting(
    "{path} and {model} are both taken from the file exactly as they are written there and are \
     never translated. Said of a file whose two halves disagree: one says which model answers, \
     the other is the list of models the person brought to this machine, and the first names \
     nothing on the second. Most often a name typed twice with one letter different.",
);

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 8] = [
    SETTINGS_NOT_READ,
    SETTINGS_NOT_UNDERSTOOD,
    SETTINGS_FROM_A_NEWER_ALO_OS,
    SETTINGS_NAME_NO_MODEL,
    SETTINGS_NAME_NO_LANGUAGE,
    SETTINGS_WEIGHTS_UNNAMED,
    SETTINGS_WEIGHTS_TWICE,
    SETTINGS_NOT_BROUGHT,
];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads — it keeps its English and its `Display` for
/// the reason every other crate's does, which is that whoever reads it is
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

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn choosing_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
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
    use alo_strings::Key;
    use std::collections::BTreeSet;

    /// What we ship is held to the rule everybody else is held to: `Word::key`
    /// does not check, because a key written in this file cannot arrive from
    /// anywhere, and this is the test that makes that true.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(Key::named(word.named()), Ok(word.key()), "{}", word.named());
        }
    }

    /// A key names one string.
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
            assert_eq!(word.key().area(), "choosing", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = choosing_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0);
    }

    /// A vocabulary that already holds one of these keeps its own.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = choosing_words().unwrap();
        assert!(matches!(
            declare_into(&mut vocabulary).unwrap_err(),
            WordsError::List(_)
        ));
    }

    /// **Every one of them tells a translator what it is about.** All eight
    /// have a gap that is data rather than language, and a translation that
    /// treated `{path}`, `{language}` or `{model}` as something to render would
    /// be a sentence a person cannot act on.
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

    /// **Every sentence about the file names the file.** A person who has been
    /// told their settings are wrong needs the one thing that lets them fix it.
    #[test]
    fn every_sentence_about_the_file_names_the_file() {
        for word in EVERY_WORD {
            assert!(word.says().contains("{path}"), "{}", word.named());
        }
    }

    /// **Every sentence that quotes a name a runtime answers to says whose it
    /// is.** `{model}` is what somebody wrote in their own file, and a
    /// translator who took it for a word to render would produce a sentence
    /// nobody can search their settings for.
    #[test]
    fn every_sentence_that_quotes_a_name_tells_a_translator_it_is_not_a_word() {
        for word in [SETTINGS_WEIGHTS_TWICE, SETTINGS_NOT_BROUGHT] {
            assert!(word.says().contains("{model}"), "{}", word.named());
            assert!(
                word.note()
                    .is_some_and(|note| note.contains("never translated")),
                "{}",
                word.named()
            );
        }
    }

    /// **Every one of them says what follows from it**, which is the half a
    /// person acts on: their machine has not chosen anything to answer
    /// questions, or nothing in the file was used. A sentence saying only that
    /// a file is wrong would leave somebody wondering whether their agent had
    /// answered anyway.
    ///
    /// The two consequences are spelled one way each rather than matched
    /// loosely, because a test that accepted any sentence with *used* in it
    /// would pass on a string that had stopped saying this at all.
    #[test]
    fn every_sentence_says_what_the_machine_did_about_it() {
        const WHAT_THE_MACHINE_DID: [&str; 2] = [
            "nothing has been chosen to answer questions",
            "nothing in the file has been used",
        ];
        for word in EVERY_WORD {
            assert!(
                WHAT_THE_MACHINE_DID
                    .iter()
                    .any(|did| word.says().contains(did)),
                "{}: {}",
                word.named(),
                word.says()
            );
        }
    }

    /// **Nothing here counts anything out loud**, which is `alo-models`' rule
    /// from item 9f: a gap named for a quantity would be English's two shapes
    /// standing in for Polish's three.
    #[test]
    fn nothing_this_crate_says_counts_something() {
        assert_eq!(choosing_words().unwrap().counted().count(), 0);
        for word in EVERY_WORD {
            assert!(
                !word.says().chars().any(|char| char.is_ascii_digit()),
                "{}",
                word.named()
            );
        }
    }
}
