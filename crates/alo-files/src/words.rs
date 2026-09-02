//! Every string this crate can say, and the English beside each one.
//!
//! `CLAUDE.md` says hardcoded English is a bug. What that means in practice is
//! not that English disappears — somebody has to write the first sentence, and
//! it is written in the language this code is written in — but that **no
//! sentence reaches a person without something having asked whether anybody
//! translated it**. `alo-strings` is the machinery for that; this file is what
//! this crate hands it.
//!
//! It is the same shape as `alo-shortcuts`' default bindings and
//! `alo-appearance`'s shipped wallpaper: what we ship lives in the code, and
//! what somebody else contributes is the difference. Here the difference is a
//! translation.
//!
//! # Why it is one file and not one sentence per file
//!
//! A translator reads this list, not this crate. The keys sort together, the
//! notes sit beside the sentences they are about, and *what alo OS says about
//! files* is one thing that changes for one reason. The variants and the verbs
//! that these sentences belong to are elsewhere, and they change for their own
//! reasons: a new way for a disk to say no is a change to [`crate::Failed`] and
//! a line here, and neither of those is the other.
//!
//! **The six verbs' words are declared from here, not copied from here.**
//! [`crate::verbs`] builds each declaration out of these constants, so the
//! English a person approves and the English a translator is given are the same
//! string rather than two strings a test hopes are equal. That matters more
//! than it looks: `alo_capability::Verb::checked` refuses a sentence that does
//! not name every argument, and the guarantee only survives translation if the
//! thing being translated is the sentence that was checked.
//!
//! # A note is part of the string
//!
//! A translator works alone, in a language nobody here reads, from a list of
//! sentences with no product in front of them. Where a sentence cannot be
//! translated from its own words — *most* means *at most*, `.zip` is not to be
//! translated, a name in a message came off somebody's disk — the note says so.
//! A note nobody wrote is a sentence somebody guesses at.

use alo_strings::{Key, Phrase, Plural, Vocabulary};

/// One string this crate can say.
///
/// The key and the English live together, because a key with its sentence
/// somewhere else is two files to change and one of them will be forgotten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Word {
    /// What names it.
    named: &'static str,
    /// What it says in the language the code is written in.
    says: &'static str,
    /// What a translator needs to know that the sentence does not tell them.
    note: Option<&'static str>,
}

impl Word {
    /// A string this crate can say.
    const fn saying(named: &'static str, says: &'static str) -> Self {
        Self {
            named,
            says,
            note: None,
        }
    }

    /// The same string, with something a translator has to be told.
    const fn noting(self, note: &'static str) -> Self {
        Self {
            note: Some(note),
            ..self
        }
    }

    /// What names it.
    ///
    /// [`Key::unchecked`], because these are literals in this file and
    /// [`every_key_is_a_key`](self#tests) walks every one of them through the
    /// checked door.
    #[must_use]
    pub fn key(&self) -> Key {
        Key::unchecked(self.named)
    }

    /// What it says in the language the code is written in.
    ///
    /// This is what [`crate::verbs`] declares a verb with, so that the sentence
    /// a person approves and the sentence a translator is handed cannot be two
    /// different sentences.
    #[must_use]
    pub const fn says(&self) -> &'static str {
        self.says
    }

    /// What a translator needs to know that the sentence does not tell them.
    #[must_use]
    pub const fn note(&self) -> Option<&'static str> {
        self.note
    }
}

/// One string this crate can say about a number of things.
///
/// Separate from [`Word`] because a countable string is declared and looked up
/// differently: two English sentences rather than one, and the reader's own
/// language decides which of *its* forms is shown. `alo-strings`' `cldr` module
/// is where that is decided; this is only the declaration.
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
// What the machine could not do — [`crate::Failed`].
// ---------------------------------------------------------------------------

/// A verb that is not one of the six.
pub const NOT_A_FILE_VERB: Word = Word::saying(
    "files.failed.not-a-file-verb",
    "nothing in the file half does {verb} — it lists, reads, finds, renames, moves and archives, \
     and anything else is somewhere else's to do",
)
.noting("{verb} is the name of a capability and is never translated.");

/// A verb performed without an argument it declares.
pub const MISSING: Word = Word::saying(
    "files.failed.missing",
    "{verb} was performed without {argument}, which it declares — this is a verb to write again, \
     not a call to make again",
)
.noting(
    "Both gaps are names in the code and are never translated. This sentence is read by whoever \
     wrote the verb rather than by whoever asked for it.",
);

/// Something is there, and it is not a folder.
pub const NOT_A_FOLDER: Word = Word::saying(
    "files.failed.not-a-folder",
    "{path} is not a folder — read it if it is a file, or list the folder it is in",
);

/// Something is there, and it is not a file.
pub const NOT_A_FILE: Word = Word::saying(
    "files.failed.not-a-file",
    "{path} is not a file — list it if it is a folder",
);

/// It was there when it was checked, and it is not there now.
pub const GONE: Word = Word::saying(
    "files.failed.gone",
    "{path} was there when it was checked and is not there now — nothing was done, so ask again",
);

/// A file too large to answer with.
///
/// The one countable string this crate has. As `alo-files` wrote it before
/// `alo-strings` existed it said *1 bytes* for a one-byte file; Polish says it
/// three ways and Irish five, and none of that is something a sentence with a
/// number stuck in it can express.
pub const TOO_BIG: Counted = Counted {
    named: "files.failed.too-big",
    number: "bytes",
    one: "{path} holds one byte and a verb reads at most {most} — open it in an application, or \
          ask for part of the folder instead",
    other: "{path} holds {bytes} bytes and a verb reads at most {most} — open it in an \
            application, or ask for part of the folder instead",
    note: "{most} is the largest a read answers with, in bytes: the sentence means \"at most this \
           many\". {bytes} is how big the file actually is.",
};

/// A file that is not text.
pub const NOT_TEXT: Word = Word::saying(
    "files.failed.not-text",
    "{path} is not text — this verb answers with what a person could read, and reading this one \
     needs an application that knows what it is",
);

/// Something is already at the name a change would create.
pub const ALREADY_THERE: Word = Word::saying(
    "files.failed.already-there",
    "there is already something at {path} — nothing here replaces a file that was not named, so \
     choose another name",
);

/// A file asked to move into the folder it is already in.
pub const ALREADY_IN: Word = Word::saying(
    "files.failed.already-in",
    "{path} is already in that folder — nothing was moved",
);

/// An archive asked to be written inside the folder it is an archive of.
pub const INTO_ITSELF: Word = Word::saying(
    "files.failed.into-itself",
    "the archive would be written inside {folder}, which is the folder it is an archive of — put \
     it somewhere that is not inside it",
);

/// An archive asked for under a name that does not say what it is.
pub const NOT_A_ZIP_NAME: Word = Word::saying(
    "files.failed.not-a-zip-name",
    "call the archive something ending in .zip — alo OS makes zip archives, and a name saying \
     otherwise would be a file whose name lies about what is in it",
)
.noting(
    "\".zip\" is what the name has to end in and is never translated. {name} is not in this \
     sentence on purpose: the name that was asked for is the thing being corrected, and repeating \
     it back reads like an instruction to use it.",
);

/// More in a folder than one archive holds.
pub const TOO_MANY: Word = Word::saying(
    "files.failed.too-many",
    "{folder} holds more than {most} things — archive one of the folders inside it instead, so \
     that what is in the archive is something a person can still recognise",
)
.noting("{most} is a count of files and folders. \"Things\" is deliberate: it is both.");

/// More bytes than one archive holds.
pub const TOO_MUCH: Word = Word::saying(
    "files.failed.too-much",
    "{folder} holds more than {most} bytes and one archive holds at most that — archive one of \
     the folders inside it instead",
)
.noting("{most} is a number of bytes.");

/// The machine said no, in its own words.
pub const THE_MACHINE_SAID_NO: Word = Word::saying(
    "files.failed.the-machine-said-no",
    "{path} could not be {doing} ({why}) — nothing else was attempted",
)
.noting(
    "{doing} is one word for what was being attempted — read, listed, renamed — and {why} is what \
     the operating system said, which arrives in whatever language it speaks and is not ours to \
     translate.",
);

// ---------------------------------------------------------------------------
// What could not be made real — [`crate::RealError`].
// ---------------------------------------------------------------------------

/// Nothing is at that path.
pub const NOTHING_THERE: Word = Word::saying(
    "files.real.nothing",
    "there is nothing at {path} — a verb reaches what is there, so name something that exists",
);

/// Something is there, and this machine would not say where it leads.
pub const UNREADABLE: Word = Word::saying(
    "files.real.unreadable",
    "{path} could not be followed ({why}) — nothing is done to a path this machine cannot resolve",
)
.noting(
    "\"Followed\" is about symbolic links: the question is where the path really leads. {why} is \
     the operating system's own words.",
);

// ---------------------------------------------------------------------------
// What this crate says when the grants refuse something.
//
// These two are the only sentences here that are not this crate's to show: they
// are handed to `alo_capability::Refused`, which carries them into the record.
// So they are the words a person reads *and* the words a security review reads
// afterwards, which is one rendering rather than two that could differ.
// ---------------------------------------------------------------------------

/// A path that is granted where it was written and leads somewhere that is not.
pub const REALLY_LEADS_ELSEWHERE: Word = Word::saying(
    "files.refused.really-leads-elsewhere",
    "{path} really leads to {really} — a grant covers where a file is, not where a link to it \
     sits, and {who} has not been granted where this one leads",
)
.noting(
    "{who} is the name of the agent that asked, which is written the way it was granted and is \
     not translated.",
);

/// A change that would put something where nothing is granted.
pub const WOULD_CREATE: Word = Word::saying(
    "files.refused.would-create",
    "{verb} would put something at {path}, and {why} — a grant covers where a file goes, not only \
     where it comes from",
)
.noting(
    "{verb} is the name of a capability and is never translated. {why} is the grants' own refusal, \
     already in the reader's language.",
);

// ---------------------------------------------------------------------------
// The six verbs: what each does, what a person approves, and what each argument
// is for. `crate::verbs` declares them from these.
// ---------------------------------------------------------------------------

/// What `list_folder` does.
pub const LIST_FOLDER: Word =
    Word::saying("files.verb.list-folder.purpose", "list what is in a folder");
/// What a person approves when `list_folder` runs. It is a read, so nobody is
/// asked to approve it — it is what the shell shows while it happens.
pub const LIST_FOLDER_SENTENCE: Word = Word::saying(
    "files.verb.list-folder.sentence",
    "list what is in {folder}",
);
/// `list_folder`'s only argument.
pub const LIST_FOLDER_FOLDER: Word = Word::saying(
    "files.verb.list-folder.argument.folder",
    "the folder to list",
);

/// What `read_file` does.
pub const READ_FILE: Word = Word::saying("files.verb.read-file.purpose", "read what is in a file");
/// What a person is shown when `read_file` runs.
pub const READ_FILE_SENTENCE: Word =
    Word::saying("files.verb.read-file.sentence", "read what is in {file}");
/// `read_file`'s only argument.
pub const READ_FILE_FILE: Word =
    Word::saying("files.verb.read-file.argument.file", "the file to read");

/// What `find_in_folder` does.
pub const FIND_IN_FOLDER: Word = Word::saying(
    "files.verb.find-in-folder.purpose",
    "find files in a folder by what they are called",
);
/// What a person is shown when `find_in_folder` runs.
pub const FIND_IN_FOLDER_SENTENCE: Word = Word::saying(
    "files.verb.find-in-folder.sentence",
    "find up to {most} files in {folder} whose name contains {named}",
)
.noting(
    "{named} is part of a file's name, not a pattern and not an expression: this verb interprets \
     nothing.",
);
/// Where `find_in_folder` looks.
pub const FIND_IN_FOLDER_FOLDER: Word = Word::saying(
    "files.verb.find-in-folder.argument.folder",
    "the folder to look in",
);
/// What `find_in_folder` looks for.
pub const FIND_IN_FOLDER_NAMED: Word = Word::saying(
    "files.verb.find-in-folder.argument.named",
    "the words a file's name has to contain",
);
/// How much `find_in_folder` answers with.
pub const FIND_IN_FOLDER_MOST: Word = Word::saying(
    "files.verb.find-in-folder.argument.most",
    "how many files to answer with at most",
);

/// What `rename_file` does.
pub const RENAME_FILE: Word = Word::saying(
    "files.verb.rename-file.purpose",
    "give a file a different name, where it already is",
);
/// **The sentence a person approves before a file is renamed.**
pub const RENAME_FILE_SENTENCE: Word =
    Word::saying("files.verb.rename-file.sentence", "rename {file} to {name}");
/// What `rename_file` renames.
pub const RENAME_FILE_FILE: Word =
    Word::saying("files.verb.rename-file.argument.file", "the file to rename");
/// What `rename_file` renames it to.
pub const RENAME_FILE_NAME: Word = Word::saying(
    "files.verb.rename-file.argument.name",
    "what to call it instead",
);

/// What `move_file` does.
pub const MOVE_FILE: Word =
    Word::saying("files.verb.move-file.purpose", "move a file into a folder");
/// **The sentence a person approves before a file is moved.**
pub const MOVE_FILE_SENTENCE: Word =
    Word::saying("files.verb.move-file.sentence", "move {file} into {into}");
/// What `move_file` moves.
pub const MOVE_FILE_FILE: Word =
    Word::saying("files.verb.move-file.argument.file", "the file to move");
/// Where `move_file` moves it.
pub const MOVE_FILE_INTO: Word = Word::saying(
    "files.verb.move-file.argument.into",
    "the folder it goes into",
);

/// What `archive_folder` does.
pub const ARCHIVE_FOLDER: Word = Word::saying(
    "files.verb.archive-folder.purpose",
    "make one archive file out of a folder",
);
/// **The sentence a person approves before an archive is made.**
pub const ARCHIVE_FOLDER_SENTENCE: Word = Word::saying(
    "files.verb.archive-folder.sentence",
    "make an archive of {folder} called {name}, in {into}",
);
/// What `archive_folder` makes an archive of.
pub const ARCHIVE_FOLDER_FOLDER: Word = Word::saying(
    "files.verb.archive-folder.argument.folder",
    "the folder to make an archive of",
);
/// Where `archive_folder` puts the archive.
pub const ARCHIVE_FOLDER_INTO: Word = Word::saying(
    "files.verb.archive-folder.argument.into",
    "the folder the archive goes into",
);
/// What `archive_folder` calls the archive.
pub const ARCHIVE_FOLDER_NAME: Word = Word::saying(
    "files.verb.archive-folder.argument.name",
    "what to call the archive, ending in .zip",
)
.noting("\".zip\" is the ending the name has to have, and is not translated.");

/// What one of the six says: what it does, what a person reads when it runs,
/// and what each of its arguments is for.
///
/// This is the same list [`crate::verbs`] declares, said rather than declared.
/// Both are built from the constants above, so there is one English sentence
/// per thing rather than two that a test has to keep equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spoken {
    /// The verb's name, which is never translated.
    pub verb: &'static str,
    /// What it does, in the words a person would use.
    pub purpose: Word,
    /// What a person reads when it runs — and approves, if it changes anything.
    pub sentence: Word,
    /// What each argument is for, by the argument's name.
    pub arguments: &'static [(&'static str, Word)],
}

impl Spoken {
    /// What this argument is for, or nothing if the verb does not take it.
    #[must_use]
    pub fn argument(&self, named: &str) -> Option<Word> {
        self.arguments
            .iter()
            .find(|(name, _)| *name == named)
            .map(|(_, word)| *word)
    }
}

/// The six, and every word each of them has.
pub const THE_SIX: [Spoken; 6] = [
    Spoken {
        verb: "list_folder",
        purpose: LIST_FOLDER,
        sentence: LIST_FOLDER_SENTENCE,
        arguments: &[("folder", LIST_FOLDER_FOLDER)],
    },
    Spoken {
        verb: "read_file",
        purpose: READ_FILE,
        sentence: READ_FILE_SENTENCE,
        arguments: &[("file", READ_FILE_FILE)],
    },
    Spoken {
        verb: "find_in_folder",
        purpose: FIND_IN_FOLDER,
        sentence: FIND_IN_FOLDER_SENTENCE,
        arguments: &[
            ("folder", FIND_IN_FOLDER_FOLDER),
            ("named", FIND_IN_FOLDER_NAMED),
            ("most", FIND_IN_FOLDER_MOST),
        ],
    },
    Spoken {
        verb: "rename_file",
        purpose: RENAME_FILE,
        sentence: RENAME_FILE_SENTENCE,
        arguments: &[("file", RENAME_FILE_FILE), ("name", RENAME_FILE_NAME)],
    },
    Spoken {
        verb: "move_file",
        purpose: MOVE_FILE,
        sentence: MOVE_FILE_SENTENCE,
        arguments: &[("file", MOVE_FILE_FILE), ("into", MOVE_FILE_INTO)],
    },
    Spoken {
        verb: "archive_folder",
        purpose: ARCHIVE_FOLDER,
        sentence: ARCHIVE_FOLDER_SENTENCE,
        arguments: &[
            ("folder", ARCHIVE_FOLDER_FOLDER),
            ("into", ARCHIVE_FOLDER_INTO),
            ("name", ARCHIVE_FOLDER_NAME),
        ],
    },
];

/// Every plain string this crate can say, in the order a translator meets them:
/// what the machine could not do, what could not be followed, what a refusal
/// says, and then the six verbs.
pub const EVERY_WORD: [Word; 41] = [
    NOT_A_FILE_VERB,
    MISSING,
    NOT_A_FOLDER,
    NOT_A_FILE,
    GONE,
    NOT_TEXT,
    ALREADY_THERE,
    ALREADY_IN,
    INTO_ITSELF,
    NOT_A_ZIP_NAME,
    TOO_MANY,
    TOO_MUCH,
    THE_MACHINE_SAID_NO,
    NOTHING_THERE,
    UNREADABLE,
    REALLY_LEADS_ELSEWHERE,
    WOULD_CREATE,
    LIST_FOLDER,
    LIST_FOLDER_SENTENCE,
    LIST_FOLDER_FOLDER,
    READ_FILE,
    READ_FILE_SENTENCE,
    READ_FILE_FILE,
    FIND_IN_FOLDER,
    FIND_IN_FOLDER_SENTENCE,
    FIND_IN_FOLDER_FOLDER,
    FIND_IN_FOLDER_NAMED,
    FIND_IN_FOLDER_MOST,
    RENAME_FILE,
    RENAME_FILE_SENTENCE,
    RENAME_FILE_FILE,
    RENAME_FILE_NAME,
    MOVE_FILE,
    MOVE_FILE_SENTENCE,
    MOVE_FILE_FILE,
    MOVE_FILE_INTO,
    ARCHIVE_FOLDER,
    ARCHIVE_FOLDER_SENTENCE,
    ARCHIVE_FOLDER_FOLDER,
    ARCHIVE_FOLDER_INTO,
    ARCHIVE_FOLDER_NAME,
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
    /// A sentence that is not one.
    #[error(transparent)]
    Sentence(#[from] alo_strings::TemplateError),
    /// A note that could not be attached.
    #[error(transparent)]
    Note(#[from] alo_strings::PhraseError),
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
pub fn file_words() -> Result<Vocabulary, WordsError> {
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
        let phrase = Phrase::says(word.key(), word.says())?;
        let phrase = match word.note() {
            Some(note) => phrase.noting(note)?,
            None => phrase,
        };
        vocabulary.says(phrase)?;
    }
    vocabulary.counts(
        Plural::counting(TOO_BIG.key(), TOO_BIG.number, TOO_BIG.one, TOO_BIG.other)?
            .noting(TOO_BIG.note)?,
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
    /// arrive from anywhere; this is the test that makes that true, and it is
    /// the same shape as `alo-shortcuts` putting every shipped binding back
    /// through `Chord::checked`.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                Key::named(word.named),
                Ok(word.key()),
                "{}: {}",
                word.named,
                Key::named(word.named).unwrap_err()
            );
        }
        assert_eq!(Key::named(TOO_BIG.named), Ok(TOO_BIG.key()));
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        assert!(!named.contains(TOO_BIG.named));
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "files", "{}", word.named);
        }
        assert_eq!(TOO_BIG.key().area(), "files");
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it — which is the whole of what this file has to get right.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = file_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len() + 1);
        assert_eq!(vocabulary.counted().count(), 1);
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = file_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **The words the six verbs are declared with are these words.** Not
    /// copies of them: [`crate::verbs`] reads these constants, so a sentence
    /// improved here is improved in the declaration, and the sentence
    /// `alo_capability` checked is the sentence a translator is given.
    #[test]
    fn the_six_say_what_they_are_declared_with() {
        let verbs = crate::verbs::file_verbs().unwrap();
        assert_eq!(THE_SIX.len(), verbs.len());
        for spoken in THE_SIX {
            let verb = verbs.of(spoken.verb).unwrap();
            assert_eq!(verb.purpose(), spoken.purpose.says(), "{}", spoken.verb);
            assert_eq!(verb.args().len(), spoken.arguments.len(), "{}", spoken.verb);
            for arg in verb.args() {
                assert_eq!(
                    spoken.argument(&arg.name).map(|word| word.says()),
                    Some(arg.purpose.as_str()),
                    "{} {}",
                    spoken.verb,
                    arg.name
                );
            }
        }
    }

    /// A note is written where the sentence cannot be translated from its own
    /// words, and the two that carry a name off somebody's disk are among them.
    #[test]
    fn the_awkward_ones_carry_a_note() {
        for word in [
            NOT_A_ZIP_NAME,
            THE_MACHINE_SAID_NO,
            UNREADABLE,
            ARCHIVE_FOLDER_NAME,
        ] {
            assert!(word.note().is_some(), "{}", word.named);
        }
        assert!(!TOO_BIG.note.is_empty());
    }
}
