//! What a person is told about **alo typing into an application**
//! (`accessible.type_text`, ADR 0153), in their language.
//!
//! In every sentence `{name}` is the field's name exactly as the application
//! shows it and `{application}` is the application's identifier; neither is
//! translated.

use alo_strings::Word;

/// The placeholder for one character no key can type.
pub const CHARACTER: &str = "character";

/// What the verb is for.
pub const TYPE_PURPOSE: Word = Word::saying(
    "adapters.fallback.type-text.purpose",
    "type a text into a field or a document in an application's window, and save it",
)
.noting("What one thing the agent can do is for, shown where its list of abilities is.");

/// What `application` is, for typing.
pub const TYPE_APPLICATION: Word = Word::saying(
    "adapters.fallback.type-text.application",
    "the application to type into",
)
.noting("Shown beside the application the agent names when it asks to type.");

/// What `name` is, for typing.
pub const TYPE_NAME: Word = Word::saying(
    "adapters.fallback.type-text.name",
    "the name of the field or document, as the application shows it",
)
.noting("Shown beside the name of the place the agent names, such as a file name in an editor.");

/// What `text` is.
pub const TYPE_THE_TEXT: Word = Word::saying(
    "adapters.fallback.type-text.text",
    "the text to type, shown to you in full before anything is typed",
)
.noting("Shown beside the text the agent proposes to type.");

/// What a person approves. The text itself is shown whole beneath it.
pub const TYPE_SENTENCE: Word = Word::saying(
    "adapters.fallback.type-text.sentence",
    "type {text} into “{name}” in {application}, and save it",
)
.noting(
    "The sentence a person approves before anything is typed; the whole text is shown beneath it. \
     {text} is already translated and says “the text below”; {name} and {application} are not \
     translated.",
);

/// How a person types without the agent.
pub const TYPE_BY_HAND: Word = Word::saying(
    "adapters.fallback.type-text.by-hand",
    "Click in the application's window and type it yourself",
)
.noting("How a person does the same thing themselves.");

/// What typing changes, on the approval card.
pub const TYPE_CHANGES: Word = Word::saying(
    "adapters.fallback.type-text.changes",
    "The text below is typed where the field's cursor is, read back, and saved only if it reads \
     back exactly as you see it here.",
)
.noting("Shown under the sentence a person approves, saying what will change.");

/// What typing leaves alone.
pub const TYPE_LEAVES: Word = Word::saying(
    "adapters.fallback.type-text.leaves",
    "Nothing is typed into any other window, into a password field, or into a terminal, and no \
     file is written behind the application's back.",
)
.noting("Shown beside the sentence about what typing changes.");

/// No field of that name.
pub const NO_SUCH_FIELD: Word = Word::saying(
    "adapters.fallback.type-text.no-such-field",
    "{application} shows no field or document named “{name}” now, so nothing was typed",
)
.noting("Said when the place the person approved typing into is not on the screen.");

/// More than one field of that name.
pub const MORE_THAN_ONE_FIELD: Word = Word::saying(
    "adapters.fallback.type-text.more-than-one",
    "{application} shows more than one field named “{name}”, so alo did not guess which, and \
     nothing was typed",
)
.noting("Said when two places on the screen have the approved name.");

/// A password field.
pub const A_PASSWORD_FIELD: Word = Word::saying(
    "adapters.fallback.type-text.password-field",
    "“{name}” in {application} is a password field, and alo never types into one, so nothing \
     was typed",
)
.noting("Said when the place the person approved typing into is a password field.");

/// A terminal.
pub const A_TERMINAL: Word = Word::saying(
    "adapters.fallback.type-text.terminal",
    "“{name}” in {application} is a terminal, and typing there runs commands, which alo does \
     only where you have let your agent use a terminal, so nothing was typed",
)
.noting("Said when the place the person approved typing into is a terminal inside an application.");

/// The field does not have the keyboard.
pub const NOT_IN_FRONT: Word = Word::saying(
    "adapters.fallback.type-text.not-in-front",
    "“{name}” in {application} does not have the keyboard, so nothing was typed. alo types only \
     where the keyboard is: bring the window forward and click in it first",
)
.noting("Said when another window, or another field, has the keyboard.");

/// The person's own window, at the default.
pub const NOT_ALOS_COPY: Word = Word::saying(
    "adapters.fallback.type-text.not-alos-copy",
    "That {application} window is yours, not alo's own copy, so nothing was typed. To let alo \
     type in your own windows, choose “alo may work in your own window” in Settings → Folders \
     alo can use",
)
.noting("Said when the window in front is the person's own and alo works only in its own copy.");

/// This machine cannot deliver keys, or the compositor is not there.
pub const NO_KEYBOARD: Word = Word::saying(
    "adapters.fallback.type-text.no-keyboard",
    "This machine could not type into {application} just now, so nothing was typed",
)
.noting("Said when the desktop has no keyboard to type through, or could not be reached.");

/// A character no key can type.
pub const CANNOT_TYPE: Word = Word::saying(
    "adapters.fallback.type-text.cannot-type",
    "The text holds a character no key can type ({character}), so nothing was typed",
)
.noting("{character} is a code point such as U+0007, not translated.");

/// The application refused the text through its own interface.
pub const DID_NOT_TAKE: Word = Word::saying(
    "adapters.fallback.type-text.did-not-take",
    "{application} did not take the text into “{name}”, so nothing was saved",
)
.noting("Said when the application answered that it did not put the text in.");

/// The text reads back differently.
pub const READ_BACK_DIFFERS: Word = Word::saying(
    "adapters.fallback.type-text.read-back-differs",
    "What “{name}” in {application} shows now is not exactly the text you approved, so it was \
     not saved. Look at the window, and undo with Ctrl+Z if you want it gone",
)
.noting("Said when the text was typed and the application changed it or lost part of it.");

/// The text cannot be read back.
pub const CANNOT_READ_BACK: Word = Word::saying(
    "adapters.fallback.type-text.cannot-read-back",
    "The text was typed into “{name}” in {application}, and the window does not show it in a \
     way alo can read back, so it was not saved. Look at the window, and save it yourself if it \
     is right",
)
.noting("Said when the text was typed and could not be checked.");

/// Every word typing declares.
pub const EVERY_WORD: [Word; 19] = [
    TYPE_PURPOSE,
    TYPE_APPLICATION,
    TYPE_NAME,
    TYPE_THE_TEXT,
    TYPE_SENTENCE,
    TYPE_BY_HAND,
    TYPE_CHANGES,
    TYPE_LEAVES,
    NO_SUCH_FIELD,
    MORE_THAN_ONE_FIELD,
    A_PASSWORD_FIELD,
    A_TERMINAL,
    NOT_IN_FRONT,
    NOT_ALOS_COPY,
    NO_KEYBOARD,
    CANNOT_TYPE,
    DID_NOT_TAKE,
    READ_BACK_DIFFERS,
    CANNOT_READ_BACK,
];
