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
use crate::fallback_verbs::THE_WHOLE_WAY;

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
    strings.say(
        &words::WHERE_IS_IT.key(),
        &Filling::of(APPLICATION, application.to_owned()).and(DESCRIPTION, description.to_owned()),
    )
}

/// The point the reader's `answer` gives, in whole per cent of the window:
/// across, then down.
///
/// **One line of whole numbers**, whatever is between them: a count, and
/// after a count of one, two numbers from 0 to 100. A minus sign or a
/// decimal point is not a whole number of per cent, and a second line is not
/// the one line asked for.
///
/// # Errors
/// [`NotFound`]: none found, more than one, or an answer that is not that one
/// line.
pub fn found(answer: &str) -> Result<(u8, u8), NotFound> {
    let mut lines = answer.lines().filter(|line| !line.trim().is_empty());
    let (Some(line), None) = (lines.next(), lines.next()) else {
        return Err(NotFound::Unreadable);
    };
    let said: Vec<char> = line.chars().collect();
    let a_fraction = said
        .windows(3)
        .any(|three| matches!(three, [before, '.', after] if before.is_ascii_digit() && after.is_ascii_digit()));
    let below_nothing = said
        .windows(2)
        .any(|two| matches!(two, ['-', after] if after.is_ascii_digit()));
    if a_fraction || below_nothing {
        return Err(NotFound::Unreadable);
    }
    let numbers: Option<Vec<u32>> = line
        .split(|one: char| !one.is_ascii_digit())
        .filter(|run| !run.is_empty())
        .map(|run| run.parse().ok())
        .collect();
    let Some(numbers) = numbers else {
        return Err(NotFound::Unreadable);
    };
    let per_cent = |number: u32| {
        u8::try_from(number)
            .ok()
            .filter(|number| *number <= THE_WHOLE_WAY)
    };
    match numbers.as_slice() {
        [0, ..] => Err(NotFound::Nowhere),
        [1, across, down] => match (per_cent(*across), per_cent(*down)) {
            (Some(across), Some(down)) => Ok((across, down)),
            _ => Err(NotFound::Unreadable),
        },
        [1, ..] | [] => Err(NotFound::Unreadable),
        [_, ..] => Err(NotFound::MoreThanOnce),
    }
}

#[cfg(test)]
#[path = "where_a_thing_is_tests.rs"]
mod tests;
