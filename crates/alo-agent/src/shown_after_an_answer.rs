//! **What the model is shown after the person answered a change**: exactly
//! what it is shown after the same change at full access.
//!
//! Measured with `alo-every-try` on 2026-11-16: shown the daemon's bare
//! `did`, a model proposed an approved move again; shown *Done after the
//! person approved it: …*, it kept going after one-change requests (open an
//! application 2 of 5 at *ask each time*, run on #1326's head 70d0986e), while
//! the same requests at full access answered in words. So a change that ran
//! after an approval is shown **byte for byte** as full access shows one: the
//! daemon's `done-without-asking`, with what the change did and the sentence
//! `alo_capability::words::DONE_AT_FULL_ACCESS` (or its *cannot be put back*
//! twin) says, in the language the code is written in. The person's approval
//! is already in the record; the model does not need it.
//!
//! A decline and a refusal at the approval are shown as
//! `alo_protocol::what_the_agent_says_back::the_step_after_an_answer` shows
//! them: a refusal, which full access has no equivalent of.

use alo_capability::words::{DONE_AT_FULL_ACCESS, DONE_AT_FULL_ACCESS_AND_IT_CANNOT_BE_PUT_BACK};
use alo_protocol::{ToAPerson, ToAnAgent};
use alo_strings::{Filling, Strings, Vocabulary};

/// The line the model is shown as the step after a change the person
/// answered, whose sentence was `sentence` — [`None`] for an answer an approve
/// or a decline does not earn.
#[must_use]
pub fn the_step_after_an_answer(told: &ToAPerson, sentence: &str) -> Option<String> {
    let word = match told {
        ToAPerson::Did(_) => DONE_AT_FULL_ACCESS,
        ToAPerson::DidAndCannotBeUndone(_) => DONE_AT_FULL_ACCESS_AND_IT_CANNOT_BE_PUT_BACK,
        _ => {
            return alo_protocol::what_the_agent_says_back::the_step_after_an_answer(
                told, sentence,
            );
        }
    };
    let (ToAPerson::Did(did) | ToAPerson::DidAndCannotBeUndone(did)) = told else {
        return None;
    };
    let said = the_words()?.say(&word.key(), &Filling::of("sentence", sentence.to_owned()));
    ToAnAgent::done_without_asking(did.clone(), &said)
        .written()
        .ok()
}

/// The capability model's own words, in the language the code is written in.
fn the_words() -> Option<Strings> {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).ok()?;
    Some(Strings::of(vocabulary))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_protocol::Done;

    /// **An approved change is shown exactly as full access shows the same
    /// change**: the same message, the same sentence, byte for byte.
    #[test]
    fn an_approved_change_is_shown_as_full_access_shows_it() {
        let did = Done::Renamed { now_at: None };
        let sentence = "Rename march.pdf to march-final.pdf";
        let mut vocabulary = Vocabulary::empty();
        alo_capability::declare_into(&mut vocabulary).unwrap();
        let strings = Strings::of(vocabulary);
        for (told, word) in [
            (ToAPerson::Did(did.clone()), DONE_AT_FULL_ACCESS),
            (
                ToAPerson::DidAndCannotBeUndone(did.clone()),
                DONE_AT_FULL_ACCESS_AND_IT_CANNOT_BE_PUT_BACK,
            ),
        ] {
            let at_full_access = ToAnAgent::done_without_asking(
                did.clone(),
                &strings.say(&word.key(), &Filling::of("sentence", sentence.to_owned())),
            )
            .written()
            .unwrap();
            assert_eq!(
                the_step_after_an_answer(&told, sentence).unwrap(),
                at_full_access
            );
        }
    }

    /// **A decline is shown as a refusal**, never as something done.
    #[test]
    fn a_decline_is_shown_as_a_refusal() {
        let shown = the_step_after_an_answer(&ToAPerson::Declined, "Move a.txt to b").unwrap();
        assert!(
            matches!(ToAnAgent::read(&shown), Ok(ToAnAgent::Refused(_))),
            "{shown}"
        );
    }
}
