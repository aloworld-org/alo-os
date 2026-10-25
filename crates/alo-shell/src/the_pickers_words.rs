//! What a person reads while they are giving the agent a folder.
//!
//! `alo-picking` already words the picker's refusals and says what picking a
//! folder will do. What it has no word for is the **surface**: the row that
//! opens the picker, the two moves a person makes in it, and the question that
//! comes after a folder is chosen — how long alo may have it.
//!
//! Those are here rather than there for the reason `crate::the_providers_words`
//! gives: `alo-picking` is the picker as a decision and deliberately nothing
//! more, and *the row you press to open it* is a fact about this window.
//!
//! # Every move is a row, so nothing new has to be learned
//!
//! Settings has four keys — previous, next, choose, close — and no letters.
//! Rather than teach the picker a fifth key for *pick this one* and a sixth
//! for *go up*, both are rows a person lands on and presses Enter. It makes
//! the list a little longer and it means the picker is operated by exactly the
//! keys the rest of the window already uses, which is the difference between a
//! surface somebody can work and one they have to be told about.
//!
//! # A note is part of the string
//!
//! *The agent* is the assistant built into alo OS and not a person. *To grant*
//! is for the person to allow the agent to reach one thing. These are read by
//! somebody standing in a folder chooser deciding what to hand over, so each
//! says what it will do rather than what it is.

use alo_strings::{Vocabulary, Word};

/// The row in Settings that opens the picker.
pub const GRANT_A_FOLDER: Word = Word::saying("picker.grant-a-folder", "Give alo a folder").noting(
    "alo is the assistant built into alo OS, not a person. The row in Settings a person presses \
     to start choosing a folder for alo to be able to read. An instruction rather than a title: \
     it is the only row in its section that does something rather than listing something.",
);

/// The row that picks the folder the person is standing in.
pub const PICK_THIS_FOLDER: Word = Word::saying("picker.pick-this-folder", "Give alo this folder")
    .noting(
        "alo is the assistant built into alo OS, not a person. The row a person presses to choose \
         the folder they are currently looking inside. It is the first row in the list so that \
         the folder they opened is the easiest one to choose.",
    );

/// The row that goes up one level.
pub const UP_ONE_FOLDER: Word = Word::saying("picker.up-one-folder", "Up one folder").noting(
    "The row a person presses to leave the folder they are looking inside and see the one that \
     holds it. Not shown when there is nothing above.",
);

/// The question asked once a folder is chosen.
pub const HOW_LONG: Word = Word::saying(
    "picker.how-long",
    "How long should alo be able to read it? It will stop on its own",
)
.noting(
    "alo is the assistant built into alo OS, not a person. The heading over the three lengths \
     below, shown after a person has chosen a folder. The second sentence matters: it tells them \
     the access ends by itself, so that choosing the longest option is not a decision they can \
     forget about forever.",
);

/// The shortest length offered.
pub const FOR_AN_HOUR: Word = Word::saying("picker.for-an-hour", "For an hour").noting(
    "One of three lengths of time a person chooses between when giving alo a folder. For somebody \
     who wants one piece of work done now and no more than that.",
);

/// A working day.
pub const FOR_A_DAY: Word = Word::saying("picker.for-a-day", "For a day").noting(
    "One of three lengths of time a person chooses between when giving alo a folder. A working \
     day, which outlasts restarting the machine.",
);

/// A month.
pub const FOR_A_MONTH: Word = Word::saying("picker.for-a-month", "For a month").noting(
    "One of three lengths of time a person chooses between when giving alo a folder. For the \
     folder somebody actually works in and would otherwise have to choose again every morning. \
     Thirty days, which is what a month means everywhere else in software.",
);

/// Said when a folder was given, naming it.
pub const IT_WAS_GIVEN: Word = Word::saying(
    "picker.it-was-given",
    "alo can now read {folder}. It is in the list below, and you can take it back there",
)
.noting(
    "alo is the assistant built into alo OS, not a person. Shown after a person has given alo a \
     folder. {folder} is replaced with the folder's name and must be kept. The second half points \
     at the list of everything granted, which is in the same section just below this.",
);

/// The gap [`IT_WAS_GIVEN`] leaves for the folder.
///
/// Named here rather than written twice: the sentence above and the code that
/// fills it are one thing, and a typo between them would put the key in front
/// of a person at the moment they are told what they just handed over.
pub const THE_FOLDER: &str = "folder";

/// Every string this file declares, in the order a translator meets them.
pub const THE_PICKERS_WORDS: [Word; 8] = [
    GRANT_A_FOLDER,
    PICK_THIS_FOLDER,
    UP_ONE_FOLDER,
    HOW_LONG,
    FOR_AN_HOUR,
    FOR_A_DAY,
    FOR_A_MONTH,
    IT_WAS_GIVEN,
];

/// Why this file's words could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ThePickersWordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Add this surface's wording atomically, before any translation is loaded.
///
/// # Errors
/// [`ThePickersWordsError`] for a malformed phrase or a key already taken.
pub fn declare_the_pickers_words(vocabulary: &mut Vocabulary) -> Result<(), ThePickersWordsError> {
    // **Prepared and swapped**, which is `declare_the_providers_words`' shape:
    // a declaration that failed halfway would leave a vocabulary holding some
    // of these and not others, and the sentence missing would be found on a
    // screen rather than here.
    let mut prepared = vocabulary.clone();
    for word in THE_PICKERS_WORDS {
        prepared.says(word.phrase()?)?;
    }
    *vocabulary = prepared;
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Every one of them declares, which is what the shell's boot relies on.
    #[test]
    fn every_word_here_can_be_declared() {
        let mut vocabulary = Vocabulary::empty();
        declare_the_pickers_words(&mut vocabulary).unwrap();
        for word in THE_PICKERS_WORDS {
            assert!(vocabulary.phrase(&word.key()).is_some(), "{word:?}");
        }
    }

    /// Declaring twice is refused rather than silently replacing, and the
    /// first failure leaves the vocabulary exactly as it was.
    #[test]
    fn declaring_the_same_words_twice_is_refused_and_changes_nothing() {
        let mut vocabulary = Vocabulary::empty();
        declare_the_pickers_words(&mut vocabulary).unwrap();
        let before = vocabulary.clone();
        assert!(declare_the_pickers_words(&mut vocabulary).is_err());
        assert_eq!(vocabulary.how_many(), before.how_many());
    }

    /// The sentence a person is told what they gave away with keeps its gap.
    ///
    /// A translator who drops `{folder}` leaves somebody reading *alo can now
    /// read* with nothing after it, at the one moment they most want to know
    /// which folder.
    #[test]
    fn the_sentence_naming_the_folder_has_somewhere_to_put_it() {
        assert!(
            IT_WAS_GIVEN.says().contains(&format!("{{{THE_FOLDER}}}")),
            "{}",
            IT_WAS_GIVEN.says()
        );
    }

    /// No key here collides with another in this file.
    #[test]
    fn no_two_words_here_share_a_key() {
        let mut keys: Vec<String> = THE_PICKERS_WORDS
            .iter()
            .map(|it| it.key().to_string())
            .collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), THE_PICKERS_WORDS.len());
    }
}
