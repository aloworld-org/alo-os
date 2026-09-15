//! Where the backend writes down every answer it gave.
//!
//! [`Recording`] is the one door: the backend hands it an [`Answered`] for each
//! request, before the application is sent the response, so an answer that
//! reached an application and is missing from the record is not a state the
//! backend can be in.
//!
//! # Why this is not `alo-record`
//!
//! `alo-record` is *what an agent did*. Every entry it can make names an agent
//! or names nobody, and `alo_record::Only::ByAnAgent` is the question a
//! person who declined the agent puts to it: *is there anything in here an agent
//! did?* Writing an application's portal request there under the agent column
//! would answer that question wrongly, which ADR 0040 part 2 — *no refusal calls
//! an application an agent* — rules out; and that crate is another lane's. So
//! an application's requests are recorded as themselves, and the proposal that
//! the record file gains an application's kind of entry is in the report of the
//! task that built this.
//!
//! [`Kept`] keeps them in memory, which is what `alo_record::Record` is too:
//! writing a record to a file is the service's, and the service that runs this
//! backend on a machine is not built yet.

use std::sync::{Mutex, PoisonError};

use crate::answered::Answered;

/// Something every answer is written into.
pub trait Recording: Send + Sync {
    /// Keep this answer.
    fn keep(&self, answered: Answered);
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
    fn keep(&self, answered: Answered) {
        // A panic elsewhere while the list was held leaves a list, and an
        // answer is still worth keeping in it.
        self.answers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(answered);
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
            kept.keep(Answered::new(
                SystemTime::UNIX_EPOCH,
                None,
                portal,
                Outcome::Unanswered(Unanswered::NotIdentified),
            ));
        }
        let portals: Vec<Portal> = kept.everything().iter().map(Answered::portal).collect();
        assert_eq!(portals, [Portal::Secret, Portal::OpenWith]);
    }
}
