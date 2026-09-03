//! One line from an agent, carried out against the turn it belongs to.
//!
//! Everything an agent can ask for is `alo_protocol::FromAnAgent`, everything
//! that can come back is `alo_protocol::ToAnAgent`, and every decision between
//! the two is `alo_turn::Turning`'s. This file is the join, and its whole value
//! is that there is exactly one of it: a second road from a message to a turn
//! would be a second place law 2 has to be got right.
//!
//! # Nothing here decides anything
//!
//! The name of a verb goes to `alo_capability::Verbs::call` exactly as it
//! arrived — not trimmed, not lower-cased, not looked at. Whether a request is
//! a read or a change is decided twice and neither time here:
//! `alo_capability::Authorised::read` refuses a change offered as a read, and
//! `alo_capability::Proposal::checked` refuses a read offered for approval.
//! `FromAnAgent::waits_for_a_person` exists and is deliberately not consulted,
//! because a door that chose from it would be a third answer to ADR 0001 §5
//! that could disagree with the other two.
//!
//! # A refusal crosses in the words of whoever refused it
//!
//! Every failure inside a turn is `alo_turn::NotDone::said`, rendered with the
//! machine's own vocabulary and handed over as one sentence. This file words
//! nothing of its own except the one thing no turn can answer, which is a
//! question put to a model on a machine where nobody has chosen anything to
//! answer questions.
//!
//! # Why a question to a model is refused here at all
//!
//! `alo_turn::Turning::asking` exists and works, and it needs three things this
//! service is not yet told: which model or provider answers, what an
//! organisation permits, and where a question may be answered. Those are
//! settings, and nothing in this repository reads them yet — queue item 21e is
//! where a machine says what it is. Until then a machine has chosen nothing,
//! and *nothing here has been chosen to answer questions* is the true sentence
//! about it rather than a placeholder: it is what a person who has picked
//! neither a model nor a provider will be told for as long as alo OS exists.

use std::time::{Duration, SystemTime};

use alo_capability::{AnswerError, Grants, ProposalId};
use alo_protocol::{FromAnAgent, ToAnAgent};
use alo_strings::{Filling, Strings};
use alo_turn::Turning;

use crate::words::NOTHING_ANSWERS_QUESTIONS;

/// Read one line as something an agent asked, and do it.
///
/// Answers with what to say back, always: a message that was not a request, a
/// verb nobody declared and a grant that ran out are all answers rather than
/// silences, which is `docs/contracts/daemon-protocol.md`'s *refused in words
/// and never dropped*.
pub fn what_an_agent_said(
    line: &str,
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    strings: &Strings,
    standing: Duration,
    now: SystemTime,
) -> ToAnAgent {
    match FromAnAgent::read(line) {
        Ok(asked) => carried_out(&asked, turning, grants, strings, standing, now),
        Err(why) => ToAnAgent::refused(&why.said(strings)),
    }
}

/// The three things an agent can ask for, each through the turn's own door.
fn carried_out(
    asked: &FromAnAgent,
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    strings: &Strings,
    standing: Duration,
    now: SystemTime,
) -> ToAnAgent {
    match asked {
        FromAnAgent::Read { verb, .. } => {
            match turning.reading(verb, &asked.given(), grants, now) {
                Ok(answer) => ToAnAgent::did(&answer),
                Err(why) => ToAnAgent::refused(&why.said(strings)),
            }
        }
        FromAnAgent::Propose { verb, .. } => {
            match turning.proposing(verb, &asked.given(), grants, standing, now) {
                Ok(number) => waiting_under(turning, number, strings, now),
                Err(why) => ToAnAgent::refused(&why.said(strings)),
            }
        }
        FromAnAgent::Ask { .. } => {
            ToAnAgent::refused(&strings.say(&NOTHING_ANSWERS_QUESTIONS.key(), &Filling::nothing()))
        }
    }
}

/// The change that is now waiting, with the sentence the person will be asked.
///
/// `alo_protocol::ToAnAgent::proposed` takes the change rather than its number,
/// so an answer cannot be composed without the sentence — which means the
/// change has to be found among what is really waiting rather than assumed from
/// the number that was just handed back.
///
/// A change that is not there is a change that was proposed to stand for no
/// time at all. It is refused with the capability model's own sentence for a
/// number nothing is waiting under, rather than with a second one written here
/// that would say the same thing differently.
fn waiting_under(
    turning: &Turning<'_, '_>,
    number: ProposalId,
    strings: &Strings,
    now: SystemTime,
) -> ToAnAgent {
    turning
        .waiting_at(now)
        .find(|waiting| waiting.id == number)
        .map_or_else(
            || {
                ToAnAgent::refused(
                    &AnswerError::NothingWaiting {
                        number: number.as_u64(),
                    }
                    .said(strings),
                )
            },
            |waiting| ToAnAgent::proposed(waiting, strings, now),
        )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_message, hour, noon, on_a_machine};

    /// **A read answers inside the turn**, and what comes back is what the
    /// machine found rather than a promise to find it.
    #[test]
    fn a_read_is_carried_out_and_answered() {
        on_a_machine("a-read", |turning, grants, strings, folder, _| {
            let said = what_an_agent_said(
                &a_message(&format!(
                    r#"{{"read":{{"verb":"list_folder","given":[{{"named":"folder","is":"{}"}}]}}}}"#,
                    folder.display()
                )),
                turning,
                grants,
                strings,
                hour(),
                noon(),
            );

            assert!(matches!(
                said.done(),
                Some(alo_protocol::Done::Listed { things, .. }) if things.len() == 1
            ));
        });
    }

    /// **A change comes back as a number and the sentence it waits on**, so the
    /// agent has something to show the person rather than a handle only this
    /// machine understands.
    #[test]
    fn a_change_comes_back_with_the_sentence_it_waits_on() {
        on_a_machine("a-change", |turning, grants, strings, _, invoice| {
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
            assert_eq!(turning.waiting_at(noon()).count(), 1);
            assert!(invoice.is_file(), "a proposal moved a file");
        });
    }

    /// **A verb nobody declared is turned away by the closed list**, and the
    /// name reaches it exactly as it was written — `/bin/sh` and all.
    #[test]
    fn a_verb_nobody_declared_is_refused_by_the_list() {
        on_a_machine("no-such-verb", |turning, grants, strings, _, _| {
            let said = what_an_agent_said(
                &a_message(r#"{"read":{"verb":"/bin/sh","given":[]}}"#),
                turning,
                grants,
                strings,
                hour(),
                noon(),
            );

            let refusal = said.refusal().unwrap();
            assert!(refusal.text().contains("/bin/sh"), "{refusal:?}");
        });
    }

    /// **An agent cannot answer its own question**, and the refusal is
    /// `alo-protocol`'s rather than one written here: the two doors are two
    /// before anything reaches a turn.
    #[test]
    fn an_agent_that_approves_something_is_refused_before_the_turn() {
        on_a_machine("self-approval", |turning, grants, strings, _, invoice| {
            what_an_agent_said(
                &a_message(&format!(
                    r#"{{"propose":{{"verb":"rename_file","given":[{{"named":"file","is":"{}"}},{{"named":"name","is":"gone.pdf"}}]}}}}"#,
                    invoice.display()
                )),
                turning,
                grants,
                strings,
                hour(),
                noon(),
            );

            let said = what_an_agent_said(
                &a_message(r#"{"approve":{"number":1}}"#),
                turning,
                grants,
                strings,
                hour(),
                noon(),
            );

            assert!(
                said.refusal()
                    .unwrap()
                    .text()
                    .contains("cannot answer a question that was put to a person")
            );
            assert_eq!(
                turning.waiting_at(noon()).count(),
                1,
                "the change stopped waiting for the person"
            );
            assert!(invoice.is_file(), "an agent approved its own change");
        });
    }

    /// **A message that is not a request is answered in words**, with
    /// `alo-protocol`'s own sentence, and nothing reaches the turn.
    #[test]
    fn a_message_that_is_not_a_request_is_answered_and_reaches_no_turn() {
        on_a_machine("gibberish", |turning, grants, strings, _, _| {
            for line in [
                "not json at all",
                r#"{"format":9,"asks":{"read":{"verb":"list_folder","given":[]}}}"#,
                r#"{"format":1,"asks":{"run":{"command":"rm -rf /"}}}"#,
            ] {
                let said = what_an_agent_said(line, turning, grants, strings, hour(), noon());
                assert!(said.refusal().is_some(), "{line}");
            }
            assert!(!turning.is_closed());
        });
    }

    /// **A question for a model is refused because nothing has been chosen to
    /// answer one**, which is the true state of a machine nobody has set a
    /// model or a provider on — and the sentence names the panel to go to.
    #[test]
    fn a_question_for_a_model_says_nothing_has_been_chosen_to_answer_one() {
        on_a_machine("a-question", |turning, grants, strings, _, _| {
            let said = what_an_agent_said(
                &a_message(r#"{"ask":{"question":"what is in this contract?"}}"#),
                turning,
                grants,
                strings,
                hour(),
                noon(),
            );

            let refusal = said.refusal().unwrap();
            assert!(refusal.text().contains("Settings"), "{refusal:?}");
            assert!(
                !refusal.is_a_bug(),
                "the sentence this crate says is not one it declared"
            );
        });
    }

    /// **A change that stands for no time at all is not waiting**, so what
    /// comes back is the capability model's own sentence for a number nothing
    /// is waiting under rather than a proposal nobody could answer.
    #[test]
    fn a_change_that_waits_for_no_time_is_not_answered_as_waiting() {
        on_a_machine("no-standing", |turning, grants, strings, _, invoice| {
            let said = what_an_agent_said(
                &a_message(&format!(
                    r#"{{"propose":{{"verb":"rename_file","given":[{{"named":"file","is":"{}"}},{{"named":"name","is":"never.pdf"}}]}}}}"#,
                    invoice.display()
                )),
                turning,
                grants,
                strings,
                Duration::from_secs(0),
                noon(),
            );

            assert!(!said.waits_for_a_person());
            assert!(said.refusal().is_some());
            assert_eq!(turning.waiting_at(noon()).count(), 0);
        });
    }
}
