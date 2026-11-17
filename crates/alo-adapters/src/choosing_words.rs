//! What a person is told about **alo choosing in an application** — an option
//! of a drop-down (`accessible.choose_option`) and an item of a list
//! (`accessible.select_item`), added 2026-11-17 — in their language.
//!
//! In every sentence `{name}` is the drop-down's or the item's name exactly as
//! the application shows it, `{option}` is the option exactly as the
//! drop-down lists it, and `{application}` is the application's identifier;
//! none of the three is translated.

use alo_strings::Word;

/// The gap holding the option chosen.
pub const OPTION: &str = "option";

// ---------------------------------------------------------------------------
// Choosing an option of a drop-down.
// ---------------------------------------------------------------------------

/// What `accessible.choose_option` is for.
pub const CHOOSE_PURPOSE: Word = Word::saying(
    "adapters.fallback.choose-option.purpose",
    "choose one option of a drop-down in an application's window",
)
.noting("What one thing the agent can do is for, shown where its list of abilities is.");

/// What `application` is, for choosing an option.
pub const CHOOSE_APPLICATION: Word = Word::saying(
    "adapters.fallback.choose-option.application",
    "the application the drop-down is in",
)
.noting("Shown beside the application the agent names when it asks to choose an option.");

/// What `name` is, for choosing an option.
pub const CHOOSE_NAME: Word = Word::saying(
    "adapters.fallback.choose-option.name",
    "the drop-down, by the name it shows or the choice it shows now",
)
.noting(
    "Shown beside the drop-down the agent names. A drop-down is a control showing one choice of \
     several, which opens into a list of them.",
);

/// What `option` is.
pub const CHOOSE_OPTION_WORD: Word = Word::saying(
    "adapters.fallback.choose-option.option",
    "the option to choose, exactly as the drop-down lists it",
)
.noting("Shown beside the option the agent names, such as a language or a paper size.");

/// What a person approves.
pub const CHOOSE_SENTENCE: Word = Word::saying(
    "adapters.fallback.choose-option.sentence",
    "choose “{option}” in the drop-down “{name}” in {application}",
)
.noting(
    "The sentence a person approves before anything is chosen. {option} and {name} are exactly \
     as the application shows them and are not translated; {application} is the application's \
     identifier, like org.gnome.Papers, and is not translated.",
);

/// What choosing changes, on the approval card.
pub const CHOOSE_CHANGES: Word = Word::saying(
    "adapters.fallback.choose-option.changes",
    "Chooses that one option of that one drop-down, as a person picking it would, and then looks \
     to see that it is chosen.",
)
.noting(
    "Shown on the card a person answers before alo chooses an option, beside a tick. What the \
     application then does with the choice is its own.",
);

/// What choosing leaves alone.
pub const CHOOSE_LEAVES: Word = Word::saying(
    "adapters.fallback.choose-option.leaves",
    "Nothing is typed, no other control is pressed or chosen, and nothing is found by where it \
     is on the screen.",
)
.noting("Shown beside the sentence about what choosing changes.");

// ---------------------------------------------------------------------------
// Selecting an item of a list.
// ---------------------------------------------------------------------------

/// What `accessible.select_item` is for.
pub const SELECT_PURPOSE: Word = Word::saying(
    "adapters.fallback.select-item.purpose",
    "select one item of a list in an application's window, such as a file in a folder, without \
     opening it",
)
.noting("What one thing the agent can do is for, shown where its list of abilities is.");

/// What `application` is, for selecting an item.
pub const SELECT_APPLICATION: Word = Word::saying(
    "adapters.fallback.select-item.application",
    "the application the list is in",
)
.noting("Shown beside the application the agent names when it asks to select an item.");

/// What `name` is, for selecting an item.
pub const SELECT_NAME: Word = Word::saying(
    "adapters.fallback.select-item.name",
    "the item, by the name or the words it shows",
)
.noting("Shown beside the item the agent names, such as a file name in a folder.");

/// What a person approves.
pub const SELECT_SENTENCE: Word = Word::saying(
    "adapters.fallback.select-item.sentence",
    "select “{name}” in {application}",
)
.noting(
    "The sentence a person approves before anything is selected. {name} is the item exactly as \
     the application shows it and is not translated; {application} is the application's \
     identifier and is not translated. Selecting marks the item as chosen; it does not open it.",
);

/// What selecting changes, on the approval card.
pub const SELECT_CHANGES: Word = Word::saying(
    "adapters.fallback.select-item.changes",
    "Selects that one item, as a person clicking it once would, and then looks to see that it is \
     selected.",
)
.noting("Shown on the card a person answers before alo selects an item, beside a tick.");

/// What selecting leaves alone.
pub const SELECT_LEAVES: Word = Word::saying(
    "adapters.fallback.select-item.leaves",
    "The item is not opened, moved or changed, nothing is typed, and nothing is found by where it \
     is on the screen.",
)
.noting("Shown beside the sentence about what selecting changes.");

// ---------------------------------------------------------------------------
// When nothing was chosen.
// ---------------------------------------------------------------------------

/// No drop-down of that name is on the screen.
pub const NO_SUCH_DROP_DOWN: Word = Word::saying(
    "adapters.fallback.choose.no-such-drop-down",
    "{application} shows no drop-down named “{name}” now, so nothing was chosen",
)
.noting("Said when the drop-down the person approved is not on the screen. Nothing happened.");

/// No item of that name is on the screen.
pub const NO_SUCH_ITEM: Word = Word::saying(
    "adapters.fallback.choose.no-such-item",
    "{application} shows no item named “{name}” now, so nothing was selected",
)
.noting(
    "Said when the item the person approved is not on the screen — it may be further down a list \
     that has not been scrolled to. Nothing happened.",
);

/// More than one drop-down or item has that name.
pub const MORE_THAN_ONE: Word = Word::saying(
    "adapters.fallback.choose.more-than-one",
    "{application} shows more than one “{name}”, so alo did not guess which, and nothing was \
     chosen",
)
.noting("Said when two controls on the screen have the same name. Nothing happened.");

/// The window is too large to be sure the one named is the only one.
pub const TOO_MUCH_TO_BE_SURE: Word = Word::saying(
    "adapters.fallback.choose.too-much-to-be-sure",
    "{application} shows more than can be read at once, so whether “{name}” is the only one is \
     not known, and nothing was chosen",
)
.noting("Said when a window is too large to read whole. Nothing happened.");

/// The drop-down or item is greyed out.
pub const CANNOT_BE_USED_NOW: Word = Word::saying(
    "adapters.fallback.choose.cannot-be-used-now",
    "“{name}” in {application} is greyed out, so nothing was chosen",
)
.noting("Said when the control cannot be used at the moment. Nothing happened.");

/// The drop-down lists no option of that name.
pub const NO_SUCH_OPTION: Word = Word::saying(
    "adapters.fallback.choose.no-such-option",
    "The drop-down “{name}” in {application} lists no option “{option}”, so nothing was chosen",
)
.noting(
    "Said when the option the person approved is not one the drop-down offers, matched exactly. \
     Nothing happened.",
);

/// The drop-down lists the option more than once.
pub const MORE_THAN_ONE_OPTION: Word = Word::saying(
    "adapters.fallback.choose.more-than-one-option",
    "The drop-down “{name}” in {application} lists “{option}” more than once, so alo did not \
     guess which, and nothing was chosen",
)
.noting("Said when two options have the same words. Nothing happened.");

/// The application offers no way to choose it from outside.
pub const CANNOT_BE_CHOSEN: Word = Word::saying(
    "adapters.fallback.choose.cannot-be-chosen",
    "{application} does not let “{name}” be chosen from outside the application, so nothing was \
     chosen. Choose it yourself in the window",
)
.noting(
    "Said when the application describes the control but offers no way for alo to choose it. \
     Nothing happened.",
);

/// The application answered that it did not choose it.
pub const DID_NOT_CHOOSE: Word = Word::saying(
    "adapters.fallback.choose.did-not-choose",
    "{application} said it did not choose “{name}”, so nothing was chosen",
)
.noting("Said when the application refused. Nothing happened.");

// ---------------------------------------------------------------------------
// After it was asked.
// ---------------------------------------------------------------------------

/// The application did not answer, so whether it chose is not known.
pub const DID_NOT_ANSWER: Word = Word::saying(
    "adapters.fallback.choose.did-not-answer",
    "{application} did not answer in time, so whether “{name}” was chosen is not known. Look at \
     the window",
)
.noting(
    "Said after alo asked the application to choose, when no answer came back. It may have been \
     chosen; say so plainly rather than as a failure.",
);

/// alo asked, and cannot see that it was chosen.
pub const NOT_SEEN_CHOSEN: Word = Word::saying(
    "adapters.fallback.choose.not-seen-chosen",
    "alo asked {application} to choose “{name}”, and the window does not show it chosen yet. \
     Look at the window before going on",
)
.noting(
    "Said after alo asked, when it looked again and could not see the choice made. The \
     application may show it differently; alo does not claim it worked.",
);

/// Every word this file declares.
pub const EVERY_WORD: [Word; 24] = [
    CHOOSE_PURPOSE,
    CHOOSE_APPLICATION,
    CHOOSE_NAME,
    CHOOSE_OPTION_WORD,
    CHOOSE_SENTENCE,
    CHOOSE_CHANGES,
    CHOOSE_LEAVES,
    SELECT_PURPOSE,
    SELECT_APPLICATION,
    SELECT_NAME,
    SELECT_SENTENCE,
    SELECT_CHANGES,
    SELECT_LEAVES,
    NO_SUCH_DROP_DOWN,
    NO_SUCH_ITEM,
    MORE_THAN_ONE,
    TOO_MUCH_TO_BE_SURE,
    CANNOT_BE_USED_NOW,
    NO_SUCH_OPTION,
    MORE_THAN_ONE_OPTION,
    CANNOT_BE_CHOSEN,
    DID_NOT_CHOOSE,
    DID_NOT_ANSWER,
    NOT_SEEN_CHOSEN,
];
