//! Every sentence this crate can say.

use alo_strings::Vocabulary;

pub use alo_strings::Word;

/// A device is not muted.
pub const NOT_MUTED: Word = Word::saying("sound.not-muted", "On").noting(
    "Shown beside a microphone or a speaker that is working normally. One word, because it sits \
     beside the device's own name in a list.",
);

/// A device is muted: the source is stopped.
pub const MUTED: Word = Word::saying("sound.muted", "Muted — nothing is heard from it").noting(
    "Shown beside a muted microphone or speaker. The second half is literal and must not be \
     softened: alo OS stops the source rather than turning it down, so nothing is carried \
     anywhere that could be recovered. Do not translate it as quietened or silenced-for-now.",
);

/// A device was pinned.
pub const PINNED: Word = Word::saying(
    "sound.pinned",
    "Always use this one when it is here",
)
.noting(
    "A switch beside a device. It means: whenever this device is connected, choose it; when it is \
     not, let the machine choose as it would have. It is not a default and it never causes \
     silence.",
);

/// The device somebody pinned is not here.
pub const PINNED_ONE_IS_AWAY: Word = Word::saying(
    "sound.pinned-one-is-away",
    "The device you always use is not connected, so this machine chose another",
)
.noting(
    "Shown once, where a person would otherwise wonder why their pinned device is not in use. It \
     is not a refusal: sound is working, through something else, and the device's name is shown \
     beside this line.",
);

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 4] = [NOT_MUTED, MUTED, PINNED, PINNED_ONE_IS_AWAY];

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
pub fn sound_words() -> Result<Vocabulary, WordsError> {
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

    /// **Every sentence has a note**, and none says a mute is a quieter volume.
    #[test]
    fn nothing_here_describes_a_mute_as_a_volume() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            let said = word.says().to_lowercase();
            for never in ["volume to zero", "turned down", "quietened"] {
                assert!(!said.contains(never), "{}: {never}", word.named());
            }
        }
        assert!(sound_words().unwrap().phrase(&MUTED.key()).is_some());
    }
}
