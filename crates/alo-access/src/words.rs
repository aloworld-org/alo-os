//! Every sentence this crate can say, and the notes a translator writes from.
//!
//! These are the sentences a person reads **while looking for the setting they
//! need**, sometimes without being able to see the screen, so they say what the
//! setting does rather than name a feature: *read the screen aloud*, not
//! *screen reader*, because somebody who has never used one is looking for the
//! first and not the second.

use alo_strings::Vocabulary;

pub use alo_strings::Word;

/// Read the screen aloud.
pub const SCREEN_READER: Word = Word::saying("access.screen-reader", "read the screen aloud")
    .noting(
        "A setting a person turns on. It starts the screen reader this machine ships with, which \
     reads what is on the screen and what the keyboard is on. Say it as what it does, because \
     somebody who has not used one before is looking for that rather than for its name.",
    );

/// Magnify what is under the pointer.
pub const MAGNIFIER: Word = Word::saying(
    "access.magnifier",
    "magnify what is under the pointer",
)
.noting(
    "A setting a person turns on. How much larger is a number shown beside this line rather than \
     inside it.",
);

/// Draw in high contrast.
pub const HIGH_CONTRAST: Word = Word::saying(
    "access.high-contrast",
    "use strong colours that are easier to read",
)
.noting(
    "A setting a person turns on. It replaces the machine's quiet colours with a palette of black \
     and white and one strong accent. Do not translate it as \"high contrast mode\" — nothing is \
     switched off, only the colours change.",
);

/// Draw text larger.
pub const LARGER_TEXT: Word = Word::saying("access.larger-text", "make all text larger").noting(
    "A setting a person turns on, which makes text half again as large everywhere. A person who \
     wants to choose a size themselves does that in the appearance settings; this is one switch \
     for somebody who cannot read the ordinary size.",
);

/// Move nothing that need not move.
pub const REDUCED_MOTION: Word =
    Word::saying("access.reduced-motion", "stop things sliding and fading").noting(
        "A setting a person turns on, for somebody who is made unwell by movement on a screen. \
     Nothing is removed — what would have slid appears instead.",
    );

/// A modifier held by pressing it.
pub const STICKY_KEYS: Word = Word::saying(
    "access.sticky-keys",
    "press keys one at a time instead of holding them together",
)
.noting(
    "A setting a person turns on, for somebody who cannot hold two keys at once. After it, a \
     modifier such as shift stays held until the next key is pressed.",
);

/// A key counts once it has been held.
pub const SLOW_KEYS: Word = Word::saying(
    "access.slow-keys",
    "ignore a key unless it is held for a moment",
)
.noting(
    "A setting a person turns on, so that a key brushed by accident does nothing. How long is a \
     number shown beside this line.",
);

/// The same key twice in a moment is one press.
pub const BOUNCE_KEYS: Word = Word::saying(
    "access.bounce-keys",
    "ignore the same key pressed twice quickly",
)
.noting(
    "A setting a person turns on, for a hand that repeats a key without meaning to. How long is a \
     number shown beside this line.",
);

/// Focus, always drawn.
pub const FOCUS_ALWAYS_VISIBLE: Word = Word::saying(
    "access.focus-always-visible",
    "always show where the keyboard is",
)
.noting(
    "A setting a person turns on. \"Where the keyboard is\" is the outline around whatever a key \
     would act on. Ordinarily it appears once somebody uses the keyboard; this draws it always.",
);

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 9] = [
    SCREEN_READER,
    MAGNIFIER,
    HIGH_CONTRAST,
    LARGER_TEXT,
    REDUCED_MOTION,
    STICKY_KEYS,
    SLOW_KEYS,
    BOUNCE_KEYS,
    FOCUS_ALWAYS_VISIBLE,
];

/// Why this crate's own list could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
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
pub fn access_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put every sentence this crate says into a vocabulary somebody else holds.
///
/// # Errors
/// [`WordsError`] where the vocabulary already has one of these keys.
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
    use crate::setting::Setting;

    /// **Every setting has a sentence, and no two share one.**
    #[test]
    fn every_setting_says_what_it_does_in_words_of_its_own() {
        let vocabulary = access_words().unwrap();
        assert_eq!(EVERY_WORD.len(), Setting::ALL.len());
        for setting in Setting::ALL {
            let word = setting.word();
            assert!(
                EVERY_WORD.iter().any(|every| every.key() == word.key()),
                "{setting:?} says something this crate does not declare"
            );
            assert!(vocabulary.phrase(&word.key()).is_some(), "{setting:?}");
        }
        let mut keys: Vec<String> = EVERY_WORD
            .iter()
            .map(|word| word.key().to_string())
            .collect();
        keys.sort();
        keys.dedup();
        assert_eq!(
            keys.len(),
            EVERY_WORD.len(),
            "two settings share a sentence"
        );
    }

    /// **No sentence names a disability**, and none says *mode*: a person is
    /// reading a list of what the machine can do, not a diagnosis.
    #[test]
    fn no_sentence_names_a_person_or_calls_itself_a_mode() {
        for word in EVERY_WORD {
            let said = format!("{} {}", word.says(), word.note().unwrap_or_default());
            for never in [
                "disabled", "impair", "handicap", "blind", "deaf", "sufferer",
            ] {
                assert!(
                    !said.to_lowercase().contains(never),
                    "{}: {never}",
                    word.key()
                );
            }
            assert!(
                !word.says().to_lowercase().contains("mode"),
                "{}",
                word.key()
            );
        }
    }
}
