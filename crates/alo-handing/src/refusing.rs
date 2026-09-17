//! Why nothing was handed over, and what the person is told.
//!
//! **Every refusal hands nothing to anybody.** Two of them refuse *before the
//! window that has the data is asked for a byte* — a target that takes none of
//! the forms, and a form nobody offered — which is `alo_clipboard::Clipboard`'s
//! rule in the same words: a compositor that asked and then converted would be
//! handing an application bytes that the window which had them never said it
//! could produce.
//!
//! And a refused drop is never half a move. What was being dragged stays where
//! it is, because the only thing that gives a source up is a completed
//! [`crate::Handed`], and none of these is one.
//!
//! Like `alo-clipboard`'s and `alo-dividing`'s refusals, a [`NotHanded`] has no
//! `Display`: the only road to words is [`NotHanded::said`], in the language the
//! person reads.

use alo_clipboard::{CouldNotGive, Kind};
use alo_strings::{Filling, Said, Strings};

use crate::uri_list::NotAFileList;
use crate::words::{self, Word};

/// Why what a person dragged did not arrive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotHanded {
    /// There is nothing under the pointer that takes a drop.
    NothingHere,
    /// This window takes none of the forms on offer.
    WillNotTakeIt,
    /// A form the source never offered was asked for.
    NotThatForm {
        /// What was asked for.
        form: Kind,
    },
    /// The window that had it did not hand it over.
    DidNotHandItOver(CouldNotGive),
    /// What an application put up as a list of files is not one.
    NotAFileList {
        /// What is wrong with it, for whoever is fixing that application. The
        /// person reads one sentence; this is not it.
        why: NotAFileList,
    },
    /// The question a file was dropped into is over.
    TheQuestionIsOver,
}

impl NotHanded {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> Word {
        match self {
            Self::NothingHere => words::NOTHING_HERE,
            Self::WillNotTakeIt => words::WILL_NOT_TAKE_IT_AFTER,
            Self::NotThatForm { .. } => words::NOT_THAT_FORM,
            Self::DidNotHandItOver(_) => words::DID_NOT_HAND_IT_OVER,
            Self::NotAFileList { .. } => words::NOT_A_FILE_LIST,
            Self::TheQuestionIsOver => words::THE_QUESTION_IS_OVER,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use std::collections::BTreeSet;

    /// Every refusal says something of its own, declared, with nothing left to
    /// fill in — a person told the wrong reason drags the same thing again.
    #[test]
    fn every_refusal_says_something_of_its_own() {
        let strings = in_english();
        let mut seen = BTreeSet::new();
        for refused in [
            NotHanded::NothingHere,
            NotHanded::WillNotTakeIt,
            NotHanded::NotThatForm {
                form: Kind::image_png(),
            },
            NotHanded::DidNotHandItOver(CouldNotGive::TheApplicationDidNot),
            NotHanded::NotAFileList {
                why: NotAFileList::NotText,
            },
            NotHanded::TheQuestionIsOver,
        ] {
            let said = refused.said(&strings);
            assert!(!said.is_a_bug(), "{refused:?}");
            assert!(said.unfilled().is_empty(), "{refused:?}: {said}");
            assert!(seen.insert(said.text().to_owned()), "{refused:?}");
        }
    }

    /// **What is wrong with an application's list of files is kept, and is not
    /// what the person reads.** One sentence for them; the detail for whoever
    /// is fixing the application that wrote it.
    #[test]
    fn what_an_application_got_wrong_is_kept_beside_the_sentence() {
        let refused = NotHanded::NotAFileList {
            why: NotAFileList::NotOnThisMachine {
                host: "otherbox".to_owned(),
            },
        };
        let said = refused.said(&in_english());
        assert!(!said.text().contains("otherbox"), "{said}");
        let NotHanded::NotAFileList { why } = &refused else {
            unreachable!("it was built as one")
        };
        assert!(why.to_string().contains("otherbox"), "{why}");
    }
}
