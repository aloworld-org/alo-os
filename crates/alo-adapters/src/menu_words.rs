//! What a person is told about **alo opening a thing's right-click menu**
//! (`accessible.open_menu`, 2026-11-17), in their language.
//!
//! In every sentence `{name}` is the thing's name exactly as the application
//! shows it and `{application}` is the application's identifier; neither is
//! translated.

use alo_strings::Word;

/// What the verb is for.
pub const MENU_PURPOSE: Word = Word::saying(
    "adapters.fallback.open-menu.purpose",
    "open the menu a right-click would open on one thing in an application's window, named as \
     the application names it, so that one of its items can be chosen",
)
.noting("What one thing the agent can do is for, shown where its list of abilities is.");

/// What `application` is.
pub const MENU_APPLICATION: Word = Word::saying(
    "adapters.fallback.open-menu.application",
    "the application the thing is in",
)
.noting("Shown beside the application the agent names when it asks to open a menu.");

/// What `name` is.
pub const MENU_NAME: Word = Word::saying(
    "adapters.fallback.open-menu.name",
    "the name of the thing whose menu is opened, exactly as the application shows it",
)
.noting("Shown beside the name the agent gives, such as a file in a list.");

/// What a person approves.
pub const MENU_SENTENCE: Word = Word::saying(
    "adapters.fallback.open-menu.sentence",
    "open the right-click menu of “{thing}” in {application}",
)
.noting(
    "The sentence a person approves before alo opens a menu. “Right-click menu” is the menu a \
     click with the other mouse button opens: use the name people use for it in your language. \
     {thing} is the thing's name as the application shows it, and {application} the \
     application's identifier; neither is translated.",
);

/// What opening a menu changes, on the approval card.
pub const MENU_CHANGES: Word = Word::saying(
    "adapters.fallback.open-menu.changes",
    "Opens that one thing's own menu, by the application's own way of opening it, as a \
     right-click would.",
)
.noting(
    "Shown on the card a person answers before alo opens a menu, beside a tick. Held to it by a \
     test.",
);

/// What opening a menu leaves alone.
pub const MENU_LEAVES: Word = Word::saying(
    "adapters.fallback.open-menu.leaves",
    "No item of the menu is chosen, nothing is typed, and nothing is found by where it is on the \
     screen.",
)
.noting("Shown beside the sentence about what opening a menu changes. Held to it by a test.");

/// Nothing of that name is showing.
pub const NOTHING_NAMED: Word = Word::saying(
    "adapters.fallback.open-menu.nothing-named",
    "{application} shows nothing named “{name}” now, so no menu was opened",
)
.noting("Said when the thing the person approved is not on the screen.");

/// More than one thing of that name offers a menu.
pub const MORE_THAN_ONE: Word = Word::saying(
    "adapters.fallback.open-menu.more-than-one",
    "{application} shows more than one “{name}” with a menu, so alo did not guess which, and no \
     menu was opened",
)
.noting("Said when two things on the screen have the approved name and a menu.");

/// Too much to be sure which.
pub const TOO_MUCH: Word = Word::saying(
    "adapters.fallback.open-menu.too-much",
    "{application} shows more than alo can read at once, so alo could not be sure which \
     “{name}” was meant, and no menu was opened",
)
.noting("Said when the window holds so many things that alo cannot know there is only one.");

/// A password field.
pub const A_PASSWORD_FIELD: Word = Word::saying(
    "adapters.fallback.open-menu.password-field",
    "“{name}” in {application} is a password field, and alo never acts on one, so no menu was \
     opened",
)
.noting("Said when the thing the person approved is a password field.");

/// The thing offers no menu alo can open.
pub const NO_MENU: Word = Word::saying(
    "adapters.fallback.open-menu.no-menu",
    "“{name}” in {application} offers no menu alo can open, so no menu was opened. Right-click \
     it yourself if it has one",
)
.noting(
    "Said when the thing is on the screen but the application gives alo no way to open its \
     menu.",
);

/// The application said no.
pub const DID_NOT_OPEN: Word = Word::saying(
    "adapters.fallback.open-menu.did-not-open",
    "{application} answered that it could not open the menu of “{name}”, so no menu was opened",
)
.noting("Said when the application itself said no.");

/// The application did not answer.
pub const DID_NOT_ANSWER: Word = Word::saying(
    "adapters.fallback.open-menu.did-not-answer",
    "alo asked {application} to open the menu of “{name}” and it did not answer, so whether the \
     menu is open is not known. Read the window to see",
)
.noting("Said after asking, when the application gave no answer in time. It may have opened.");

/// Every word opening a menu declares.
pub const EVERY_WORD: [Word; 13] = [
    MENU_PURPOSE,
    MENU_APPLICATION,
    MENU_NAME,
    MENU_SENTENCE,
    MENU_CHANGES,
    MENU_LEAVES,
    NOTHING_NAMED,
    MORE_THAN_ONE,
    TOO_MUCH,
    A_PASSWORD_FIELD,
    NO_MENU,
    DID_NOT_OPEN,
    DID_NOT_ANSWER,
];
