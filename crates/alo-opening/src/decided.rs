//! The answer to *given this file, what are my options?* — and whether its
//! name told the truth.
//!
//! A [`Decided`] is two shapes, and the second cannot be mistaken for the
//! first. A file whose name claims to be one thing and whose bytes are another
//! is [`Decided::NotWhatItsNameSays`], with the outcome **inside** it: whoever
//! shows the answer has to take the finding apart to reach what the machine can
//! do, so there is no road from a mismatched file to an open window that does
//! not pass the mismatch first. That is what *the mismatch is the finding
//! rather than a thing to silently correct* means in a type.
//!
//! The sentences come in the same order: the finding, then the outcome.

use alo_strings::{Filling, Said, Strings};

use crate::naming::Named;
use crate::outcome::Outcome;
use crate::words;

/// What this machine decided about a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Decided {
    /// The file's name agrees with its bytes, or claims nothing.
    AsItIs(Outcome),
    /// The file's name claims one thing, and its bytes are another.
    NotWhatItsNameSays {
        /// What the name claims.
        named: Named,
        /// What this machine can do with what the file really is.
        outcome: Outcome,
    },
}

impl Decided {
    /// The sentences a person reads, in the order they read them.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Vec<Said> {
        match self {
            Self::AsItIs(outcome) => outcome.said(strings),
            Self::NotWhatItsNameSays { named, outcome } => {
                let filling = Filling::nothing().and_said(words::NAMED, &named.said(strings));
                let finding = match outcome.appears().said(strings) {
                    Some(what) => strings.say(
                        &words::NAMED_AS_SOMETHING_ELSE.key(),
                        &filling.and_said(words::WHAT, &what),
                    ),
                    None => strings.say(&words::NAMED_AS_WHAT_IT_IS_NOT.key(), &filling),
                };
                let mut said = vec![finding];
                said.extend(outcome.said(strings));
                said
            }
        }
    }

    /// Whether a name and what a file turned out to be are a finding, and the
    /// answer either way.
    pub(crate) fn of(named: Option<Named>, outcome: Outcome) -> Self {
        match named {
            Some(named) if !named.agrees_with(outcome.appears()) => {
                Self::NotWhatItsNameSays { named, outcome }
            }
            _ => Self::AsItIs(outcome),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::appears::Macros;
    use crate::kind::Kind;
    use crate::outcome::Cannot;
    use crate::testing::in_english;

    /// **A name that tells the truth, or says nothing, is no finding.**
    #[test]
    fn a_truthful_name_is_no_finding() {
        let opened = Outcome::OpensAsItIs {
            kind: Kind::Pdf,
            macros: Macros::NoneSeen,
        };
        assert_eq!(
            Decided::of(Some(Named::A(Kind::Pdf)), opened),
            Decided::AsItIs(opened)
        );
        assert_eq!(Decided::of(None, opened), Decided::AsItIs(opened));
    }

    /// **A name that lies is the finding, and it is said first**, before what
    /// the machine can do with what the file really is.
    #[test]
    fn a_name_that_lies_is_said_first() {
        let strings = in_english();
        let decided = Decided::of(
            Some(Named::A(Kind::Pdf)),
            Outcome::CannotOpen(Cannot::AProgram),
        );
        assert!(matches!(decided, Decided::NotWhatItsNameSays { .. }));
        let said: Vec<String> = decided
            .said(&strings)
            .iter()
            .map(|said| said.text().to_owned())
            .collect();
        assert_eq!(
            said,
            [
                "This file is named as a PDF document, but it is a program",
                "This file is a program. Opening a file never runs a program, so it has not been \
                 opened",
                "If a document was expected, whoever sent it can send the document itself instead"
            ]
        );
    }

    /// A file named as something that is nothing recognisable is said to be
    /// not one, rather than given a name it has not earned.
    #[test]
    fn a_name_for_bytes_nothing_recognises_is_not_one() {
        let strings = in_english();
        let said: Vec<String> = Decided::of(
            Some(Named::A(Kind::WordDocument)),
            Outcome::CannotOpen(Cannot::Unrecognised),
        )
        .said(&strings)
        .iter()
        .map(|said| said.text().to_owned())
        .collect();
        assert_eq!(
            said.first().map(String::as_str),
            Some("This file is named as a Word document, but it is not one")
        );
    }
}
