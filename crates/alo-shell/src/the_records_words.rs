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

/// The sentence under the heading when the record does not go all the way
/// back: [`WHAT_IS_IN_IT`] without its promise, because the promise would be
/// false, and the record's own sentence saying so follows it.
pub const WHAT_IS_IN_IT_NOW: Word = Word::saying(
    "the-record.introduction-not-all-of-it",
    "Every change alo made, everything that left this computer, and every request that was \
     refused — newest first.",
)
.noting(
    "Under the title of the record window, when the oldest part of the record has aged out. The \
     sentence after it says how far back the list goes.",
);

/// Every sentence this file declares, in the order a translator meets them.
pub const THE_RECORDS_WORDS: [Word; 16] = [
    WHAT_ALO_DID,
    WHAT_IS_IN_IT,
    WHAT_IS_IN_IT_NOW,
    ALREADY_SENT,
    REFUSED,
    KEPT_ON_THIS_COMPUTER,
    PUT_BACK_ASKS_FIRST,
    PUT_ONE_BACK,
    PUT_THESE_BACK,
    KEPT_BESIDE_IT,
    WRITTEN_DOWN_TOO,
    PUT_BACK,
    CANCEL,
    PUTTING_BACK,
    OLDER_THAN_PUT_BACK,
    NO_LONGER_KEPT,
];

/// The button at the end of a change that can be put back (`560:242`).
pub const PUT_BACK_ASKS_FIRST: Word = Word::saying("the-record.put-back-asks", "Put back…").noting(
    "A button at the end of one change in the record window. It asks first: pressing it \
         opens a short confirmation, and nothing changes until the person confirms. The three \
         dots say that it asks.",
);

/// The confirmation's question, for one folder (`560:764`).
pub const PUT_ONE_BACK: Word = Word::saying(
    "the-record.put-one-back",
    "Put {folder} back as it was on {date} at {time}?",
)
.noting(
    "The question on the confirmation in the record window. {folder} is a folder's name on this \
     computer and is never translated. {date} and {time} are already written the way the person's \
     region writes them: the moment this computer kept the folder, before the change.",
);

/// The confirmation's question, for several folders.
pub const PUT_THESE_BACK: Word = Word::saying(
    "the-record.put-these-back",
    "Put these folders back as they were on {date} at {time}?",
)
.noting(
    "The question on the confirmation in the record window when the change touched several \
     folders, which go back together or not at all; each is named on its own line below. {date} \
     and {time} are already written the way the person's region writes them.",
);

/// Where what is in a folder now will be.
pub const KEPT_BESIDE_IT: Word = Word::saying(
    "the-record.kept-beside-it",
    "What is in {folder} now is kept beside it, as {aside}.",
)
.noting(
    "One line per folder on the confirmation: nothing is deleted by putting back, and this says \
     where what is there now can be found afterwards. {folder} and {aside} are names on this \
     computer and are never translated.",
);

/// That putting back is recorded.
pub const WRITTEN_DOWN_TOO: Word = Word::saying(
    "the-record.written-down-too",
    "Putting back is written in this list too.",
)
.noting("The last line of the confirmation in the record window, the list being that window.");

/// The confirmation's way on.
pub const PUT_BACK: Word = Word::saying("the-record.put-back", "Put back").noting(
    "A button on the confirmation in the record window. It returns the folders to how they \
     were. Keep it short, like a button.",
);

/// The confirmation's way back.
pub const CANCEL: Word = Word::saying("the-record.cancel", "Cancel")
    .noting("A button beside \"Put back\" on the confirmation. Nothing is changed.");

/// While the service puts the folders back.
pub const PUTTING_BACK: Word = Word::saying("the-record.putting-back", "Putting back…")
    .noting("Shown on the confirmation while the folders are being put back.");

/// A change written down before Put back existed.
pub const OLDER_THAN_PUT_BACK: Word = Word::saying(
    "the-record.older-than-put-back",
    "This entry is older than Put back, so it cannot be put back from here.",
)
.noting(
    "Under a change in the record window that was written down before this computer started \
     noting, beside each change, what it kept to put it back. \"Put back\" is the button's name.",
);

/// What was kept has been let go of.
pub const NO_LONGER_KEPT: Word = Word::saying(
    "the-record.no-longer-kept",
    "What was kept before this is no longer on this computer, so it cannot be put back.",
)
.noting(
    "Under a change in the record window whose kept copy this computer has let go of — it keeps \
     them only for as long as the person chose, or until the disk needs the room.",
);

/// Every count this file declares, in the order they are drawn.
pub const THE_RECORDS_COUNTS: [Counted; 3] =
    [CHANGES_YOU_ALLOWED, REQUESTS_THAT_LEFT, REFUSED_TODAY];

/// The window's own words in the person's language, looked up once a frame.
///
/// **Here, beside their declarations, and not in the window's files**:
/// `tests/record_source.rs` holds every `record_*` file to saying nothing of
/// its own, so the window is handed its few sentences rather than looking any
/// up — and what it is handed is exactly this list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheRecordsSentences {
    /// The title, in the strip and as the heading.
    pub(crate) title: String,
    /// The sentence under the heading.
    pub(crate) introduction: String,
    /// The sentence under the heading when the record does not go all the
    /// way back.
    pub(crate) introduction_not_all_of_it: String,
    /// The footer.
    pub(crate) footer: String,
    /// *Refused*.
    pub(crate) refused: String,
    /// *Already sent*.
    pub(crate) already_sent: String,
    /// *Today*, beside today's counts — `alo-formats`' word, the one the day
    /// headings use.
    pub(crate) today: String,
    /// *Put back…*, at the end of a change that can be put back.
    pub(crate) put_back_asks: String,
    /// *Put back*, on the confirmation.
    pub(crate) put_back: String,
    /// *Cancel*, on the confirmation.
    pub(crate) cancel: String,
    /// *Putting back…*, while it is done.
    pub(crate) putting_back: String,
    /// The last line of the confirmation.
    pub(crate) written_down_too: String,
    /// Under a change written down before Put back existed.
    pub(crate) older: String,
    /// Under a change whose kept copy is gone.
    pub(crate) no_longer_kept: String,
}

impl TheRecordsSentences {
    /// Every sentence, in `strings`' language.
    pub(crate) fn of(strings: &alo_strings::Strings) -> Self {
        let said = |word: Word| {
            strings
                .say(&word.key(), &alo_strings::Filling::nothing())
                .into_text()
        };
        Self {
            title: said(WHAT_ALO_DID),
            introduction: said(WHAT_IS_IN_IT),
            introduction_not_all_of_it: said(WHAT_IS_IN_IT_NOW),
            footer: said(KEPT_ON_THIS_COMPUTER),
            refused: said(REFUSED),
            already_sent: said(ALREADY_SENT),
            today: said(alo_formats::words::TODAY),
            put_back_asks: said(PUT_BACK_ASKS_FIRST),
            put_back: said(PUT_BACK),
            cancel: said(CANCEL),
            putting_back: said(PUTTING_BACK),
            written_down_too: said(WRITTEN_DOWN_TOO),
            older: said(OLDER_THAN_PUT_BACK),
            no_longer_kept: said(NO_LONGER_KEPT),
        }
    }
}

/// The confirmation's question and its line per folder, in `strings`'
/// language: `folders` as (name, set-aside name), the moment as the person
/// writes a date and a time.
pub(crate) fn the_question(
    strings: &alo_strings::Strings,
    folders: &[(String, String)],
    (date, time): (&str, &str),
) -> (String, Vec<String>) {
    let when = |filling: alo_strings::Filling| filling.and("date", date).and("time", time);
    let question = match folders {
        [(only, _)] => strings.say(
            &PUT_ONE_BACK.key(),
            &when(alo_strings::Filling::of("folder", only.clone())),
        ),
        _ => strings.say(
            &PUT_THESE_BACK.key(),
            &when(alo_strings::Filling::nothing()),
        ),
    };
    let beside = folders
        .iter()
        .map(|(folder, aside)| {
            strings
                .say(
                    &KEPT_BESIDE_IT.key(),
                    &alo_strings::Filling::of("folder", folder.clone()).and("aside", aside.clone()),
                )
                .into_text()
        })
        .collect();
    (question.into_text(), beside)
}

/// One of today's counts, `how_many`, in `strings`' language.
pub(crate) fn counted(strings: &alo_strings::Strings, counted: Counted, how_many: u64) -> String {
    strings
        .count(
            &counted.key(),
            &alo_strings::Counting::of(how_many),
            &alo_strings::Filling::nothing(),
        )
        .into_text()
}

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
