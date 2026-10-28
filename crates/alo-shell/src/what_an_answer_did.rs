//! What a person is told in the conversation once they answered a waiting
//! change.
//!
//! **Nothing was said at all until 2026-10-26.** Walking a booted machine, a
//! person pressed *Allow once*, the daemon refused the change, and the screen
//! showed only that the answers had gone: the reply was read and dropped, and
//! the task kept saying *Waiting for your permission*. A person who cannot see
//! whether their approval worked has not been given a choice, only a button.
//!
//! So every answer ends in one line from alo, and the line says which of five
//! things happened. A refusal is the daemon's own words, unchanged, because a
//! reason re-worded here would be one nobody gave.

use alo_changing::Answered;
use alo_strings::{Filling, Strings};

use crate::the_cards_words::{
    IT_WAS_DONE, IT_WAS_DONE_AND_CANNOT_BE_UNDONE, NO_WORD_BACK, YOU_SAID_NO,
};

/// The line alo says once the daemon has answered, or failed to.
pub(crate) fn what_an_answer_did(answered: Option<&Answered>, strings: &Strings) -> String {
    let word = match answered {
        Some(Answered::Refused { told }) => return told.clone(),
        Some(Answered::It(_)) => IT_WAS_DONE,
        // **Since 2026-10-28.** The daemon ran the change and could not keep a
        // copy from before it; *Done.* alone would read as a change `alo-back`
        // can take back (ADR 0045's third term).
        Some(Answered::ItCannotBeUndone(_)) => IT_WAS_DONE_AND_CANNOT_BE_UNDONE,
        Some(Answered::Declined) => YOU_SAID_NO,
        None => NO_WORD_BACK,
    };
    strings
        .say(&word.key(), &Filling::nothing())
        .text()
        .to_owned()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_changing::Done;
    use alo_strings::Vocabulary;

    /// This surface's words, untranslated.
    fn in_english() -> Strings {
        let mut vocabulary = Vocabulary::empty();
        crate::the_cards_words::declare_the_cards_words(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// **Each of the four is told apart**, and none of them reads as another —
    /// above all, *nobody answered* never reads as *done* or as *you said no*.
    #[test]
    fn each_outcome_says_something_different() {
        let strings = in_english();
        let done = what_an_answer_did(
            Some(&Answered::It(Box::new(Done::Renamed { now_at: None }))),
            &strings,
        );
        let declined = what_an_answer_did(Some(&Answered::Declined), &strings);
        let silent = what_an_answer_did(None, &strings);

        assert_eq!(done, "Done.");
        assert_eq!(declined, "You said no, so nothing was changed.");
        assert!(silent.contains("did not say"), "{silent}");
        assert_ne!(silent, done);
        assert_ne!(silent, declined);
    }

    /// **A change that cannot be undone says so, and plainly.** It is told
    /// apart from *Done.*, it says it was done, and it never softens into
    /// *may not*.
    #[test]
    fn a_change_that_cannot_be_undone_says_so() {
        let strings = in_english();
        let done = what_an_answer_did(
            Some(&Answered::It(Box::new(Done::Renamed { now_at: None }))),
            &strings,
        );
        let not_kept = what_an_answer_did(
            Some(&Answered::ItCannotBeUndone(Box::new(Done::Renamed {
                now_at: None,
            }))),
            &strings,
        );

        assert_eq!(
            not_kept,
            "Done. This change cannot be undone: alo could not keep a copy of your files from \
             before it."
        );
        assert_ne!(not_kept, done);
        assert!(!not_kept.contains("may"), "{not_kept}");
        assert!(!not_kept.contains("might"), "{not_kept}");
    }

    /// **A refusal is the daemon's own words, unchanged** — the walk's own
    /// refusal, as the record held it.
    #[test]
    fn a_refusal_is_told_in_the_daemons_words() {
        let told = "/home/alo/Pictures really leads to /var/home/alo/Pictures";
        let said = what_an_answer_did(
            Some(&Answered::Refused {
                told: told.to_owned(),
            }),
            &in_english(),
        );
        assert_eq!(said, told);
    }
}
