//! **The loop and the run door** (ADR 0149): a command that ran is a step, a
//! command waiting for the person's OK ends the turn as a proposal does, and a
//! refused command is a step.
//!
//! Against a door that answers from a list, as `a_request_is_driven_to_an_end.rs`
//! does, so what this proves is the order of the loop and nothing about a
//! model. The waiting command is a real one, put on a real `Approvals`, so the
//! `Standing` scripted here is the shape the daemon sends.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err or variant is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_agent::{HowFar, NotAnswered, TheDoor, WhatHappened, driving};
use alo_capability::{ACommand, Approvals, Grantee, Verbs, WhatACommandDid};
use alo_instructing::{Instructions, NOT_FOR_THE_PERSON, WhatItMayReach};
use alo_protocol::{Done, FromAnAgent, Standing, ToAnAgent, Wording};
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

/// A fixed moment.
fn the_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A model's answer, as the machine hands one back. Where it came from is not
/// what these tests are about, so any declared sentence stands in for it.
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

/// The line a model writes to run a command.
const A_RUN: &str =
    r#"{"format":1,"asks":{"run":{"command":"mkdir -p /home/anna/Documents/demo"}}}"#;

/// The line a model writes to answer the person.
const AN_ANSWER: &str = r#"{"format":1,"asks":{"answer":{"text":"Made it."}}}"#;

/// Drive `request` against `door` under what a turn shows.
fn driven(door: &mut ADoorThatAnswers) -> WhatHappened {
    driving(
        "make a folder called demo in my Documents",
        &the_verbs(),
        &WhatItMayReach::not_said(),
        door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
    )
}

/// **A command that ran is a step**: what it did reaches the model on the next
/// round, and the turn goes on to its answer.
#[test]
fn a_command_that_ran_is_a_step_and_the_turn_goes_on() {
    let ran = ToAnAgent::did(Done::of_a_command(&WhatACommandDid::of(
        Some(0),
        String::new(),
        false,
        false,
    )));
    let ran_line = ran.written().unwrap();
    let mut door = answering(vec![
        the_model_said(A_RUN),
        Ok(ran),
        the_model_said(AN_ANSWER),
    ]);
    let happened = driven(&mut door);
    let WhatHappened::ItWasAnsweredInWords { said, after } = happened else {
        panic!("{happened:?}");
    };
    assert_eq!(said, "Made it.");
    assert_eq!(after.len(), 1, "the run was not kept as a step");
    assert!(
        door.asked.iter().any(|asked| asked
            == &FromAnAgent::Run {
                command: "mkdir -p /home/anna/Documents/demo".to_owned()
            }),
        "the command never reached the machine: {:?}",
        door.asked
    );
    let shown_next = door
        .asked
        .iter()
        .filter_map(FromAnAgent::question)
        .nth(1)
        .unwrap();
    assert!(
        shown_next.contains(&ran_line),
        "the model was not shown what the command did"
    );
}

/// **A command waiting for the OK ends the turn as a proposal does**, with the
/// number the person answers by and the command as the sentence.
#[test]
fn a_command_waiting_for_the_ok_ends_the_turn_as_a_change_waiting() {
    let strings = everything_this_machine_says();
    let mut approvals = Approvals::default();
    let number = approvals
        .propose_a_command(
            ACommand::of("mkdir -p /home/anna/Documents/demo").unwrap(),
            &Grantee::named("@files"),
            the_moment(),
            Duration::from_secs(300),
        )
        .unwrap();
    let waiting = approvals.command_of(number).unwrap();
    let standing = Standing::of_a_command(waiting, &strings, the_moment());
    let mut door = answering(vec![
        the_model_said(A_RUN),
        Ok(ToAnAgent::Proposed(standing)),
    ]);
    let happened = driven(&mut door);
    let WhatHappened::AChangeIsWaiting {
        number: answered_by,
        sentence,
        after,
        stands_for,
    } = happened
    else {
        panic!("{happened:?}");
    };
    assert_eq!(answered_by, number.as_u64());
    assert_eq!(
        sentence,
        "run this command: mkdir -p /home/anna/Documents/demo"
    );
    assert!(after.is_empty());
    assert_eq!(stands_for, Some(Duration::from_secs(300)));
    // Nothing more was asked once it waited.
    assert_eq!(door.asked.len(), 2);
}

/// **A refused command is a step**, and the model is told the sentence.
#[test]
fn a_refused_command_is_a_step_and_the_model_is_told_why() {
    let strings = everything_this_machine_says();
    let refusal = strings.say(
        &alo_capability::words::THE_SEALED_BOX_CANNOT_RUN_PROGRAMS_HERE_YET.key(),
        &Filling::of("program", "mkdir"),
    );
    let refused = ToAnAgent::refused(&refusal);
    let refused_line = refused.written().unwrap();
    let mut door = answering(vec![
        the_model_said(A_RUN),
        Ok(refused),
        the_model_said(
            r#"{"format":1,"asks":{"answer":{"text":"I can't run that here: choose 'ask each time' in Settings."}}}"#,
        ),
    ]);
    let happened = driven(&mut door);
    let WhatHappened::ItWasAnsweredInWords { after, .. } = happened else {
        panic!("{happened:?}");
    };
    assert_eq!(after.len(), 1);
    let shown_next = door
        .asked
        .iter()
        .filter_map(FromAnAgent::question)
        .nth(1)
        .unwrap();
    assert!(shown_next.contains(&refused_line), "{shown_next}");
}

/// **The ADR 0149 guard still holds beside the run door**: an answer about the
/// verbs is told back as a step, and the run that follows still reaches the
/// machine.
#[test]
fn an_answer_about_the_verbs_is_still_not_for_the_person() {
    let mut door = answering(vec![
        the_model_said(
            r#"{"format":1,"asks":{"answer":{"text":"Folders cannot be created with the given verbs."}}}"#,
        ),
        the_model_said(A_RUN),
        Ok(ToAnAgent::did(Done::of_a_command(&WhatACommandDid::of(
            Some(0),
            String::new(),
            false,
            false,
        )))),
        the_model_said(AN_ANSWER),
    ]);
    let happened = driven(&mut door);
    let WhatHappened::ItWasAnsweredInWords { said, after } = happened else {
        panic!("{happened:?}");
    };
    assert_eq!(said, "Made it.");
    assert_eq!(after.len(), 2);
    let shown_next = door
        .asked
        .iter()
        .filter_map(FromAnAgent::question)
        .nth(1)
        .unwrap();
    assert!(shown_next.contains(NOT_FOR_THE_PERSON), "{shown_next}");
    assert!(
        door.asked
            .iter()
            .any(|asked| matches!(asked, FromAnAgent::Run { .. }))
    );
}
