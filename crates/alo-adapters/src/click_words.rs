//! What a person is told about **alo clicking at a point of an application's
//! window** (`accessible.click_at`, ADR 0147 as amended 2026-11-17: *"Allow
//! clicking by position"*), in their language.
//!
//! In every sentence `{application}` is the application's identifier, which is
//! not translated, and `{across}` and `{down}` are whole numbers from 0 to 100.

use alo_strings::Word;

/// What the verb is for.
pub const CLICK_PURPOSE: Word = Word::saying(
    "adapters.fallback.click-at.purpose",
    "click once at one point of the window of an application that describes nothing alo can \
     read, such as a game, at a place a picture of that window showed: a point given as how far \
     across and how far down the window it is, in per cent",
)
.noting(
    "What one thing the agent can do is for, shown where its list of abilities is. alo is the \
     assistant and is never written with a capital letter.",
);

/// What `application` is, for a click.
pub const CLICK_APPLICATION: Word = Word::saying(
    "adapters.fallback.click-at.application",
    "the application whose window is clicked",
)
.noting("Shown beside the application the agent names when it asks to click.");

/// What `across` is.
pub const CLICK_ACROSS: Word = Word::saying(
    "adapters.fallback.click-at.across",
    "how far across the window, from its left edge, in per cent: 0 to 100",
)
.noting("Shown beside the number the agent chooses for how far across the window to click.");

/// What `down` is.
pub const CLICK_DOWN: Word = Word::saying(
    "adapters.fallback.click-at.down",
    "how far down the window, from its top edge, in per cent: 0 to 100",
)
.noting("Shown beside the number the agent chooses for how far down the window to click.");

/// What a person approves.
pub const CLICK_SENTENCE: Word = Word::saying(
    "adapters.fallback.click-at.sentence",
    "click {across}% across and {down}% down the {application} window",
)
.noting(
    "The sentence a person approves before alo clicks once in an application's window. {across} \
     and {down} are whole numbers from 0 to 100, and {application} is the application's \
     identifier, which is not translated.",
);

/// What clicking changes, on the approval card.
pub const CLICK_CHANGES: Word = Word::saying(
    "adapters.fallback.click-at.changes",
    "Clicks once at that point of the application's window that has the keyboard, as a person \
     clicking there would, then takes a picture of the window to see what changed. alo cannot \
     know for certain what it pressed.",
)
.noting(
    "Shown on the card a person answers before alo clicks by position, beside a tick. The last \
     sentence is a plain warning, not an apology. Held to it by a test.",
);

/// What clicking leaves alone.
pub const CLICK_LEAVES: Word = Word::saying(
    "adapters.fallback.click-at.leaves",
    "No other window is clicked, nothing is dragged or held down, and nothing is clicked in a \
     terminal or in a window with a password field.",
)
.noting("Shown beside the sentence about what clicking changes. Held to it by a test.");

/// Nothing visibly changed after the click.
pub const NOT_KNOWN: Word = Word::saying(
    "adapters.fallback.click-at.not-known",
    "alo clicked in {application}, and nothing in its window visibly changed, so what the click \
     did is not known",
)
.noting(
    "Kept in the record, and said to alo, when a picture of the window taken after the click is \
     the same as one taken before it.",
);

/// Pictures are not switched on for this application.
pub const NOT_SWITCHED_ON: Word = Word::saying(
    "adapters.fallback.click-at.not-switched-on",
    "alo cannot click inside {application}: pictures and keys are off for it, so nothing was \
     clicked. To let alo use pictures of its window and click in it, switch on “Pictures and \
     keys” for {application} in Settings → What alo can use",
)
.noting(
    "Said when the person has not let alo use pictures and keys in this application. \"Pictures \
     and keys\" and \"What alo can use\" are names in Settings: use exactly their translations \
     there.",
);

/// Nobody reads pictures, so nothing could see what a click did.
pub const NO_ONE_READS: Word = Word::saying(
    "adapters.fallback.click-at.no-one-reads",
    "alo cannot click inside {application}: nobody has been chosen to read pictures of its window, \
     so alo could not see what a click did, and nothing was clicked. To choose, open “Who reads \
     the pictures” in Settings → What alo can use",
)
.noting(
    "Said when the person has chosen \"No one\" under \"Who reads the pictures\". Use exactly the \
     translations of the Settings names.",
);

/// It describes itself, so it is pressed by name.
pub const IT_DESCRIBES_ITSELF: Word = Word::saying(
    "adapters.fallback.click-at.describes-itself",
    "{application} names what is in its window, so alo presses things there by their names and \
     clicked nothing by position",
)
.noting("Said when the agent asked to click by position in a window alo can read in words.");

/// A password field in one of its windows.
pub const A_PASSWORD_FIELD: Word = Word::saying(
    "adapters.fallback.click-at.password-field",
    "A window of {application} has a password field, and alo never clicks by position in one, so \
     nothing was clicked",
)
.noting("Said when a window of the application has a password field.");

/// Another window has the keyboard.
pub const NOT_IN_FRONT: Word = Word::saying(
    "adapters.fallback.click-at.not-in-front",
    "No window of {application} has the keyboard, so nothing was clicked. alo clicks only in the \
     window in front: bring it forward first",
)
.noting("Said when another application's window has the keyboard, or none has.");

/// The person's own window, at the default.
pub const NOT_ALOS_COPY: Word = Word::saying(
    "adapters.fallback.click-at.not-alos-copy",
    "That {application} window is yours, not alo's own copy, so nothing was clicked. To let alo \
     work in your own windows, choose “alo may work in your own window” in Settings → What alo \
     can use",
)
.noting(
    "Said when the window in front is the person's own and alo works only in its own copy. \
     \"What alo can use\" is the name of the Settings page: use exactly its translation there.",
);

/// The person is holding a button down.
pub const THE_PERSON_IS_PRESSING: Word = Word::saying(
    "adapters.fallback.click-at.the-person-is-pressing",
    "You are holding a button down, so alo clicked nothing in {application} rather than press \
     into what you are doing",
)
.noting("Said when the person was pressing a mouse button at the moment alo would have clicked.");

/// This machine could not click.
pub const NO_CLICK: Word = Word::saying(
    "adapters.fallback.click-at.no-click",
    "This machine could not click in {application} just now, so nothing was clicked",
)
.noting("Said when the desktop could not deliver a click, or could not be reached.");

/// Asked for from another machine.
pub const ACROSS_A_PAIRING: Word = Word::saying(
    "adapters.fallback.click-at.across-a-pairing",
    "alo never clicks by position in a window on this computer for another computer, so nothing \
     was clicked in {application}",
)
.noting("Said when another computer paired with this one asked alo to click by position here.");

/// Every word clicking declares.
pub const EVERY_WORD: [Word; 17] = [
    CLICK_PURPOSE,
    CLICK_APPLICATION,
    CLICK_ACROSS,
    CLICK_DOWN,
    CLICK_SENTENCE,
    CLICK_CHANGES,
    CLICK_LEAVES,
    NOT_KNOWN,
    NOT_SWITCHED_ON,
    NO_ONE_READS,
    IT_DESCRIBES_ITSELF,
    A_PASSWORD_FIELD,
    NOT_IN_FRONT,
    NOT_ALOS_COPY,
    THE_PERSON_IS_PRESSING,
    NO_CLICK,
    ACROSS_A_PAIRING,
];
