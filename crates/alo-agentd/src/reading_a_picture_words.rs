//! What a person, and the reader, are told when **a picture alo took is put to
//! the reader the person chose** (ADR 0147 §4). `{application}` is the
//! application's identifier and `{provider}` the service's own name; neither is
//! translated.

use alo_strings::Word;

/// What the reader is asked about the picture.
///
/// Translated like every other sentence: the reader is asked in the person's
/// language, so it answers in it.
pub const WHAT_THE_READER_IS_ASKED: Word = Word::saying(
    "agentd.picture.what-the-reader-is-asked",
    "This is a picture of one window of the application {application}. Say what the window \
     shows: every piece of text in it, and every thing a person could press or choose, with what \
     it says. For each, say where its centre is as two whole numbers from 0 to 100: how many per \
     cent of the way across from the left edge, then down from the top edge. Say only what is in the picture, and say \
     so where something cannot be read.",
)
.noting(
    "The question put to an AI model together with a picture of a window, so that the model \
     describes the window in words. The model answers in the language of this question. \
     {application} is the application's identifier and is not translated.",
);

/// What the reader is asked about the picture, where the call said what it
/// is to answer.
pub const WHAT_THE_READER_IS_ASKED_TO_ANSWER: Word = Word::saying(
    "agentd.picture.what-the-reader-is-asked-to-answer",
    "{asked} What is wanted from this picture: {question}",
)
.noting(
    "Put to an AI model with a picture of a window. {asked} is the whole sentence \
     agentd.picture.what-the-reader-is-asked, already translated; {question} is what the \
     person wants to know, in their own words, and is not translated.",
);

/// The person's own model cannot be sent a picture yet.
pub const NOT_TO_THIS_COMPUTER_YET: Word = Word::saying(
    "agentd.picture.not-to-this-computer-yet",
    "alo cannot yet send a picture to your model on this computer, so the picture of \
     {application} was not read and nothing was sent. To have it read, choose a service under \
     “Who reads the pictures” in Settings → What alo can use",
)
.noting(
    "Said when the person chose their own model on this computer to read pictures. alo, the \
     assistant, cannot hand that model a picture yet. \"Who reads the pictures\" and \"What alo \
     can use\" are names in Settings: use exactly their translations there.",
);

/// The service chosen to read is not one the person added.
pub const THE_READER_IS_NOT_ADDED: Word = Word::saying(
    "agentd.picture.the-reader-is-not-added",
    "“{provider}”, chosen to read pictures, is not among the services added in alo and AI, so \
     the picture of {application} was not read and nothing was sent",
)
.noting(
    "Said when the service the person chose to read pictures has since been removed. \"alo and \
     AI\" is the name of a Settings page: use exactly its translation there.",
);

/// The service chosen to read names no model.
pub const THE_READER_HAS_NO_MODEL: Word = Word::saying(
    "agentd.picture.the-reader-has-no-model",
    "“{provider}”, chosen to read pictures, names no model to read them with, so the picture of \
     {application} was not read and nothing was sent. Add a model to it in alo and AI",
)
.noting(
    "Said when the service the person chose to read pictures lists no AI model. \"alo and AI\" \
     is the name of a Settings page: use exactly its translation there.",
);

/// Every word reading a picture declares.
pub const EVERY_WORD: [Word; 5] = [
    WHAT_THE_READER_IS_ASKED,
    WHAT_THE_READER_IS_ASKED_TO_ANSWER,
    NOT_TO_THIS_COMPUTER_YET,
    THE_READER_IS_NOT_ADDED,
    THE_READER_HAS_NO_MODEL,
];
