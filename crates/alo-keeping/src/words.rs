//! Every string this crate can say, and the English beside each one.
//!
//! `CLAUDE.md` says hardcoded English is a bug. This is the list that stops it
//! being one here: a key, the sentence in the language the code is written in,
//! and the note a translator needs. `alo-strings` does the rest.
//!
//! It is the seventh list of its kind, and the first written by a crate that
//! did not exist before `alo-strings` did — so there was never any English here
//! to move, and nothing in this crate has ever had a `Display`.
//!
//! # Nothing here says a machine did nothing
//!
//! Two of these sentences are the reason the crate exists. A record that has
//! been shortened and a record that was never written look identical from the
//! outside: both are short. So [`crate::Head`] says which of the two it is,
//! [`crate::Damage`] says when part of one could not be read, and both are
//! sentences a person is shown rather than a flag on a struct somebody may or
//! may not draw. A record that quietly lost its first six months would answer
//! *what did the agent do in March* with *nothing*, and a person would believe
//! it.
//!
//! # What the machine said, four times rather than once with a gap
//!
//! `alo-files` says *{path} could not be {doing}* and fills `{doing}` with an
//! English word — *read*, *listed*, *renamed* — inside an otherwise translated
//! sentence. That is one string instead of six, and it is a small hole in
//! *hardcoded English is a bug*.
//!
//! This crate does four things to a file and no more, so there are four whole
//! sentences and no gap holding an English verb. It is `alo-egress`' decision
//! from item 9h met again: the indicator line is three sentences rather than a
//! stem with a place glued on, because the join is not something a program can
//! pick for a language it does not know. `{why}` stays, because it is what the
//! operating system itself said and is not ours to translate.

use alo_strings::{Key, Plural, Vocabulary};

/// One string a crate can say.
///
/// Lifted into `alo-strings` by item 9d. Re-exported here because this crate's
/// own files, and the tests that read this list, name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

/// One string this crate can say about a number of things.
///
/// Separate from [`Word`] because a countable string is declared and looked up
/// differently: two English sentences rather than one, and the reader's own
/// language decides which of *its* forms is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counted {
    /// What names it.
    named: &'static str,
    /// The gap the number goes in.
    number: &'static str,
    /// What it says about one thing.
    one: &'static str,
    /// What it says about any other number of things.
    other: &'static str,
    /// What a translator needs to know.
    note: &'static str,
}

impl Counted {
    /// What names it.
    #[must_use]
    pub fn key(&self) -> Key {
        Key::unchecked(self.named)
    }
}

// ---------------------------------------------------------------------------
// How long the record is kept — [`crate::Keeping`].
//
// The rule an organisation or a person named (ADR 0004), said back to them.
// ---------------------------------------------------------------------------

/// Nothing ages out.
pub const FOREVER: Word = Word::saying(
    "keeping.forever",
    "everything this machine's agent has done is kept",
)
.noting(
    "What a machine is set to, shown where somebody can change it. Present tense and plain: this is \
     the setting that keeps the most, and it is what a machine ships with.",
);

/// Anything older than this many days is removed.
pub const FOR_DAYS: Counted = Counted {
    named: "keeping.for-days",
    number: "days",
    one: "what this machine's agent has done is kept for one day, and then removed",
    other: "what this machine's agent has done is kept for {days} days, and then removed",
    note: "{days} is a number of whole days. The second half matters as much as the first: somebody \
           reading this is deciding how much evidence their machine will still have when they come \
           to ask a question of it.",
};

/// A record kept for no time at all.
pub const NO_DAYS_AT_ALL: Word = Word::saying(
    "keeping.no-days-at-all",
    "say how many days this record is kept for, or that it is kept for good — no days at all is a \
     machine that keeps no evidence of what its agent did",
)
.noting(
    "Read by somebody who typed 0 into a setting. It is not a scolding: the second half says what \
     the number they chose would actually mean, because it does not read like a way of turning the \
     record off and that is what it is.",
);

// ---------------------------------------------------------------------------
// Where the record starts — [`crate::Head`].
//
// The first line of the file, and the one thing in it that pruning may never
// remove.
// ---------------------------------------------------------------------------

/// Nothing has ever been removed.
pub const WHOLE: Word = Word::saying(
    "keeping.record.whole",
    "this is everything that has happened on this machine — nothing has been removed from it",
)
.noting(
    "Shown above a record somebody is reading. The claim is the point: it is what makes the other \
     sentence, about a record that has been shortened, worth anything.",
);

/// Something has.
pub const SHORTENED: Word = Word::saying(
    "keeping.record.shortened",
    "this record does not go all the way back — what happened before it starts has aged out and \
     been removed",
)
.noting(
    "The moment it starts at is a date, shown beside this sentence rather than inside it, because \
     how a date is written belongs to the reader's region and not to their language. Read by \
     somebody who may be about to conclude that nothing happened.",
);

// ---------------------------------------------------------------------------
// What could not be read back — [`crate::Damage`].
//
// Nought, one or two whole sentences, drawn one under another. Never joined:
// the conjunction is not punctuation a program can pick (item 9c).
// ---------------------------------------------------------------------------

/// Lines that are there and cannot be read.
pub const UNREADABLE: Word = Word::saying(
    "keeping.damage.unreadable",
    "part of what was written here cannot be read, so this record is not all of what happened — \
     nothing more will be removed from it until somebody has looked at it",
)
.noting(
    "The strongest sentence in this list, and it should read that way: lines in the middle of a \
     record do not become unreadable by accident very often. How many, and which, are numbers \
     shown beside it.",
);

/// The last line, cut off partway.
pub const UNFINISHED: Word = Word::saying(
    "keeping.damage.unfinished",
    "the last thing written here did not finish being written, which is what a machine losing \
     power in the middle of something looks like",
)
.noting(
    "Ordinary rather than alarming, and different from the sentence above it: one entry was being \
     written when the machine stopped. Everything written before it is intact.",
);

// ---------------------------------------------------------------------------
// Why there is no record to write to, or to read — [`crate::NotKept`].
//
// Every one of these is read by somebody holding a machine that is not keeping
// evidence, so each says what state the record is in and not only what failed.
// ---------------------------------------------------------------------------

/// Nothing is at that path.
pub const NOT_THERE: Word = Word::saying(
    "keeping.not-there",
    "there is no record at {path} — a machine with no record is not a machine that has done \
     nothing, so this is worth finding out about before it is replaced with an empty one",
)
.noting(
    "{path} is a place on this machine's own disk and is never translated. The second half is the \
     whole point of the sentence: the obvious response to a missing record is to make a new one, \
     and that is how a deleted record becomes an innocent one.",
);

/// Something is there, and it is not a record.
pub const NOT_A_RECORD: Word = Word::saying(
    "keeping.not-a-record",
    "{path} is not an alo OS record, so nothing has been written to it — every record begins with \
     a line saying what it is",
)
.noting("{path} is a place on this machine's own disk and is never translated.");

/// A record from a version of alo OS this one does not know.
pub const FROM_A_NEWER_ALO: Word = Word::saying(
    "keeping.from-a-newer-alo",
    "the record at {path} was written by a newer alo OS than this one, so nothing has been written \
     to it — adding to a record in a shape this version does not know would leave one neither \
     version can read",
)
.noting(
    "{path} is a place on this machine's own disk and is never translated. Which shape it is in is \
     a number, shown beside this sentence rather than inside it.",
);

/// A record with unreadable lines in it, which is never shortened.
pub const DAMAGED: Word = Word::saying(
    "keeping.damaged",
    "nothing was removed from the record at {path}, because part of it cannot be read — a record \
     with something wrong in it is looked at before it is shortened, not tidied up",
)
.noting(
    "{path} is a place on this machine's own disk and is never translated. This is a refusal to do \
     something destructive, so it reads as a decision rather than as a failure.",
);

/// The record could not be opened at all.
pub const NOT_OPENED: Word = Word::saying(
    "keeping.could-not-be-opened",
    "the record at {path} could not be opened, so nothing this machine's agent does can be written \
     down — {why}",
)
.noting(
    "{path} is a place on this machine's own disk. {why} is what the operating system said, which \
     arrives in whatever language it speaks and is not ours to translate. Neither is translated.",
);

/// One entry could not be added.
pub const NOT_ADDED_TO: Word = Word::saying(
    "keeping.could-not-be-added-to",
    "something that happened could not be added to the record at {path}, so the record is no \
     longer all of what happened — {why}",
)
.noting(
    "{path} and {why} are as in the sentence about a record that could not be opened. The middle \
     clause is what a person needs: from this moment the record is incomplete, and a record nobody \
     knows is incomplete is worse than no record.",
);

/// The record could not be read back.
pub const NOT_READ: Word = Word::saying(
    "keeping.could-not-be-read",
    "the record at {path} could not be read — {why}",
)
.noting(
    "{path} and {why} are as in the sentence about a record that could not be opened. Shorter than \
     the others because nothing has changed: what happened is still written down, and this is \
     about the reading of it.",
);

/// The record could not be shortened.
pub const NOT_SHORTENED: Word = Word::saying(
    "keeping.could-not-be-shortened",
    "the record at {path} could not be shortened, and nothing was removed from it — {why}",
)
.noting(
    "{path} and {why} are as in the sentence about a record that could not be opened. \"Nothing was \
     removed\" is the reassuring half and is true: a shortening that fails partway leaves the \
     record as it was.",
);

/// Every plain string this crate can say, in the order this file declares them.
///
/// The array is what a test reads down and what [`declare_into`] walks, so a
/// word declared above and left out here is a string nothing can look up. The
/// one countable string is not here — it is declared beneath, because it is
/// declared differently.
pub const EVERY_WORD: [Word; 14] = [
    FOREVER,
    NO_DAYS_AT_ALL,
    WHOLE,
    SHORTENED,
    UNREADABLE,
    UNFINISHED,
    NOT_THERE,
    NOT_A_RECORD,
    FROM_A_NEWER_ALO,
    DAMAGED,
    NOT_OPENED,
    NOT_ADDED_TO,
    NOT_READ,
    NOT_SHORTENED,
];

/// Why this crate's own words could not be declared.
///
/// None of these can happen to the list above — the tests at the bottom of this
/// file are what say so. It is a `Result` rather than an unwrap because a
/// library that panics on its own string table takes the daemon with it, and
/// because [`declare_into`] can genuinely fail against a vocabulary that
/// already holds one of these keys.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A countable string that could not be declared.
    #[error(transparent)]
    Counting(#[from] alo_strings::PluralError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn keeping_words() -> Result<Vocabulary, WordsError> {
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
    vocabulary.counts(
        Plural::counting(
            FOR_DAYS.key(),
            FOR_DAYS.number,
            FOR_DAYS.one,
            FOR_DAYS.other,
        )?
        .noting(FOR_DAYS.note)?,
    )?;
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// **What we ship is held to the rule everybody else is held to.**
    /// [`Word::key`] does not check, because a key written in this file cannot
    /// arrive from anywhere; this is the test that makes that true.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                Key::named(word.named()),
                Ok(word.key()),
                "{}: {}",
                word.named(),
                Key::named(word.named()).unwrap_err()
            );
        }
        assert_eq!(Key::named(FOR_DAYS.named), Ok(FOR_DAYS.key()));
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let mut named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert!(named.insert(FOR_DAYS.named));
        assert_eq!(named.len(), EVERY_WORD.len() + 1);
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "keeping", "{}", word.named());
        }
        assert_eq!(FOR_DAYS.key().area(), "keeping");
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it — which is the whole of what this file has to get right.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = keeping_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len() + 1);
        assert_eq!(vocabulary.counted().count(), 1);
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = keeping_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Every string here carries a note**, as `alo-appearance`'s do. A
    /// translator with no machine in front of them cannot tell that `{path}` is
    /// a place on a disk, that `{why}` is the operating system talking, or that
    /// the sentence about a shortened record is read by somebody who is about
    /// to conclude that nothing happened.
    #[test]
    fn every_string_this_crate_says_carries_a_note() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
        assert!(!FOR_DAYS.note.is_empty());
    }

    /// **No plain sentence here counts or dates anything.** `alo-models`
    /// settled the counting half in item 9f; the dating half is this crate's,
    /// because the one thing it would have had to write is the moment a
    /// shortened record starts at, and how a date is written belongs to the
    /// region rather than to the language.
    #[test]
    fn nothing_said_plainly_counts_or_dates_anything() {
        for word in EVERY_WORD {
            assert!(
                !word.says().chars().any(|char| char.is_ascii_digit()),
                "{}",
                word.named()
            );
            for gap in ["days", "since", "date", "how-many", "count", "format"] {
                assert!(
                    !word.says().contains(&format!("{{{gap}}}")),
                    "{}: {gap}",
                    word.named()
                );
            }
        }
    }

    /// **The one that counts is the only one that counts**, and it says what
    /// happens at the end of the time as well as how long it is — a sentence
    /// that said only *kept for 30 days* leaves the reader to guess whether the
    /// thirty-first day removes anything.
    #[test]
    fn the_countable_string_says_what_happens_when_the_time_is_up() {
        assert!(FOR_DAYS.other.contains("{days}"));
        assert!(FOR_DAYS.one.contains("one day"));
        for form in [FOR_DAYS.one, FOR_DAYS.other] {
            assert!(form.contains("removed"), "{form}");
        }
    }

    /// **Every sentence about a record that could not be kept names the file.**
    /// A person told only that something failed cannot tell whether it is this
    /// machine's record or a copy of somebody else's.
    #[test]
    fn every_refusal_about_a_record_names_the_file_it_is_about() {
        for word in [
            NOT_THERE,
            NOT_A_RECORD,
            FROM_A_NEWER_ALO,
            DAMAGED,
            NOT_OPENED,
            NOT_ADDED_TO,
            NOT_READ,
            NOT_SHORTENED,
        ] {
            assert!(word.says().contains("{path}"), "{}", word.named());
        }
    }
}
