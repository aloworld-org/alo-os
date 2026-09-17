//! The sentences a person reads about a fine-tune, and about revoking one's
//! grant afterwards.

use alo_strings::Vocabulary;

pub use alo_strings::Word;

/// What revoking a folder grant does, and does not do, after a fine-tune
/// ([ADR 0047](../../../docs/decisions/0047-a-revoked-grant-stops-the-next-fine-tune-and-does-not-unlearn-the-last.md)).
pub const REVOKING_AFTER_A_FINE_TUNE: Word = Word::saying(
    "adapting.revoking-does-not-unlearn",
    "This stops new training. A model that has already learned from these documents still has \
     what it learned — delete the adapted model to be rid of it.",
)
.noting(
    "Shown where a person revokes a grant over a folder their model was trained on, at the moment \
     they revoke it. Every word of this is load-bearing and none of it may be softened. Do NOT \
     translate it as the model forgetting, as unlearning, or as the documents being removed from \
     the model: none of those is true, and a person who believed it might send that model to \
     somebody. \"Adapted model\" is the model that was trained on their documents; deleting it \
     leaves the machine answering with the model it had before.",
);

/// The action offered in the same breath.
pub const DELETE_THE_ADAPTED_MODEL: Word = Word::saying(
    "adapting.delete-the-adapted-model",
    "delete the adapted model",
)
.noting(
    "A button, shown beside the sentence above. It removes the weights the fine-tune produced; \
     the machine then answers with the model it had before. It does not touch the person's \
     documents.",
);

/// What a person is shown before anything trains.
pub const WHAT_IT_WILL_LEARN_FROM: Word = Word::saying(
    "adapting.what-it-will-learn-from",
    "what your model will learn from",
)
.noting(
    "The heading above the list of files a fine-tune will read, shown before it starts. The \
     number of files is shown beside this line rather than inside it.",
);

/// A file that was found and will not be trained on.
pub const NOT_TRAINED_ON: Word = Word::saying("adapting.not-trained-on", "found and not used")
    .noting(
        "The heading above the files a fine-tune will not learn from. Each is listed with its own \
     reason; none is left out silently, because a person would otherwise believe their model had \
     read something it never saw.",
    );

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 4] = [
    REVOKING_AFTER_A_FINE_TUNE,
    DELETE_THE_ADAPTED_MODEL,
    WHAT_IT_WILL_LEARN_FROM,
    NOT_TRAINED_ON,
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
pub fn adapting_words() -> Result<Vocabulary, WordsError> {
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
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **No sentence in this crate implies that a model forgets** (ADR 0047,
    /// decision 4). A person who believed it might send that model to somebody.
    #[test]
    fn nothing_here_says_a_model_forgets() {
        // The sentence a person reads, and not the note: the note says the word
        // `unlearn` on purpose, to tell a translator not to use it.
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for never in [
                "will forget",
                "forgets",
                "unlearn",
                "removes your documents from",
                "no longer has",
            ] {
                assert!(!said.contains(never), "{}: says `{never}`", word.key());
            }
        }
    }

    /// **And the note warns a translator off the same words**, because a
    /// sentence that is true in English and softened in Latvian is a person in
    /// Latvia believing something false (ADR 0047, decision 4).
    #[test]
    fn the_note_tells_a_translator_not_to_soften_it() {
        let note = REVOKING_AFTER_A_FINE_TUNE
            .note()
            .expect("the sentence that must not be softened carries a note")
            .to_lowercase();
        for warned in ["forget", "unlearn", "removed from the model"] {
            assert!(
                note.contains(warned),
                "the note does not warn about `{warned}`"
            );
        }
    }

    /// **The revocation sentence says all three things**: what stops, what does
    /// not, and the one thing a person can do.
    #[test]
    fn the_revocation_sentence_says_what_stops_what_does_not_and_what_can_be_done() {
        let said = REVOKING_AFTER_A_FINE_TUNE.says();
        assert!(said.contains("stops new training"));
        assert!(said.contains("still has what it learned"));
        assert!(said.contains("delete the adapted model"));
        assert!(
            adapting_words()
                .unwrap()
                .phrase(&DELETE_THE_ADAPTED_MODEL.key())
                .is_some()
        );
    }
}
