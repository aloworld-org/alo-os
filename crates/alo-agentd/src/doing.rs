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
//! two things and nothing else, and both are things no turn can say: a machine
//! where nobody has chosen anything to answer questions, and a question that
//! was put nowhere because parts of alo OS disagreed about where it should go.
//!
//! # A question to a model goes to what the person chose
//!
//! `crate::questions` is what the person's settings and this machine's runtime
//! come out of, and it answers one of four things: nothing was chosen, nothing
//! is running, the settings file does not hold, or here is the choice and here
//! is what answers it. Only the last reaches a turn, and the other three are
//! sentences somebody already says — this file writes none of them.
//!
//! *Nothing here has been chosen to answer questions* is not a placeholder and
//! never was: it is what a person who has picked neither a model nor a provider
//! is told for as long as alo OS exists.

use std::time::{Duration, SystemTime};

use alo_capability::{AnswerError, Grants, ProposalId};
use alo_models::RuntimeError;
use alo_protocol::{FromAnAgent, ToAnAgent};
use alo_strings::{Filling, Said, Strings};
use alo_turn::{Answers, NoAnswer, Turning};

use crate::questions::{Questions, WhatAnswers};
use crate::words::{NOTHING_ANSWERS_QUESTIONS, NOTHING_WAS_ASKED};

/// Read one line as something an agent asked, and do it.
///
/// Answers with what to say back, always: a message that was not a request, a
/// verb nobody declared and a grant that ran out are all answers rather than
/// silences, which is `docs/contracts/daemon-protocol.md`'s *refused in words
/// and never dropped*.
pub fn what_an_agent_said(
    line: &str,
    turning: &mut Turning<'_, '_>,
    questions: &mut Questions,
    grants: &Grants,
    strings: &Strings,
    standing: Duration,
    now: SystemTime,
) -> ToAnAgent {
    match FromAnAgent::read(line) {
        Ok(asked) => carried_out(&asked, turning, questions, grants, strings, standing, now),
        Err(why) => ToAnAgent::refused(&why.said(strings)),
    }
}

/// The three things an agent can ask for, each through the turn's own door.
fn carried_out(
    asked: &FromAnAgent,
    turning: &mut Turning<'_, '_>,
    questions: &mut Questions,
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
        FromAnAgent::Ask { question } => put_to_a_model(question, turning, questions, strings, now),
    }
}

/// A question, put to whatever this person chose — or refused in one sentence.
///
/// The three refusals are three different things to go and fix, and each is
/// worded by whoever knows it: *choose something* is this crate's, because
/// this crate is where a machine that has been asked and has nothing is;
/// *the runtime is not reachable* is `alo-models`', because a runtime being up
/// is its fact and it already has the sentence; *your settings file says this*
/// is `alo-choosing`'s, and names the file and the line.
fn put_to_a_model(
    question: &str,
    turning: &mut Turning<'_, '_>,
    questions: &mut Questions,
    strings: &Strings,
    now: SystemTime,
) -> ToAnAgent {
    match questions.what_answers() {
        WhatAnswers::Nothing => {
            ToAnAgent::refused(&strings.say(&NOTHING_ANSWERS_QUESTIONS.key(), &Filling::nothing()))
        }
        WhatAnswers::NotRunning => ToAnAgent::refused(&RuntimeError::Unreachable.said(strings)),
        WhatAnswers::NotSet(why) => ToAnAgent::refused(&why.said(strings)),
        WhatAnswers::OnThisMachine {
            chosen,
            runtime,
            places,
        } => match chosen.asking(Some(places.policy())) {
            // Nothing is composed out of what a model said, here or anywhere:
            // the text crosses as the model's own words, and the line naming
            // where it came from is a sentence of ours beside it.
            Ok(permission) => match turning.asking(
                question,
                chosen.model(),
                permission,
                &Answers::Runtime(runtime),
                &places,
                now,
            ) {
                Ok(answer) => {
                    ToAnAgent::answered(answer.text(), &answer.came_from(strings), answer.model())
                }
                Err(why) => ToAnAgent::refused(&nothing_answered(&why, strings)),
            },
            // What an organisation permits, refusing what the person chose —
            // and the sentence names the rule rather than the machine.
            Err(why) => ToAnAgent::refused(&why.said(strings)),
        },
    }
}

/// The sentence for a question that was not answered.
///
/// [`alo_turn::NoAnswer`] words all of them but one. The exception is a
/// miswiring, which is this repository disagreeing with itself and has no
/// sentence of its own because there is nothing for a person to do about it —
/// so this crate says the one thing that is true and useful: it went nowhere,
/// and it is not theirs to fix.
fn nothing_answered(why: &NoAnswer, strings: &Strings) -> Said {
    why.said(strings)
        .unwrap_or_else(|| strings.say(&NOTHING_WAS_ASKED.key(), &Filling::nothing()))
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
    use crate::testing::{
        a_directory_of_our_own, a_message, a_runtime_saying, hour, noon, nothing_has_been_chosen,
        on_a_machine, on_a_machine_that_answers,
    };
    use alo_choosing::{Chosen, Which};
    use alo_record::Record;

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
                &mut nothing_has_been_chosen(),
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
                &mut nothing_has_been_chosen(),
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
                &mut nothing_has_been_chosen(),
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
                &mut nothing_has_been_chosen(),
                grants,
                strings,
                hour(),
                noon(),
            );

            let said = what_an_agent_said(
                &a_message(r#"{"approve":{"number":1}}"#),
                turning,
                &mut nothing_has_been_chosen(),
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
                let said = what_an_agent_said(
                    line,
                    turning,
                    &mut nothing_has_been_chosen(),
                    grants,
                    strings,
                    hour(),
                    noon(),
                );
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
                &mut nothing_has_been_chosen(),
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
                &mut nothing_has_been_chosen(),
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

    /// A machine where this person chose these weights and this answers them.
    fn holding(model: &str, said: Result<String, RuntimeError>) -> Questions {
        Questions::already_found(
            Chosen::of(Which::Brought, model).unwrap(),
            a_runtime_saying(said),
            None,
        )
    }

    /// **A question reaches the model the person chose, and the answer comes
    /// back in the model's own words** — with the model named beside it, so
    /// whoever is reading knows what answered.
    #[test]
    fn a_question_is_put_to_what_the_person_chose_and_the_answer_comes_back() {
        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let said = what_an_agent_said(
                &a_message(r#"{"ask":{"question":"what is in this contract?"}}"#),
                turning,
                &mut holding("my-finetune", Ok("a sublet clause".to_owned())),
                &Grants::default(),
                strings,
                hour(),
                noon(),
            );

            assert!(
                matches!(
                    &said,
                    ToAnAgent::Answered { text, model, .. }
                        if text == "a sublet clause" && model == "my-finetune"
                ),
                "{said:?}"
            );
        });

        // Law 1's other half: it was answered here, so the entry says so and
        // nothing on it is about a destination.
        assert_eq!(record.len(), 1, "a question left no record");
    }

    /// **A model that does not answer is a refusal in words**, and the sentence
    /// is `alo-models`' own rather than one written in this file.
    #[test]
    fn a_model_that_does_not_answer_is_refused_in_the_runtimes_own_words() {
        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let said = what_an_agent_said(
                &a_message(r#"{"ask":{"question":"what is in this contract?"}}"#),
                turning,
                &mut holding("my-finetune", Err(RuntimeError::Unreachable)),
                &Grants::default(),
                strings,
                hour(),
                noon(),
            );

            let refusal = said.refusal().unwrap();
            assert!(!refusal.is_a_bug(), "{refusal:?}");
            assert!(!refusal.text().contains("Settings"), "{refusal:?}");
        });

        // Nothing was sent and nothing was answered, so there is nothing an
        // entry could truthfully say — `alo-turn`'s decision, honoured here.
        assert_eq!(
            record.len(),
            0,
            "a question that went nowhere left a record"
        );
    }

    /// **A settings file that does not hold is refused with its own sentence**,
    /// naming the file — not with *you have chosen nothing*, which would be
    /// false and would send the person to a panel that already agrees with them.
    #[test]
    fn a_settings_file_that_does_not_hold_names_itself_rather_than_settings() {
        let config = a_directory_of_our_own("doing-bad-settings");
        let folder = config.join(alo_choosing::THE_FOLDER);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(
            folder.join(alo_choosing::THE_SETTINGS),
            "format = 1\n[answers\n",
        )
        .unwrap();
        let mut questions = Questions::of_a_session(
            Some(config.clone().into_os_string()),
            None,
            alo_models::Catalogue::built_in().unwrap(),
            None,
        );

        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let said = what_an_agent_said(
                &a_message(r#"{"ask":{"question":"what is in this contract?"}}"#),
                turning,
                &mut questions,
                &Grants::default(),
                strings,
                hour(),
                noon(),
            );

            let refusal = said.refusal().unwrap();
            assert!(!refusal.is_a_bug(), "{refusal:?}");
            assert!(
                refusal.text().contains(&config.display().to_string()),
                "{refusal:?}"
            );
        });
    }

    /// **A question is never a change**, whatever answered it: nothing waits
    /// for the person and the turn is still open for the next thing.
    #[test]
    fn a_question_answered_leaves_nothing_waiting_for_a_person() {
        let mut record = Record::default();
        on_a_machine_that_answers(&mut record, |turning, _, strings| {
            let said = what_an_agent_said(
                &a_message(r#"{"ask":{"question":"what is in this contract?"}}"#),
                turning,
                &mut holding("my-finetune", Ok("a sublet clause".to_owned())),
                &Grants::default(),
                strings,
                hour(),
                noon(),
            );

            assert!(!said.waits_for_a_person());
            assert_eq!(turning.waiting_at(noon()).count(), 0);
            assert!(!turning.is_closed());
        });
    }
}
