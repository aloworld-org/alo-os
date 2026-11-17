//! What a person is told about **alo typing into a field without saving**
//! (`accessible.type_without_saving`, 2026-11-17), where it differs from
//! typing that saves (`crate::typing_words`, whose other sentences both share).
//!
//! In every sentence `{name}` is the field's name exactly as the application
//! shows it and `{application}` is the application's identifier; neither is
//! translated.

use alo_strings::Word;

/// What the verb is for.
pub const UNSAVED_PURPOSE: Word = Word::saying(
    "adapters.fallback.type-without-saving.purpose",
    "type a text into a field in an application's window — a search box, a form, an address, a \
     message — and save nothing",
)
.noting("What one thing the agent can do is for, shown where its list of abilities is.");

/// What a person approves. The text itself is shown whole beneath it.
pub const UNSAVED_SENTENCE: Word = Word::saying(
    "adapters.fallback.type-without-saving.sentence",
    "type {text} into “{name}” in {application}",
)
.noting(
    "The sentence a person approves before anything is typed; the whole text is shown beneath it. \
     Nothing is saved. {text} is already translated and says “the text below”; {name} and \
     {application} are not translated.",
);

/// What typing without saving changes, on the approval card.
pub const UNSAVED_CHANGES: Word = Word::saying(
    "adapters.fallback.type-without-saving.changes",
    "The text below is typed where the field's cursor is and read back. Nothing is saved.",
)
.noting(
    "Shown under the sentence a person approves, saying what will change. Held to it by a test.",
);

/// The text reads back differently.
pub const UNSAVED_READ_BACK_DIFFERS: Word = Word::saying(
    "adapters.fallback.type-without-saving.read-back-differs",
    "What “{name}” in {application} shows now is not exactly the text you approved. Look at the \
     window, and undo with Ctrl+Z if you want it gone",
)
.noting("Said when the text was typed and the application changed it or lost part of it.");

/// The text cannot be read back.
pub const UNSAVED_CANNOT_READ_BACK: Word = Word::saying(
    "adapters.fallback.type-without-saving.cannot-read-back",
    "The text was typed into “{name}” in {application}, and the window does not show it in a \
     way alo can read back. Look at the window to check it",
)
.noting("Said when the text was typed and could not be checked.");

/// Every word typing without saving declares.
pub const EVERY_WORD: [Word; 5] = [
    UNSAVED_PURPOSE,
    UNSAVED_SENTENCE,
    UNSAVED_CHANGES,
    UNSAVED_READ_BACK_DIFFERS,
    UNSAVED_CANNOT_READ_BACK,
];
