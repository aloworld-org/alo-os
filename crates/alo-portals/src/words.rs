//! Every string this crate can say, and the English beside each one.
//!
//! Three groups.
//!
//! **One sentence for each portal** — what it lets an application do, in the
//! words a person reads beside it wherever this machine lists the portals it
//! answers. Each says what the application may do and, where a reader could
//! reasonably worry about more, what it may not.
//!
//! **One refusal of a request this crate judged**: [`NOTHING_GRANTED`], for an
//! application nobody has granted anything. The other judged refusals are
//! `alo-capability`'s own ([`alo_capability::NotAllowed`]), because that is
//! the crate that decides, and a second sentence about the same refusal written
//! here could come to disagree with it.
//!
//! **The refusals of something that was never a request** — an identifier that
//! is not one, a path where there should be none, a relative path. They are
//! read in the record by whoever is looking at what an application tried, and
//! each names what a well-formed request would have carried.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What each portal lets an application do — [`crate::Portal`].
// ---------------------------------------------------------------------------

/// The file chooser, and the documents it hands over.
pub const FILE_CHOOSER: Word = Word::saying(
    "portals.portal.file-chooser",
    "Lets an application open or save a file you choose, and reach that file and nothing beside it",
)
.noting(
    "A portal is how an application on alo OS asks for something outside itself. The file is \
     the one the person picked; the application is not given the folder it is in.",
);

/// Open-with and default applications.
pub const OPEN_WITH: Word = Word::saying(
    "portals.portal.open-with",
    "Lets an application open a file you have granted it in the application that opens that kind \
     of file",
);

/// Notifications.
pub const NOTIFICATIONS: Word = Word::saying(
    "portals.portal.notifications",
    "Lets an application send you notifications",
);

/// Printing.
pub const PRINT: Word = Word::saying(
    "portals.portal.print",
    "Lets an application print a document you have granted it",
);

/// One picture of the screen.
pub const SCREENSHOT: Word = Word::saying(
    "portals.portal.screenshot",
    "Lets an application take one picture of the screen",
)
.noting("Once — recording or sharing the screen is a different portal and a different grant.");

/// The screen, continuously.
pub const SCREEN_CAPTURE: Word = Word::saying(
    "portals.portal.screen-capture",
    "Lets an application record or share the screen for as long as the grant lasts",
);

/// The camera.
pub const CAMERA: Word = Word::saying(
    "portals.portal.camera",
    "Lets an application use the camera",
);

/// The microphone.
pub const MICROPHONE: Word = Word::saying(
    "portals.portal.microphone",
    "Lets an application use the microphone",
);

/// The clipboard.
pub const CLIPBOARD: Word = Word::saying(
    "portals.portal.clipboard",
    "Lets an application read what you copied, and copy something for you",
);

/// The trash.
pub const TRASH: Word = Word::saying(
    "portals.portal.trash",
    "Lets an application move a file you have granted it to the trash",
)
.noting("To the trash, from which it can be taken back — not deleted.");

/// The desktop background.
pub const WALLPAPER: Word = Word::saying(
    "portals.portal.wallpaper",
    "Lets an application set the desktop background",
);

/// Appearance settings.
pub const SETTINGS: Word = Word::saying(
    "portals.portal.settings",
    "Lets an application read your appearance settings, such as light or dark, so it can match \
     them",
)
.noting("Only appearance: not any other setting on the machine, and never changing one.");

/// Keeping the machine awake.
pub const INHIBIT: Word = Word::saying(
    "portals.portal.inhibit",
    "Lets an application keep the machine awake, for example during a presentation",
);

/// The network monitor.
pub const NETWORK_MONITOR: Word = Word::saying(
    "portals.portal.network-monitor",
    "Lets an application know whether the machine is connected to a network, and never what is \
     sent over it",
);

/// The power-profile monitor.
pub const POWER_PROFILE_MONITOR: Word = Word::saying(
    "portals.portal.power-profile-monitor",
    "Lets an application know the power profile, so it can use less power while you are saving it",
);

// ---------------------------------------------------------------------------
// A request judged and refused — [`crate::Refused`].
// ---------------------------------------------------------------------------

/// An application that holds no grant at all.
pub const NOTHING_GRANTED: Word = Word::saying(
    "portals.refused.nothing-granted",
    "{application} has not been granted anything on this machine, so what it asked for was \
     refused without asking you",
)
.noting(
    "{application} is the identifier of an application, like org.gnome.Cheese, and is never \
     translated; it is an application, not a person. \"Without asking you\" is literal: no \
     question was put on the screen, because an application nobody has granted anything is \
     refused before a question is.",
);

// ---------------------------------------------------------------------------
// Something that was never a request — [`crate::NotARequest`].
// ---------------------------------------------------------------------------

/// No application was named.
pub const NO_APPLICATION: Word = Word::saying(
    "portals.not-a-request.no-application",
    "A request has to say which application is asking, and this one said none",
);

/// The application's identifier is not one.
pub const NOT_AN_IDENTIFIER: Word = Word::saying(
    "portals.not-a-request.not-an-identifier",
    "A request names an application by an identifier like org.gnome.Cheese, with no spaces in it",
)
.noting("org.gnome.Cheese is an example identifier and is not translated.");

/// A portal over a path, asked without one.
pub const NEEDS_A_PATH: Word = Word::saying(
    "portals.not-a-request.needs-a-path",
    "This portal is about a file, so a request to it has to say which file",
);

/// A portal that is not over a path, asked with one.
pub const NOT_OVER_A_PATH: Word = Word::saying(
    "portals.not-a-request.not-over-a-path",
    "This portal is not about a file, so a request to it names none",
);

/// A path that is not a full path.
pub const NOT_A_FULL_PATH: Word = Word::saying(
    "portals.not-a-request.not-a-full-path",
    "A request names a file by its full path, so it means the same thing wherever it is read",
);

/// A path with `..` in it.
pub const COULD_LEAD_ELSEWHERE: Word = Word::saying(
    "portals.not-a-request.could-lead-elsewhere",
    "A path with .. in it can lead somewhere else, so a request names a file by its own path",
)
.noting("\"..\" is how a path says \"the folder above\" and is never translated.");

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 22] = [
    FILE_CHOOSER,
    OPEN_WITH,
    NOTIFICATIONS,
    PRINT,
    SCREENSHOT,
    SCREEN_CAPTURE,
    CAMERA,
    MICROPHONE,
    CLIPBOARD,
    TRASH,
    WALLPAPER,
    SETTINGS,
    INHIBIT,
    NETWORK_MONITOR,
    POWER_PROFILE_MONITOR,
    NOTHING_GRANTED,
    NO_APPLICATION,
    NOT_AN_IDENTIFIER,
    NEEDS_A_PATH,
    NOT_OVER_A_PATH,
    NOT_A_FULL_PATH,
    COULD_LEAD_ELSEWHERE,
];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// say so. It keeps its English and its `Display`: whoever reads it is fixing
/// the list.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn portal_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// `alo-saying` is the one place that calls all of these.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced.
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
    use std::collections::BTreeSet;

    /// Every key is a key, in this crate's area, and named once.
    #[test]
    fn every_key_is_a_key_in_this_crates_area_once() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
            assert_eq!(word.key().area(), "portals", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// The list declares, and a vocabulary that has it already keeps its own.
    #[test]
    fn the_whole_list_declares_once() {
        let mut vocabulary = portal_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Nothing here calls an application an agent.** An application asking
    /// through a portal is not an AI assistant, and the sentences it is judged
    /// by must not read as though it were.
    #[test]
    fn nothing_here_calls_an_application_an_agent() {
        for word in EVERY_WORD {
            assert!(
                !word.says().to_lowercase().contains("agent"),
                "{}",
                word.named()
            );
        }
    }

    /// Each is a sentence a person reads on its own, so it begins like one.
    #[test]
    fn every_string_begins_a_sentence() {
        for word in EVERY_WORD {
            let first = word.says().chars().next().unwrap();
            assert!(
                first.is_uppercase() || word.named() == NOTHING_GRANTED.named(),
                "{} does not begin a sentence",
                word.named()
            );
        }
    }
}
