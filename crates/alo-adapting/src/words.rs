//! The sentences a person reads about a fine-tune, and about revoking one's
//! grant afterwards.

use alo_strings::Vocabulary;

pub use alo_strings::Word;

/// What deleting one folder's adapter does
/// ([ADR 0048](../../../docs/decisions/0048-an-adapter-is-the-learning-and-the-base-weights-are-never-touched.md)).
pub const DELETING_THIS_ADAPTER: Word = Word::saying(
    "adapting.deleting-this-adapter",
    "Delete this adapter and your model no longer has what these documents taught it. \
     Everything else it learned stays.",
)
.noting(
    "Shown where a person deletes what one folder taught their model, and beside a revocation of \
     that folder's grant. It is literally true: what was learned from these documents is a file \
     of its own, applied when the model answers, and deleting it leaves the model as it was \
     without it. \"Adapter\" is that file — if your language has no short word for it, a phrase \
     like \"what it learned from these documents\" is better than a borrowed technical term. Do \
     not translate this as the model forgetting: nothing forgets, a file is removed.",
);

/// What revoking the grant over a trained folder does.
pub const REVOKING_AFTER_A_FINE_TUNE: Word = Word::saying(
    "adapting.revoking-a-trained-folder",
    "This stops new training on these documents. What they already taught your model is kept in \
     an adapter you can delete.",
)
.noting(
    "Shown where a person revokes a grant over a folder their model was trained on. Two facts and \
     no apology: training stops, and what was already learned is in a file they control. The \
     button that deletes it is shown beside this line. Do not soften it into the model \
     forgetting, and do not imply that revoking alone removed what was learned — it did not, and \
     a person who believed that might hand the model to somebody.",
);

/// The action offered in the same breath.
pub const DELETE_THE_ADAPTER: Word = Word::saying(
    "adapting.delete-the-adapter",
    "delete what these documents taught",
)
.noting(
    "A button, shown beside both sentences above. It deletes one adapter: what one granted folder \
     taught. The model itself is untouched — it is byte-for-byte what it shipped as — and every \
     other folder's adapter stays.",
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
pub const EVERY_WORD: [Word; 5] = [
    DELETING_THIS_ADAPTER,
    REVOKING_AFTER_A_FINE_TUNE,
    DELETE_THE_ADAPTER,
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
            for never in ["will forget", "forgets", "unlearn", "cannot be undone"] {
                assert!(!said.contains(never), "{}: says `{never}`", word.key());
            }
        }
    }

    /// **And the notes warn a translator off the same words**, because a
    /// sentence that is true in English and softened in Latvian is a person in
    /// Latvia believing something false (ADR 0048).
    #[test]
    fn the_notes_tell_a_translator_not_to_soften_them() {
        for word in [DELETING_THIS_ADAPTER, REVOKING_AFTER_A_FINE_TUNE] {
            let note = word
                .note()
                .expect("a sentence about what was learned carries a note")
                .to_lowercase();
            assert!(
                note.contains("forget"),
                "{}: the note does not warn against saying the model forgets",
                word.key()
            );
        }
    }

    /// **What a person is told is what they can do** (ADR 0048, decision 6):
    /// deleting one adapter takes back one folder, and leaves the rest.
    #[test]
    fn the_sentences_say_what_a_person_can_do_and_what_stays() {
        let deleting = DELETING_THIS_ADAPTER.says();
        assert!(deleting.contains("no longer has what these documents taught"));
        assert!(
            deleting.contains("Everything else it learned stays"),
            "the sentence does not say that one folder is all that goes"
        );

        let revoking = REVOKING_AFTER_A_FINE_TUNE.says();
        assert!(revoking.contains("stops new training"));
        assert!(
            revoking.contains("you can delete"),
            "a person is told what stops without being told what they can do"
        );

        assert!(
            adapting_words()
                .unwrap()
                .phrase(&DELETE_THE_ADAPTER.key())
                .is_some(),
            "the action is not in this crate's vocabulary"
        );
    }
}
