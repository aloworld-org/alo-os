//! Why nothing opens a file, as a value with a sentence.
//!
//! `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`, task 4:
//! *the answer for a kind nothing opens is a sentence in the vocabulary rather
//! than a fallback to a text editor.* A text editor handed a PDF shows a person
//! a screen of noise and teaches them that the machine does not know what their
//! file is. So there is no fallback anywhere in this crate: a kind no installed
//! application declares and no person chose is [`NothingOpens::NoApplication`],
//! and it says so.
//!
//! Two more answers are not about applications at all. A file that is a program,
//! is empty, is damaged, is protected with a password or is no kind this machine
//! recognises is not a kind anything could be chosen for, and
//! [`NothingOpens::TheFile`] carries `alo-opening`'s own finding and sentence —
//! *opening a file never runs a program* is that crate's to say, and a second
//! wording here could come to disagree with it. A file that would not be read
//! is [`NothingOpens::Unreadable`], likewise.

use alo_opening::{Cannot, Kind, Unreadable};
use alo_strings::{Filling, Said, Strings};

use crate::words;

/// Why nothing opens a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NothingOpens {
    /// A kind no installed application declares and the person chose nothing
    /// installed for.
    NoApplication {
        /// The kind of file.
        kind: Kind,
        /// What the person chose for it, when they chose something that is no
        /// longer installed.
        chosen: Option<String>,
    },
    /// The file is not a kind anything opens: a program, nothing, damaged,
    /// protected with a password, or unrecognised.
    TheFile(Cannot),
    /// The file would not be read, so nothing was decided about it.
    Unreadable(Unreadable),
}

impl NothingOpens {
    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NoApplication { kind, chosen } => {
                let filling = Filling::nothing().and_said("what", &kind.said(strings));
                match chosen {
                    None => strings.say(&words::NOTHING_OPENS.key(), &filling),
                    Some(chosen) => strings.say(
                        &words::NOTHING_OPENS_CHOICE_NOT_INSTALLED.key(),
                        &filling.and("chosen", chosen.clone()),
                    ),
                }
            }
            Self::TheFile(cannot) => cannot.said(strings),
            Self::Unreadable(unreadable) => unreadable.said(strings),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// **A kind nothing opens is a sentence**, naming the kind, and naming
    /// what the person chose when that is why.
    #[test]
    fn a_kind_nothing_opens_is_said() {
        let strings = in_english();
        let none = NothingOpens::NoApplication {
            kind: Kind::OpenDocumentSpreadsheet,
            chosen: None,
        }
        .said(&strings);
        assert!(!none.is_a_bug(), "{none}");
        assert!(none.unfilled().is_empty(), "{none}");
        assert!(none.text().contains("OpenDocument spreadsheet"), "{none}");

        let gone = NothingOpens::NoApplication {
            kind: Kind::Pdf,
            chosen: Some("org.kde.okular".to_owned()),
        }
        .said(&strings);
        assert!(!gone.is_a_bug(), "{gone}");
        assert!(gone.text().contains("org.kde.okular"), "{gone}");
        assert_ne!(none.text(), gone.text());
    }
}
