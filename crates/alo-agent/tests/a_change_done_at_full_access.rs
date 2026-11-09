//! **The loop at full access** (ADR 0149): a change the machine carried out
//! without the OK is a step, the turn goes on, and what the person is told
//! about it comes back however the turn ends — while a change left waiting
//! still ends the turn as it always did.
//!
//! Against a door that answers from a list, as `a_command_where_no_verb_fits.rs`
//! does, so what this proves is the order of the loop and nothing about a
//! model.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err or variant is the failure being reported"
)]

use alo_agent::{HowFar, NotAnswered, TheDoor, WhatHappened, driving};
use alo_capability::Verbs;
use alo_files::Answer;
use alo_instructing::{Instructions, WhatItMayReach};
use alo_protocol::{Done, FromAnAgent, ToAnAgent, Wording};
use alo_strings::{Filling, Strings};

/// A door that answers from a list and remembers every request it was given.
struct ADoorThatAnswers {
    /// What it will answer, oldest first.
    answers: Vec<Result<ToAnAgent, NotAnswered>>,
    /// Every request it was given, oldest first.
    asked: Vec<FromAnAgent>,
}

impl TheDoor for ADoorThatAnswers {
    fn asking(&mut self, request: &FromAnAgent) -> Result<ToAnAgent, NotAnswered> {
        self.asked.push(request.clone());
        if self.answers.is_empty() {
            return Err(NotAnswered::NotReached {
                why: "the list of answers ran out".to_owned(),
            });
        }
        self.answers.remove(0)
    }
}

/// A door that will answer these, in order.
fn answering(answers: Vec<Result<ToAnAgent, NotAnswered>>) -> ADoorThatAnswers {
    ADoorThatAnswers {
        answers,
        asked: Vec::new(),
    }
}

/// The verbs a real machine has.
fn the_verbs() -> Verbs {
    alo_files::file_verbs().unwrap()
}

/// Every word a machine answering these messages has loaded.
fn everything_this_machine_says() -> Strings {
    let mut vocabulary = alo_files::file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_protocol::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A model's answer, as the machine hands one back.
fn the_model_said(text: &str) -> Result<ToAnAgent, NotAnswered> {
    Ok(ToAnAgent::Answered {
        text: text.to_owned(),
        came_from: Wording::of(&everything_this_machine_says().say(
            &alo_capability::words::NOTHING_RAN_WOULD_NOT_START.key(),
            &Filling::nothing(),
        )),
        model: "a-model".to_owned(),
    })
}

/// What the machine answers a rename it carried out at full access with.
fn renamed_without_asking() -> ToAnAgent {
    let said = everything_this_machine_says().say(
        &alo_capability::words::DONE_AT_FULL_ACCESS.key(),
        &Filling::of("sentence", "rename /home/anna/march.pdf to march-final.pdf"),
    );
    ToAnAgent::done_without_asking(
        Done::of_a_file_verb(&Answer::Renamed("/home/anna/march-final.pdf".into())),
        &said,
    )
}

/// The line a model writes to rename the invoice.
const A_RENAME: &str = r#"{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/march.pdf"},{"named":"name","is":"march-final.pdf"}]}}}"#;

/// The line a model writes to answer the person.
const AN_ANSWER: &str = r#"{"format":1,"asks":{"answer":{"text":"It is renamed."}}}"#;

/// Drive the request against `door` under what a turn shows.
fn driven(door: &mut ADoorThatAnswers) -> WhatHappened {
    driving(
        "rename my march invoice to march-final",
        &the_verbs(),
        &WhatItMayReach::not_said(),
        door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
    )
}

/// **A change carried out at full access is a step, and the turn goes on**:
/// the model is shown what was done on the next round, it answers, and the
/// person's sentence comes back from the turn — once.
#[test]
fn a_change_done_at_full_access_is_a_step_and_the_turn_goes_on() {
    let done = renamed_without_asking();
    let done_line = done.written().unwrap();
    let mut door = answering(vec![
        the_model_said(A_RENAME),
        Ok(done),
        the_model_said(AN_ANSWER),
    ]);
    let happened = driven(&mut door);
    let told = happened.done_without_asking();
    let WhatHappened::ItWasAnsweredInWords { said, after } = happened else {
        panic!("{happened:?}");
    };
    assert_eq!(said, "It is renamed.");
    assert_eq!(after.len(), 1, "the change was not kept as a step");
    assert_eq!(told.len(), 1, "{told:?}");
    assert!(
        told.iter()
            .all(|told| told.contains("Done without asking you first")
                && told.contains("march-final.pdf")),
        "{told:?}"
    );
    let shown_next = door
        .asked
        .iter()
        .filter_map(FromAnAgent::question)
        .nth(1)
        .unwrap();
    assert!(
        shown_next.contains(&done_line),
        "the model was not shown what the change did"
    );
}

/// **A change that waits still ends the turn**, as before full access existed,
/// and nothing is said to have been done without asking.
#[test]
fn a_change_that_waits_still_ends_the_turn() {
    let (approvals, strings) = {
        let strings = everything_this_machine_says();
        let mut approvals = alo_capability::Approvals::default();
        let call = the_verbs()
            .call(
                "rename_file",
                &[
                    ("file", alo_capability::Given::text("/home/anna/march.pdf")),
                    ("name", alo_capability::Given::text("march-final.pdf")),
                ],
            )
            .unwrap();
        let mut grants = alo_capability::Grants::default();
        let now = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000);
        let agent = alo_capability::Grantee::named("@files");
        grants.grant(
            alo_capability::Grant::checked(
                "@files",
                alo_capability::Reach::Folder("/home/anna".into()),
                now,
                std::time::Duration::from_secs(3600),
            )
            .unwrap(),
        );
        approvals.propose(
            alo_capability::Proposal::checked(
                &call,
                &agent,
                &grants,
                now,
                std::time::Duration::from_secs(300),
            )
            .unwrap(),
        );
        (approvals, strings)
    };
    let now = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000);
    let waiting = approvals.waiting_at(now).next().unwrap();
    let mut door = answering(vec![
        the_model_said(A_RENAME),
        Ok(ToAnAgent::proposed(waiting, &strings, now)),
    ]);
    let happened = driven(&mut door);
    assert!(happened.is_waiting_for_the_person(), "{happened:?}");
    assert!(happened.done_without_asking().is_empty());
}

/// **What was done is told even when the turn then fails**: a change carried
/// out at full access, then a refusal, and the person still reads both.
#[test]
fn what_was_done_is_told_when_the_turn_then_ends_in_a_refusal() {
    let refused = ToAnAgent::refused(&everything_this_machine_says().say(
        &alo_capability::words::NOTHING_RAN_WOULD_NOT_START.key(),
        &Filling::nothing(),
    ));
    let mut door = answering(vec![
        the_model_said(A_RENAME),
        Ok(renamed_without_asking()),
        Ok(refused),
    ]);
    let happened = driven(&mut door);
    assert!(
        matches!(happened, WhatHappened::ItWasRefused { .. }),
        "{happened:?}"
    );
    assert_eq!(happened.done_without_asking().len(), 1, "{happened:?}");
}
