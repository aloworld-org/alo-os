//! What a person is told about **alo clicking on one thing in an
//! application's window, by what it looks like** (`accessible.click_at`, ADR
//! 0147 as amended 2026-11-17: *"Allow clicking by position"*, and 2026-11-21:
//! *"Yes, click by description"*), in their language — and what the reader the
//! person chose is asked, to find where that thing is.
//!
//! In every sentence `{application}` is what a person calls the application —
//! the name its entry gives it, or its identifier where it has none — and is
//! not translated; `{description}` is the thing alo was asked to click, in the
//! words it was described in, and is not translated either; `{across}` and
//! `{down}` are whole numbers from 0 to 100.

use alo_strings::Word;

/// What the verb is for.
pub const CLICK_PURPOSE: Word = Word::saying(
    "adapters.fallback.click-by-description.purpose",
    "click once on one thing in the window of an application that describes nothing alo can \
     read, such as a button in a game, described as a person sees it: the reader the person \
     chose finds where it is in a picture of that window, and alo clicks there",
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

/// What `description` is.
pub const CLICK_DESCRIPTION: Word = Word::saying(
    "adapters.fallback.click-by-description.description",
    "the one thing to click, described as a person sees it in the window, such as the words on \
     it: the Start race button",
)
.noting(
    "Shown beside what the agent describes when it asks to click. \"the Start race button\" is \
     an example: translate it as an example in your language.",
);

/// What a person approves.
pub const CLICK_SENTENCE: Word = Word::saying(
    "adapters.fallback.click-by-description.sentence",
    "click “{description}” in {application}",
)
.noting(
    "The sentence a person approves before alo clicks once in an application's window. \
     {description} is the thing to click, in the words the agent described it in, and \
     {application} is what the person calls the application; neither is translated.",
);

/// What clicking changes, on the approval card.
pub const CLICK_CHANGES: Word = Word::saying(
    "adapters.fallback.click-by-description.changes",
    "Has the reader you chose find that thing in a picture of the application's window that \
     has the keyboard, then clicks once at its centre, as a person clicking there would, and \
     takes a picture again to see what changed. alo cannot know for certain what it pressed.",
)
.noting(
    "Shown on the card a person answers before alo clicks, beside a tick. \"The reader you \
     chose\" is whoever the person chose under \"Who reads the pictures\". The last sentence is \
     a plain warning, not an apology.",
);

/// What clicking leaves alone.
pub const CLICK_LEAVES: Word = Word::saying(
    "adapters.fallback.click-by-description.leaves",
    "Nothing is clicked where the reader finds no such thing, or more than one. No other window \
     is clicked, nothing is dragged or held down, and nothing is clicked in a terminal or in a \
     window with a password field.",
)
.noting("Shown beside the sentence about what clicking changes.");

/// What the reader is asked, with the picture, to find the thing.
///
/// Answered in **one line**, which `crate::where_a_thing_is::found` reads:
/// how many things match, then the centre of the one that does.
pub const WHERE_IS_IT: Word = Word::saying(
    "adapters.fallback.click-by-description.where-is-it",
    "This is a picture of one window of the application {application}. Find in it: \
     {description}. Answer with one line and nothing else: how many things in the picture match \
     that, then, only if exactly one does, where its centre is as two whole numbers from 0 to \
     100 — how many per cent of the way across from the left edge, then down from the top edge. \
     For example: 1 50 40 for one match, 0 for none, 2 for two.",
)
.noting(
    "Put to an AI model together with a picture of a window, so that it says where one thing \
     is. The model's answer is read by a program, so keep the three examples exactly as they \
     are, with their numbers. {application} and {description} are not translated.",
);

/// Where alo clicked, kept in the record.
pub const CLICKED_THERE: Word = Word::saying(
    "adapters.fallback.click-by-description.clicked-there",
    "alo clicked {across}% across and {down}% down the {application} window, where the reader \
     found “{description}”",
)
.noting(
    "Kept in the record after alo clicked by position. {across} and {down} are whole numbers \
     from 0 to 100.",
);

/// The reader found no such thing.
pub const NOT_FOUND: Word = Word::saying(
    "adapters.fallback.click-by-description.not-found",
    "The reader found nothing like “{description}” in the {application} window, so nothing was \
     clicked",
)
.noting("Said when the reader the person chose found no such thing in the picture.");

/// The reader found more than one.
pub const FOUND_MORE_THAN_ONCE: Word = Word::saying(
    "adapters.fallback.click-by-description.more-than-once",
    "The reader found more than one thing like “{description}” in the {application} window, so \
     nothing was clicked. Describe the one meant more closely",
)
.noting("Said when the reader the person chose found several things that match.");

/// The reader's answer said nothing alo could read.
pub const NOT_UNDERSTOOD: Word = Word::saying(
    "adapters.fallback.click-by-description.not-understood",
    "The reader's answer did not say where “{description}” is in the {application} window, so \
     nothing was clicked",
)
.noting(
    "Said when the reader the person chose answered, but not in the one line asked for, so alo \
     could not tell where to click.",
);

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

/// Nobody reads pictures, so nothing could find the thing.
pub const NO_ONE_READS: Word = Word::saying(
    "adapters.fallback.click-by-description.no-one-reads",
    "alo cannot click inside {application}: nobody has been chosen to read pictures of its window, \
     so nothing could find where to click, and nothing was clicked. To choose, open “Who reads \
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

/// A control named in a window that names none of its controls.
pub const NAMES_NOTHING: Word = Word::saying(
    "adapters.fallback.click-by-description.names-nothing",
    "This window of {application} names none of its controls, so nothing called “{name}” can be \
     found in it. Read its windows, then click the thing by describing what it looks like",
)
.noting(
    "Said, before anybody is asked, when alo names a button, a field or an item in a window that \
     a read has just found to name none of its controls, as a game does. {application} is the \
     application's identifier and {name} the name alo gave; neither is translated.",
);

/// Every word clicking declares.
pub const EVERY_WORD: [Word; 22] = [
    CLICK_PURPOSE,
    CLICK_APPLICATION,
    CLICK_DESCRIPTION,
    CLICK_SENTENCE,
    CLICK_CHANGES,
    CLICK_LEAVES,
    WHERE_IS_IT,
    CLICKED_THERE,
    NOT_FOUND,
    FOUND_MORE_THAN_ONCE,
    NOT_UNDERSTOOD,
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
    NAMES_NOTHING,
];
