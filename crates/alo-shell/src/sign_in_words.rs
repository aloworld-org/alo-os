//! What the sign-in screen says, in words somebody can translate.
//!
//! Every label the design draws, declared here rather than written into
//! `crate::sign_in_raster` as English. The screen drew none of these until
//! 2026-10-12: the entry had no placeholder, the row below it was empty, and
//! *Accessibility*, *Switch person* and the power control were laid out and
//! hit-tested while nothing drew their labels — controls a person could press
//! and could not see.
//!
//! # The greeting is a sentence, not a name with a comma in front of it
//!
//! `alo, {name}` reads as a greeting in English and in nothing else
//! necessarily: a language may put the name first, may need a different
//! particle, or may not greet by given name at all. So it is one string with
//! one gap, and a translator moves the gap.
//!
//! # `disan` is sample content
//!
//! The owner's direction is explicit that the name in the design file is an
//! example. The gap takes the person's own display name, with their own
//! casing, from their account.

use alo_strings::{Vocabulary, VocabularyError, Word, WordError};

/// The greeting above the entry, by the person's own name.
pub const GREETING: Word = Word::saying("shell.sign-in.greeting", "alo, {name}").noting(
    "The greeting on the screen a machine boots to, above the place a person types. {name} is \
     their own display name, in their own casing — move the gap where your language needs the \
     name, and greet the way your language greets.",
);

/// What the entry says before anything is typed into it.
pub const ENTER_YOUR_PIN: Word = Word::saying("shell.sign-in.enter-your-pin", "Enter your PIN")
    .noting(
        "Inside the empty field on the sign-in screen, telling a person what to type. A PIN is a \
         short secret of digits; use whatever your language calls that rather than translating \
         the three letters.",
    );

/// The line that opens the other ways this person can sign in.
pub const OTHER_WAYS: Word = Word::saying("shell.sign-in.other-ways", "Other ways to sign in")
    .noting(
        "A line under the entry on the sign-in screen. Pressing it offers the other ways this \
         person has set up — a password, or a fingerprint. It never signs anybody in by itself.",
    );

/// The control that opens what helps a person see, hear and type.
pub const ACCESSIBILITY: Word = Word::saying("shell.sign-in.accessibility", "Accessibility")
    .noting(
        "A control at the bottom left of the sign-in screen, reachable before anybody has signed \
         in, opening the settings for seeing, hearing and typing.",
    );

/// The control that goes back to choosing who is signing in.
pub const SWITCH_PERSON: Word = Word::saying("shell.sign-in.switch-person", "Switch person")
    .noting(
        "A control at the bottom right of the sign-in screen. It clears whatever was typed and \
         goes back to choosing which person is signing in.",
    );

/// **The one sentence a new machine says while its screen reader is off**
/// (ADR 0155; the owner, 2026-11-14: *"Off, with one spoken hint"*).
pub const HEAR_THE_SCREEN: Word = Word::saying(
    "shell.sign-in.hear-the-screen",
    "Press Control, Alt and S to hear the screen read aloud.",
)
.noting(
    "Spoken aloud, never shown: once, on the first screen of a new computer, after a pause in \
     which nobody pressed anything, so that somebody who cannot see the screen knows how to \
     turn the screen reader on. Name the three keys the way a person says them aloud in your \
     language, in this order — Control, Alt, then the letter S — and keep it to one sentence.",
);

/// What the screen reader says as it is turned off before sign-in.
pub const SCREEN_READER_OFF: Word =
    Word::saying("shell.sign-in.screen-reader-off", "Screen reader off.").noting(
        "Spoken aloud just before the screen reader stops, when somebody turns it off on the \
     first-start screens or the sign-in screen, so they hear that it worked. Short.",
    );

/// What the screen says when the reader is on and there is no sound to play
/// it through.
pub const NO_SOUND_FOR_THE_READER: Word = Word::saying(
    "shell.sign-in.no-sound-for-the-reader",
    "The screen reader is on, and this computer has nothing to play it on.",
)
.noting(
    "Shown on the first-start and sign-in screens, above Accessibility, when somebody turned \
     the screen reader on and the computer has no speakers or headphones it can use. Says what \
     happened, not what to do.",
);

/// Every string this screen draws or says.
pub const SIGN_IN_WORDS: [Word; 8] = [
    GREETING,
    ENTER_YOUR_PIN,
    OTHER_WAYS,
    ACCESSIBILITY,
    SWITCH_PERSON,
    HEAR_THE_SCREEN,
    SCREEN_READER_OFF,
    NO_SOUND_FOR_THE_READER,
];

/// Invalid declaration or collision while registering this screen's wording.
#[derive(Debug, thiserror::Error)]
pub enum SignInWordsError {
    /// Invalid source phrase or translator note.
    #[error(transparent)]
    Word(#[from] WordError),
    /// An existing declaration already owns this key.
    #[error(transparent)]
    Vocabulary(#[from] VocabularyError),
}

/// Add this screen's wording atomically, before any translation is loaded.
///
/// A collision leaves every existing declaration untouched and adds nothing,
/// which is `crate::window_control_reader_words`' rule and the reason it is
/// safe to call on a vocabulary somebody else has already filled.
///
/// # Errors
/// [`SignInWordsError`] for a malformed phrase or a key already taken.
pub fn declare_sign_in_words(vocabulary: &mut Vocabulary) -> Result<(), SignInWordsError> {
    let mut prepared = vocabulary.clone();
    for word in SIGN_IN_WORDS {
        prepared.says(word.phrase()?)?;
    }
    *vocabulary = prepared;
    Ok(())
}

#[cfg(test)]
#[path = "sign_in_words_tests.rs"]
mod tests;
