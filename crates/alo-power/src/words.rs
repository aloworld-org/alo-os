//! Every sentence this crate can say.
//!
//! None of them names a daemon, a kernel file or a governor. A person with a
//! battery has a battery.

use alo_strings::Vocabulary;

pub use alo_strings::Word;

/// The battery is getting low.
pub const GETTING_LOW: Word = Word::saying("power.getting-low", "The battery is getting low")
    .noting(
        "Said once, when the battery first reaches a fifth. Once: a machine that repeats this has \
     taught the person to dismiss it, and then the one that mattered arrives in a queue of ones \
     that did not.",
    );

/// The battery is nearly gone.
pub const NEARLY_GONE: Word = Word::saying(
    "power.nearly-gone",
    "The battery is nearly gone. Save what you are doing",
)
.noting(
    "Said once, near the end. The second sentence is an instruction and should stay one: this is \
     the last thing this machine will say about it.",
);

/// How long is left, where the machine will say.
pub const HOW_LONG_IS_LEFT: Word = Word::saying(
    "power.how-long-is-left",
    "About this long at what this machine is doing now",
)
.noting(
    "Shown beside a time. The qualification is the point and must survive translation: the time \
     is an extrapolation from the last few minutes, and opening something heavy changes it.",
);

/// This machine will not guess how long is left.
pub const HOW_LONG_IS_NOT_SAID: Word = Word::saying(
    "power.how-long-is-not-said",
    "This machine will not guess how long is left while what it is doing keeps changing",
)
.noting(
    "Shown in place of a time. It is an admission on purpose: everyone has watched two hours \
     become thirty minutes, and a machine that says nothing is more use than one that says \
     something it does not mean.",
);

/// The saver profile.
pub const PROFILE_SAVER: Word = Word::saying("power.profile.saver", "Save power")
    .noting("One of three profiles, on a switch. The machine runs quieter, cooler and slower.");

/// The balanced profile.
pub const PROFILE_BALANCED: Word = Word::saying("power.profile.balanced", "Balanced")
    .noting("One of three profiles, and the one a machine is in when nobody has chosen.");

/// The performance profile.
pub const PROFILE_PERFORMANCE: Word =
    Word::saying("power.profile.performance", "Everything it has").noting(
        "One of three profiles: the machine runs as fast as it can, for as long as the battery \
         lasts. Shown only on machines that have it.",
    );

/// A charge limit, on machines that have one.
pub const STOPS_CHARGING_AT: Word = Word::saying(
    "power.stops-charging-at",
    "Stop charging at this much, which makes a battery last years longer",
)
.noting(
    "Beside a charge limit, on machines whose hardware has one. On machines without, nothing is \
     shown at all — never this sentence greyed out.",
);

/// The model is what is draining it.
pub const THE_MODEL_IS_WHY: Word = Word::saying(
    "power.the-model-is-why",
    "Your model is what is using this machine",
)
.noting(
    "Shown beside the model's name when the battery is going and the model is the reason. Your \
     model, not the model: it is the person's own, running on their machine, which is the whole \
     product.",
);

/// Something else is what is draining it.
pub const SOMETHING_IS_WHY: Word = Word::saying(
    "power.something-is-why",
    "This is what is using this machine",
)
.noting("Shown beside a program's name. The name is shown separately and is never translated.");

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 10] = [
    GETTING_LOW,
    NEARLY_GONE,
    HOW_LONG_IS_LEFT,
    HOW_LONG_IS_NOT_SAID,
    PROFILE_SAVER,
    PROFILE_BALANCED,
    PROFILE_PERFORMANCE,
    STOPS_CHARGING_AT,
    THE_MODEL_IS_WHY,
    SOMETHING_IS_WHY,
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
pub fn power_words() -> Result<Vocabulary, WordsError> {
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

    /// **Every sentence has a note, and none names what runs underneath.**
    #[test]
    fn nothing_here_names_a_daemon_or_a_kernel_file() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            let said = word.says().to_lowercase();
            for never in ["governor", "daemon", "/sys", "acpi", "tlp"] {
                assert!(!said.contains(never), "{}: {never}", word.named());
            }
        }
        assert!(power_words().unwrap().phrase(&NEARLY_GONE.key()).is_some());
    }
}
