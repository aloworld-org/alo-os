//! **Where one described thing is in a picture of a window** — the question
//! put to the reader the person chose, and the reading of its answer, for
//! `accessible.click_at` (ADR 0147, amended 2026-11-21: *"Yes, click by
//! description"*).
//!
//! The model describes the thing it wants clicked, as a person sees it; it is
//! never told, and never says, where. **The reader is asked**, through the road
//! every picture takes (the indicator and the record), with
//! [`the_question`]; its answer is one line, read by [`found`]: how many things
//! match, then the centre of the one that does, in whole per cent of the
//! window.
//!
//! **Exactly one gives a point.** None, more than one, or an answer that is not
//! that one line each refuse, in the person's words, and nothing is clicked: a
//! reader that wandered is not a reader whose numbers alo clicks at.
//!
//! **One function for both sides**: the daemon asks the reader with
//! [`the_question`] and the turn reads its answer with [`found`]; the every-try
//! check reads its scripted reader's answer with [`found`] too.

use alo_strings::{Filling, Said, Strings, Word};

use crate::click_words as words;

/// The name every click sentence fills with the application.
pub const APPLICATION: &str = "application";

/// The name every click sentence fills with the description.
pub const DESCRIPTION: &str = "description";

/// Why the reader's answer gives no point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotFound {
    /// It found no such thing.
    Nowhere,
    /// It found more than one.
    MoreThanOnce,
    /// Its answer was not the one line asked for.
    Unreadable,
}

impl NotFound {
    /// What a person is told.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Nowhere => words::NOT_FOUND,
            Self::MoreThanOnce => words::FOUND_MORE_THAN_ONCE,
            Self::Unreadable => words::NOT_UNDERSTOOD,
        }
    }

    /// What a person is told, filled with the application as they call it
    /// and the description.
    #[must_use]
    pub fn said(self, application: &str, description: &str, strings: &Strings) -> Said {
        strings.say(
            &self.word().key(),
            &Filling::of(APPLICATION, application.to_owned())
                .and(DESCRIPTION, description.to_owned()),
        )
    }
}

/// What the reader is asked, beside the picture: where `description` is in
/// the window of `application` (as a person calls it).
#[must_use]
pub fn the_question(application: &str, description: &str, strings: &Strings) -> Said {
    let _ = (application, description);
    strings.say(&words::WHERE_IS_IT.key(), &Filling::nothing())
}

/// The point the reader's `answer` gives, in whole per cent of the window:
/// across, then down.
///
/// # Errors
/// [`NotFound`]: none found, more than one, or an answer that is not one line
/// of whole numbers — a count, and after a count of one, two numbers from 0 to
/// 100.
pub fn found(answer: &str) -> Result<(u8, u8), NotFound> {
    let _ = answer;
    Err(NotFound::Unreadable)
}

#[cfg(test)]
#[path = "where_a_thing_is_tests.rs"]
mod tests;
