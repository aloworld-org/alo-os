//! Everything read back off the answers file, in the language the person reads.
//!
//! [`ReadBack::said`] gives back [`ReadBackSaid`]: every answer that read, said
//! ([`crate::KeptAnswer::said`]), with two things beside them that decide how
//! the list may be read.
//!
//! - **Whether the file still reaches back to its first answer.** A file
//!   shortened under the machine's record rule (`crate::shortening`) says so in
//!   its first line, and here that becomes a sentence, with the moment it now
//!   starts at kept as a moment and the rule said by `alo-keeping`. It is the
//!   difference between *no application asked anything in March* and *this
//!   list does not reach March*, which is `alo_keeping::Head::said`'s reason
//!   said of applications.
//! - **How many lines did not read**, as a count beside the answers that did,
//!   never dropped and never in place of them.
//!
//! Nothing here draws the list, and nothing decides who may read the file.

use std::time::SystemTime;

use alo_strings::{Counting, Filling, Said, Strings};

use crate::answers_file::ReadBack;
use crate::kept_answer_said::AnswerSaid;
use crate::words;

/// Everything read back, said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadBackSaid {
    /// Whether the file is whole or shortened.
    reaches: Said,
    /// The moment a shortened file starts at.
    since: Option<SystemTime>,
    /// The rule a shortened file was shortened under.
    under: Option<Said>,
    /// Every answer that read, oldest first.
    answers: Vec<AnswerSaid>,
    /// How many lines did not read, where any did not.
    not_read: Option<Said>,
}

impl ReadBackSaid {
    /// Whether the file still holds everything it was given, or was shortened
    /// and does not go all the way back.
    #[must_use]
    pub const fn reaches(&self) -> &Said {
        &self.reaches
    }

    /// The moment a shortened file now starts at, as a moment for the reader's
    /// region to write — [`None`] for a file nothing has been removed from.
    #[must_use]
    pub const fn since(&self) -> Option<SystemTime> {
        self.since
    }

    /// The rule the file was shortened under, where it was.
    #[must_use]
    pub const fn under(&self) -> Option<&Said> {
        self.under.as_ref()
    }

    /// Every answer that read, oldest first.
    #[must_use]
    pub fn answers(&self) -> &[AnswerSaid] {
        &self.answers
    }

    /// How many lines did not read, counted in the reader's language —
    /// [`None`] only where every line read.
    #[must_use]
    pub const fn not_read(&self) -> Option<&Said> {
        self.not_read.as_ref()
    }
}

impl ReadBack {
    /// Whether nothing has ever been removed from the file.
    #[must_use]
    pub const fn is_whole(&self) -> bool {
        self.since.is_none()
    }

    /// Everything read back, in the language the person reads.
    ///
    /// Never fails and never panics: a `Strings` missing a word answers with
    /// the key, marked as a bug, as every `said` on this machine does.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> ReadBackSaid {
        let reaches = if self.is_whole() {
            words::WHOLE
        } else {
            words::SHORTENED
        };
        let not_read = (!self.unreadable.is_empty()).then(|| {
            strings.count(
                &words::LINES_NOT_READ.key(),
                &Counting::of(u64::try_from(self.unreadable.len()).unwrap_or(u64::MAX)),
                &Filling::nothing(),
            )
        });
        ReadBackSaid {
            reaches: strings.say(&reaches.key(), &Filling::nothing()),
            since: self.since,
            under: self.under.map(|under| under.said(strings)),
            answers: self
                .answers
                .iter()
                .map(|answer| answer.said(strings))
                .collect(),
            not_read,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_keeping::Keeping;

    /// **Whole says whole; shortened says so, and the moment stays a moment.**
    #[test]
    fn whole_and_shortened_are_two_sentences() {
        let strings = Strings::of(crate::portal_words().unwrap());
        let mut read = ReadBack {
            answers: Vec::new(),
            unreadable: Vec::new(),
            since: None,
            under: None,
        };
        let whole = read.said(&strings);
        assert_eq!(whole.reaches().text(), words::WHOLE.says());
        assert_eq!(whole.since(), None);
        assert_eq!(whole.under(), None);
        assert_eq!(whole.not_read(), None);

        read.since = Some(SystemTime::UNIX_EPOCH);
        read.under = Keeping::for_days(7).ok();
        read.unreadable = vec![4];
        let shortened = read.said(&strings);
        assert_eq!(shortened.reaches().text(), words::SHORTENED.says());
        assert_eq!(shortened.since(), Some(SystemTime::UNIX_EPOCH));
        assert!(shortened.under().is_some());
        assert!(!shortened.not_read().unwrap().is_a_bug());
    }
}
