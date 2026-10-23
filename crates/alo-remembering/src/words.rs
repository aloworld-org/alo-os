//! Every string this crate can say, and the English beside each one.
//!
//! Ten of them, and all ten are about **one file**: the level a person chose
//! for running code, at `/var/lib/alo/how-code-runs.toml`. Nothing here says
//! anything about the **grants** beside it, and that absence is the whole
//! argument of this file.
//!
//! # Why the grants are not worded, and the level is
//!
//! `crate::NotRemembered` has twenty arms and they are reached by two different
//! roads with two different readers:
//!
//! | road | who reads the refusal |
//! |---|---|
//! | the grants, the pairings, the machine names | `alo-agentd`, at start, into a service log |
//! | the level a person chose for running code | **a person, on the Settings screen** |
//!
//! The grants road's arms already say so in their own documentation —
//! `NotAGrant` carries its sentence *"in the language this code is written in —
//! nobody reading a service log has a vocabulary loaded"*. A daemon that
//! refuses to start because a hand-edited grant names both an agent and an
//! application writes one line for whoever is debugging it, and that line is
//! correctly English.
//!
//! The level is different because a surface draws it.
//! `alo-shell`'s `how_code_runs_in_settings` holds a refusal it **cannot say**,
//! and `format!("{why:?}")` on a screen is worse than silence. So the six arms
//! that road can produce are worded here and the fourteen it cannot are not —
//! following `alo-choosing`'s rule in its own words: *a sentence for a refusal
//! that cannot happen is a string a translator was handed for nothing.*
//!
//! Which six, measured rather than assumed: `crate::believing` produces
//! `ALink`, `NotRead`, `NotThere`, `NotWritten`, `SomebodyElses` and
//! `WritableByOthers`, and `crate::how_code_runs` adds `NotTheLevel`.
//! `NotThere` is not among the worded, because on the level road it is not a
//! refusal at all — it is `Ok(None)`, a person who has not chosen.
//! `tests/every_refusal_a_person_can_meet_has_a_sentence.rs` holds the split,
//! so an arm that gains a road gains a sentence or fails a test.
//!
//! # They all name the file
//!
//! A path is data and is never translated, in the way a filename is not
//! translated in `alo-files` and an address is not in `alo-egress`. It is in
//! every one of them because it is the thing a person needs in order to act,
//! and this file is one of several on a machine.
//!
//! # Two endings, and they say opposite things
//!
//! **Eight** are about reading and end **so this machine does not know how you
//! want code run**. **Two** are about writing and end **so how code runs has
//! not been changed**, which says the opposite: whatever was chosen is still in
//! force and the thing that failed was the change. `alo-choosing` splits its
//! own sentences the same way and for the same reason.
//!
//! *This paragraph said six and four until the test that counts them was
//! written, which answered eight and two. The prose was wrong and the count is
//! asserted now — `every_sentence_says_what_follows_from_it` fails if either
//! number moves, so the two cannot drift apart again.*
//!
//! **Neither ending says what the machine does instead**, and that is
//! deliberate. `alo_capability::HowCodeRuns`' default is the sealed box, but
//! nothing in the product reads this file to decide a run yet — `alo-try` pins
//! the sealed box because the word `try` means one — so a sentence promising
//! *and so it runs in a sealed box* would be a claim about behaviour that is
//! not wired. Said plainly here so that whoever wires it adds the clause rather
//! than discovering the sentence was always half a promise.

use alo_strings::Word;

/// The file is a symbolic link, and is read only as a real file.
pub const LEVEL_A_LINK: Word = Word::saying(
    "remembering.level.a-link",
    "{path} is a link to somewhere else, and this is read only as a real file, so this machine \
     does not know how you want code run",
)
.noting(
    "{path} is a file on this machine and is never translated. Said of the file holding which of \
     three levels the person chose for running code. A link is refused because whoever can point \
     this name at a file they own decides what an agent may do on this machine. Not a mistake a \
     person makes by accident — say it as a fact about the file rather than as an accusation.",
);

/// The file could not be read.
pub const LEVEL_NOT_READ: Word = Word::saying(
    "remembering.level.not-read",
    "{path} could not be read, so this machine does not know how you want code run",
)
.noting(
    "{path} is a file on this machine and is never translated. A disk or a permission rather than \
     anything a person typed. The second half says what follows from it, which is the thing they \
     are about to notice.",
);

/// The file belongs to somebody other than root or the login reading it.
pub const LEVEL_SOMEBODY_ELSES: Word = Word::saying(
    "remembering.level.somebody-elses",
    "{path} belongs to somebody else, so this machine does not know how you want code run",
)
.noting(
    "{path} is a file on this machine and is never translated. This file says how much of this \
     machine an agent may use, so one its owner could not have written is refused rather than \
     believed. Translate 'belongs to somebody else' as ownership of the file, not as blame.",
);

/// The file can be written by its group or by the world.
pub const LEVEL_WRITABLE_BY_OTHERS: Word = Word::saying(
    "remembering.level.writable-by-others",
    "{path} can be changed by others on this machine, so this machine does not know how you want \
     code run",
)
.noting(
    "{path} is a file on this machine and is never translated. The mode is deliberately not shown: \
     a person acts on the fact that others can write it, and the octal number is for whoever is \
     debugging. This is the refusal that matters most on this file — somebody who could write it \
     could grant an agent full trust.",
);

/// The file is there and says no level.
pub const LEVEL_SAYS_NOTHING: Word = Word::saying(
    "remembering.level.says-nothing",
    "{path} is there and says no level, so this machine does not know how you want code run",
)
.noting(
    "{path} is a file on this machine and is never translated. Said of a file somebody edited into \
     saying nothing — not of a machine where nobody has chosen, which has no file and is told \
     nothing at all. The difference is the point: this person had an answer in the file and it is \
     gone.",
);

/// A level spelt in a way this does not know.
pub const LEVEL_NO_SUCH_LEVEL: Word = Word::saying(
    "remembering.level.no-such-level",
    "{path} says {said}, which is not one of the three levels, so this machine does not know how \
     you want code run",
)
.noting(
    "{path} and {said} are both taken from the file exactly as written and are never translated. \
     The three level words are not translated either: they are what the file holds. Nothing is \
     guessed at — a file saying nearly the right word is refused rather than read as the nearest \
     one, because guessing would be choosing for the person.",
);

/// The file is not the shape this file takes.
pub const LEVEL_NOT_THE_SHAPE: Word = Word::saying(
    "remembering.level.not-the-shape",
    "{path} is not the shape this file takes, so this machine does not know how you want code run",
)
.noting(
    "{path} is a file on this machine and is never translated. Said of a file that is not TOML, or \
     is TOML of the wrong shape. The parser's own words are not shown to a person: they name rows \
     and columns in a file most people did not open.",
);

/// Written for an alo OS this is not.
pub const LEVEL_FROM_A_NEWER_ALO_OS: Word = Word::saying(
    "remembering.level.from-a-newer-alo-os",
    "{path} was written by a newer alo OS than this one, so this machine does not know how you \
     want code run",
)
.noting(
    "{path} is a file on this machine and is never translated. Refused before the level in it is \
     looked at, because a later alo OS may have a level this one has no word for. The format \
     numbers are not shown: what a person can act on is that the file is from elsewhere.",
);

/// The level could not be written down.
pub const LEVEL_NOT_WRITTEN_DOWN: Word = Word::saying(
    "remembering.level.not-written-down",
    "the level you chose could not be written down, so how code runs has not been changed",
)
.noting(
    "No path, because this is a fault in alo OS rather than anything about a file — the level \
     would not turn into text. The ending says the opposite of the reading refusals above: \
     whatever was chosen before is still in force.",
);

/// The file could not be written.
pub const LEVEL_NOT_WRITTEN: Word = Word::saying(
    "remembering.level.not-written",
    "{path} could not be written, so how code runs has not been changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A disk, a permission, or a folder \
     that is not there — which is refused rather than made. The ending says whatever was chosen \
     before is still in force, which is the opposite of what the reading refusals say.",
);

/// Every word this crate can say.
pub const EVERY_WORD: [Word; 10] = [
    LEVEL_A_LINK,
    LEVEL_NOT_READ,
    LEVEL_SOMEBODY_ELSES,
    LEVEL_WRITABLE_BY_OTHERS,
    LEVEL_SAYS_NOTHING,
    LEVEL_NO_SUCH_LEVEL,
    LEVEL_NOT_THE_SHAPE,
    LEVEL_FROM_A_NEWER_ALO_OS,
    LEVEL_NOT_WRITTEN_DOWN,
    LEVEL_NOT_WRITTEN,
];

/// Why a word of this crate's would not go into a vocabulary.
///
/// English, with a `Display`, for every other crate's reason: whoever reads it
/// is whoever is fixing the list.
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
/// Its own vocabulary rather than only [`declare_into`], because
/// `alo-saying`'s `the_machine_says_what_the_crates_say_between_them` sums
/// every crate's own count and compares it to the machine's — which is how a
/// crate registered in one place and forgotten in another is caught. It caught
/// this one.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn remembering_words() -> Result<alo_strings::Vocabulary, WordsError> {
    let mut vocabulary = alo_strings::Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared it
/// first said what that string is.
pub fn declare_into(vocabulary: &mut alo_strings::Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "words_tests.rs"]
mod tests;
