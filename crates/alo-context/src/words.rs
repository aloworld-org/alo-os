//! Every string this crate can say, and the English beside each one.
//!
//! The same shape as `alo-applications`' list and for the same reason: somebody
//! has to write the first sentence and it is written in the language this code
//! is written in, but **no sentence reaches a person without something having
//! asked whether anybody translated it**.
//!
//! # The rows a person reads before they answer
//!
//! Four of these are not refusals, and they are the ones that matter most.
//! *Context is offered, never watched* (ADR 0001 §4) is only true to a person
//! if they can **see what they are offering**, so [`crate::Context::shown`]
//! hands a shell one row per part and one row saying nothing was offered. They
//! are rows rather than one sentence with a list in it, which is
//! `alo-shortcuts`' rule: the separator and the conjunction are not punctuation
//! a program can pick.
//!
//! # The gaps that are never translated
//!
//! `{document}` holds a path off this machine and `{application}` an identifier
//! off it. Neither is ours to translate — the rule `alo-files` holds a filename
//! to — and each sentence holding one says so to whoever translates it, because
//! a translator with no product in front of them has nothing else to tell a gap
//! that holds data from a gap that holds a word.
//!
//! # What is deliberately not here
//!
//! **The selected text has no string of its own.** The row says *that* a
//! selection was offered and never what was in it: a selection can be a
//! megabyte of somebody's contract, and putting it into a line a shell draws
//! would be showing a person their own document back at them in a space meant
//! for one sentence.

use alo_strings::{Key, Plural, Vocabulary};

/// One string a crate can say.
///
/// Re-exported because this crate's own files, and the tests that read its
/// list, name it as `crate::words::Word`. It lives in `alo-strings` since item
/// 9d.
pub use alo_strings::Word;

/// One string this crate can say about a number of things.
///
/// Separate from [`Word`] because a countable string is declared and looked up
/// differently: two English sentences rather than one, and the reader's own
/// language decides which of *its* forms is shown. `alo-keeping`'s shape.
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

/// What a translator has to be told about the gap that holds a path.
const A_PATH: &str = "{document} is the path to a file on this machine, like /home/anna/march.pdf. \
                      It is not translated and it is not rewritten: it is what the person's own \
                      filesystem calls the document, and a verb naming anything else would name a \
                      different file.";

/// What a translator has to be told about the gap that holds an identifier.
const AN_IDENTIFIER: &str = "{application} is the identifier this machine knows an application by, like \
     org.blender.Blender. It is never translated, and it is not the name a person would say.";

// ---------------------------------------------------------------------------
// What a person is shown of what they offered — [`crate::Context::shown`].
// ---------------------------------------------------------------------------

/// The row naming the document that was open.
pub const THE_DOCUMENT: Word = Word::saying(
    "context.the-document",
    "the document you have open: {document}",
)
.noting(A_PATH);

/// The row saying a selection was offered.
pub const THE_SELECTION: Word = Word::saying("context.the-selection", "the text you had selected")
    .noting(
        "The selected text itself is deliberately not in this row — it can be pages long, and a \
         person knows what they selected. This row says only that it went with the question.",
    );

/// The row naming the window that was in front.
pub const THE_WINDOW: Word = Word::saying(
    "context.the-window",
    "the window in front of you: {window}",
)
.noting(
    "{window} is the application's own name and the identifier this machine knows it by, together \
     — see context.window-called. Neither half is translated.",
);

/// The row shown when an invocation offered nothing at all.
pub const NOTHING_OFFERED: Word = Word::saying(
    "context.nothing-offered",
    "nothing from your screen was offered with this question",
)
.noting(
    "Shown when a person pressed the key with nothing open, nothing selected and nothing in front \
     of them. It is reassurance rather than an error, and it should read as a statement of fact \
     rather than as something having gone wrong.",
);

/// One window, as a person is shown one.
pub const WINDOW_CALLED: Word = Word::saying("context.window-called", "{title} ({application})")
    .noting(
        "{title} is what the window calls itself, in whatever language the application was written \
         in, and is not ours to translate. {application} is the identifier — the thing a grant \
         would be made over. Both are shown because two applications can call themselves the same \
         thing and no two share an identifier. Move the brackets if your language writes them \
         differently.",
    );

// ---------------------------------------------------------------------------
// What could not be offered as the window in front — [`crate::NotOffered`].
// ---------------------------------------------------------------------------

/// A window belonging to no application.
pub const NO_WINDOW: Word = Word::saying(
    "context.window.no-application",
    "a window with no application cannot be offered — name the application the window belongs to, \
     which is what a verb would have to name to reach it",
);

/// A window whose application no verb could name.
pub const NOT_AN_IDENTIFIER: Word = Word::saying(
    "context.window.not-an-identifier",
    "{application} is not an identifier — an identifier has no spaces and no folders in it, so no \
     verb could ever name this window",
)
.noting(AN_IDENTIFIER);

// ---------------------------------------------------------------------------
// What could not be offered as the open document — [`crate::NotOffered`].
//
// The same rules a grant over a single file is held to, said where the document
// is offered rather than where the grant is made.
// ---------------------------------------------------------------------------

/// A document with no path at all.
pub const NO_DOCUMENT: Word = Word::saying(
    "context.document.nothing-named",
    "there is no document to offer — offer the file that is open, or offer nothing",
);

/// A document named by a path that starts nowhere.
pub const NOT_A_FULL_PATH: Word = Word::saying(
    "context.document.not-a-full-path",
    "{document} is not a full path — offer the whole path to the open document, because a path \
     that starts anywhere else means a different file depending on where it is read",
)
.noting(A_PATH);

/// A document whose path steps upwards.
pub const COULD_LEAD_ELSEWHERE: Word = Word::saying(
    "context.document.could-lead-elsewhere",
    "{document} has .. in it and could lead somewhere else — offer the path the document really \
     has",
)
.noting(A_PATH);

/// A document that is the whole machine.
pub const NOT_A_DOCUMENT: Word = Word::saying(
    "context.document.the-whole-machine",
    "{document} is the whole machine rather than a document — offer the file that is open, because \
     what is offered at invocation becomes a grant over exactly that file",
)
.noting(A_PATH);

// ---------------------------------------------------------------------------
// The one thing here that is counted.
// ---------------------------------------------------------------------------

/// A selection too long to offer whole.
///
/// Counted rather than written with a number stuck into one English sentence,
/// because *character* takes a different form for different numbers in most of
/// the languages this ships in and in several of them for more than two
/// (item 9a).
pub const SELECTION_SHORTENED: Counted = Counted {
    named: "context.selection-shortened",
    number: "characters",
    one: "{characters} character of what you selected was left out — only the first part of it was \
          offered",
    other: "{characters} characters of what you selected were left out — only the first part of it \
            was offered",
    note: "Shown when somebody selected more text than is offered in one turn. {characters} is the \
           number left out, not the number offered. It matters that this is not silent: a bounded \
           answer that does not say it was bounded reads exactly like a complete one, and somebody \
           would conclude the agent had read the whole document.",
};

/// Every plain string this crate can say, in the order a translator meets them:
/// what a person is shown of what they offered, then what could not be offered.
///
/// The array is what a test reads down and what [`declare_into`] walks, so a
/// word declared above and left out here is a string nothing can look up. The
/// one countable string is not here — it is declared beneath, because it is
/// declared differently.
pub const EVERY_WORD: [Word; 11] = [
    THE_DOCUMENT,
    THE_SELECTION,
    THE_WINDOW,
    NOTHING_OFFERED,
    WINDOW_CALLED,
    NO_WINDOW,
    NOT_AN_IDENTIFIER,
    NO_DOCUMENT,
    NOT_A_FULL_PATH,
    COULD_LEAD_ELSEWHERE,
    NOT_A_DOCUMENT,
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
pub fn context_words() -> Result<Vocabulary, WordsError> {
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
            SELECTION_SHORTENED.key(),
            SELECTION_SHORTENED.number,
            SELECTION_SHORTENED.one,
            SELECTION_SHORTENED.other,
        )?
        .noting(SELECTION_SHORTENED.note)?,
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
        assert_eq!(
            Key::named(SELECTION_SHORTENED.named),
            Ok(SELECTION_SHORTENED.key())
        );
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let mut named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        assert!(named.insert(SELECTION_SHORTENED.named));
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "context", "{}", word.named());
        }
        assert_eq!(SELECTION_SHORTENED.key().area(), "context");
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it — which is the whole of what this file has to get right.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = context_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len() + 1);
        assert_eq!(vocabulary.counted().count(), 1);
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = context_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Every sentence with a data gap in it carries the note that says so.**
    /// A translator who took `{document}` for a word would translate around it
    /// wrongly in every language that inflects a name, and there is nothing in
    /// the sentence itself to warn them.
    #[test]
    fn every_sentence_holding_data_says_what_the_gap_is() {
        for word in EVERY_WORD {
            if word.says().contains("{document}") {
                assert!(
                    word.note().is_some_and(|note| note.contains("path")),
                    "{} has a path gap and no note about it",
                    word.named()
                );
            }
            if word.says().contains("{application}") {
                assert!(
                    word.note().is_some_and(|note| note.contains("identifier")),
                    "{} has an identifier gap and no note about it",
                    word.named()
                );
            }
        }
    }

    /// **The row about a selection never holds the selection**, and the note is
    /// what stops a translator from helpfully adding a gap for it.
    #[test]
    fn the_selection_row_says_that_there_was_one_and_not_what_it_was() {
        assert!(!THE_SELECTION.says().contains('{'));
        let note = THE_SELECTION.note().unwrap();
        assert!(note.contains("not in this row"), "{note}");
    }

    /// **The bound says it was reached, and the note says why that is not
    /// optional.** `alo-files`' rule about a listing that was cut short, met
    /// where the thing cut short is a person's own text.
    #[test]
    fn the_shortened_selection_counts_what_was_left_out_and_says_so() {
        assert!(SELECTION_SHORTENED.one.contains("{characters}"));
        assert!(SELECTION_SHORTENED.other.contains("{characters}"));
        assert!(
            SELECTION_SHORTENED
                .note
                .contains("left out, not the number")
        );
    }
}
