//! One answer read back off the disk, in the language the person reads.
//!
//! A [`KeptAnswer`] holds identities. [`KeptAnswer::said`] gives back what a
//! person reading *what did my applications ask for, and what were they told*
//! is shown for one of them ([`AnswerSaid`]): who asked, what the portal lets
//! an application do (task 1's sentence), and what it was answered with
//! (`crate::kept_outcome_said`, in the words the backend answered with).
//!
//! **The moment is kept as a moment**, never written into a sentence: how a
//! date is written belongs to the reader's region, which is not the same thing
//! as their language, and whatever draws the list knows the region.
//!
//! Nothing here draws a list, and nothing decides who may read the file.

use std::time::SystemTime;

use alo_strings::{Said, Strings};

use crate::kept_answer::KeptAnswer;
use crate::kept_outcome_said::naming;
use crate::words;

/// One answer, said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnswerSaid {
    /// When it was answered.
    at: SystemTime,
    /// Who asked.
    who: Said,
    /// What the portal it asked lets an application do.
    portal: Said,
    /// What it was answered with.
    answer: Said,
}

impl AnswerSaid {
    /// When it was answered, as a moment for the reader's region to write.
    #[must_use]
    pub const fn at(&self) -> SystemTime {
        self.at
    }

    /// Who asked: the application by its identifier, or *an application that
    /// could not be named*.
    #[must_use]
    pub const fn who(&self) -> &Said {
        &self.who
    }

    /// What the portal it asked lets an application do.
    #[must_use]
    pub const fn portal(&self) -> &Said {
        &self.portal
    }

    /// What it was answered with.
    #[must_use]
    pub const fn answer(&self) -> &Said {
        &self.answer
    }

    /// The three sentences, in the order they are read: who, the portal, the
    /// answer.
    #[must_use]
    pub fn sentences(&self) -> [&Said; 3] {
        [&self.who, &self.portal, &self.answer]
    }
}

impl KeptAnswer {
    /// This answer, in the language the person reads.
    ///
    /// Never fails and never panics: a `Strings` missing a word answers with
    /// the key, marked as a bug, as every `said` on this machine does.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> AnswerSaid {
        let application = self.application();
        AnswerSaid {
            at: self.at(),
            who: strings.say(&words::ASKED_BY.key(), &naming(application, strings)),
            portal: self.portal().said(strings),
            answer: self.outcome().said_for(application, self.portal(), strings),
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
    use crate::answered::{Answered, Outcome, Unanswered};
    use crate::portal::Portal;
    use alo_capability::Applicant;
    use std::time::Duration;

    /// **Who, the portal and the answer**, and the moment left a moment.
    #[test]
    fn an_answer_is_said_as_who_the_portal_and_what_it_was_told() {
        let strings = Strings::of(crate::portal_words().unwrap());
        let at = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
        let kept = KeptAnswer::from(&Answered::new(
            at,
            Some(Applicant::named("org.gnome.Fractal")),
            Portal::Secret,
            Outcome::Unanswered(Unanswered::KeyringUnavailable),
        ));
        let said = kept.said(&strings);
        assert_eq!(said.at(), at);
        assert_eq!(said.who().text(), "Asked by org.gnome.Fractal");
        assert_eq!(said.portal(), &Portal::Secret.said(&strings));
        assert_eq!(
            said.answer(),
            &Unanswered::KeyringUnavailable.said(&strings)
        );
        for sentence in said.sentences() {
            assert!(!sentence.text().contains("1760000000"), "{sentence}");
        }
    }
}
