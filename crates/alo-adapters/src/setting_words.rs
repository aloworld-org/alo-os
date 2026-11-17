//! What a person is told about **alo moving a slider or a number box**
//! (`accessible.set_value`, added 2026-11-17), in their language.
//!
//! In every sentence `{name}` is the control's name exactly as the application
//! shows it, `{application}` the application's identifier, and `{value}`,
//! `{least}` and `{most}` numbers written as digits; none is translated.

use alo_strings::Word;

/// The gap holding the number asked for.
pub const VALUE: &str = "value";

/// The gap holding the least the control takes.
pub const LEAST: &str = "least";

/// The gap holding the most the control takes.
pub const MOST: &str = "most";

/// What `accessible.set_value` is for.
pub const SET_PURPOSE: Word = Word::saying(
    "adapters.fallback.set-value.purpose",
    "move a slider or a number box in an application's window to a number",
)
.noting("What one thing the agent can do is for, shown where its list of abilities is.");

/// What `application` is, for setting a value.
pub const SET_APPLICATION: Word = Word::saying(
    "adapters.fallback.set-value.application",
    "the application the slider or number box is in",
)
.noting("Shown beside the application the agent names when it asks to move a slider.");

/// What `name` is, for setting a value.
pub const SET_NAME: Word = Word::saying(
    "adapters.fallback.set-value.name",
    "the slider or number box, by the name it shows",
)
.noting("Shown beside the control the agent names, such as Volume or Brightness.");

/// What `value` is.
pub const SET_VALUE_WORD: Word = Word::saying(
    "adapters.fallback.set-value.value",
    "the whole number to move it to, inside the range it shows",
)
.noting("Shown beside the number the agent asks for.");

/// What a person approves.
pub const SET_SENTENCE: Word = Word::saying(
    "adapters.fallback.set-value.sentence",
    "move “{name}” in {application} to {value}",
)
.noting(
    "The sentence a person approves before anything moves. {name} is the control's name exactly \
     as the application shows it, {application} its identifier, and {value} a number in digits; \
     none is translated.",
);

/// What setting changes, on the approval card.
pub const SET_CHANGES: Word = Word::saying(
    "adapters.fallback.set-value.changes",
    "Moves that one slider or number box to that number, as a person moving it would, and then \
     reads it back.",
)
.noting("Shown on the card a person answers before alo moves a slider, beside a tick.");

/// What setting leaves alone.
pub const SET_LEAVES: Word = Word::saying(
    "adapters.fallback.set-value.leaves",
    "Nothing is typed, no other control is moved or pressed, a number outside the control's own \
     range is refused, and nothing is found by where it is on the screen.",
)
.noting("Shown beside the sentence about what setting changes.");

/// No slider or number box of that name is on the screen.
pub const NO_SUCH_CONTROL: Word = Word::saying(
    "adapters.fallback.set-value.no-such-control",
    "{application} shows no slider or number box named “{name}” now, so nothing was moved",
)
.noting("Said when the control the person approved is not on the screen. Nothing happened.");

/// More than one has that name.
pub const MORE_THAN_ONE: Word = Word::saying(
    "adapters.fallback.set-value.more-than-one",
    "{application} shows more than one “{name}”, so alo did not guess which, and nothing was \
     moved",
)
.noting("Said when two controls on the screen have the same name. Nothing happened.");

/// The window is too large to be sure.
pub const TOO_MUCH_TO_BE_SURE: Word = Word::saying(
    "adapters.fallback.set-value.too-much-to-be-sure",
    "{application} shows more than can be read at once, so whether “{name}” is the only one is \
     not known, and nothing was moved",
)
.noting("Said when a window is too large to read whole. Nothing happened.");

/// It is greyed out.
pub const CANNOT_BE_USED_NOW: Word = Word::saying(
    "adapters.fallback.set-value.cannot-be-used-now",
    "“{name}” in {application} is greyed out, so nothing was moved",
)
.noting("Said when the control cannot be used at the moment. Nothing happened.");

/// It only shows a number: a progress bar or a meter.
pub const ONLY_SHOWS_A_NUMBER: Word = Word::saying(
    "adapters.fallback.set-value.only-shows-a-number",
    "“{name}” in {application} only shows how far something has got, and cannot be moved, so \
     nothing was moved",
)
.noting(
    "Said when the person named a progress bar or a meter, which shows a number nobody can \
     change. Nothing happened.",
);

/// It says no range, so it cannot be moved from outside.
pub const CANNOT_BE_MOVED: Word = Word::saying(
    "adapters.fallback.set-value.cannot-be-moved",
    "{application} does not let “{name}” be moved from outside the application, so nothing was \
     moved. Move it yourself in the window",
)
.noting(
    "Said when the application describes the control but offers no way for alo to move it. \
     Nothing happened.",
);

/// The number is outside the range.
pub const OUTSIDE_ITS_RANGE: Word = Word::saying(
    "adapters.fallback.set-value.outside-its-range",
    "“{name}” in {application} goes from {least} to {most}, so {value} is outside it, and \
     nothing was moved",
)
.noting("Said when the number asked for is outside what the control takes. Nothing happened.");

/// The application refused the number.
pub const DID_NOT_TAKE: Word = Word::saying(
    "adapters.fallback.set-value.did-not-take",
    "{application} did not take {value} for “{name}”, so nothing was moved",
)
.noting("Said when the application refused. Nothing happened.");

/// The application did not answer.
pub const DID_NOT_ANSWER: Word = Word::saying(
    "adapters.fallback.set-value.did-not-answer",
    "{application} did not answer in time, so whether “{name}” moved to {value} is not known. \
     Look at the window",
)
.noting(
    "Said after alo asked, when no answer came back. It may have moved; say so plainly rather \
     than as a failure.",
);

/// It reads back as another number.
pub const READS_BACK_DIFFERENTLY: Word = Word::saying(
    "adapters.fallback.set-value.reads-back-differently",
    "alo asked {application} to move “{name}” to {value}, and it shows a different number now. \
     Look at the window before going on",
)
.noting(
    "Said after alo asked, when it read the control back and found another number — an \
     application may round to its own steps. alo does not claim it worked.",
);

/// Every word this file declares.
pub const EVERY_WORD: [Word; 17] = [
    SET_PURPOSE,
    SET_APPLICATION,
    SET_NAME,
    SET_VALUE_WORD,
    SET_SENTENCE,
    SET_CHANGES,
    SET_LEAVES,
    NO_SUCH_CONTROL,
    MORE_THAN_ONE,
    TOO_MUCH_TO_BE_SURE,
    CANNOT_BE_USED_NOW,
    ONLY_SHOWS_A_NUMBER,
    CANNOT_BE_MOVED,
    OUTSIDE_ITS_RANGE,
    DID_NOT_TAKE,
    DID_NOT_ANSWER,
    READS_BACK_DIFFERENTLY,
];
