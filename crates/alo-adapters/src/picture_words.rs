//! What a person is told about **alo taking one picture of an application's
//! window** (`accessible.picture_window`, the screen road's picture —
//! [ADR 0147](../../../docs/decisions/0147-the-screen-road-and-who-reads-the-screen.md)
//! §1), in their language.
//!
//! In every sentence `{application}` is the application's identifier, which is
//! not translated.

use alo_strings::Word;

/// What the verb is for.
pub const PICTURE_PURPOSE: Word = Word::saying(
    "adapters.fallback.picture-window.purpose",
    "take one picture of the window of an application that describes nothing alo can read, such \
     as a game, and have it read in words by whoever the person chose to read pictures",
)
.noting(
    "What one thing the agent can do is for, shown where its list of abilities is. \"Describes \
     nothing alo can read\" means the application tells the computer nothing about the words and \
     buttons in its window. alo is the assistant and is never written with a capital letter.",
);

/// What `application` is, for a picture.
pub const PICTURE_APPLICATION: Word = Word::saying(
    "adapters.fallback.picture-window.application",
    "the application whose window is pictured",
)
.noting("Shown beside the application the agent names when it asks to take a picture.");

/// What a person approves.
pub const PICTURE_SENTENCE: Word = Word::saying(
    "adapters.fallback.picture-window.sentence",
    "take a picture of the {application} window and have it read",
)
.noting(
    "The sentence a person approves before alo takes one picture of an application's window. \
     {application} is the application's identifier and is not translated. Who reads the picture \
     is the person's own choice in Settings.",
);

/// What taking a picture changes, on the approval card.
pub const PICTURE_CHANGES: Word = Word::saying(
    "adapters.fallback.picture-window.changes",
    "Takes one picture of the application's window that has the keyboard, and has it read by \
     whoever you chose under “Who reads the pictures” in Settings → What alo can use. If that is \
     a service, the picture is sent to it.",
)
.noting(
    "Shown on the card a person answers before alo takes a picture, beside a tick. \"Who reads \
     the pictures\" and \"What alo can use\" are names in Settings: use exactly their \
     translations there. Held to it by a test.",
);

/// What taking a picture leaves alone.
pub const PICTURE_LEAVES: Word = Word::saying(
    "adapters.fallback.picture-window.leaves",
    "No other window is in the picture, nothing is pressed or typed, the picture is not kept \
     once it is read, and a window with a password field is never pictured.",
)
.noting("Shown beside the sentence about what taking a picture changes. Held to it by a test.");

/// Pictures are not switched on for this application.
pub const NOT_SWITCHED_ON: Word = Word::saying(
    "adapters.fallback.picture-window.not-switched-on",
    "alo cannot see inside {application}: it describes nothing alo can read, and pictures are off \
     for it, so no picture was taken. To let alo use pictures of its window, switch on “Pictures \
     and keys” for {application} in Settings → What alo can use",
)
.noting(
    "Said when the person has not let alo use pictures of this application's window. \"Pictures \
     and keys\" and \"What alo can use\" are names in Settings: use exactly their translations \
     there.",
);

/// Nobody was chosen to read pictures.
pub const NO_ONE_READS: Word = Word::saying(
    "adapters.fallback.picture-window.no-one-reads",
    "alo cannot see inside {application}: nobody has been chosen to read pictures of its window, \
     so no picture was taken. To choose, open “Who reads the pictures” in Settings → What alo can \
     use",
)
.noting(
    "Said when the person has chosen \"No one\" under \"Who reads the pictures\", which is how a \
     computer starts. Use exactly the translations of the Settings names.",
);

/// It describes itself, so it is read rather than pictured.
pub const IT_DESCRIBES_ITSELF: Word = Word::saying(
    "adapters.fallback.picture-window.describes-itself",
    "{application} describes what its window shows, so alo reads its words instead and took no \
     picture",
)
.noting(
    "Said when the agent asked for a picture of a window alo can read in words. A picture is \
     only for windows that describe nothing.",
);

/// A password field in one of its windows.
pub const A_PASSWORD_FIELD: Word = Word::saying(
    "adapters.fallback.picture-window.password-field",
    "A window of {application} has a password field, and alo never takes a picture of one, so no \
     picture was taken",
)
.noting("Said when a window of the application has a password field.");

/// Another window has the keyboard.
pub const NOT_IN_FRONT: Word = Word::saying(
    "adapters.fallback.picture-window.not-in-front",
    "No window of {application} has the keyboard, so no picture was taken. alo takes a picture \
     only of the window in front: bring it forward first",
)
.noting("Said when another application's window has the keyboard, or none has.");

/// The person's own window, at the default.
pub const NOT_ALOS_COPY: Word = Word::saying(
    "adapters.fallback.picture-window.not-alos-copy",
    "That {application} window is yours, not alo's own copy, so no picture was taken. To let alo \
     work in your own windows, choose “alo may work in your own window” in Settings → What alo \
     can use",
)
.noting(
    "Said when the window in front is the person's own and alo works only in its own copy. \
     \"What alo can use\" is the name of the Settings page: use exactly its translation there.",
);

/// This machine could not take the picture.
pub const NO_PICTURE: Word = Word::saying(
    "adapters.fallback.picture-window.no-picture",
    "This machine could not take a picture of {application} just now, so none was taken",
)
.noting("Said when the desktop could not draw the window, or could not be reached.");

/// Asked for from another machine.
pub const ACROSS_A_PAIRING: Word = Word::saying(
    "adapters.fallback.picture-window.across-a-pairing",
    "alo never takes a picture of a window on this computer for another computer, so no picture of {application} was taken",
)
.noting(
    "Said when another computer paired with this one asked alo to take a picture of a window here. Pictures of windows are only ever taken for the person at this computer.",
);

/// Every word taking a picture declares.
pub const EVERY_WORD: [Word; 13] = [
    PICTURE_PURPOSE,
    PICTURE_APPLICATION,
    PICTURE_SENTENCE,
    PICTURE_CHANGES,
    PICTURE_LEAVES,
    NOT_SWITCHED_ON,
    NO_ONE_READS,
    IT_DESCRIBES_ITSELF,
    A_PASSWORD_FIELD,
    NOT_IN_FRONT,
    NOT_ALOS_COPY,
    NO_PICTURE,
    ACROSS_A_PAIRING,
];
