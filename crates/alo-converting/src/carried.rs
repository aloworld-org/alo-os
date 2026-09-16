//! What a copy carried from the original, and what it could not — by name.
//!
//! ADR 0039 §4. Two answers and no third: [`Carried::Everything`], said in a
//! sentence of its own, and [`Carried::NotEverything`], which lists every
//! [`NotCarried`] by name. **There is no variant meaning *not checked*.** A
//! conversion whose inventories did not both complete never reaches this type
//! — it is a refusal to show the copy — so *lost nothing* and *did not check*
//! cannot read the same, because only one of them can be said at all.

use std::collections::BTreeSet;

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// The longest font name carried in a finding, in bytes.
///
/// A name in a document is whatever the sender wrote; a longer one is not a
/// font anybody could recognise, and it would be a paragraph in a sentence.
pub const LONGEST_FONT_NAME: usize = 64;

/// What a copy carried from its original.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Carried {
    /// Everything: both inventories completed and nothing differs.
    Everything,
    /// Not everything, and this is what was not carried — never empty, in one
    /// order, each thing once.
    NotEverything(NotEverything),
}

/// A non-empty list of what a copy could not carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotEverything(BTreeSet<NotCarried>);

/// One thing a copy could not carry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NotCarried {
    /// A font the original sets text in, of which the copy contains nothing:
    /// its text is shown in another.
    FontSubstituted(FontName),
    /// A field whose value depends on when or where the document is open,
    /// fixed in the copy at what it showed at conversion.
    FieldFixed(Field),
    /// Macros, which were not run and are not in the copy.
    Macros,
    /// Content the original takes from somewhere else, which was not fetched.
    LinkedNotFetched(Linked),
    /// Comments, which a page does not show.
    Comments,
    /// Tracked changes, which a page does not show as changes.
    TrackedChanges,
}

/// A font family's name, as the document gave it, made safe to say.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontName(String);

/// A field whose value depends on when or where a document is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Field {
    /// Today's date.
    Date,
    /// The time now.
    Time,
    /// The document's own file name.
    FileName,
    /// An author's or a user's name.
    Author,
    /// A mail merge field.
    MergeField,
    /// A formula calling for the current date and time.
    TheCurrentMoment,
    /// A formula calling for a random number.
    ARandomNumber,
}

/// What a piece of linked content is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Linked {
    /// A picture.
    Picture,
    /// Data from another file.
    Data,
    /// A template the document is attached to.
    Template,
    /// Something else the document takes from another place.
    SomethingElse,
}

impl FontName {
    /// A name as a document wrote it: trimmed, with anything that is not a
    /// printable character removed, and cut at [`LONGEST_FONT_NAME`] bytes.
    ///
    /// [`None`] when nothing is left, because a font with no name is not one a
    /// person could be told about.
    #[must_use]
    pub fn from_document(written: &str) -> Option<Self> {
        let mut name = String::new();
        for letter in written.trim().chars().filter(|letter| !letter.is_control()) {
            if name.len() + letter.len_utf8() > LONGEST_FONT_NAME {
                break;
            }
            name.push(letter);
        }
        let name = name.trim().to_owned();
        (!name.is_empty()).then_some(Self(name))
    }

    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The name as fonts are compared: lowercase letters and digits only, so
    /// `Aptos Narrow` in a document and `AptosNarrow-Bold` in a PDF are the
    /// same family.
    #[must_use]
    pub fn compared(&self) -> String {
        compared(&self.0)
    }
}

/// A font name as fonts are compared.
pub(crate) fn compared(name: &str) -> String {
    name.chars()
        .filter(|letter| letter.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

impl Carried {
    /// What was not carried, as an answer: [`Carried::Everything`] when the
    /// list is empty.
    #[must_use]
    pub fn of(not_carried: BTreeSet<NotCarried>) -> Self {
        if not_carried.is_empty() {
            Self::Everything
        } else {
            Self::NotEverything(NotEverything(not_carried))
        }
    }

    /// What was not carried, in order — nothing for [`Carried::Everything`].
    pub fn not_carried(&self) -> impl Iterator<Item = &NotCarried> {
        match self {
            Self::Everything => None,
            Self::NotEverything(list) => Some(list.0.iter()),
        }
        .into_iter()
        .flatten()
    }

    /// What a person reads, in order: that everything was carried, or that
    /// not everything was and then each thing that was not.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Vec<Said> {
        match self {
            Self::Everything => vec![plainly(strings, words::CARRIED_EVERYTHING)],
            Self::NotEverything(list) => {
                let mut said = vec![plainly(strings, words::CARRIED_NOT_EVERYTHING)];
                said.extend(list.0.iter().map(|not| not.said(strings)));
                said
            }
        }
    }
}

impl NotCarried {
    /// The sentence a person reads about it.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::FontSubstituted(font) => strings.say(
                &words::NOT_CARRIED_FONT.key(),
                &Filling::of(words::FONT, font.as_str().to_owned()),
            ),
            Self::FieldFixed(field) => strings.say(
                &words::NOT_CARRIED_FIELD.key(),
                &Filling::nothing().and_said(words::FIELD, &plainly(strings, field.word())),
            ),
            Self::Macros => plainly(strings, words::NOT_CARRIED_MACROS),
            Self::LinkedNotFetched(linked) => plainly(strings, linked.word()),
            Self::Comments => plainly(strings, words::NOT_CARRIED_COMMENTS),
            Self::TrackedChanges => plainly(strings, words::NOT_CARRIED_TRACKED_CHANGES),
        }
    }
}

impl Field {
    /// Every field kind, in one order.
    pub const EVERY: [Self; 7] = [
        Self::Date,
        Self::Time,
        Self::FileName,
        Self::Author,
        Self::MergeField,
        Self::TheCurrentMoment,
        Self::ARandomNumber,
    ];

    /// The word this field kind is named by.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Date => words::FIELD_DATE,
            Self::Time => words::FIELD_TIME,
            Self::FileName => words::FIELD_FILE_NAME,
            Self::Author => words::FIELD_AUTHOR,
            Self::MergeField => words::FIELD_MERGE,
            Self::TheCurrentMoment => words::FIELD_THE_CURRENT_MOMENT,
            Self::ARandomNumber => words::FIELD_A_RANDOM_NUMBER,
        }
    }
}

impl Linked {
    /// Every kind of linked content, in one order.
    pub const EVERY: [Self; 4] = [
        Self::Picture,
        Self::Data,
        Self::Template,
        Self::SomethingElse,
    ];

    /// The sentence for this kind of linked content not being fetched.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Picture => words::NOT_CARRIED_LINKED_PICTURE,
            Self::Data => words::NOT_CARRIED_LINKED_DATA,
            Self::Template => words::NOT_CARRIED_LINKED_TEMPLATE,
            Self::SomethingElse => words::NOT_CARRIED_LINKED_SOMETHING_ELSE,
        }
    }
}

/// A sentence with no gaps.
fn plainly(strings: &Strings, word: Word) -> Said {
    strings.say(&word.key(), &Filling::nothing())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// **Lost nothing is its own sentence, and it is not the start of a list.**
    #[test]
    fn everything_and_not_everything_read_differently() {
        let strings = in_english();
        let everything = Carried::of(BTreeSet::new());
        assert_eq!(everything, Carried::Everything);
        assert_eq!(everything.not_carried().count(), 0);
        let said = everything.said(&strings);
        assert_eq!(said.len(), 1);

        let lost = Carried::of(BTreeSet::from([
            NotCarried::Comments,
            NotCarried::FontSubstituted(FontName::from_document("Garamond").unwrap()),
        ]));
        let lost_said = lost.said(&strings);
        assert_eq!(lost_said.len(), 3);
        assert_ne!(
            said.first().map(Said::text),
            lost_said.first().map(Said::text)
        );
        assert!(
            lost_said
                .iter()
                .any(|said| said.text().contains("Garamond"))
        );
    }

    /// Each thing is said once, whatever order it was found in.
    #[test]
    fn each_thing_is_said_once_in_one_order() {
        let one = Carried::of(BTreeSet::from([
            NotCarried::Comments,
            NotCarried::FieldFixed(Field::Date),
            NotCarried::Comments,
        ]));
        let other = Carried::of(BTreeSet::from([
            NotCarried::FieldFixed(Field::Date),
            NotCarried::Comments,
        ]));
        assert_eq!(one, other);
        assert_eq!(one.not_carried().count(), 2);
    }

    /// **A font name from a document is made safe to say**: no control
    /// characters, no line breaks, and not a paragraph.
    #[test]
    fn a_font_name_from_a_document_is_made_safe_to_say() {
        assert_eq!(
            FontName::from_document("  Garamond\n").unwrap().as_str(),
            "Garamond"
        );
        assert_eq!(
            FontName::from_document("Evil\nlost comments")
                .unwrap()
                .as_str(),
            "Evillost comments"
        );
        assert!(FontName::from_document(" \u{7}\t ").is_none());
        let long = "G".repeat(500);
        assert_eq!(
            FontName::from_document(&long).unwrap().as_str().len(),
            LONGEST_FONT_NAME
        );
        let wide = "é".repeat(100);
        assert!(FontName::from_document(&wide).unwrap().as_str().len() <= LONGEST_FONT_NAME);
    }

    /// Families compare by their letters, not their spelling.
    #[test]
    fn families_compare_by_their_letters() {
        assert_eq!(
            FontName::from_document("Aptos Narrow").unwrap().compared(),
            compared("AptosNarrow")
        );
        assert_ne!(compared("Garamond"), compared("NotoSerif"));
        assert_eq!(compared("Times New Roman"), "timesnewroman");
    }

    /// Every field kind and every linked kind has its own sentence.
    #[test]
    fn every_kind_has_its_own_sentence() {
        let strings = in_english();
        let mut seen: Vec<String> = Field::EVERY
            .iter()
            .map(|field| {
                NotCarried::FieldFixed(*field)
                    .said(&strings)
                    .text()
                    .to_owned()
            })
            .chain(Linked::EVERY.iter().map(|linked| {
                NotCarried::LinkedNotFetched(*linked)
                    .said(&strings)
                    .text()
                    .to_owned()
            }))
            .collect();
        let before = seen.len();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), before);
    }
}
