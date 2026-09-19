//! Every string this crate can say, and the English beside each one.
//!
//! Three groups. **What would not close**, which is what a log-out says when it
//! stops; **the one setting**, as Settings offers it; and **the person's own
//! file**, with what is said when it did not read or could not be written.
//!
//! # What is deliberately not here
//!
//! **A sentence for a locked machine.** Logging out and switching user at a
//! locked screen are both answered with `alo_locking::NotWhileLocked`, which
//! says the one thing the lock screen already says. A second sentence naming
//! what was refused would tell a stranger at the desk that somebody is signed in
//! with work open, which is exactly what task 1 of this plan decided a lock
//! screen does not say.
//!
//! **Anything about what was open.** No sentence here has a gap a document, a
//! title or a screen could be filled into, and the only thing ever filled in
//! about an application is its identifier — which the sandbox vouches for, and
//! which is never the name whoever packaged it chose. A sentence with room for
//! *Invoices — March.odt* in it would undo the whole of `crate::open`.
//!
//! **A countdown.** Nothing here says *closing in 30 seconds*, because nothing
//! here closes anything after a wait.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What would not close — `crate::WouldNotClose`. One sentence per application,
// so no language has to form a plural alo OS chose for it.
// ---------------------------------------------------------------------------

/// One application did not close when it was asked.
pub const WOULD_NOT_CLOSE: Word = Word::saying(
    "leaving.would-not-close",
    "{application} has not closed, and may have work you have not saved",
)
.noting(
    "{application} is the identifier of an application, like org.example.Editor, and is never \
     translated. Shown when the person chose to log out and this application did not close — \
     usually because it is waiting for them to say whether to save something. Logging out has \
     stopped and nothing has been forced to close. Say it as a fact rather than as an error.",
);

/// Some applications had already closed before the log-out stopped.
pub const SOME_HAVE_ALREADY_CLOSED: Word = Word::saying(
    "leaving.some-have-already-closed",
    "Some of your applications have already closed",
)
.noting(
    "Said beside the list of applications that would not close, when others had already closed by \
     then. It is there so that a person who decides not to log out after all is not surprised to \
     find some of their applications gone; an application that has closed cannot be put back.",
);

/// The person may log out anyway.
pub const LOG_OUT_ANYWAY: Word = Word::saying(
    "leaving.log-out-anyway",
    "Log out anyway, and lose whatever has not been saved",
)
.noting(
    "What the person chooses when applications would not close and they want to log out \
     regardless. It is deliberately blunt: choosing it ends the session over an application that \
     was waiting to save. Keep the consequence in the label rather than in a separate warning.",
);

// ---------------------------------------------------------------------------
// The one setting — `crate::Settings`.
// ---------------------------------------------------------------------------

/// Whether applications open again at the next sign-in.
pub const REOPEN: Word = Word::saying(
    "leaving.setting.reopen",
    "Open my applications again when I sign in",
)
.noting(
    "A switch in Settings, off as alo OS ships. When it is on, alo OS writes down which \
     applications were open and on which screen when the person logs out, and opens them again at \
     their next sign-in. It never writes down what was in them — no document, no page, no \
     message — so an application that reopens a document does that itself.",
);

// ---------------------------------------------------------------------------
// The person's own file, `leaving.toml` — `crate::FileNotRead` and
// `crate::FileNotWritten`. Each names the file, and each says what the machine
// does instead.
// ---------------------------------------------------------------------------

/// The disk would not give the file up.
pub const KEPT_NOT_READ: Word = Word::saying(
    "leaving.kept.not-read",
    "what you chose about signing out, at {path}, could not be read, so nothing has been reopened",
)
.noting(
    "{path} is a file on this machine and is never translated. A disk or a permission rather than \
     anything a person typed. The second clause is the one they act on: their applications are not \
     opening themselves this time.",
);

/// The file is there and is not these settings.
pub const KEPT_NOT_UNDERSTOOD: Word = Word::saying(
    "leaving.kept.not-understood",
    "what you chose about signing out, at {path}, is not something alo OS can read, so nothing in \
     the file has been used and nothing has been reopened",
)
.noting(
    "{path} is a file on this machine and is never translated. \"alo OS\" is the product's name \
     and is never translated. The important clause is that nothing in the file was used: alo OS \
     did not take the half it understood.",
);

/// The file stopped being settings at a line.
pub const KEPT_NOT_UNDERSTOOD_AT: Word = Word::saying(
    "leaving.kept.not-understood-at",
    "what you chose about signing out, at {path}, stops making sense at line {line}, so nothing \
     in the file has been used and nothing has been reopened",
)
.noting(
    "{path} is a file on this machine and is never translated. {line} is a plain whole number, \
     counted from one the way a text editor counts lines.",
);

/// The file says it is a shape this alo OS does not read.
pub const KEPT_ANOTHER_FORMAT: Word = Word::saying(
    "leaving.kept.another-format",
    "what you chose about signing out, at {path}, was written for a different alo OS than this \
     one, so nothing in the file has been used and nothing has been reopened",
)
.noting(
    "{path} is a file on this machine and is never translated. Most often a newer alo OS wrote \
     the file.",
);

/// The file names something that is not one of this file's keys.
pub const KEPT_UNKNOWN_KEY: Word = Word::saying(
    "leaving.kept.unknown-key",
    "what you chose about signing out, at {path}, says {key}, which is not something alo OS keeps \
     about signing out, so nothing in the file has been used",
)
.noting(
    "{path} is a file on this machine and {key} is a word as it was typed into it; neither is \
     translated. The key is named because it is what a person has to find in the file to fix it.",
);

/// The disk would not take the changed file.
pub const KEPT_NOT_WRITTEN: Word = Word::saying(
    "leaving.kept.not-written",
    "what you chose about signing out could not be written to {path}, so nothing has been changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A full disk or a folder the person \
     cannot write. The second clause is what they act on: the file is exactly as it was.",
);

/// The change could not be written as a file this alo OS reads back.
pub const KEPT_NOT_EXPRESSIBLE: Word = Word::saying(
    "leaving.kept.not-expressible",
    "this alo OS could not write that into {path} in a way it can read back again, so nothing has \
     been changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A fault in alo OS rather than \
     anything the person did, said plainly: the change was refused before the file was touched.",
);

/// The file a change would replace is there and did not read, so it was kept.
pub const KEPT_NOT_REPLACED: Word = Word::saying(
    "leaving.kept.not-replaced",
    "what you chose about signing out, at {path}, could not be read, so that change has not been \
     written over it and nothing has been changed — correct the file, or put this back as alo OS \
     ships it",
)
.noting(
    "{path} is a file on this machine and is never translated. Said when a person changes this \
     setting while the file, most often one they edited by hand, does not read: alo OS keeps their \
     file rather than replacing it, so a typing mistake in it is not lost. The last clause gives \
     the two ways on.",
);

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 12] = [
    WOULD_NOT_CLOSE,
    SOME_HAVE_ALREADY_CLOSED,
    LOG_OUT_ANYWAY,
    REOPEN,
    KEPT_NOT_READ,
    KEPT_NOT_UNDERSTOOD,
    KEPT_NOT_UNDERSTOOD_AT,
    KEPT_ANOTHER_FORMAT,
    KEPT_UNKNOWN_KEY,
    KEPT_NOT_WRITTEN,
    KEPT_NOT_EXPRESSIBLE,
    KEPT_NOT_REPLACED,
];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// are what say so.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn leaving_words() -> Result<Vocabulary, WordsError> {
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
/// nothing is replaced.
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
    use std::collections::BTreeSet;

    use super::*;

    /// What we ship is held to the rule everybody else is held to.
    #[test]
    fn every_key_is_a_key_under_this_crates_area() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
            assert_eq!(word.key().area(), "leaving", "{}", word.named());
        }
    }

    /// The list declares once, each key once, and a second declaration replaces
    /// nothing.
    #[test]
    fn the_whole_list_declares_once() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        let mut vocabulary = leaving_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// Every word carries a note: none of these can be translated from its own
    /// words alone, and the one about reopening has to say what is *not* kept.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Only an application's identifier and a file's place are ever filled
    /// in.** No sentence here has room for a document, a window's title, a page
    /// a person was reading or the name of a screen — which is the whole of
    /// `crate::open`'s promise, held one floor up in the strings.
    #[test]
    fn nothing_but_an_identifier_a_path_a_line_or_a_key_is_filled_in() {
        let allowed: BTreeSet<&str> = ["application", "path", "line", "key"].into();
        for word in EVERY_WORD {
            for gap in word.phrase().unwrap().source().gaps() {
                assert!(allowed.contains(gap.as_str()), "{}: {gap}", word.named());
            }
        }
    }
}
