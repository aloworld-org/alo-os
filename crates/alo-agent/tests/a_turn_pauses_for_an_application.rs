//! **A call for an application refused pauses the turn, and only where asked**
//! — the owner's decision of 2026-11-16
//! (`docs/design/when-alo-asks-for-an-application.md`).
//!
//! Driven as `alo-agent`'s `main` drives it: pausing only where the shell said
//! it tells the answer (`HowFar::pausing_for_an_application`). If the person
//! lets alo use the application, the identical call is sent again and waits
//! for its own approval; if not, the refusal is a step and the turn goes on.
//! **Nothing here grants anything**: the door is the agent's, and a grant is
//! the person's, made on their own door.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err or variant is the failure being reported"
)]

use alo_agent::{
    HowFar, NotAnswered, TheDoor, WhatHappened, a_proposal_answered, driving_until_stopped,
};
use alo_capability::Verbs;
use alo_instructing::{Instructions, TheConversationSoFar, WhatItMayReach};
use alo_protocol::{FromAnAgent, ToAnAgent, Wording};
use alo_strings::{Filling, Strings};

/// Discovery metadata cannot stand in for an approval or an execution result.
#[test]
fn a_proposal_answered_with_capabilities_stops_without_reporting_success() {
    let answer = a_proposal_answered(
        "propose a recording action",
        Ok(ToAnAgent::Capabilities { verbs: Vec::new() }),
        Vec::new(),
    );
    assert!(matches!(
        answer.unwrap_err().as_ref(),
        WhatHappened::TheMachineCouldNotBeAsked { after, .. } if after.is_empty()
    ));
}

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

/// The verbs a whole machine offers, the application verbs among them.
fn the_verbs() -> Verbs {
    alo_turn::the_verbs_a_whole_machine_offers().unwrap()
}

/// Every word the machine answering these says.
fn words() -> Strings {
    let mut vocabulary = alo_files::file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_protocol::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A model's answer, as the machine hands one back.
fn the_model_said(text: &str) -> Result<ToAnAgent, NotAnswered> {
    Ok(ToAnAgent::Answered {
        text: text.to_owned(),
        came_from: Wording::of(&words().say(
            &alo_capability::words::NOTHING_RAN_WOULD_NOT_START.key(),
            &Filling::nothing(),
        )),
        model: "a-model".to_owned(),
    })
}

/// The daemon refusing, in some words.
fn refused() -> Result<ToAnAgent, NotAnswered> {
    Ok(ToAnAgent::Refused(Wording::of(&words().say(
        &alo_capability::words::NOTHING_RAN_WOULD_NOT_START.key(),
        &Filling::nothing(),
    ))))
}

/// The daemon's answer to a proposal: waiting under this number.
fn waiting_as(number: u64, sentence: &str) -> Result<ToAnAgent, NotAnswered> {
    let line = format!(
        "{{\"format\":1,\"tells\":{{\"proposed\":{{\"number\":{number},\"sentence\":{{\"text\":\
         {sentence:?},\"came_from\":\"the-source\"}}}}}}}}"
    );
    Ok(ToAnAgent::read(&line).unwrap())
}

/// The line a model writes to open VSCodium.
const OPEN_CODIUM: &str = r#"{"format":1,"asks":{"propose":{"verb":"open_application","given":[{"named":"application","is":"com.vscodium.codium"}]}}}"#;

/// The line a model writes to rename a file: no application in it.
const RENAME_MARCH: &str = r#"{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/march.pdf"},{"named":"name","is":"march-final.pdf"}]}}}"#;

/// The line a model writes to answer the person.
const AN_ANSWER: &str = r#"{"format":1,"asks":{"answer":{"text":"I could not open it."}}}"#;

/// Drive `answers` from the start, this far.
fn driven(
    answers: Vec<Result<ToAnAgent, NotAnswered>>,
    how_far: HowFar,
) -> (WhatHappened, ADoorThatAnswers) {
    let mut door = ADoorThatAnswers {
        answers,
        asked: Vec::new(),
    };
    let happened = driving_until_stopped(
        &TheConversationSoFar::nothing(),
        None,
        "open my project in VSCodium",
        &the_verbs(),
        &WhatItMayReach::not_said(),
        &mut door,
        how_far,
        Instructions::SHOWN_TO_A_TURN,
        &|| false,
    );
    (happened, door)
}

/// The proposal a model's line reads as.
fn the_proposal(line: &str) -> FromAnAgent {
    FromAnAgent::read(line).unwrap()
}

/// **Pausing, a refused call for an application ends the drive as a pause**,
/// carrying the identical call to send again — and nothing more is asked.
#[test]
fn pausing_a_refused_call_for_an_application_pauses_the_turn() {
    let (happened, door) = driven(
        vec![the_model_said(OPEN_CODIUM), refused()],
        HowFar::THE_ORDINARY.pausing_for_an_application(),
    );
    let WhatHappened::AnApplicationWasRefused {
        proposed,
        again,
        after,
        ..
    } = &happened
    else {
        panic!("it did not pause: {happened:?}");
    };
    assert_eq!(proposed, OPEN_CODIUM);
    assert_eq!(again, &the_proposal(OPEN_CODIUM));
    assert!(after.is_empty(), "the refusal was made a step: {after:?}");
    assert_eq!(
        door.asked.len(),
        2,
        "something more was asked: {:?}",
        door.asked
    );
    assert!(happened.is_waiting_for_the_person());
}

/// **Not pausing — a shell that does not tell the answer — the refusal is a
/// step and the turn goes on**, exactly as before.
#[test]
fn not_pausing_a_refused_call_for_an_application_is_a_step() {
    let (happened, _) = driven(
        vec![
            the_model_said(OPEN_CODIUM),
            refused(),
            the_model_said(AN_ANSWER),
        ],
        HowFar::THE_ORDINARY,
    );
    let WhatHappened::ItWasAnsweredInWords { after, .. } = &happened else {
        panic!("{happened:?}");
    };
    assert_eq!(after.len(), 1, "{after:?}");
}

/// **A refused call that names no application does not pause**, even where
/// pausing was asked for.
#[test]
fn a_refused_call_with_no_application_does_not_pause() {
    let (happened, _) = driven(
        vec![
            the_model_said(RENAME_MARCH),
            refused(),
            the_model_said(AN_ANSWER),
        ],
        HowFar::THE_ORDINARY.pausing_for_an_application(),
    );
    assert!(
        matches!(happened, WhatHappened::ItWasAnsweredInWords { .. }),
        "{happened:?}"
    );
}

/// **Once the person lets alo use it, the identical call is sent again and
/// waits for its own approval** — the same reading of a proposal's answer the
/// loop makes. Nothing about the first refusal approves it.
#[test]
fn sent_again_the_call_waits_for_its_own_approval() {
    let (happened, _) = driven(
        vec![the_model_said(OPEN_CODIUM), refused()],
        HowFar::THE_ORDINARY.pausing_for_an_application(),
    );
    let WhatHappened::AnApplicationWasRefused {
        proposed, again, ..
    } = &happened
    else {
        panic!("{happened:?}");
    };
    let mut door = ADoorThatAnswers {
        answers: vec![waiting_as(4, "Open VSCodium")],
        asked: Vec::new(),
    };
    let sent = a_proposal_answered(proposed, door.asking(again), happened.after().to_vec());
    let Err(ended) = sent else {
        panic!("it did not wait for an approval");
    };
    let WhatHappened::AChangeIsWaiting {
        number, proposed, ..
    } = *ended
    else {
        panic!("it ended otherwise than waiting for an approval");
    };
    assert_eq!(number, 4);
    assert_eq!(proposed, OPEN_CODIUM);
    assert_eq!(
        door.asked,
        [the_proposal(OPEN_CODIUM)],
        "it was not the identical call"
    );
}

/// **Not given, the refusal is the next step**, beside the model's own call,
/// and the turn goes on from it.
#[test]
fn not_given_the_refusal_is_the_next_step() {
    let (happened, _) = driven(
        vec![the_model_said(OPEN_CODIUM), refused()],
        HowFar::THE_ORDINARY.pausing_for_an_application(),
    );
    let steps = happened.going_on_without_it().unwrap();
    assert_eq!(steps.len(), 1);
    let last = steps.last().unwrap().what_came_back();
    assert!(last.contains("refused"), "{last}");
}
