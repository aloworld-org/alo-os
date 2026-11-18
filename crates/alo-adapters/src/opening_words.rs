//! What a person is told about **alo opening an item in an application** — a
//! file in a folder, a row of an archive (`accessible.open_item`, added
//! 2026-11-17) — in their language.
//!
//! In every sentence `{name}` is the item exactly as the application shows it
//! and `{application}` the application's identifier; neither is translated.

use alo_strings::Word;

/// What `accessible.open_item` is for.
pub const OPEN_PURPOSE: Word = Word::saying(
    "adapters.fallback.open-item.purpose",
    "open one item of a list in an application's window, as a double-click would, such as a \
     file in a folder",
)
.noting("What one thing the agent can do is for, shown where its list of abilities is.");

/// What `application` is, for opening an item.
pub const OPEN_APPLICATION: Word = Word::saying(
    "adapters.fallback.open-item.application",
    "the application the item is in",
)
.noting("Shown beside the application the agent names when it asks to open an item.");

/// What `name` is, for opening an item.
pub const OPEN_NAME: Word = Word::saying(
    "adapters.fallback.open-item.name",
    "the item, by the name or the words it shows",
)
.noting("Shown beside the item the agent names, such as a file name in a folder.");

/// What a person approves.
pub const OPEN_SENTENCE: Word = Word::saying(
    "adapters.fallback.open-item.sentence",
    "open “{name}” in {application}",
)
.noting(
    "The sentence a person approves before anything is opened. {name} is the item exactly as the \
     application shows it and is not translated; {application} is the application's identifier \
     and is not translated. Opening is what a double-click does.",
);

/// What opening changes, on the approval card.
pub const OPEN_CHANGES: Word = Word::saying(
    "adapters.fallback.open-item.changes",
    "Opens that one item, as a person double-clicking it would. What it opens with is the \
     application's choice: a file may open in another application, and a program may start.",
)
.noting(
    "Shown on the card a person answers before alo opens an item, beside a tick. The second \
     sentence says plainly what opening can start, so the person decides knowing it.",
);

/// What opening leaves alone.
pub const OPEN_LEAVES: Word = Word::saying(
    "adapters.fallback.open-item.leaves",
    "No other item is opened, nothing is typed, nothing is moved or deleted, and nothing is found \
     by where it is on the screen.",
)
.noting("Shown beside the sentence about what opening changes.");

/// No item of that name is on the screen.
pub const NO_SUCH_ITEM: Word = Word::saying(
    "adapters.fallback.open-item.no-such-item",
    "{application} shows no item named “{name}” now, so nothing was opened",
)
.noting(
    "Said when the item the person approved is not on the screen — it may be further down a list \
     that has not been scrolled to. Nothing happened.",
);

/// More than one item has that name.
pub const MORE_THAN_ONE: Word = Word::saying(
    "adapters.fallback.open-item.more-than-one",
    "{application} shows more than one “{name}”, so alo did not guess which, and nothing was \
     opened",
)
.noting("Said when two items on the screen have the same name. Nothing happened.");

/// The window is too large to be sure.
pub const TOO_MUCH_TO_BE_SURE: Word = Word::saying(
    "adapters.fallback.open-item.too-much-to-be-sure",
    "{application} shows more than can be read at once, so whether “{name}” is the only one is \
     not known, and nothing was opened",
)
.noting("Said when a window is too large to read whole. Nothing happened.");

/// It is greyed out.
pub const CANNOT_BE_USED_NOW: Word = Word::saying(
    "adapters.fallback.open-item.cannot-be-used-now",
    "“{name}” in {application} is greyed out, so nothing was opened",
)
.noting("Said when the item cannot be used at the moment. Nothing happened.");

/// The application offers no way to open it from outside.
pub const CANNOT_BE_OPENED: Word = Word::saying(
    "adapters.fallback.open-item.cannot-be-opened",
    "{application} does not let “{name}” be opened from outside the application, so nothing was \
     opened. Double-click it yourself in the window",
)
.noting(
    "Said when the application describes the item but offers no way for alo to open it. Nothing \
     happened.",
);

/// The application answered that it did not open it.
pub const DID_NOT_OPEN: Word = Word::saying(
    "adapters.fallback.open-item.did-not-open",
    "{application} said it did not open “{name}”, so nothing was opened",
)
.noting("Said when the application refused. Nothing happened.");

/// The application did not answer.
pub const DID_NOT_ANSWER: Word = Word::saying(
    "adapters.fallback.open-item.did-not-answer",
    "{application} did not answer in time, so whether “{name}” was opened is not known. Look at \
     the window",
)
.noting(
    "Said after alo asked the application to open the item, when no answer came back. It may \
     have been opened; say so plainly rather than as a failure.",
);

/// Every word this file declares.
pub const EVERY_WORD: [Word; 13] = [
    OPEN_PURPOSE,
    OPEN_APPLICATION,
    OPEN_NAME,
    OPEN_SENTENCE,
    OPEN_CHANGES,
    OPEN_LEAVES,
    NO_SUCH_ITEM,
    MORE_THAN_ONE,
    TOO_MUCH_TO_BE_SURE,
    CANNOT_BE_USED_NOW,
    CANNOT_BE_OPENED,
    DID_NOT_OPEN,
    DID_NOT_ANSWER,
];
