//! What a person is told about **alo pressing a key in an application's
//! window** (`accessible.press_key`, 2026-11-17), in their language.
//!
//! In every sentence `{application}` is the application's identifier, which is
//! not translated, and `{key}` is one of the seven keys' own words, already
//! translated.

use alo_strings::Word;

/// What the verb is for.
pub const KEY_PURPOSE: Word = Word::saying(
    "adapters.fallback.press-key.purpose",
    "press one key — Enter, Escape, Tab or an arrow — in the window of an application that has \
     the keyboard, as a person would",
)
.noting(
    "What one thing the agent can do is for, shown where its list of abilities is. Enter, \
     Escape and Tab are the names of keys on a keyboard: use the names a keyboard in your \
     language prints, or the usual ones.",
);

/// What `application` is, for a key.
pub const KEY_APPLICATION: Word = Word::saying(
    "adapters.fallback.press-key.application",
    "the application whose window the key is pressed in",
)
.noting("Shown beside the application the agent names when it asks to press a key.");

/// What `key` is.
pub const KEY_KEY: Word = Word::saying("adapters.fallback.press-key.key", "the key to press, once")
    .noting("Shown beside the key the agent chooses.");

/// Enter.
pub const ENTER: Word = Word::saying("adapters.fallback.press-key.enter", "Enter")
    .noting("The Enter key on a keyboard, put into “press {key} in …”.");

/// Escape.
pub const ESCAPE: Word = Word::saying("adapters.fallback.press-key.escape", "Escape")
    .noting("The Escape (Esc) key on a keyboard, put into “press {key} in …”.");

/// Tab.
pub const TAB: Word = Word::saying("adapters.fallback.press-key.tab", "Tab")
    .noting("The Tab key on a keyboard, put into “press {key} in …”.");

/// The up arrow.
pub const UP: Word = Word::saying("adapters.fallback.press-key.up", "the up arrow")
    .noting("The arrow key pointing up, put into “press {key} in …”.");

/// The down arrow.
pub const DOWN: Word = Word::saying("adapters.fallback.press-key.down", "the down arrow")
    .noting("The arrow key pointing down, put into “press {key} in …”.");

/// The left arrow.
pub const LEFT: Word = Word::saying("adapters.fallback.press-key.left", "the left arrow")
    .noting("The arrow key pointing left, put into “press {key} in …”.");

/// The right arrow.
pub const RIGHT: Word = Word::saying("adapters.fallback.press-key.right", "the right arrow")
    .noting("The arrow key pointing right, put into “press {key} in …”.");

/// What a person approves.
pub const KEY_SENTENCE: Word = Word::saying(
    "adapters.fallback.press-key.sentence",
    "press {key} in {application}",
)
.noting(
    "The sentence a person approves before alo presses a key. {key} is one key's name, already \
     translated, such as “Enter” or “the down arrow”. {application} is the application's \
     identifier and is not translated.",
);

/// What pressing a key changes, on the approval card.
pub const KEY_CHANGES: Word = Word::saying(
    "adapters.fallback.press-key.changes",
    "Presses that one key once in the application's window that has the keyboard, as a person \
     pressing it would.",
)
.noting(
    "Shown on the card a person answers before alo presses a key, beside a tick. What the \
     window then does with the key is the application's. Held to it by a test.",
);

/// What pressing a key leaves alone.
pub const KEY_LEAVES: Word = Word::saying(
    "adapters.fallback.press-key.leaves",
    "No other key is pressed, no other window is sent the key, and nothing is pressed in a \
     password field or a terminal.",
)
.noting("Shown beside the sentence about what pressing a key changes. Held to it by a test.");

/// A password field has the keyboard.
pub const A_PASSWORD_FIELD_HAS_THE_KEYBOARD: Word = Word::saying(
    "adapters.fallback.press-key.password-field",
    "A password field has the keyboard in {application}, and alo never acts on one, so no key \
     was pressed",
)
.noting("Said when the place the key would go into is a password field.");

/// A terminal has the keyboard.
pub const A_TERMINAL_HAS_THE_KEYBOARD: Word = Word::saying(
    "adapters.fallback.press-key.terminal",
    "A terminal has the keyboard in {application}, and a key there can run a command, which alo \
     does only where you have let your agent use a terminal, so no key was pressed",
)
.noting("Said when the place the key would go into is a terminal inside an application.");

/// Another window has the keyboard.
pub const NOT_IN_FRONT: Word = Word::saying(
    "adapters.fallback.press-key.not-in-front",
    "No window of {application} has the keyboard, so no key was pressed. alo presses keys only \
     where the keyboard is: bring the window forward first",
)
.noting("Said when another application's window has the keyboard, or none has.");

/// The person's own window, at the default.
pub const NOT_ALOS_COPY: Word = Word::saying(
    "adapters.fallback.press-key.not-alos-copy",
    "That {application} window is yours, not alo's own copy, so no key was pressed. To let alo \
     press keys in your own windows, choose “alo may work in your own window” in Settings → \
     What alo can use",
)
.noting(
    "Said when the window in front is the person's own and alo works only in its own copy. \
     \"What alo can use\" is the name of the Settings page: use exactly its translation there.",
);

/// This machine cannot deliver keys.
pub const NO_KEYBOARD: Word = Word::saying(
    "adapters.fallback.press-key.no-keyboard",
    "This machine could not press a key in {application} just now, so no key was pressed",
)
.noting("Said when the desktop has no keyboard to press through, or could not be reached.");

/// Every word pressing a key declares.
pub const EVERY_WORD: [Word; 18] = [
    KEY_PURPOSE,
    KEY_APPLICATION,
    KEY_KEY,
    ENTER,
    ESCAPE,
    TAB,
    UP,
    DOWN,
    LEFT,
    RIGHT,
    KEY_SENTENCE,
    KEY_CHANGES,
    KEY_LEAVES,
    A_PASSWORD_FIELD_HAS_THE_KEYBOARD,
    A_TERMINAL_HAS_THE_KEYBOARD,
    NOT_IN_FRONT,
    NOT_ALOS_COPY,
    NO_KEYBOARD,
];
