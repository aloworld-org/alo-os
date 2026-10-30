//! What the form that adds a provider says, in words somebody can translate.
//!
//! Twelve sentences, and none of them is drawn from a string literal: the labels
//! beside the four fields, the heading above them, the one line that says what
//! a person is about to do, the four reasons a save did not happen, the caution
//! about a service at this machine's address, and the caution about plain http
//! to another computer on the person's own network.
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
///
/// **The four refusals are in here**, which is the whole point of the list: a
/// word declared and left out of it reaches a person as its own key, and this
/// form drew six of those on a booted machine on 2026-10-23 for that fault one
/// layer up. The length is written out so that adding a `Word` above and
/// forgetting this line does not compile.
pub const THE_PROVIDERS_WORDS: [Word; 12] = [
    ADDING_A_PROVIDER,
    WHAT_TO_DO_WITH_IT,
    ITS_NAME,
    ITS_ADDRESS,
    ITS_KEY,
    ITS_MODEL,
    NOT_SAVED_NO_KEYRING,
    NOT_SAVED_KEY_NOT_KEPT,
    NOT_SAVED_SETTINGS_WOULD_NOT_READ,
    NOT_SAVED_NOWHERE_TO_KEEP,
    THIS_MACHINE_CANNOT_SEE_WHERE_IT_GOES,
    OTHERS_ON_THIS_NETWORK_COULD_READ_IT,
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

/// What a person is told when their provider was not saved.
///
/// **Four sentences and not one**, because the four are four different machines
/// and a person does something different about each. *Something went wrong* is
/// the sentence this set exists to avoid.
///
/// They live here rather than in `alo-desktop` because that binary is where the
/// writes are made, not where words belong — its own
/// `what_did_not_happen` is English for a service log and says so. A person
/// reads these.
pub const NOT_SAVED_NO_KEYRING: Word = Word::saying(
    "shell.provider.not-saved.no-keyring",
    "This computer has nowhere to keep a key yet, so nothing was saved. What you typed is still here.",
)
.noting("Shown under the form when the machine has no keyring at all.");

/// The key could not be kept, for a reason the keyring gave.
pub const NOT_SAVED_KEY_NOT_KEPT: Word = Word::saying(
    "shell.provider.not-saved.key-not-kept",
    "The key could not be kept, so nothing else was tried. What you typed is still here.",
)
.noting("Shown when the keyring is there and refused the key.");

/// The settings file is there and would not read.
pub const NOT_SAVED_SETTINGS_WOULD_NOT_READ: Word = Word::saying(
    "shell.provider.not-saved.settings",
    "Your settings file could not be read, so it was left exactly as it was. What you typed is still here.",
)
.noting("Shown when the key was kept and the settings file refused — ADR 0038: a file that will not read is never written over.");

/// This person has nowhere to keep settings.
pub const NOT_SAVED_NOWHERE_TO_KEEP: Word = Word::saying(
    "shell.provider.not-saved.nowhere",
    "This sign-in has no folder to keep settings in, so nothing was saved. What you typed is still here.",
)
.noting("Shown for a session with no home directory.");

/// Said above the fields while the address is this machine's own.
///
/// **A warning and not a refusal** (owner's decision, 2026-10-29; ADR 0021). A
/// service at this machine's address is asked there, so its questions go on no
/// road out and the indicator has nothing to show — and if that service passes
/// them on to the internet, nothing in alo OS can see them go. Saving still
/// works: it is the person's service on the person's machine (law 5).
pub const THIS_MACHINE_CANNOT_SEE_WHERE_IT_GOES: Word = Word::saying(
    "shell.provider.this-machine-cannot-see",
    "alo cannot see where this service sends your questions. If it passes them on to the internet, the network indicator will not show it.",
)
.noting(
    "Shown above the form that adds a provider while its address is this computer's own — a \
     program the person runs on this computer, such as a local model server. alo OS hands the \
     question to that program and cannot see what the program does next. \"The network \
     indicator\" is the part of the screen that shows what alo OS sends over the network. This \
     is a caution, not an error: the person can still save. Keep \"alo\" as written, in lower \
     case.",
);

/// Said above the fields while the address is plain `http://` to another
/// computer on the person's own network.
///
/// **A warning and not a refusal** (owner's decision, 2026-10-30; ADR 0021).
/// Such a provider can be saved, and its questions and its key then cross that
/// network with nothing protecting them. The sentence says so in those terms,
/// and says who could read them, because that is the cost the person is
/// choosing to accept. Saving is still theirs (law 5).
pub const OTHERS_ON_THIS_NETWORK_COULD_READ_IT: Word = Word::saying(
    "shell.provider.unencrypted-on-this-network",
    "This address is not https, so your questions and your key cross your network unencrypted. Anyone else on that network could read them.",
)
.noting(
    "Shown above the form that adds a provider while its address starts with http:// and is \
     another computer on the person's own home or office network, such as 192.168.1.20. \
     \"Unencrypted\" means sent as plain text that is not protected in any way. \"Your key\" is \
     the secret the provider gave them. \"Anyone else on that network\" means any other device or \
     person connected to the same network — a guest's phone, another computer. https and http are \
     not translated. This is a caution, not an error: the person can still save.",
);
