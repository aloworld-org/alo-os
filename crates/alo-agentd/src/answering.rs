//! One line from the person's shell, answered against the turn that is under
//! way.
//!
//! [`crate::doing`]'s twin, and it differs in one thing that shapes the whole
//! file: **there may be no turn.** A shell is signed in for as long as the
//! person is, and an agent's turn lasts a few seconds in the middle of that, so
//! every question here has to have an answer for a machine on which nothing is
//! happening.
//!
//! # A number off the wire is not a handle
//!
//! `alo_protocol::FromAPerson` carries a number, because a number is what a
//! shell drew beside a sentence. It is **found** among what is really waiting
//! rather than turned into a handle, and a number nothing is waiting under is
//! the capability model's own refusal. That is what stops a shell — or anything
//! that can reach the person's door — from answering a change by guessing at a
//! number.
//!
//! # Nothing is waiting is an answer, not a refusal
//!
//! *What is waiting* on a machine with no turn under way is an empty list, and
//! it is the same empty list a turn with nothing outstanding gives. A shell
//! drawing the person's changes must not have to tell those apart, because to
//! the person they are one fact: there is nothing to answer.
//!
//! **Answering** something when there is no turn is different, and is refused:
//! a number that is not waiting is a number that is not waiting, whether the
//! turn ended a moment ago or never began. What the person is told is
//! `alo_capability::AnswerError::NothingWaiting`'s sentence, which is the one
//! they would have been told inside a turn — so a change that lapsed while they
//! were reading it does not produce a different explanation depending on
//! whether the agent has hung up yet.

use std::time::SystemTime;

use alo_capability::{AnswerError, Grants, ProposalId};
use alo_protocol::{FromAPerson, ToAPerson};
use alo_strings::Strings;
use alo_turn::Turning;

/// Read one line as something the person answered, and answer it.
///
/// Answers with what to say back, always. A message that was not a request is
/// refused in `alo-protocol`'s own words, and nothing reaches the turn.
pub fn what_a_person_said(
    line: &str,
    turning: Option<&mut Turning<'_, '_>>,
    grants: &Grants,
    strings: &Strings,
    now: SystemTime,
) -> ToAPerson {
    match FromAPerson::read(line) {
        Ok(answered) => answered_to(answered, turning, grants, strings, now),
        Err(why) => ToAPerson::refused(&why.said(strings)),
    }
}

/// The three things a person can send, each against the turn if there is one.
fn answered_to(
    answered: FromAPerson,
    turning: Option<&mut Turning<'_, '_>>,
    grants: &Grants,
    strings: &Strings,
    now: SystemTime,
) -> ToAPerson {
    let Some(turning) = turning else {
        return match answered.number() {
            Some(number) => nothing_is_waiting(number, strings),
            None => ToAPerson::waiting(std::iter::empty(), strings, now),
        };
    };

    match answered {
        FromAPerson::Waiting => ToAPerson::waiting(turning.waiting_at(now), strings, now),
        FromAPerson::Approve { number } => match under(turning, number, now) {
            Some(waiting) => match turning.approving(waiting, grants, now) {
                Ok(answer) => ToAPerson::did(&answer),
                Err(why) => ToAPerson::refused(&why.said(strings)),
            },
            None => nothing_is_waiting(number, strings),
        },
        FromAPerson::Decline { number } => match under(turning, number, now) {
            Some(waiting) => match turning.declining(waiting, now) {
                Ok(()) => ToAPerson::Declined,
                Err(why) => ToAPerson::refused(&why.said(strings)),
            },
            None => nothing_is_waiting(number, strings),
        },
    }
}

/// The change really waiting under this number, if one is.
///
/// The borrow ends with the search, so what comes back is a handle the turn
/// made rather than one this file invented from a number somebody sent.
fn under(turning: &Turning<'_, '_>, number: u64, now: SystemTime) -> Option<ProposalId> {
    turning
        .waiting_at(now)
        .find(|waiting| waiting.id.as_u64() == number)
        .map(|waiting| waiting.id)
}

/// The capability model's own sentence for a number nothing is waiting under.
///
/// Borrowed rather than written here for item 9e's reason: the screen and the
/// record render the same value, so neither can be a language the other is not
/// — and a person who answers a change twice reads the same sentence whichever
/// road the second answer took.
fn nothing_is_waiting(number: u64, strings: &Strings) -> ToAPerson {
    ToAPerson::refused(&AnswerError::NothingWaiting { number }.said(strings))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::doing::what_an_agent_said;
    use crate::testing::{a_message, hour, in_english, noon, on_a_machine};
    use std::path::Path;

    /// Propose a rename, and answer with the number it is waiting under.
    fn a_change_waiting(
        turning: &mut Turning<'_, '_>,
        grants: &Grants,
        strings: &Strings,
        invoice: &Path,
    ) -> u64 {
        let said = what_an_agent_said(
            &a_message(&format!(
                r#"{{"propose":{{"verb":"rename_file","given":[{{"named":"file","is":"{}"}},{{"named":"name","is":"march-final.pdf"}}]}}}}"#,
                invoice.display()
            )),
            turning,
            grants,
            strings,
            hour(),
            noon(),
        );
        assert!(said.waits_for_a_person());
        turning.waiting_at(noon()).next().unwrap().id.as_u64()
    }

    /// **A change proposed on one door is approved on the other**, and the file
    /// only moves when the second message arrives.
    #[test]
    fn a_change_is_approved_and_then_it_runs() {
        on_a_machine("approved", |turning, grants, strings, _, invoice| {
            let number = a_change_waiting(turning, grants, strings, invoice);
            assert!(invoice.is_file(), "a proposal moved a file");

            let said = what_a_person_said(
                &a_message(&format!(r#"{{"approve":{{"number":{number}}}}}"#)),
                Some(turning),
                grants,
                strings,
                noon(),
            );

            assert!(said.done().is_some(), "{said:?}");
            assert!(!invoice.is_file(), "the file did not move on the disk");
        });
    }

    /// **One approval is one execution.** The second answer finds nothing
    /// waiting under that number, which is the same sentence a number nobody
    /// ever proposed gets.
    #[test]
    fn the_same_number_cannot_be_approved_twice() {
        on_a_machine("twice", |turning, grants, strings, _, invoice| {
            let number = a_change_waiting(turning, grants, strings, invoice);
            let approve = a_message(&format!(r#"{{"approve":{{"number":{number}}}}}"#));

            assert!(
                what_a_person_said(&approve, Some(turning), grants, strings, noon())
                    .done()
                    .is_some()
            );
            let again = what_a_person_said(&approve, Some(turning), grants, strings, noon());
            assert!(again.refusal().is_some(), "{again:?}");
        });
    }

    /// **No is a whole answer**, and it is written down: nothing runs, and the
    /// number stops waiting.
    #[test]
    fn a_change_the_person_declined_runs_nothing() {
        on_a_machine("declined", |turning, grants, strings, _, invoice| {
            let number = a_change_waiting(turning, grants, strings, invoice);

            let said = what_a_person_said(
                &a_message(&format!(r#"{{"decline":{{"number":{number}}}}}"#)),
                Some(turning),
                grants,
                strings,
                noon(),
            );

            assert_eq!(said, ToAPerson::Declined);
            assert!(invoice.is_file(), "a declined change ran anyway");
            assert_eq!(turning.waiting_at(noon()).count(), 0);
        });
    }

    /// **What is waiting is the turn's own list**, each change carrying the
    /// sentence the person is being asked about rather than a number alone.
    #[test]
    fn what_is_waiting_comes_back_with_its_sentences() {
        on_a_machine("waiting", |turning, grants, strings, _, invoice| {
            a_change_waiting(turning, grants, strings, invoice);

            let said = what_a_person_said(
                &a_message(r#"{"waiting":{}}"#),
                Some(turning),
                grants,
                strings,
                noon(),
            );

            let changes = said.changes().unwrap();
            assert_eq!(changes.len(), 1);
            assert!(
                changes
                    .first()
                    .unwrap()
                    .sentence()
                    .text()
                    .contains("march-final.pdf")
            );
        });
    }

    /// **Nothing is waiting on a machine with no turn**, and it is an answer
    /// rather than a refusal: a shell drawing the person's changes must not
    /// have to tell *no turn* from *nothing outstanding*, because to them they
    /// are one fact.
    #[test]
    fn a_machine_with_no_turn_says_nothing_is_waiting() {
        let strings = in_english();
        let grants = Grants::default();

        let said = what_a_person_said(
            &a_message(r#"{"waiting":{}}"#),
            None,
            &grants,
            &strings,
            noon(),
        );

        assert_eq!(said.changes(), Some([].as_slice()));
    }

    /// **Answering when there is no turn is refused**, in the same words a
    /// number nothing is waiting under gets inside one — so a person whose
    /// agent has hung up while they were reading is told what happened rather
    /// than told something about a daemon.
    #[test]
    fn answering_with_no_turn_under_way_is_refused_in_the_ordinary_words() {
        let strings = in_english();
        let grants = Grants::default();

        for line in [
            a_message(r#"{"approve":{"number":7}}"#),
            a_message(r#"{"decline":{"number":7}}"#),
        ] {
            let said = what_a_person_said(&line, None, &grants, &strings, noon());
            let refusal = said.refusal().unwrap();
            assert!(refusal.text().contains('7'), "{refusal:?}");
            assert!(!refusal.is_a_bug());
        }
    }

    /// **A person cannot ask for a read**, and the refusal is `alo-protocol`'s:
    /// the two doors are two before anything reaches a turn.
    #[test]
    fn a_person_asking_for_a_read_is_refused_before_the_turn() {
        on_a_machine("wrong-door", |turning, grants, strings, folder, _| {
            let said = what_a_person_said(
                &a_message(&format!(
                    r#"{{"read":{{"verb":"list_folder","given":[{{"named":"folder","is":"{}"}}]}}}}"#,
                    folder.display()
                )),
                Some(turning),
                grants,
                strings,
                noon(),
            );

            assert!(said.refusal().is_some(), "{said:?}");
        });
    }

    /// **A number nobody proposed is refused**, and it is refused by looking at
    /// what is really waiting rather than by trusting the number.
    #[test]
    fn a_number_nobody_proposed_answers_nothing() {
        on_a_machine("invented", |turning, grants, strings, _, invoice| {
            a_change_waiting(turning, grants, strings, invoice);

            let said = what_a_person_said(
                &a_message(r#"{"approve":{"number":424242}}"#),
                Some(turning),
                grants,
                strings,
                noon(),
            );

            assert!(said.refusal().is_some(), "{said:?}");
            assert!(invoice.is_file(), "an invented number ran a change");
            assert_eq!(
                turning.waiting_at(noon()).count(),
                1,
                "the real change stopped waiting"
            );
        });
    }
}
