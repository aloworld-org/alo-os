//! Every string this crate can say, and the English beside each one.
//!
//! Thirteen, in two groups. **Three are what a person is told when a picture
//! was taken** — saved, copied, or both, because *both* is a thing somebody
//! asked for and never a thing that happened by itself. **Ten are refusals**:
//! seven about the picture this machine would not take or could not keep, and
//! three about the part of alo OS that reads the screen not being able to
//! answer.
//!
//! The shape is `alo-in-use`'s and is copied rather than re-decided: constants
//! under one area, `alo_strings::Word` because these are literals in this file,
//! [`declare_into`] for the one vocabulary the machine has, and tests at the
//! bottom holding the list to the rules every other list is held to.
//!
//! # Why the file's name is not in this list
//!
//! A screenshot's file name is a date and nothing else ([`crate::naming`]), so
//! there is no word in it to translate — and that is the point rather than an
//! omission. A name with *Screenshot* in it would be one language's word on
//! every machine, and a name with a window's title in it would be the thing the
//! plan forbids outright: a file name that says what was on the screen.
//!
//! # A note is part of the string
//!
//! A translator works alone, with no alo machine in front of them. *The lock
//! screen* is what a machine shows when somebody has to sign in again before
//! they can use it; *a picture of your screen* is what everybody else calls a
//! screenshot, and the sentences deliberately do not use that word, because it
//! is English jargon that has been borrowed unevenly into other languages.
//!
//! **No sentence and no note here names anything alo OS rents.**
//! `alo_saying::what_a_person_would_have_to_learn` is the check, and the plan's
//! task 7 asks it of every sentence these crates say.

use alo_strings::Vocabulary;

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What a person is told when a picture was taken — [`crate::Taken`].
//
// Each is the whole of what one message says, announced on its own by a screen
// reader, so each is a capitalised sentence.
// ---------------------------------------------------------------------------

/// The picture went to a file.
pub const SAVED: Word = Word::saying(
    "capturing.saved",
    "A picture of your screen was saved as {name}",
)
.noting(
    "Shown after a picture of the screen has been taken and written into the folder the person \
     chose for them. {name} is the file's name, which is a date and a time and is not translated. \
     \"A picture of your screen\" is deliberately not the English word screenshot: that word has \
     been borrowed unevenly into other languages, and the plain description reads in all of them.",
);

/// The picture went to the clipboard.
pub const COPIED: Word = Word::saying(
    "capturing.copied",
    "A picture of your screen was copied, ready to paste",
)
.noting(
    "Shown after a picture of the screen has been taken and put where copying puts things, so the \
     person's next paste is that picture. Nothing was written to a disk, which is the difference \
     from capturing.saved. \"A picture of your screen\" is deliberately not the English word \
     screenshot.",
);

/// The picture went to a file and to the clipboard, because somebody asked for
/// both.
pub const SAVED_AND_COPIED: Word = Word::saying(
    "capturing.saved-and-copied",
    "A picture of your screen was saved as {name}, and copied ready to paste",
)
.noting(
    "Shown after a picture of the screen has been both written into the folder the person chose \
     and put where copying puts things. It is only ever read when the person asked for both: \
     nothing on this machine does both on its own. {name} is the file's name, which is a date and \
     a time and is not translated.",
);

// ---------------------------------------------------------------------------
// Why no picture was taken, or none was kept — [`crate::NotTaken`].
//
// Read where the picture would have appeared. Each says what is so and then
// what to do about it, because somebody told only that their machine will not
// do the thing they asked has been told they are stuck.
// ---------------------------------------------------------------------------

/// What [`crate::NotTaken::TheLockScreen`] says.
pub const THE_LOCK_SCREEN: Word = Word::saying(
    "capturing.the-lock-screen",
    "A picture of the lock screen cannot be taken. Sign in, and take the picture then",
)
.noting(
    "Read in place of the picture. The lock screen is what this machine shows when somebody has \
     to sign in again before they can use it; while it is up, the screen belongs to nobody yet, \
     so nothing may photograph it — including alo OS itself. Not a fault, and not a permission \
     the person can be given: the second sentence is the whole of what they do.",
);

/// What [`crate::NotTaken::AnotherPersonsWindow`] says.
pub const ANOTHER_PERSONS_WINDOW: Word = Word::saying(
    "capturing.another-persons-window",
    "That window belongs to somebody else signed in on this machine, so a picture of it cannot be \
     taken. Take a picture of your own screen instead",
)
.noting(
    "Read in place of the picture. Several people can be signed in to one alo machine at once, \
     each with their own windows; this is read when the window asked for is one of somebody \
     else's. The other person is deliberately not named — being refused must not tell anybody who \
     else is signed in.",
);

/// What [`crate::NotTaken::NotAnAreaOfTheScreen`] says.
pub const NOT_AN_AREA_OF_THE_SCREEN: Word = Word::saying(
    "capturing.not-an-area-of-the-screen",
    "That is not an area of the screen. Choose an area that is on the screen, and try again",
)
.noting(
    "Read in place of the picture, when the part of the screen asked for has no width or no \
     height, or hangs over an edge of the screen. The commonest way to meet it is letting go of a \
     selection without having dragged it.",
);

/// What [`crate::NotTaken::NotAFolderForPictures`] says.
pub const NOT_A_FOLDER_FOR_PICTURES: Word = Word::saying(
    "capturing.not-a-folder-for-pictures",
    "Pictures cannot be kept there. Choose a folder inside your own files, and try again",
)
.noting(
    "Read in place of the picture, when the folder chosen to keep pictures in is the very top of \
     a disk rather than a folder inside the person's own files. \"Your own files\" means the \
     place on this machine that belongs to them, as against the parts of it that belong to the \
     machine.",
);

/// What [`crate::NotTaken::NoRoomForAName`] says.
pub const NO_ROOM_FOR_A_NAME: Word = Word::saying(
    "capturing.no-room-for-a-name",
    "The picture could not be saved: that folder already holds one under every name this second \
     can make. Move some of them somewhere else, and try again",
)
.noting(
    "Read in place of the picture. Pictures of the screen are named after the moment they were \
     taken, so several taken in one second are told apart by a number after the time; this is \
     read when every one of those names is already a file in the folder. Rare, and a fact about \
     the person's own folder rather than a fault of the machine.",
);

/// What [`crate::NotTaken::NotWritten`] says.
pub const NOT_WRITTEN: Word = Word::saying(
    "capturing.not-written",
    "The picture could not be saved in that folder. Check that the folder is still there and that \
     the disk has room, and try again",
)
.noting(
    "Read in place of the picture, when writing the file into the folder the person chose did not \
     work — the folder was removed or renamed since they chose it, the disk is full, or it is not \
     theirs to write in. The two things to check are the two that account for almost all of it.",
);

/// What [`crate::NotTaken::NotCopied`] says.
pub const NOT_COPIED: Word = Word::saying(
    "capturing.not-copied",
    "The picture could not be copied. Take it again, and report it if it happens a second time",
)
.noting(
    "Read in place of the picture, when it was taken and then could not be put where copying puts \
     things. It is alo OS's own bug rather than anything the person did, which is why the second \
     sentence asks them to report it — but taking the picture again is worth trying first, and \
     costs them nothing.",
);

// ---------------------------------------------------------------------------
// Why the screen itself could not be read — [`crate::NotGrabbed`].
//
// Three, because a part that is not running, one that did not answer and one
// that answered nothing are three different things to fix, and a person told
// the same sentence for all three goes and does the wrong one.
// ---------------------------------------------------------------------------

/// What [`crate::NotGrabbed::NothingReadsTheScreen`] says.
pub const NOTHING_READS_THE_SCREEN: Word = Word::saying(
    "capturing.nothing-reads-the-screen",
    "A picture of the screen could not be taken: the part of alo OS that reads the screen is not \
     running. Sign out and back in, and it comes back with it",
)
.noting(
    "Read in place of the picture. The part of alo OS that reads the screen is one component and \
     is deliberately not named: naming it would ask the reader to learn what it is before they \
     could understand why their machine will not take a picture. The second sentence must survive \
     translation — it is the only thing the person can do.",
);

/// What [`crate::NotGrabbed::NoPicture`] says.
pub const NO_PICTURE: Word = Word::saying(
    "capturing.no-picture",
    "A picture of the screen could not be taken: the part of alo OS that reads the screen did not \
     answer. Sign out and back in to start it again",
)
.noting(
    "Read in place of the picture. Different from capturing.nothing-reads-the-screen: that one is \
     not running at all, this one is there and did not reply. A person fixes them the same way \
     and is not being told the same thing.",
);

/// What [`crate::NotGrabbed::NothingCameBack`] says.
pub const NOTHING_CAME_BACK: Word = Word::saying(
    "capturing.nothing-came-back",
    "A picture of the screen could not be taken: the part of alo OS that reads the screen \
     answered with nothing at all. Report this, because until it is fixed this machine cannot \
     take a picture of anything",
)
.noting(
    "Read in place of the picture. It is alo OS's own bug rather than anything the person did, \
     and the second sentence says the serious half. Do not soften it into \"something went \
     wrong\".",
);

/// Everything a person is told when a picture was taken.
pub const EVERY_TOLD: [Word; 3] = [SAVED, COPIED, SAVED_AND_COPIED];

/// Every refusal: why no picture was taken, or none was kept.
pub const EVERY_REFUSAL: [Word; 10] = [
    THE_LOCK_SCREEN,
    ANOTHER_PERSONS_WINDOW,
    NOT_AN_AREA_OF_THE_SCREEN,
    NOT_A_FOLDER_FOR_PICTURES,
    NO_ROOM_FOR_A_NAME,
    NOT_WRITTEN,
    NOT_COPIED,
    NOTHING_READS_THE_SCREEN,
    NO_PICTURE,
    NOTHING_CAME_BACK,
];

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 13] = [
    SAVED,
    COPIED,
    SAVED_AND_COPIED,
    THE_LOCK_SCREEN,
    ANOTHER_PERSONS_WINDOW,
    NOT_AN_AREA_OF_THE_SCREEN,
    NOT_A_FOLDER_FOR_PICTURES,
    NO_ROOM_FOR_A_NAME,
    NOT_WRITTEN,
    NOT_COPIED,
    NOTHING_READS_THE_SCREEN,
    NO_PICTURE,
    NOTHING_CAME_BACK,
];

/// The area every key in this crate is under.
///
/// Named once so the tests below and anybody reading a translator's file are
/// looking at the same string.
pub const AREA: &str = "capturing";

/// The gap a message leaves for the file's name.
pub const NAME: &str = "name";

/// Why this crate's own words could not be declared.
///
/// Neither of these can happen to the list above — the tests at the bottom of
/// this file are what say so. It is a `Result` rather than an unwrap because a
/// library that panics on its own string table takes the shell with it, and
/// because [`declare_into`] can genuinely fail against a vocabulary that
/// already holds one of these keys.
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
pub fn capturing_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// The machine has one vocabulary and every crate adds its own to it —
/// `alo-saying` is the one place that calls all of these.
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
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_strings::Key;
    use std::collections::BTreeSet;

    /// **What we ship is held to the rule everybody else is held to.**
    /// [`Word::key`] does not check, because a key written in this file cannot
    /// arrive from anywhere; this is the test that makes that true.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(Key::named(word.named()), Ok(word.key()), "{}", word.named());
        }
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// **The two groups add up to the whole list**, so a word added to one and
    /// forgotten in the other is a failure here rather than a string nothing
    /// declares.
    #[test]
    fn the_groups_are_the_whole_list() {
        let grouped: BTreeSet<&str> = EVERY_TOLD
            .iter()
            .chain(EVERY_REFUSAL.iter())
            .map(|word| word.named())
            .collect();
        let everything: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(grouped, everything);
        assert_eq!(EVERY_TOLD.len() + EVERY_REFUSAL.len(), EVERY_WORD.len());
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), AREA, "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = capturing_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0, "nothing here counts");
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = capturing_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **The two messages that name a file leave exactly one gap, and it is the
    /// same gap.** A message whose gap were named something else would be
    /// filled with nothing and read *saved as {name}* on somebody's screen.
    #[test]
    fn the_messages_that_name_a_file_leave_one_gap_for_it() {
        for word in [SAVED, SAVED_AND_COPIED] {
            let phrase = word.phrase().unwrap();
            assert_eq!(
                phrase.source().gaps(),
                [NAME.to_owned()],
                "{}",
                word.named()
            );
        }
        assert!(COPIED.phrase().unwrap().source().gaps().is_empty());
    }

    /// **Every refusal is a whole sentence with nothing to fill in** — a gap in
    /// a refusal would put `{}` in front of a person at the exact moment
    /// something already went wrong.
    #[test]
    fn a_refusal_names_nothing_and_needs_nothing_filled() {
        for word in EVERY_REFUSAL {
            let phrase = word.phrase().unwrap();
            assert!(phrase.source().gaps().is_empty(), "{}", word.named());
        }
    }

    /// **Every refusal says what to do.** Somebody told only that their machine
    /// will not take a picture has been told they are stuck. Each is two
    /// clauses — what is so, and then the action — so none can shrink to a bare
    /// report without failing here.
    #[test]
    fn every_refusal_says_what_to_do() {
        for word in EVERY_REFUSAL {
            let after = word.says().split_once(". ").map(|(_, after)| after);
            assert!(
                after.is_some_and(|after| !after.trim().is_empty()),
                "{} is one clause, so it reports without instructing",
                word.named()
            );
        }
    }

    /// **Every sentence stands alone.** Each is the whole of what one message
    /// says, announced with nothing above or beside it, so each is a
    /// capitalised sentence.
    #[test]
    fn every_sentence_stands_alone() {
        for word in EVERY_WORD {
            let first = word.says().chars().next().unwrap();
            assert!(
                first.is_uppercase(),
                "{} does not begin a sentence",
                word.named()
            );
        }
    }

    /// **Every word carries a note for the translator.** None of these can be
    /// translated from its own words alone: each is read at a moment the
    /// translator has to be able to picture, and each names something — the
    /// lock screen, pasting, a picture of the screen — that has a meaning here.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Nothing here counts anything.** A number written into an English
    /// sentence is one that cannot be translated into a language with three
    /// plural forms, and nothing this crate says has a number in it.
    #[test]
    fn nothing_here_counts_anything() {
        for word in EVERY_WORD {
            assert!(
                !word.says().chars().any(|char| char.is_ascii_digit()),
                "{}",
                word.named()
            );
        }
    }

    /// **No sentence uses the English jargon for a picture of a screen.** It
    /// has been borrowed unevenly into other languages, and the plain
    /// description reads in all of them — including in English.
    #[test]
    fn no_sentence_uses_the_english_jargon_for_a_picture_of_a_screen() {
        for word in EVERY_WORD {
            assert!(
                !word.says().to_lowercase().contains("screenshot"),
                "{} says screenshot",
                word.named()
            );
        }
    }

    /// **Being refused somebody else's window names nobody.** Being refused a
    /// window that is not yours must not tell you whose it is, so the sentence
    /// has no gap and no name in it.
    #[test]
    fn being_refused_somebody_elses_window_names_nobody() {
        let phrase = ANOTHER_PERSONS_WINDOW.phrase().unwrap();
        assert!(phrase.source().gaps().is_empty());
        assert!(ANOTHER_PERSONS_WINDOW.says().contains("somebody else"));
    }

    /// **Nothing here says anything about what is leaving this machine.**
    /// Taking a picture of the screen sends nothing anywhere, and a sentence
    /// here that read like the egress indicator's would be saying otherwise.
    #[test]
    fn nothing_here_says_anything_about_what_is_leaving() {
        for word in EVERY_WORD {
            let says = word.says().to_lowercase();
            for absent in ["leaving", "upload", "share", "cloud"] {
                assert!(!says.contains(absent), "{} says {absent}", word.named());
            }
        }
    }
}
