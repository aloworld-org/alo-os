//! What letting go here would do, while the person is still holding it.
//!
//! A drag is answered before it is committed, for `alo-dividing`'s reason in
//! another shape: a person dragging a file across four windows is asking, four
//! times, *what happens if I let go now* — and a system that answers only by
//! doing it has made every drag a thing to undo.
//!
//! So this is the pointer's own sentence. It says whether the window takes what
//! is being dragged, whether letting go copies or moves it, and — the one that
//! matters most in this product — that letting go on the agent's surface offers
//! it for this question and hands over nothing else.
//!
//! **Nothing here changes anything.** [`crate::Drag::over`] takes `&self` and
//! the application is never asked for a byte: what is being dragged is not read
//! until a person lets go.

use alo_clipboard::{Kind, Taking};
use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// What would happen if the person let go where the pointer is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Over {
    /// This window takes it, in this form, and letting go copies or moves it.
    WouldHandItOver {
        /// The form it would be handed over in — the first the source offered
        /// that this window takes.
        form: Kind,
        /// Whether the window it came from would keep its own.
        taking: Taking,
    },
    /// Letting go offers it to the agent for this question, and grants nothing.
    WouldOfferItForThisQuestion {
        /// The form the agent would read it in.
        form: Kind,
    },
    /// This window takes none of the forms on offer.
    WouldNotTakeIt,
    /// There is nothing under the pointer that takes a drop.
    NothingHere,
}

impl Over {
    /// The string this crate declares for it: the label beside the pointer.
    ///
    /// [`None`] over nothing, because there is nothing to say — a pointer over
    /// the empty desktop shows that letting go does nothing, and a sentence
    /// about it would be the machine narrating a person's own arm.
    #[must_use]
    pub const fn word(&self) -> Option<Word> {
        match self {
            Self::WouldHandItOver {
                taking: Taking::Copied,
                ..
            } => Some(words::COPY_HERE),
            Self::WouldHandItOver {
                taking: Taking::Cut,
                ..
            } => Some(words::MOVE_HERE),
            Self::WouldOfferItForThisQuestion { .. } => Some(words::ASK_ABOUT_THIS),
            Self::WouldNotTakeIt => Some(words::WILL_NOT_TAKE_IT),
            Self::NothingHere => None,
        }
    }

    /// What the pointer says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Option<Said> {
        self.word()
            .map(|word| strings.say(&word.key(), &Filling::nothing()))
    }

    /// The form letting go would use, when letting go would do anything.
    #[must_use]
    pub const fn form(&self) -> Option<&Kind> {
        match self {
            Self::WouldHandItOver { form, .. } | Self::WouldOfferItForThisQuestion { form } => {
                Some(form)
            }
            Self::WouldNotTakeIt | Self::NothingHere => None,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use std::collections::BTreeSet;

    /// Every answer the pointer can give says something of its own, with
    /// nothing left to fill in — except over nothing, where there is nothing to
    /// say.
    #[test]
    fn every_answer_says_something_of_its_own() {
        let strings = in_english();
        let mut seen = BTreeSet::new();
        for over in [
            Over::WouldHandItOver {
                form: Kind::text(),
                taking: Taking::Copied,
            },
            Over::WouldHandItOver {
                form: Kind::files(),
                taking: Taking::Cut,
            },
            Over::WouldOfferItForThisQuestion { form: Kind::text() },
            Over::WouldNotTakeIt,
        ] {
            let said = over.said(&strings).expect("a label");
            assert!(!said.is_a_bug(), "{over:?}");
            assert!(said.unfilled().is_empty(), "{over:?}: {said}");
            assert!(seen.insert(said.text().to_owned()), "{over:?}");
        }
        assert_eq!(Over::NothingHere.said(&strings), None);
    }

    /// Copying and moving are two different sentences, because they are two
    /// different things to have done to somebody's file.
    #[test]
    fn copying_and_moving_do_not_read_the_same() {
        let strings = in_english();
        let copying = Over::WouldHandItOver {
            form: Kind::files(),
            taking: Taking::Copied,
        };
        let moving = Over::WouldHandItOver {
            form: Kind::files(),
            taking: Taking::Cut,
        };
        assert_ne!(copying.said(&strings), moving.said(&strings));
        assert_eq!(copying.form(), Some(&Kind::files()));
        assert_eq!(Over::WouldNotTakeIt.form(), None);
        assert_eq!(Over::NothingHere.form(), None);
    }
}
