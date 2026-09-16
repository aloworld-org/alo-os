//! Every string the accessibility fallback can say.
//!
//! Three groups: the two verbs' own words (what they are for, the sentence,
//! each argument, how a person does it by hand, and the kinds of control), what
//! a person or the agent is told when something could not be read or pressed,
//! and what is said about a part of a window that cannot be read. Collected
//! into the crate's vocabulary by `crate::words`, which also holds every one of
//! them to naming none of the machinery.

use alo_strings::Word;

/// The gap holding an application's identifier.
pub const APPLICATION: &str = "application";

/// The gap holding the kind of control.
pub const KIND: &str = "kind";

/// The gap holding the control's name.
pub const NAME: &str = "name";

/// The gap holding a verb's name.
pub const VERB: &str = "verb";

// ---------------------------------------------------------------------------
// Reading what a window shows.
// ---------------------------------------------------------------------------

/// What `accessible.read_window` is for.
pub const READ_PURPOSE: Word = Word::saying(
    "adapters.fallback.read-window.purpose",
    "read what an application's windows show",
)
.noting(
    "What one thing the agent can do is for, shown in the list of what an agent may do. It \
     reads the words, buttons and fields on the screen in an application's windows, for an \
     application the person granted.",
);

/// What `application` is, for reading.
pub const READ_APPLICATION: Word = Word::saying(
    "adapters.fallback.read-window.application",
    "the application whose windows to read",
)
.noting("Shown beside the one thing the agent names when it asks to read a window.");

/// What is recorded, and shown, when a window is read.
pub const READ_SENTENCE: Word = Word::saying(
    "adapters.fallback.read-window.sentence",
    "read what {application} shows in its windows",
)
.noting(
    "Written in the record when the agent reads an application's windows. {application} is the \
     application's identifier, like org.gnome.Loupe, and is not translated.",
);

/// How a person reads a window without the agent.
pub const READ_BY_HAND: Word = Word::saying(
    "adapters.fallback.read-window.by-hand",
    "Look at the application's window, or have the screen reader read it aloud",
)
.noting("How a person does the same thing themselves, with or without seeing the screen.");

// ---------------------------------------------------------------------------
// Pressing a control.
// ---------------------------------------------------------------------------

/// What `accessible.activate_control` is for.
pub const PRESS_PURPOSE: Word = Word::saying(
    "adapters.fallback.activate-control.purpose",
    "press a button or another control in an application's window",
)
.noting(
    "What one thing the agent can do is for. It is for applications that have no list of their \
     own of what an agent can do in them.",
);

/// What `application` is, for pressing.
pub const PRESS_APPLICATION: Word = Word::saying(
    "adapters.fallback.activate-control.application",
    "the application the control is in",
)
.noting("Shown beside the application the agent names when it asks to press a control.");

/// What `kind` is.
pub const PRESS_KIND: Word = Word::saying(
    "adapters.fallback.activate-control.kind",
    "what kind of control it is",
)
.noting("Shown beside the kind of control the agent names: a button, a check box, a tab.");

/// What `name` is.
pub const PRESS_NAME: Word = Word::saying(
    "adapters.fallback.activate-control.name",
    "the name the control shows",
)
.noting("Shown beside the name of the control the agent names, as the application shows it.");

/// What a person approves.
pub const PRESS_SENTENCE: Word = Word::saying(
    "adapters.fallback.activate-control.sentence",
    "press the {kind} named “{name}” in {application}",
)
.noting(
    "The sentence a person approves before anything happens. {kind} is one of the kinds of \
     control, like “button” or “check box”, already translated. {name} is the control's name \
     exactly as the application shows it and is not translated. {application} is the \
     application's identifier and is not translated.",
);

/// How a person presses a control without the agent.
pub const PRESS_BY_HAND: Word = Word::saying(
    "adapters.fallback.activate-control.by-hand",
    "Press the control in the application's window, with the pointer or from the keyboard",
)
.noting("How a person does the same thing themselves.");

/// A button.
pub const A_BUTTON: Word = Word::saying("adapters.fallback.kind.button", "button")
    .noting("A kind of control, put into “press the {kind} named …”.");

/// A check box.
pub const A_CHECK_BOX: Word = Word::saying("adapters.fallback.kind.check-box", "check box")
    .noting("A kind of control that is ticked or not, put into “press the {kind} named …”.");

/// A radio button.
pub const A_RADIO_BUTTON: Word =
    Word::saying("adapters.fallback.kind.radio-button", "radio button").noting(
        "A kind of control: one of a set of options of which only one is chosen, put into \
         “press the {kind} named …”.",
    );

/// A switch.
pub const A_SWITCH: Word = Word::saying("adapters.fallback.kind.switch", "switch")
    .noting("A kind of control that is on or off, put into “press the {kind} named …”.");

/// A menu item.
pub const A_MENU_ITEM: Word = Word::saying("adapters.fallback.kind.menu-item", "menu item")
    .noting("One item of a menu, put into “press the {kind} named …”.");

/// A tab.
pub const A_TAB: Word = Word::saying("adapters.fallback.kind.tab", "tab")
    .noting("One tab of a set of tabs, put into “press the {kind} named …”.");

/// A link.
pub const A_LINK: Word = Word::saying("adapters.fallback.kind.link", "link")
    .noting("A link, put into “press the {kind} named …”.");

// ---------------------------------------------------------------------------
// When nothing was read or pressed.
// ---------------------------------------------------------------------------

/// An authority for another verb.
pub const NOT_THIS_VERB: Word = Word::saying(
    "adapters.fallback.not-this-verb",
    "{verb} is not reading or pressing something in an application's window, so nothing was done",
)
.noting(
    "A fault in this machine rather than in anything the person did. {verb} is the name of a \
     thing an agent can do and is not translated.",
);

/// The application has an adapter of its own.
pub const HAS_ITS_OWN: Word = Word::saying(
    "adapters.fallback.has-its-own",
    "{application} has its own list of what the agent can do in it, so nothing was read or \
     pressed in its windows instead",
)
.noting(
    "Said when the agent tried to read or press something in an application that has its own, \
     narrower list of things an agent may do. {application} is its identifier and is not \
     translated. Nothing happened.",
);

/// No window of the application is on the screen.
pub const NO_WINDOW: Word = Word::saying(
    "adapters.fallback.no-window",
    "{application} has no window on the screen, so there was nothing to read or press",
)
.noting(
    "Said when the application is not running or none of its windows is showing. \
     {application} is its identifier and is not translated. Nothing happened.",
);

/// What applications show cannot be read on this machine.
pub const NOT_READABLE_HERE: Word = Word::saying(
    "adapters.fallback.not-readable-here",
    "What applications show could not be read on this machine just now, so nothing was read or \
     pressed in {application}",
)
.noting(
    "Said when the part of the machine that tells screen readers what is on the screen could \
     not be reached. {application} is its identifier and is not translated. Nothing happened.",
);

/// The application did not answer while being read.
pub const DID_NOT_ANSWER_READING: Word = Word::saying(
    "adapters.fallback.did-not-answer-reading",
    "{application} did not answer in time, so nothing was read or pressed in it",
)
.noting(
    "Said when the application stopped answering while what it shows was being read. \
     {application} is its identifier and is not translated. Nothing was pressed.",
);

/// No control of that kind and name is showing.
pub const NO_SUCH_CONTROL: Word = Word::saying(
    "adapters.fallback.no-such-control",
    "{application} shows no {kind} named “{name}” now, so nothing was pressed",
)
.noting(
    "Said when the control the person approved pressing is not on the screen when it would be \
     pressed. {kind} is already translated; {name} and {application} are not translated.",
);

/// More than one control of that kind and name is showing.
pub const MORE_THAN_ONE: Word = Word::saying(
    "adapters.fallback.more-than-one",
    "{application} shows more than one {kind} named “{name}”, so nothing was pressed rather \
     than guessing which",
)
.noting(
    "Said when two or more controls match what the person approved. The machine never guesses. \
     {kind} is already translated; {name} and {application} are not translated.",
);

/// Too much is showing to be sure which control is meant.
pub const TOO_MUCH_TO_BE_SURE: Word = Word::saying(
    "adapters.fallback.too-much-to-be-sure",
    "{application} shows more than can be read at once, so whether its {kind} named “{name}” is \
     the only one is not known, and nothing was pressed",
)
.noting(
    "Said when an application's windows are too large to read whole, so the machine cannot be \
     sure the control is the only one of its name. {kind} is already translated; {name} and \
     {application} are not translated.",
);

/// The control is greyed out.
pub const CANNOT_BE_USED_NOW: Word = Word::saying(
    "adapters.fallback.cannot-be-used-now",
    "The {kind} named “{name}” in {application} cannot be used right now, so it was not pressed",
)
.noting(
    "Said when the control is shown greyed out. {kind} is already translated; {name} and \
     {application} are not translated.",
);

/// The control offers no way to be pressed.
pub const CANNOT_BE_PRESSED: Word = Word::saying(
    "adapters.fallback.cannot-be-pressed",
    "The {kind} named “{name}” in {application} cannot be pressed from outside the application, \
     so nothing was pressed",
)
.noting(
    "Said when the application does not let the control be pressed except by a person. {kind} \
     is already translated; {name} and {application} are not translated.",
);

/// The application said the control was not pressed.
pub const DID_NOT_PRESS: Word = Word::saying(
    "adapters.fallback.did-not-press",
    "{application} did not press the {kind} named “{name}”",
)
.noting(
    "Said when the application itself answered that the control was not pressed. {kind} is \
     already translated; {name} and {application} are not translated.",
);

/// The application did not answer when the control was pressed.
pub const DID_NOT_ANSWER_PRESSING: Word = Word::saying(
    "adapters.fallback.did-not-answer-pressing",
    "{application} did not answer in time, so whether the {kind} named “{name}” was pressed is \
     not known",
)
.noting(
    "Said when the request to press reached the application and no answer came back. It may \
     have been pressed; say so plainly rather than as a failure. {kind} is already translated; \
     {name} and {application} are not translated.",
);

// ---------------------------------------------------------------------------
// What could not be read.
// ---------------------------------------------------------------------------

/// A control with no name.
pub const LIMIT_NO_NAME: Word = Word::saying(
    "adapters.fallback.limit.no-name",
    "a control with no name, which the agent can neither describe nor press",
)
.noting(
    "Said about one thing in a window that has no name, so the agent cannot say what it is or \
     ask to press it. The agent never guesses at it by where it is on the screen.",
);

/// An area the application draws itself.
pub const LIMIT_NOT_DESCRIBED: Word = Word::saying(
    "adapters.fallback.limit.not-described",
    "an area the application draws itself and does not describe, which the agent cannot read",
)
.noting(
    "Said about a part of a window, like a drawing or a game, whose contents the application \
     does not describe to screen readers. The agent never guesses at it by looking at the \
     screen.",
);

/// A part of the window another program shows.
pub const LIMIT_ANOTHER_PROGRAM: Word = Word::saying(
    "adapters.fallback.limit.another-program",
    "a part of the window shown by another program, which was not read",
)
.noting(
    "Said about a part of a window that a different program puts there, so it is not the \
     application the person granted.",
);

/// Not everything was read.
pub const NOT_ALL_READ: Word = Word::saying(
    "adapters.fallback.not-all-read",
    "{application} shows more than can be read at once, so not all of it was read",
)
.noting("Said beside what was read. {application} is its identifier and is not translated.");

/// Every word the fallback declares.
pub const EVERY_WORD: [Word; 33] = [
    READ_PURPOSE,
    READ_APPLICATION,
    READ_SENTENCE,
    READ_BY_HAND,
    PRESS_PURPOSE,
    PRESS_APPLICATION,
    PRESS_KIND,
    PRESS_NAME,
    PRESS_SENTENCE,
    PRESS_BY_HAND,
    A_BUTTON,
    A_CHECK_BOX,
    A_RADIO_BUTTON,
    A_SWITCH,
    A_MENU_ITEM,
    A_TAB,
    A_LINK,
    NOT_THIS_VERB,
    HAS_ITS_OWN,
    NO_WINDOW,
    NOT_READABLE_HERE,
    DID_NOT_ANSWER_READING,
    NO_SUCH_CONTROL,
    MORE_THAN_ONE,
    TOO_MUCH_TO_BE_SURE,
    CANNOT_BE_USED_NOW,
    CANNOT_BE_PRESSED,
    DID_NOT_PRESS,
    DID_NOT_ANSWER_PRESSING,
    LIMIT_NO_NAME,
    LIMIT_NOT_DESCRIBED,
    LIMIT_ANOTHER_PROGRAM,
    NOT_ALL_READ,
];
