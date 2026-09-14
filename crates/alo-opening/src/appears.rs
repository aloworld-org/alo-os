//! What a file's own bytes say it is — including the answers that are not a
//! kind.
//!
//! Six answers and no seventh. There is no *probably a Word document*: a file
//! whose contents meet a rule in `crate::looking` is what that rule says, a
//! file whose contents meet none is [`Appears::Unrecognised`], and a file that
//! begins as one thing and breaks part way through is [`Appears::Damaged`] —
//! which is itself a finding, because *damaged* and *not recognised* send a
//! person in opposite directions.

use alo_strings::{Filling, Said, Strings};

use crate::kind::Kind;
use crate::words::{self, Word};

/// What a file's bytes say it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Appears {
    /// One of the kinds this crate recognises.
    A(Kind),
    /// A program a computer could run. Never a [`Kind`]: nothing on this
    /// machine is ever described as opening one, because opening a file never
    /// runs it.
    AProgram,
    /// A current Office document locked with a password by its author.
    APasswordProtectedDocument,
    /// A file with nothing in it.
    Nothing,
    /// Bytes that meet no rule this crate has.
    Unrecognised,
    /// A file that begins as something recognisable and does not hold together
    /// after that.
    Damaged(Container),
}

/// What a damaged file is known to be.
///
/// Less than a [`Kind`], and deliberately: a zip cut short cannot be looked
/// inside far enough to say whether it was a Word document or a spreadsheet,
/// and a sentence that named one would be a guess.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Container {
    /// A PDF document.
    Pdf,
    /// A compressed file: a zip archive, or a document stored as one.
    Compressed,
    /// A file in the storage format older Office documents use.
    OlderOffice,
}

impl Container {
    /// The word this container is named by.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Pdf => words::KIND_PDF,
            Self::Compressed => words::CONTAINER_COMPRESSED,
            Self::OlderOffice => words::CONTAINER_OLDER_OFFICE,
        }
    }
}

/// Whether a document was seen to carry macros.
///
/// Two answers, and the second is worded as what it is: *none were seen*.
/// Macros in an older PowerPoint presentation live inside its one stream rather
/// than beside it, so this crate cannot see them without reading the document,
/// and a type that said *none* would claim something nobody checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Macros {
    /// The document carries macros.
    Inside,
    /// None were seen where this kind of document keeps them.
    NoneSeen,
}

impl Appears {
    /// What the bytes are, named in the language the person reads — or
    /// [`None`] for the two answers that are not a thing a file is: a file
    /// with nothing in it, and bytes nothing recognises.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Option<Said> {
        let word = match self {
            Self::A(kind) => kind.word(),
            Self::AProgram => words::A_PROGRAM,
            Self::APasswordProtectedDocument => words::A_PASSWORD_PROTECTED_DOCUMENT,
            Self::Damaged(container) => container.word(),
            Self::Nothing | Self::Unrecognised => return None,
        };
        Some(strings.say(&word.key(), &Filling::nothing()))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// **What a file is has a name exactly when it is a thing**, and the name
    /// is the one a person reads.
    #[test]
    fn what_a_file_is_is_named_when_it_is_a_thing() {
        let strings = in_english();
        assert_eq!(
            Appears::A(Kind::PngImage).said(&strings).unwrap().text(),
            "a PNG image"
        );
        assert_eq!(
            Appears::AProgram.said(&strings).unwrap().text(),
            "a program"
        );
        assert_eq!(
            Appears::Damaged(Container::Compressed)
                .said(&strings)
                .unwrap()
                .text(),
            "a compressed file"
        );
        assert_eq!(
            Appears::Damaged(Container::Pdf)
                .said(&strings)
                .unwrap()
                .text(),
            "a PDF document"
        );
        assert!(Appears::Nothing.said(&strings).is_none());
        assert!(Appears::Unrecognised.said(&strings).is_none());
    }
}
