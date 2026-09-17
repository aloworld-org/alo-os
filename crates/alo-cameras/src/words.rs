//! Every sentence this crate can say.

use alo_strings::Vocabulary;

pub use alo_strings::Word;

/// The camera is off for everything.
pub const THE_CAMERA_IS_OFF: Word = Word::saying(
    "cameras.camera-is-off",
    "The camera is off. Nothing on this machine can use it, whatever you have allowed it",
)
.noting(
    "Shown wherever an application asked for the camera and a person has the switch off. The \
     second sentence is the promise and must not be softened: it covers applications that were \
     granted the camera, and it is why a person can trust the switch instead of checking.",
);

/// The microphone is off for everything.
pub const THE_MICROPHONE_IS_OFF: Word = Word::saying(
    "cameras.microphone-is-off",
    "The microphone is off. Nothing on this machine can use it, whatever you have allowed it",
)
.noting("The same sentence as the camera's, about the microphone. Keep them parallel.");

/// This machine has no camera.
pub const NO_CAMERA_HERE: Word = Word::saying(
    "cameras.none-here",
    "This machine has no camera",
)
.noting(
    "Shown where a list of cameras would be, on a machine with none. Not an empty list: a person \
     shown one goes looking for a cable.",
);

/// Whether a camera has a shutter or a light is the camera's own business.
pub const WHAT_IT_HAS_OF_ITS_OWN: Word = Word::saying(
    "cameras.what-it-has-of-its-own",
    "This machine cannot tell whether this camera has a shutter or a light of its own",
)
.noting(
    "Shown beside a camera in settings. It is an admission and it is deliberate: a camera's light \
     is wired to the camera, and a machine claiming to know about it would be claiming the one \
     thing a person can check with their own eyes.",
);

/// The camera was let go of, so there is nothing to open.
pub const THE_DEVICE_WAS_LET_GO: Word = Word::saying(
    "cameras.let-go",
    "This machine has let go of the camera. There is nothing for a program to open until you turn \
     it back on",
)
.noting(
    "Shown once when a person turns the camera off on a machine that can release the hardware. It \
     is the difference between a machine that refuses requests and a machine where there is \
     nothing to request.",
);

/// The machine could not let go of the device, and the switch is a door.
pub const ONLY_THE_DOOR: Word = Word::saying(
    "cameras.only-the-door",
    "This machine cannot release this device, so off is kept at the door. A program you run \
     yourself outside the workspace could still reach it",
)
.noting(
    "Shown where the switch is off and the machine could not release the hardware. It is an \
     honest limit rather than a failure, and it must be translated as one: alo OS ships a \
     terminal on purpose, and a person with a shell has a machine.",
);

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 6] = [
    THE_CAMERA_IS_OFF,
    THE_MICROPHONE_IS_OFF,
    NO_CAMERA_HERE,
    WHAT_IT_HAS_OF_ITS_OWN,
    THE_DEVICE_WAS_LET_GO,
    ONLY_THE_DOOR,
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
pub fn camera_words() -> Result<Vocabulary, WordsError> {
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

    /// **Every sentence has a note, and none of them names a device number or
    /// the stack underneath.**
    #[test]
    fn nothing_here_names_what_a_person_does_not_have_to_know() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            let said = word.says().to_lowercase();
            for never in ["/dev/", "v4l2", "pipewire", "wireplumber", "udev"] {
                assert!(!said.contains(never), "{}: {never}", word.named());
            }
        }
        assert!(
            camera_words()
                .unwrap()
                .phrase(&THE_CAMERA_IS_OFF.key())
                .is_some()
        );
    }
}
