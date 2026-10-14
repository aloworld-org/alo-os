//! Every string this crate can say, and the English beside each one.
//!
//! `CLAUDE.md` says hardcoded English is a bug. This is the list that stops it
//! being one here: a key, the sentence in the language the code is written in,
//! and the note a translator needs. `alo-strings` does the rest.
//!
//! # These four words carry the product's central claim
//!
//! They are not labels in the ordinary sense. The gutter is the one place in
//! the terminal a program cannot write, and these are the only words it ever
//! shows — so a person reading `program` is being told something they are meant
//! to be able to rely on, and a person reading `alo` or `system` is being told
//! something a program is not able to say about itself.
//!
//! Two consequences for a translator, and both are in the notes:
//!
//! **They are one word each, in a narrow column.** The design gives the label
//! 80 of the gutter's 96 (`measures::THE_LABELS_ROOM`). A translation that
//! needs a phrase has nowhere to put it, so the note asks for the shortest
//! truthful word in the language rather than the most literal one.
//!
//! **They are lowercase because the design draws them lowercase** — `you`,
//! `alo`, `system`, `program`, read off the transcript node itself. In a
//! language where case carries meaning, the language wins: the lowercase here
//! is English typography, not a rule about the word.

use alo_strings::{Vocabulary, Word};

/// A program's own output.
pub const A_PROGRAM: Word = Word::saying("terminal.who.program", "program").noting(
    "The word in the terminal's gutter beside output a program produced. One word, in a column \
     80 units wide, so the shortest truthful word is better than the most literal one. It is \
     shown so a person can tell a program's own text apart from alo's, which is why it must not \
     be translated as something that could also describe alo — \"output\" or \"result\" would \
     both lose the whole point of the word.",
);

/// alo's own interpretation.
pub const ALO: Word = Word::saying("terminal.who.alo", "alo").noting(
    "The word in the terminal's gutter beside something alo itself wrote. \"alo\" is the \
     product's name and is not translated; it is lowercase everywhere it appears. The note \
     exists so that a translator does not translate it by mistake.",
);

/// An attributed observation, with its scope and the time it was made.
pub const THE_SYSTEM: Word = Word::saying("terminal.who.system", "system").noting(
    "The word in the terminal's gutter beside an observation something measured -- a port that \
     was listened on, a permission that was read -- rather than something alo worked out. One \
     word, in a column 80 units wide. It means the machine itself observed this, so a word \
     meaning \"the operating system\" is right and a word meaning \"automatic\" is not.",
);

/// What the person submitted.
pub const THE_PERSON: Word = Word::saying("terminal.who.you", "you").noting(
    "The word in the terminal's gutter beside what the person themselves typed or submitted. \
     Addressed to the reader, so the second person: it says \"you\" to the person reading it, \
     not \"the user\". Where a language distinguishes formal and familiar address, alo uses the \
     one it uses elsewhere for the person who owns the machine. One word, in a column 80 units \
     wide.",
);

/// A command parked on a read from a terminal.
pub const WAITING_FOR_THE_KEYBOARD: Word =
    Word::saying("terminal.waiting.keyboard", "waiting for the keyboard").noting(
        "Said about a command that looks stuck. It is parked reading from the terminal, so it is \
         waiting for the person to type something. Not an error and not a fault: the sentence \
         exists so that somebody does not stop a command that was only waiting for them.",
    );

/// A command parked on a read from a socket.
pub const WAITING_FOR_THE_NETWORK: Word =
    Word::saying("terminal.waiting.network", "waiting for the network").noting(
        "Said about a command that looks stuck. It is parked reading from a socket, so it is \
         waiting for something on a network or on another machine. The phrase means whatever it \
         is talking to, not this machine's own connection being down.",
    );

/// A command parked on another process, or on a lock one holds.
pub const WAITING_FOR_ANOTHER_PROCESS: Word = Word::saying(
    "terminal.waiting.another-process",
    "waiting for another program, or for a lock",
)
.noting(
    "Said about a command that looks stuck. It is waiting for a different program to finish, to \
     write something, or to let go of a lock on a file. One sentence covers all three because \
     the evidence cannot separate them, and inventing a distinction would be a guess.",
);

/// A command that is not waiting at all.
pub const STILL_COMPUTING: Word = Word::saying("terminal.waiting.computing", "still computing")
    .noting(
        "Said about a command that looks stuck and is not: the machine has it on a processor and \
         it is working. This is a measurement rather than a shrug -- the kernel says the command \
         is running -- and it is the sentence that stops somebody killing something nearly done.",
    );

/// Nothing found settles it.
pub const NOBODY_CAN_TELL: Word = Word::saying(
    "terminal.waiting.not-known",
    "waiting for something alo cannot identify",
)
.noting(
    "Said about a command that looks stuck when the evidence does not settle why. It must not be \
     translated as an error or as a failure to look: alo did look, and what it found does not \
     answer the question. A translation that reads like \"unknown error\" would say something \
     this does not.",
);

/// Every string this crate can say, in the order this file declares them.
///
/// The array is what a test reads down and what [`declare_into`] walks, so a
/// word declared above and left out here is a string nothing can look up.
pub const EVERY_WORD: [Word; 9] = [
    A_PROGRAM,
    ALO,
    THE_SYSTEM,
    THE_PERSON,
    WAITING_FOR_THE_KEYBOARD,
    WAITING_FOR_THE_NETWORK,
    WAITING_FOR_ANOTHER_PROCESS,
    STILL_COMPUTING,
    NOBODY_CAN_TELL,
];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads — it is read by whoever is fixing the list
/// above, so it keeps its English and its `Display`.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn terminal_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// The shell has one vocabulary and every crate adds its own to it, which is
/// what the area at the front of a key is for.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared it
/// first said what that string is.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "words_tests.rs"]
mod tests;
