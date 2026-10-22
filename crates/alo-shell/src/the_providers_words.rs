//! What the form that adds a provider says, in words somebody can translate.
//!
//! Six sentences, and none of them is drawn from a string literal: the labels
//! beside the four fields, the heading above them, and the one line that says
//! what a person is about to do.
//!
//! # Why these exist at all
//!
//! Until this form, a person on a fresh alo machine could not make the
//! machine answer them. `settings_answering` says so in its own words —
//! *adding a provider, bringing weights and pairing are flows of their own …
//! and none is reachable from this list* — and the image carries no model
//! (ADR 0095), so the only answer offered was *not at all*.
//!
//! # What is deliberately not said
//!
//! **No sentence tells a person where to get a key.** That would be alo OS
//! recommending a provider, which ADR 0009 refuses: *we ship the mechanism,
//! never a default that decides for them.* A person adding a provider has one
//! in mind already.
//!
//! **And nothing says the key is safe.** It is kept in the machine's keyring
//! (ADR 0022) and the settings file holds only a reference, which is true and
//! is not a reassurance to put on a screen: a sentence promising safety is a
//! sentence somebody has to keep.

use alo_strings::{Vocabulary, VocabularyError, Word, WordError};

/// The heading above the four fields.
pub const ADDING_A_PROVIDER: Word = Word::saying("shell.provider.adding", "Add a provider").noting(
    "The heading over the form where a person adds a service that answers their questions \
         — an AI provider they already have an account with. A short instruction, the way a \
         button is worded.",
);

/// The line under the heading, saying what happens.
pub const WHAT_TO_DO_WITH_IT: Word = Word::saying(
    "shell.provider.what-to-do",
    "Fill these in, then press Enter. Escape leaves it.",
)
.noting(
    "Under the heading of the form that adds a provider. Tells a person the two keys that \
     finish or abandon it. Enter and Escape are the keys on a keyboard; leave their names as \
     they are written on keyboards in your language.",
);

/// The first field.
pub const ITS_NAME: Word = Word::saying("shell.provider.name", "Name").noting(
    "The label beside the field where a person types what to call this provider. It is their \
     own name for it and appears when it answers them. One or two words.",
);

/// The second field.
pub const ITS_ADDRESS: Word = Word::saying("shell.provider.address", "Address").noting(
    "The label beside the field where a person types the web address of the provider's \
     interface — the base address, such as https://api.example.com. One or two words.",
);

/// The third field.
pub const ITS_KEY: Word = Word::saying("shell.provider.key", "Key").noting(
    "The label beside the field where a person pastes the secret key their provider gave them. \
     What they type here is shown as dots. One or two words.",
);

/// The fourth field.
pub const ITS_MODEL: Word = Word::saying("shell.provider.model", "Model").noting(
    "The label beside the field where a person types which model to ask for, exactly as their \
     provider names it. One or two words.",
);

/// Every sentence this form says.
pub const THE_PROVIDERS_WORDS: [Word; 6] = [
    ADDING_A_PROVIDER,
    WHAT_TO_DO_WITH_IT,
    ITS_NAME,
    ITS_ADDRESS,
    ITS_KEY,
    ITS_MODEL,
];

/// Why this form's wording would not declare.
#[derive(Debug, thiserror::Error)]
pub enum ProvidersWordsError {
    /// Invalid source phrase or translator note.
    #[error(transparent)]
    Word(#[from] WordError),
    /// An existing declaration already owns this key.
    #[error(transparent)]
    Vocabulary(#[from] VocabularyError),
}

/// Add this form's wording atomically, before any translation is loaded.
///
/// # Errors
/// [`ProvidersWordsError`] for a malformed phrase or a key already taken.
pub fn declare_the_providers_words(vocabulary: &mut Vocabulary) -> Result<(), ProvidersWordsError> {
    // **Prepared and swapped**, which is `declare_first_start_words`' shape:
    // a declaration that failed halfway would leave a vocabulary holding some
    // of these and not others, and the sentence missing would be found on a
    // screen rather than here.
    let mut prepared = vocabulary.clone();
    for word in THE_PROVIDERS_WORDS {
        prepared.says(word.phrase()?)?;
    }
    *vocabulary = prepared;
    Ok(())
}
