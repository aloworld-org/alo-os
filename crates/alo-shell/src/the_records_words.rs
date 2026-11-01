//! What the record window says in its own voice: its title, the sentence under
//! it, today's three counts, the two words an entry can carry at its end, and
//! the footer — as the owner approved them on 2026-11-01 (`560:142`).
//!
//! **Nothing here words an entry.** Every clause, sentence and refusal in the
//! list is `alo-recounting`'s or the record's own (`crate::record_window`); what
//! this file says is about the window, or counts entries without describing any
//! one of them.

use alo_strings::{Key, Plural, Vocabulary, Word};

/// The window's title, in its strip and as its heading.
pub const WHAT_ALO_DID: Word = Word::saying("the-record.title", "What alo did").noting(
    "The title of the window that lists everything alo, the assistant built into alo OS, did on \
     this computer: every change, everything that left it, every request refused. \"alo\" is \
     the product's name and stays as it is, in lower case.",
);

/// The sentence under the heading.
pub const WHAT_IS_IN_IT: Word = Word::saying(
    "the-record.introduction",
    "Every change alo made, everything that left this computer, and every request that was \
     refused — newest first. Nothing has been removed from this list.",
)
.noting(
    "Under the title of the record window. \"Nothing has been removed\" is a promise: the list \
     is the whole record, and refusals are in it as plainly as what ran.",
);

/// At the end of an entry that left this computer.
pub const ALREADY_SENT: Word = Word::saying("the-record.already-sent", "Already sent").noting(
    "A small label at the end of an entry where something left this computer for a provider. \
     It says that this cannot be called back: putting back rewinds files on this computer and \
     cannot reach anything that already left.",
);

/// Beside an entry that was refused.
pub const REFUSED: Word = Word::saying("the-record.refused", "Refused").noting(
    "A small label under an entry alo asked to do and that was refused, so nothing happened. \
     One word, like a status.",
);

/// The footer.
pub const KEPT_ON_THIS_COMPUTER: Word = Word::saying(
    "the-record.footer",
    "Kept on this computer. Putting back rewinds files here; it cannot call back anything that \
     already left.",
)
.noting(
    "The line at the foot of the record window. The record is kept on this computer only. \
     \"Putting back\" is returning a folder to how it was before a change; something already \
     sent to a provider cannot be recalled.",
);

/// One of today's counts: a number, and what it counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counted {
    /// What names it.
    named: &'static str,
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

/// The gap the number goes in, in every count.
pub const HOW_MANY: &str = "how_many";

/// Today's changes the person said yes to.
pub const CHANGES_YOU_ALLOWED: Counted = Counted {
    named: "the-record.today.allowed",
    one: "1 change you allowed",
    other: "{how_many} changes you allowed",
    note: "A count beside \"Today\" at the top of the record window: how many changes alo made \
           today that the person approved. {how_many} is the number.",
};

/// Today's requests that left this computer.
pub const REQUESTS_THAT_LEFT: Counted = Counted {
    named: "the-record.today.left",
    one: "1 request left this computer",
    other: "{how_many} requests left this computer",
    note: "A count beside \"Today\" at the top of the record window: how many times today \
           something left this computer for a provider. {how_many} is the number.",
};

/// Today's refusals.
pub const REFUSED_TODAY: Counted = Counted {
    named: "the-record.today.refused",
    one: "1 refused",
    other: "{how_many} refused",
    note: "A count beside \"Today\" at the top of the record window: how many things alo asked \
           to do today were refused, so did not happen. {how_many} is the number.",
};

/// Every sentence this file declares, in the order a translator meets them.
pub const THE_RECORDS_WORDS: [Word; 5] = [
    WHAT_ALO_DID,
    WHAT_IS_IN_IT,
    ALREADY_SENT,
    REFUSED,
    KEPT_ON_THIS_COMPUTER,
];

/// Every count this file declares, in the order they are drawn.
pub const THE_RECORDS_COUNTS: [Counted; 3] =
    [CHANGES_YOU_ALLOWED, REQUESTS_THAT_LEFT, REFUSED_TODAY];

/// Why this file's words could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TheRecordsWordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A count that is not one.
    #[error(transparent)]
    Plural(#[from] alo_strings::PluralError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Add these words atomically, before any translation is loaded.
///
/// # Errors
/// [`TheRecordsWordsError`] for a malformed phrase or count, or a key already
/// taken.
pub fn declare_the_records_words(vocabulary: &mut Vocabulary) -> Result<(), TheRecordsWordsError> {
    let mut prepared = vocabulary.clone();
    for word in THE_RECORDS_WORDS {
        prepared.says(word.phrase()?)?;
    }
    for counted in THE_RECORDS_COUNTS {
        prepared.counts(
            Plural::counting(counted.key(), HOW_MANY, counted.one, counted.other)?
                .noting(counted.note)?,
        )?;
    }
    *vocabulary = prepared;
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_strings::{Counting, Filling, Strings};

    /// Every word and count declares, and twice is refused and changes nothing.
    #[test]
    fn every_word_declares_once() {
        let mut vocabulary = Vocabulary::empty();
        declare_the_records_words(&mut vocabulary).unwrap();
        for word in THE_RECORDS_WORDS {
            assert!(vocabulary.phrase(&word.key()).is_some(), "{word:?}");
        }
        let before = vocabulary.how_many();
        assert!(declare_the_records_words(&mut vocabulary).is_err());
        assert_eq!(vocabulary.how_many(), before);
    }

    /// **A count says one thing and many things differently**, as the design
    /// writes both.
    #[test]
    fn a_count_says_one_and_many() {
        let mut vocabulary = Vocabulary::empty();
        declare_the_records_words(&mut vocabulary).unwrap();
        let strings = Strings::of(vocabulary);
        let said = |counted: Counted, how_many: u64| {
            strings
                .count(&counted.key(), &Counting::of(how_many), &Filling::nothing())
                .into_text()
        };
        assert_eq!(said(CHANGES_YOU_ALLOWED, 1), "1 change you allowed");
        assert_eq!(said(REQUESTS_THAT_LEFT, 2), "2 requests left this computer");
        assert_eq!(said(REFUSED_TODAY, 0), "0 refused");
    }
}
