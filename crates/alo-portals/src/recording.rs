//! Where the backend writes down every answer it gave.
//!
//! [`Recording`] is the one door: the backend hands it an [`Answered`] for each
//! request, before the application is sent the response, so an answer that
//! reached an application and is missing from the record is not a state the
//! backend can be in.
//!
//! # A record that cannot keep an answer refuses it
//!
//! [`Recording::keep`] can fail, and says so ([`NotRecorded`]). The backend
//! reads that as a refusal: the application is sent the portal's refusal
//! rather than what was decided, no secret is handed over and no file is
//! opened. Law 3's *every execution and every refusal leaves a record* is a
//! promise about what a person can read later, and an answer given while the
//! record was failing would break it silently.
//!
//! # Two records
//!
//! - [`Kept`] keeps answers in memory, and is gone when the backend stops. It is
//!   what a test that asks about answers in the same process holds.
//! - `AnswersFile` (Unix) keeps them in a file, one line each, and a backend
//!   started again over the same file finds what the last one wrote. It is
//!   what a machine runs; `docs/contracts/portal-answers-file.md` is its shape.
//!
//! # Why this is not `alo-record`
//!
//! `alo-record` is *what an agent did*. Every entry it can make names an agent
//! or names nobody, and `alo_record::Only::ByAnAgent` is the question a
//! person who declined the agent puts to it: *is there anything in here an agent
//! did?* Writing an application's portal request there under the agent column
//! would answer that question wrongly, which ADR 0040 part 2 — *no refusal calls
//! an application an agent* — rules out; and that crate is another lane's. So
//! an application's requests are recorded as themselves, in a file of their
//! own beside the agent's record.

use std::sync::{Mutex, PoisonError};

use crate::answered::Answered;
use crate::not_recorded::NotRecorded;

/// Something every answer is written into.
pub trait Recording: Send + Sync {
    /// Keep this answer, and say whether it was kept.
    ///
    /// # Errors
    /// [`NotRecorded`] when the answer is not in the record. The backend then
    /// sends the application the portal's refusal instead of the answer.
    fn keep(&self, answered: Answered) -> Result<(), NotRecorded>;
}

/// Every answer, in the order the backend gave them, in memory.
#[derive(Debug, Default)]
pub struct Kept {
    /// The answers.
    answers: Mutex<Vec<Answered>>,
}

impl Kept {
    /// Nothing answered yet.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// Every answer so far, oldest first.
    #[must_use]
    pub fn everything(&self) -> Vec<Answered> {
        self.answers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl Recording for Kept {
    fn keep(&self, answered: Answered) -> Result<(), NotRecorded> {
        // A panic elsewhere while the list was held leaves a list, and an
        // answer is still worth keeping in it.
        self.answers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(answered);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::answered::{Outcome, Unanswered};
    use crate::portal::Portal;
    use std::time::SystemTime;

    /// **Kept in the order given**, and nothing is dropped.
    #[test]
    fn every_answer_is_kept_in_order() {
        let kept = Kept::nothing();
        assert!(kept.everything().is_empty());
        for portal in [Portal::Secret, Portal::OpenWith] {
            assert_eq!(
                kept.keep(Answered::new(
                    SystemTime::UNIX_EPOCH,
                    None,
                    portal,
                    Outcome::Unanswered(Unanswered::NotIdentified),
                )),
                Ok(())
            );
        }
        let portals: Vec<Portal> = kept.everything().iter().map(Answered::portal).collect();
        assert_eq!(portals, [Portal::Secret, Portal::OpenWith]);
    }
}
