//! What the first-start sheets say, in words somebody can translate.
//!
//! Read from the renders of `183:10675` and `186:10737`. Each is the design's
//! own wording; none is invented here, and none is written into the drawing.
//!
//! # Why a machine says this at all
//!
//! `alo_greeting::makes_an_account` has been able to make a machine's first
//! account since 2026-10-10 — refused when the store is not empty, written to
//! the disk before anybody is signed in, on the uid `agentd.toml` names rather
//! than one a surface chose. **Nothing had ever called it.** A machine with no
//! accounts showed one sentence saying *make an account* and offered nowhere
//! to make one, which is what stood between alo OS and a computer somebody
//! could use.

use alo_strings::{Vocabulary, VocabularyError, Word, WordError};

/// The line at the top right of every sheet.
pub const STAGE: Word = Word::saying("shell.first-start.stage", "alo OS").noting(
    "The product's name, at the top right of every first-start sheet. A name, so leave it.",
);

/// The small line above the heading on the name sheet.
pub const NAME_EYEBROW: Word = Word::saying("shell.first-start.name.eyebrow", "MAKE IT YOURS")
    .noting(
        "A short line above the heading, telling a person which part of setting up they are in. \
         The upper case is this phrase's own; use whatever your language does for a quiet label \
         rather than upper-casing a translation that should not be.",
    );

/// The name sheet's heading.
pub const NAME_TITLE: Word =
    Word::saying("shell.first-start.name.title", "What should we call you?").noting(
        "The heading of the sheet where a person gives their name, the first thing alo OS asks \
         anybody. A question, asked warmly.",
    );

/// What the name is for.
pub const NAME_INTRODUCTION: Word = Word::saying(
    "shell.first-start.name.introduction",
    "This name appears when you sign in. You can change it later.",
)
.noting(
    "Under the heading on the name sheet. It says where the name shows and that it is not final.",
);

/// The row a person types their name into.
pub const NAME_CHOICE: Word = Word::saying("shell.first-start.name.choice", "Your name")
    .noting("The label above the field a person types their name into.");

/// What that row says before anything is typed.
pub const NAME_HINT: Word = Word::saying("shell.first-start.name.hint", "Type your name")
    .noting("Inside the empty name field, telling a person what to type.");

/// What making an account here means.
pub const NAME_CONSEQUENCE: Word = Word::saying(
    "shell.first-start.name.consequence",
    "Your account is created on this computer. No online account is required.",
)
.noting(
    "Under the choices on the name sheet. It is a promise, and it is true: the account is kept in \
     a file on this machine and is not sent anywhere.",
);

/// The PIN sheet's small line.
pub const PIN_EYEBROW: Word = Word::saying("shell.first-start.pin.eyebrow", "Optional · PIN")
    .noting(
        "A short line above the heading on the PIN sheet. The sentence case is this phrase's own; \
         it is not a label that should be upper-cased.",
    );

/// The PIN sheet's heading.
pub const PIN_TITLE: Word = Word::saying("shell.first-start.pin.title", "Add a PIN?")
    .noting("The heading of the sheet where a person chooses a PIN. A question, and declinable.");

/// What a PIN is for.
pub const PIN_INTRODUCTION: Word = Word::saying(
    "shell.first-start.pin.introduction",
    "Use a PIN to sign in on this computer. Keep your password for the situations that require it.",
)
.noting("Under the heading on the PIN sheet, saying what a PIN is for and that it does not replace a password.");

/// The row a new PIN is typed into.
pub const PIN_CHOICE: Word = Word::saying("shell.first-start.pin.choice", "New PIN")
    .noting("The label above the field a person types a new PIN into.");

/// What that row says before anything is typed.
pub const PIN_HINT: Word = Word::saying("shell.first-start.pin.hint", "Enter your PIN")
    .noting("Inside the empty new-PIN field. A PIN is a short secret of digits; use whatever your language calls that.");

/// What a PIN means for this machine.
pub const PIN_CONSEQUENCE: Word = Word::saying(
    "shell.first-start.pin.consequence",
    "A PIN signs you in on this computer only.",
)
.noting(
    "Under the choices on the PIN sheet. It says the PIN is this machine's and goes nowhere else.",
);

/// The action that goes on to the next sheet.
pub const CONTINUE: Word = Word::saying("shell.first-start.continue", "Continue")
    .noting("The action that accepts what is on this sheet and goes on to the next.");

/// The action that makes the account.
pub const SAVE: Word = Word::saying("shell.first-start.save", "Save PIN").noting(
    "The action on the PIN sheet that keeps the PIN and makes the account on this machine.",
);

/// The line along the bottom of every sheet.
pub const FOOTER: Word = Word::saying(
    "shell.first-start.footer",
    "Local setup. No online account required.",
)
.noting("A quiet line at the bottom right of every first-start sheet, repeating that nothing here needs an account elsewhere.");

/// Every string these sheets draw.
pub const FIRST_START_WORDS: [Word; 15] = [
    STAGE,
    NAME_EYEBROW,
    NAME_TITLE,
    NAME_INTRODUCTION,
    NAME_CHOICE,
    NAME_HINT,
    NAME_CONSEQUENCE,
    PIN_EYEBROW,
    PIN_TITLE,
    PIN_INTRODUCTION,
    PIN_CHOICE,
    PIN_HINT,
    PIN_CONSEQUENCE,
    CONTINUE,
    SAVE,
];

/// Invalid declaration or collision while registering this wording.
#[derive(Debug, thiserror::Error)]
pub enum FirstStartWordsError {
    /// Invalid source phrase or translator note.
    #[error(transparent)]
    Word(#[from] WordError),
    /// An existing declaration already owns this key.
    #[error(transparent)]
    Vocabulary(#[from] VocabularyError),
}

/// Add these sheets' wording atomically, before any translation is loaded.
///
/// # Errors
/// [`FirstStartWordsError`] for a malformed phrase or a key already taken.
pub fn declare_first_start_words(vocabulary: &mut Vocabulary) -> Result<(), FirstStartWordsError> {
    let mut prepared = vocabulary.clone();
    for word in FIRST_START_WORDS {
        prepared.says(word.phrase()?)?;
    }
    prepared.says(FOOTER.phrase()?)?;
    *vocabulary = prepared;
    Ok(())
}
