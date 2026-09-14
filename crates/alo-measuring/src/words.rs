//! Every string this crate can say, and the English beside each one.
//!
//! A window that shows what is running shows numbers, and a number needs no
//! translation. What needs one is every place a number is **not** shown: a
//! process that ended, a count the kernel withheld, a line the file does not
//! have, a rate there has been no time for. Those are sentences a person reads
//! in place of a number, and each is declared here so that nothing reaches a
//! person without something having asked whether anybody translated it.
//!
//! # A note is part of the string
//!
//! A translator works alone, with no product in front of them. Where a sentence
//! cannot be translated from its own words — *the kernel* is a thing, *since
//! the last reading* is an interval the person chose — the note says so.

use alo_strings::{Key, Plural, Vocabulary};

/// One string a crate can say.
///
/// Re-exported because this crate's own files, and the tests that read this
/// list, name it as `crate::words::Word`.
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

    /// What names it, as written.
    #[must_use]
    pub const fn named(&self) -> &'static str {
        self.named
    }

    /// What a translator needs to know.
    #[must_use]
    pub const fn note(&self) -> &'static str {
        self.note
    }
}

// ---------------------------------------------------------------------------
// Why nothing could be measured — `crate::NotMeasured`.
// ---------------------------------------------------------------------------

/// The host this is running on is not alo OS.
pub const NOT_ON_THIS_HOST: Word = Word::saying(
    "measuring.not-on-this-host",
    "What is running can only be read on alo OS itself.",
)
.noting(
    "Said when this code runs somewhere that is not a Linux machine, which in practice is a \
     developer's laptop. Nobody using alo OS sees it. \"alo OS\" is a product name and is never \
     translated.",
);

/// The kernel's list of what is running could not be read.
pub const UNREADABLE: Word = Word::saying(
    "measuring.unreadable",
    "What is running could not be read from {at}.",
)
.noting(
    "{at} is the path of a file the kernel keeps, such as /proc/stat, and is never translated. \
     Said when that file is missing or cannot be opened, which on a working machine does not \
     happen.",
);

/// Two readings with no time between them.
pub const NO_INTERVAL: Word = Word::saying(
    "measuring.no-interval",
    "Two readings need time between them.",
)
.noting(
    "A rate, such as bytes per second or a share of the processor, is two readings divided by \
     the time between them, and this is said when that time was given as none at all. It is a \
     mistake in the code that asked rather than anything a person did.",
);

/// Two readings that are the same moment.
pub const SAME_MOMENT: Word = Word::saying(
    "measuring.same-moment",
    "Both readings are from the same moment, so nothing can be said about a rate.",
)
.noting(
    "Said when the kernel's own count of processor time did not move between two readings, \
     which means they were taken at the same instant or the same reading was given twice. Like \
     \"measuring.no-interval\", a mistake in the code that asked.",
);

// ---------------------------------------------------------------------------
// What a window shows in place of a number — `crate::Number`, `crate::Gone`.
// ---------------------------------------------------------------------------

/// A process that ended between two readings.
pub const GONE: Word = Word::saying("measuring.process.gone", "Ended since the last reading.")
    .noting(
        "Shown on the row of a process that was running at the earlier of two readings and was \
         not at the later one. \"The last reading\" is the earlier of the two, taken a moment \
         before. A phrase rather than a sentence: it stands in a table cell.",
    );

/// A process that began between two readings, so no rate can be given yet.
pub const NOT_YET: Word = Word::saying(
    "measuring.number.not-yet",
    "Started since the last reading.",
)
.noting(
    "Shown in place of a rate, such as bytes per second or a share of the processor, for a \
     process that did not exist at the earlier of two readings. The next reading will show a \
     number. A phrase rather than a sentence: it stands in a table cell.",
);

/// A number the kernel would not show.
pub const WITHHELD: Word =
    Word::saying("measuring.number.withheld", "The kernel did not show this.").noting(
        "Shown in place of a number that the kernel keeps for a process but would not let this \
     reader open, usually because the process belongs to another person on the machine. \"The \
     kernel\" is the core of the operating system and may be translated as such. A phrase rather \
     than a sentence: it stands in a table cell.",
    );

/// A number the kernel does not keep for this process.
pub const NOT_SAID: Word = Word::saying(
    "measuring.number.not-said",
    "Not counted for this process.",
)
.noting(
    "Shown in place of a number that the kernel's file simply does not have a line for. A thread \
     inside the kernel itself has no memory of its own to count, for instance. Not a fault, and \
     the phrase must not read as one. It stands in a table cell.",
);

// ---------------------------------------------------------------------------
// Whose network traffic a count is — `crate::Network`.
// ---------------------------------------------------------------------------

/// A network count that is this process's alone.
pub const NETWORK_THIS_PROCESS_ALONE: Word = Word::saying(
    "measuring.network.this-process-alone",
    "Network traffic counted for this process alone.",
)
.noting(
    "Shown beside a process's bytes sent and received when no other running process shares its \
     network, as with a sandboxed application in a network of its own. Read against \
     \"measuring.network.shared\", which is the other case.",
);

/// A network count shared with other processes.
///
/// The kernel counts network traffic per network namespace rather than per
/// process, and most processes share one; the count beside such a process is
/// the count for all of them together. The sentence says how many.
pub const NETWORK_SHARED: Counted = Counted {
    named: "measuring.network.shared",
    number: "others",
    one: "Network traffic counted together with one other process.",
    other: "Network traffic counted together with {others} other processes.",
    note: "Shown beside a process's bytes sent and received when the kernel's count covers this \
           process and {others} more, because they share one network. The number is how many \
           others, not how many in all.",
};

// ---------------------------------------------------------------------------
// What is filling a folder — `crate::Holding`, `crate::Counted`.
// ---------------------------------------------------------------------------

/// A file counted under another of its names.
pub const ELSEWHERE: Word = Word::saying(
    "measuring.filling.counted-elsewhere",
    "Counted under another of its names, {at}.",
)
.noting(
    "Shown beside a file that has more than one name on the disk (a hard link), on every name \
     but the one its bytes were counted under. {at} is the path of that other name and is never \
     translated. The file takes up its space once, so it is counted once. A phrase rather than \
     a sentence: it stands beside a size.",
);

/// A folder that could not be read.
pub const NOT_READ: Word = Word::saying(
    "measuring.filling.not-read",
    "Could not be read, so nothing inside it is counted.",
)
.noting(
    "Shown beside a folder the machine would not let this reader open, usually because it \
     belongs to another person on the machine. Its size is shown as unknown rather than as \
     nothing, and the phrase must not read as a fault of the person's. It stands beside a size.",
);

/// A folder on another filesystem.
pub const ANOTHER_FILESYSTEM: Word = Word::saying(
    "measuring.filling.another-filesystem",
    "On another filesystem, so what is inside it is not counted here.",
)
.noting(
    "Shown beside a folder where a different disk or a different part of the system is \
     attached: the kernel's own view of every process, a plugged-in drive, another machine's \
     files. What is inside it takes no space on the disk being counted. \"Filesystem\" is the \
     technical term and may be translated as such. It stands beside a size.",
);

/// A folder no count can finish.
pub const NOT_FINISHED: Word = Word::saying(
    "measuring.filling.not-finished",
    "The count stopped before everything inside it was seen.",
)
.noting(
    "Shown beside a folder that holds more things directly inside it than one count looks at, \
     which no count can list to its end, so its size is the size of what was seen and not of \
     the whole. A folder that is merely large, with more than that in subfolders, is counted \
     whole and never shows this. Read with \"measuring.filling.cut-short\", which is said once \
     about the whole count. It stands beside a size.",
);

/// The folder asked about could not be counted at all.
pub const NOT_COUNTED: Word = Word::saying(
    "measuring.filling.not-counted",
    "What is filling {at} could not be counted. {why}",
)
.noting(
    "Said when the folder a person asked about could not be opened at all: it is not there, it \
     is a file rather than a folder, or the machine refused. {at} is its path and is never \
     translated. {why} is a whole sentence saying which, already in the reader's language.",
);

/// A count with a folder no walk could list to its end.
pub const CUT_SHORT: Counted = Counted {
    named: "measuring.filling.cut-short",
    number: "most",
    one: "A folder under it holds more than one thing at a single level, so these sizes are not \
          the whole of it.",
    other: "A folder under it holds more than {most} things at a single level, so these sizes \
            are not the whole of it.",
    note: "Said once, above a tree of sizes, when a folder under the one counted holds more \
           things directly inside it than one count looks at, which no count can list to its \
           end. {most} is that limit; a folder holding more than it in subfolders is counted \
           whole. The sizes shown are true for what was seen and smaller than the whole, and \
           each such folder is marked with \"measuring.filling.not-finished\".",
};

/// Things whose names cannot be shown, left out of the count.
pub const UNNAMED: Counted = Counted {
    named: "measuring.filling.unnamed",
    number: "unnamed",
    one: "One thing whose name cannot be shown is not counted.",
    other: "{unnamed} things whose names cannot be shown are not counted.",
    note: "Said once, above a tree of sizes, when the folder holds files or folders whose names \
           contain characters a screen cannot show safely. They are left out rather than shown \
           under a name that could mislead. {unnamed} is how many.",
};

// ---------------------------------------------------------------------------
// The two verbs: what each does, what a person is shown, and what each
// argument is for. `crate::verbs` declares them from these.
// ---------------------------------------------------------------------------

/// What `what_is_running` does.
pub const WHAT_IS_RUNNING: Word = Word::saying(
    "measuring.verb.what-is-running.purpose",
    "list what is running on this machine and what each process is using",
)
.noting(
    "The one-sentence purpose of the verb, shown where a person reads what their agent can do. \
     \"What it is using\" is memory, processor, disk and network, each read from the kernel.",
);

/// What a person is shown when `what_is_running` runs. It is a read, so
/// nobody is asked to approve it — it is what the shell shows while it
/// happens.
pub const WHAT_IS_RUNNING_SENTENCE: Word = Word::saying(
    "measuring.verb.what-is-running.sentence",
    "list what is running and what it is using, read from {proc}",
)
.noting(
    "{proc} is the path of the kernel's own directory of what is running — on alo OS it is \
     \"/proc\" — and is never translated. It is in the sentence because it is the folder the \
     verb reads and the folder the person granted.",
);

/// Where `what_is_running` reads.
pub const WHAT_IS_RUNNING_PROC: Word = Word::saying(
    "measuring.verb.what-is-running.argument.proc",
    "the kernel's own directory of what is running, which on this machine is /proc",
)
.noting(
    "Shown beside the argument when an agent asks for it. \"/proc\" is a path and is never \
     translated.",
);

/// What `what_is_filling` does.
pub const WHAT_IS_FILLING: Word = Word::saying(
    "measuring.verb.what-is-filling.purpose",
    "count what is filling a folder, as a tree of sizes",
)
.noting(
    "The one-sentence purpose of the verb, shown where a person reads what their agent can do. \
     \"A tree of sizes\" is the folder, each thing inside it and its size, and so on down.",
);

/// What a person is shown when `what_is_filling` runs.
pub const WHAT_IS_FILLING_SENTENCE: Word = Word::saying(
    "measuring.verb.what-is-filling.sentence",
    "count what is filling {folder}",
)
.noting("{folder} is a path and is never translated. Nothing is changed by counting it.");

/// What `what_is_filling` counts.
pub const WHAT_IS_FILLING_FOLDER: Word = Word::saying(
    "measuring.verb.what-is-filling.argument.folder",
    "the folder to count",
)
.noting("Shown beside the argument when an agent asks for it.");

/// A verb that is not this crate's to answer.
pub const NOT_THIS_CRATES: Word = Word::saying(
    "measuring.verb.not-this-crates",
    "nothing here measures {verb} — it says what is running and what is filling a folder, and \
     anything else is somewhere else's to do",
)
.noting("{verb} is the name of a capability and is never translated.");

/// A verb performed without an argument it declares.
pub const MISSING: Word = Word::saying(
    "measuring.verb.missing",
    "{verb} was performed without {argument}, which it declares — this is a verb to write again, \
     not a call to make again",
)
.noting(
    "Both gaps are names in the code and are never translated. This sentence is read by whoever \
     wrote the verb rather than by whoever asked for it.",
);

/// Everything this crate can say in one sentence each.
pub const EVERY_WORD: [Word; 22] = [
    NOT_ON_THIS_HOST,
    UNREADABLE,
    NO_INTERVAL,
    SAME_MOMENT,
    GONE,
    NOT_YET,
    WITHHELD,
    NOT_SAID,
    NETWORK_THIS_PROCESS_ALONE,
    ELSEWHERE,
    NOT_READ,
    ANOTHER_FILESYSTEM,
    NOT_FINISHED,
    NOT_COUNTED,
    WHAT_IS_RUNNING,
    WHAT_IS_RUNNING_SENTENCE,
    WHAT_IS_RUNNING_PROC,
    WHAT_IS_FILLING,
    WHAT_IS_FILLING_SENTENCE,
    WHAT_IS_FILLING_FOLDER,
    NOT_THIS_CRATES,
    MISSING,
];

/// Everything this crate can say about a number of things.
pub const EVERY_COUNTED: [Counted; 3] = [NETWORK_SHARED, CUT_SHORT, UNNAMED];

/// Why this crate's list could not be declared.
#[derive(Debug, thiserror::Error)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A countable string that is not one.
    #[error(transparent)]
    Counted(#[from] alo_strings::PluralError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
///
/// [`WordsError`], which the list above cannot cause.
pub fn measuring_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
///
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared it
/// first said what that string is.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    for counted in EVERY_COUNTED {
        vocabulary.counts(
            Plural::counting(counted.key(), counted.number, counted.one, counted.other)?
                .noting(counted.note)?,
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::{EVERY_COUNTED, EVERY_WORD, WordsError, declare_into, measuring_words};
    use alo_strings::{Key, Vocabulary};

    /// Every key in this file is a key, checked here because a key written in
    /// this file cannot arrive from anywhere else.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(Key::named(word.named()), Ok(word.key()), "{}", word.named());
        }
        for counted in EVERY_COUNTED {
            assert_eq!(
                Key::named(counted.named()),
                Ok(counted.key()),
                "{}",
                counted.named()
            );
        }
    }

    /// A key names one string.
    #[test]
    fn the_list_declares_into_a_vocabulary_once() {
        assert_eq!(measuring_words().unwrap().how_many(), 25);
        let mut vocabulary = Vocabulary::empty();
        declare_into(&mut vocabulary).unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Every string here carries a note.** A translator with no product in
    /// front of them cannot tell that *the last reading* is a moment the
    /// person chose, or that *not counted* is not a fault.
    #[test]
    fn every_string_carries_a_note_for_whoever_translates_it() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
        for counted in EVERY_COUNTED {
            assert!(!counted.note().is_empty(), "{}", counted.named());
        }
    }

    /// Every key is under this crate's own area, so nothing here can collide
    /// with another crate's list by accident.
    #[test]
    fn every_key_is_this_crates_own() {
        for word in EVERY_WORD {
            assert!(
                word.named().starts_with("measuring."),
                "{} is not under this crate's area",
                word.named()
            );
        }
        for counted in EVERY_COUNTED {
            assert!(
                counted.named().starts_with("measuring."),
                "{}",
                counted.named()
            );
        }
    }
}
