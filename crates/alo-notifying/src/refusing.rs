//! Why a notification was never accepted, in words a person can act on.
//!
//! Five refusals, and every one of them is a fault in the program that sent
//! it rather than anything the person did — which is why every one of them
//! names that program. *A notification could not be shown* with nothing beside
//! it is a sentence that leaves somebody staring at a machine they now trust
//! slightly less for no reason they can find.
//!
//! **A refused notification is not shown, not held and not written down.**
//! There is no half-accepted notification: the road through
//! [`crate::arriving`] either produces a [`crate::Notification`] or produces
//! one of these, and nothing else is reachable.
//!
//! There is no `Display`, for the reason the rest of this crate has none: the
//! only road to words is [`NotSent::said`], in the language the person reads.

use alo_strings::{Filling, Said, Strings, Word};

use crate::words;

/// The gap a refusal fills with the program that sent it.
const APPLICATION: &str = "application";

/// The gap [`NotSent::TooManyThingsToDo`] fills with how many were offered.
const OFFERED: &str = "offered";

/// The gap [`NotSent::TooManyThingsToDo`] fills with how many there may be.
const MOST: &str = "most";

/// Why a notification was not accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotSent {
    /// What arrived was a permission to do something that is not notifying.
    NotANotification {
        /// The program that sent it.
        application: String,
    },
    /// The notification had nothing readable at the top of it.
    NoTitle {
        /// The program that sent it.
        application: String,
    },
    /// Something it offered the person had no words on it.
    NoLabel {
        /// The program that sent it.
        application: String,
    },
    /// Something it offered the person had no name its sender knows it by.
    NoActionName {
        /// The program that sent it.
        application: String,
    },
    /// It offered more things to do than a notification may.
    TooManyThingsToDo {
        /// The program that sent it.
        application: String,
        /// How many it offered.
        offered: usize,
        /// How many it may offer.
        most: usize,
    },
}

impl NotSent {
    /// The program that sent it, by the identifier this machine knows it by.
    #[must_use]
    pub fn application(&self) -> &str {
        match self {
            Self::NotANotification { application }
            | Self::NoTitle { application }
            | Self::NoLabel { application }
            | Self::NoActionName { application }
            | Self::TooManyThingsToDo { application, .. } => application,
        }
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> Word {
        match self {
            Self::NotANotification { .. } => words::NOT_A_NOTIFICATION,
            Self::NoTitle { .. } => words::NO_TITLE,
            Self::NoLabel { .. } => words::NO_LABEL,
            Self::NoActionName { .. } => words::NO_ACTION_NAME,
            Self::TooManyThingsToDo { .. } => words::TOO_MANY_THINGS_TO_DO,
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let mut filling = Filling::of(APPLICATION, self.application().to_owned());
        if let Self::TooManyThingsToDo { offered, most, .. } = self {
            filling = filling
                .and(OFFERED, offered.to_string())
                .and(MOST, most.to_string());
        }
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// Every refusal, for the tests that walk all of them.
    fn every_refusal() -> Vec<NotSent> {
        let application = "org.example.Mail".to_owned();
        vec![
            NotSent::NotANotification {
                application: application.clone(),
            },
            NotSent::NoTitle {
                application: application.clone(),
            },
            NotSent::NoLabel {
                application: application.clone(),
            },
            NotSent::NoActionName {
                application: application.clone(),
            },
            NotSent::TooManyThingsToDo {
                application,
                offered: 9,
                most: crate::action::MOST_THINGS_TO_DO,
            },
        ]
    }

    /// **Every refusal is said with every gap filled and the program named.**
    /// A refusal with a hole in it is a refusal nobody can act on, and one
    /// that does not name the program is one nobody can attribute.
    #[test]
    fn every_refusal_is_said_with_the_program_named() {
        let strings = in_english();
        for refused in every_refusal() {
            let said = refused.said(&strings);
            assert!(words::EVERY_WORD.contains(&refused.word()), "{refused:?}");
            assert!(!said.is_a_bug(), "{refused:?} is not declared");
            assert!(said.unfilled().is_empty(), "{refused:?}: {said}");
            assert!(
                said.text().contains("org.example.Mail"),
                "{refused:?}: {said}"
            );
        }
    }

    /// **No two refusals read the same.** Five sentences for five faults: a
    /// programmer told the same thing for an empty title and for a permission
    /// to use the camera has been told nothing.
    #[test]
    fn no_two_refusals_read_the_same() {
        let strings = in_english();
        let mut seen: Vec<String> = Vec::new();
        for refused in every_refusal() {
            let text = refused.said(&strings).into_text();
            assert!(!seen.contains(&text), "two refusals both read {text}");
            seen.push(text);
        }
    }

    /// **The one refusal about a count says both numbers**, because *too many*
    /// without them is an argument rather than a fact.
    #[test]
    fn the_refusal_about_a_count_says_both_numbers() {
        let strings = in_english();
        let said = NotSent::TooManyThingsToDo {
            application: "org.example.Mail".to_owned(),
            offered: 9,
            most: 3,
        }
        .said(&strings);
        assert!(said.text().contains('9'), "{said}");
        assert!(said.text().contains('3'), "{said}");
    }
}
