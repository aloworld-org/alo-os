//! What a person is told about **alo scrolling in an application's window**
//! (`accessible.scroll_to` and `accessible.scroll_page`), in their language.
//!
//! In every sentence `{name}` is a thing's name exactly as the application
//! shows it and `{application}` is the application's identifier; neither is
//! translated.

use alo_strings::Word;

/// What scrolling to a named thing is for.
pub const SCROLL_TO_PURPOSE: Word = Word::saying(
    "adapters.fallback.scroll-to.purpose",
    "scroll an application's window until one thing, named as the application names it, is on \
     the screen — for reading or pressing what is further down or further up",
)
.noting("What one thing the agent can do is for, shown where its list of abilities is.");

/// What `application` is, for scrolling to a thing.
pub const SCROLL_TO_APPLICATION: Word = Word::saying(
    "adapters.fallback.scroll-to.application",
    "the application whose window is scrolled",
)
.noting("Shown beside the application the agent names when it asks to scroll.");

/// What `name` is, for scrolling to a thing.
pub const SCROLL_TO_NAME: Word = Word::saying(
    "adapters.fallback.scroll-to.name",
    "the name of the thing to bring onto the screen, exactly as the application shows it",
)
.noting("Shown beside the name the agent gives, such as a heading, a file in a list or a button.");

/// What a person approves before alo scrolls to a thing.
pub const SCROLL_TO_SENTENCE: Word = Word::saying(
    "adapters.fallback.scroll-to.sentence",
    "scroll {application} until “{name}” is on the screen",
)
.noting(
    "The sentence a person approves before alo scrolls. {name} is the thing's name exactly as \
     the application shows it and is not translated. {application} is the application's \
     identifier and is not translated.",
);

/// What scrolling by a page is for.
pub const SCROLL_PAGE_PURPOSE: Word = Word::saying(
    "adapters.fallback.scroll-page.purpose",
    "scroll an application's window by about one screen, down or up, so that what was off the \
     screen can be read",
)
.noting("What one thing the agent can do is for, shown where its list of abilities is.");

/// What `application` is, for scrolling by a page.
pub const SCROLL_PAGE_APPLICATION: Word = Word::saying(
    "adapters.fallback.scroll-page.application",
    "the application whose window is scrolled",
)
.noting("Shown beside the application the agent names when it asks to scroll by a page.");

/// What `toward` is.
pub const SCROLL_PAGE_TOWARD: Word = Word::saying(
    "adapters.fallback.scroll-page.toward",
    "which way to scroll: down or up",
)
.noting("Shown beside the direction the agent chooses.");

/// The direction down.
pub const DOWN: Word = Word::saying("adapters.fallback.scroll-page.down", "down")
    .noting("A direction, put into “scroll {toward} by one page in …”.");

/// The direction up.
pub const UP: Word = Word::saying("adapters.fallback.scroll-page.up", "up")
    .noting("A direction, put into “scroll {toward} by one page in …”.");

/// What a person approves before alo scrolls by a page.
pub const SCROLL_PAGE_SENTENCE: Word = Word::saying(
    "adapters.fallback.scroll-page.sentence",
    "scroll {toward} by one page in {application}",
)
.noting(
    "The sentence a person approves before alo scrolls. {toward} is “down” or “up”, already \
     translated. {application} is the application's identifier and is not translated.",
);

/// What scrolling changes, on the approval card — both verbs.
pub const SCROLL_CHANGES: Word = Word::saying(
    "adapters.fallback.scroll.changes",
    "Moves what one window of the application shows, as scrolling would, so that more of it \
     can be read.",
)
.noting(
    "Shown on the card a person answers before alo scrolls in an application, beside a tick. \
     Held to it by a test.",
);

/// What scrolling leaves alone — both verbs.
pub const SCROLL_LEAVES: Word = Word::saying(
    "adapters.fallback.scroll.leaves",
    "Nothing is typed, nothing is pressed, no text is changed, and nothing is found by where it \
     is on the screen.",
)
.noting("Shown beside the sentence about what scrolling changes. Held to it by a test.");

/// No thing of that name.
pub const NOTHING_NAMED: Word = Word::saying(
    "adapters.fallback.scroll-to.nothing-named",
    "{application} has nothing named “{name}”, on the screen or off it, so nothing was scrolled",
)
.noting(
    "Said when the thing the person approved scrolling to is not in the application's windows \
     at all.",
);

/// More than one thing of that name.
pub const MORE_THAN_ONE: Word = Word::saying(
    "adapters.fallback.scroll-to.more-than-one",
    "{application} has more than one thing named “{name}”, so alo did not guess which, and \
     nothing was scrolled",
)
.noting("Said when two things in the application's windows have the approved name.");

/// Too much to be sure which.
pub const TOO_MUCH: Word = Word::saying(
    "adapters.fallback.scroll-to.too-much",
    "{application} holds more than alo can read at once, so alo could not be sure which \
     “{name}” was meant, and nothing was scrolled",
)
.noting(
    "Said when the application's windows hold so many things that alo cannot know there is \
     only one with the approved name.",
);

/// A password field.
pub const A_PASSWORD_FIELD: Word = Word::saying(
    "adapters.fallback.scroll-to.password-field",
    "“{name}” in {application} is a password field, and alo never acts on one, so nothing was \
     scrolled",
)
.noting("Said when the thing the person approved scrolling to is a password field.");

/// Nothing further down.
pub const NOTHING_FURTHER_DOWN: Word = Word::saying(
    "adapters.fallback.scroll-page.nothing-further-down",
    "{application} shows nothing further down to scroll to, so nothing was scrolled",
)
.noting("Said when the window is already at its end, or does not say what is below what it shows.");

/// Nothing further up.
pub const NOTHING_FURTHER_UP: Word = Word::saying(
    "adapters.fallback.scroll-page.nothing-further-up",
    "{application} shows nothing further up to scroll to, so nothing was scrolled",
)
.noting(
    "Said when the window is already at its start, or does not say what is above what it shows.",
);

/// The application said it could not.
pub const DID_NOT_SCROLL: Word = Word::saying(
    "adapters.fallback.scroll.did-not",
    "{application} answered that it could not scroll there, so nothing was scrolled",
)
.noting("Said when the application itself said no to scrolling.");

/// Asked, and not seen to have moved.
pub const NOT_SEEN_TO_MOVE: Word = Word::saying(
    "adapters.fallback.scroll.not-seen-to-move",
    "alo asked {application} to scroll, and its window does not yet say that what was off the \
     screen is on it now. Read the window again to see what it shows",
)
.noting(
    "Said after scrolling, when the application said yes but does not describe the thing as \
     on the screen afterwards. It may have scrolled.",
);

/// Every word scrolling declares.
pub const EVERY_WORD: [Word; 21] = [
    SCROLL_TO_PURPOSE,
    SCROLL_TO_APPLICATION,
    SCROLL_TO_NAME,
    SCROLL_TO_SENTENCE,
    SCROLL_PAGE_PURPOSE,
    SCROLL_PAGE_APPLICATION,
    SCROLL_PAGE_TOWARD,
    DOWN,
    UP,
    SCROLL_PAGE_SENTENCE,
    SCROLL_CHANGES,
    SCROLL_LEAVES,
    NOTHING_NAMED,
    MORE_THAN_ONE,
    TOO_MUCH,
    A_PASSWORD_FIELD,
    NOTHING_FURTHER_DOWN,
    NOTHING_FURTHER_UP,
    DID_NOT_SCROLL,
    NOT_SEEN_TO_MOVE,
    ALREADY_SHOWING,
];

/// Said when the thing is found already on the screen and the application
/// will not move it — kept apart from [`DID_NOT_SCROLL`] so the person is not
/// told something failed that was already the case.
pub const ALREADY_SHOWING: Word = Word::saying(
    "adapters.fallback.scroll-to.already-showing",
    "“{name}” is already on the screen in {application}",
)
.noting(
    "Said after scrolling to a thing that was already on the screen, when the application did \
     not move it. Nothing went wrong.",
);
