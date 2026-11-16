//! **A request with several changes goes on after each answer.**
//!
//! `docs/contracts/asking-socket.md`, *The turn goes on after an answer*: a
//! shell that says the answer follows tells the agent what the person's answer
//! did, as the daemon wrote it, and the agent shows the model that as a step and
//! drives on. These drive the loop the way `alo-agent`'s `main` does:
//! [`driving_until_stopped`] to the first change, then [`driving_on`] from the
//! steps it took and the answer.
//!
//! Each change still waits for its own one approval, given on the person's door
//! in another process. Nothing here can approve: the door is the agent's.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err or variant is the failure being reported"
)]

use alo_agent::{HowFar, NotAnswered, TheDoor, WhatHappened, driving_on, driving_until_stopped};
use alo_capability::Verbs;
use alo_files::Answer;
use alo_instructing::{Instructions, TheConversationSoFar, WhatItMayReach};
use alo_protocol::what_the_agent_says_back::{the_answer_line, the_answer_told};
use alo_protocol::{Done, FromAnAgent, ToAPerson, ToAnAgent, Wording};
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

/// The daemon's answer to a proposal: waiting under this number.
fn waiting_as(number: u64, sentence: &str) -> Result<ToAnAgent, NotAnswered> {
    let line = format!(
        "{{\"format\":1,\"tells\":{{\"proposed\":{{\"number\":{number},\"sentence\":{{\"text\":\
         {sentence:?},\"came_from\":\"the-source\"}}}}}}}}"
    );
    Ok(ToAnAgent::read(&line).unwrap())
}

/// The line a model writes to rename the March invoice.
const RENAME_MARCH: &str = r#"{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/march.pdf"},{"named":"name","is":"march-final.pdf"}]}}}"#;

/// The line a model writes to rename the April invoice.
const RENAME_APRIL: &str = r#"{"format":1,"asks":{"propose":{"verb":"rename_file","given":[{"named":"file","is":"/home/anna/april.pdf"},{"named":"name","is":"april-final.pdf"}]}}}"#;

/// The line a model writes to answer the person.
const AN_ANSWER: &str =
    r#"{"format":1,"asks":{"answer":{"text":"March is renamed; April is as it was."}}}"#;

/// What the person asked.
const REQUEST: &str = "rename my march and april invoices to -final";

/// Drive from the start.
fn first(door: &mut ADoorThatAnswers) -> WhatHappened {
    driving_until_stopped(
        &TheConversationSoFar::nothing(),
        None,
        REQUEST,
        &the_verbs(),
        &WhatItMayReach::not_said(),
        door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
        &|| false,
    )
}

/// Drive on after the person's answer, as `main` does: the steps so far, and
/// the proposal beside the answer the shell's line carried.
fn on_after(door: &mut ADoorThatAnswers, happened: &WhatHappened, line: &str) -> WhatHappened {
    let told = the_answer_told(line).unwrap();
    let Some(steps) = happened.going_on_after(&told) else {
        panic!("nothing was waiting: {happened:?}");
    };
    driving_on(
        &TheConversationSoFar::nothing(),
        None,
        REQUEST,
        &the_verbs(),
        &WhatItMayReach::not_said(),
        door,
        HowFar::THE_ORDINARY,
        Instructions::SHOWN_TO_A_TURN,
        &|| false,
        steps,
    )
}

/// The changes a door was asked to propose or run, as written, in order.
fn proposals(door: &ADoorThatAnswers) -> Vec<String> {
    door.asked
        .iter()
        .filter(|it| matches!(it, FromAnAgent::Propose { .. } | FromAnAgent::Run { .. }))
        .map(|it| it.written().unwrap())
        .collect()
}

/// The question the model was last shown.
fn the_last_question(door: &ADoorThatAnswers) -> String {
    door.asked
        .iter()
        .rev()
        .find_map(|it| match it {
            FromAnAgent::Ask { question, .. } => Some(question.clone()),
            _ => None,
        })
        .unwrap()
}

/// What the March rename did, as the daemon tells the person.
fn march_renamed() -> ToAPerson {
    ToAPerson::Did(Done::of_a_file_verb(&Answer::Renamed(
        "/home/anna/march-final.pdf".into(),
    )))
}

/// **The second change is proposed after the first is approved, and the
/// model is shown what the first did.** Each change goes to the daemon once.
#[test]
fn after_an_approval_the_turn_goes_on_to_the_next_change() {
    let mut door = ADoorThatAnswers {
        answers: vec![
            the_model_said(RENAME_MARCH),
            waiting_as(7, "Rename march.pdf to march-final.pdf"),
            the_model_said(RENAME_APRIL),
            waiting_as(8, "Rename april.pdf to april-final.pdf"),
        ],
        asked: Vec::new(),
    };
    let happened = first(&mut door);
    assert!(
        matches!(&happened, WhatHappened::AChangeIsWaiting { number: 7, proposed, .. } if proposed == RENAME_MARCH),
        "{happened:?}"
    );

    let line = the_answer_line(&march_renamed()).unwrap();
    let next = on_after(&mut door, &happened, &line);

    assert!(
        matches!(next, WhatHappened::AChangeIsWaiting { number: 8, .. }),
        "{next:?}"
    );
    assert_eq!(proposals(&door).len(), 2, "{:?}", proposals(&door));
    let shown = the_last_question(&door);
    assert!(
        shown.contains(
            &alo_agent::shown_after_an_answer::the_step_after_an_answer(
                &march_renamed(),
                "Rename march.pdf to march-final.pdf",
            )
            .unwrap(),
        ) && !shown.contains("without asking")
            && !shown.contains("done-without-asking"),
        "the model was not shown what the approved change did: {shown}"
    );
}

/// **Refuse the second change, and the turn ends with nothing done for it.**
/// The model is shown the decline, answers in words, and nothing more is
/// proposed or run. The decline itself is the daemon's to record, on the
/// person's door, before the shell says anything to the agent.
#[test]
fn a_refused_second_change_ends_the_turn_with_nothing_done_for_it() {
    let mut door = ADoorThatAnswers {
        answers: vec![
            the_model_said(RENAME_MARCH),
            waiting_as(7, "Rename march.pdf to march-final.pdf"),
            the_model_said(RENAME_APRIL),
            waiting_as(8, "Rename april.pdf to april-final.pdf"),
            the_model_said(AN_ANSWER),
        ],
        asked: Vec::new(),
    };
    let happened = first(&mut door);
    let second = on_after(
        &mut door,
        &happened,
        &the_answer_line(&march_renamed()).unwrap(),
    );
    let before_the_refusal = door.asked.len();

    let declined = the_answer_line(&ToAPerson::Declined).unwrap();
    let ended = on_after(&mut door, &second, &declined);

    assert!(
        matches!(&ended, WhatHappened::ItWasAnsweredInWords { said, .. } if said.contains("April is as it was")),
        "{ended:?}"
    );
    let after_the_refusal: Vec<&FromAnAgent> = door.asked.iter().skip(before_the_refusal).collect();
    assert!(
        after_the_refusal
            .iter()
            .all(|it| matches!(it, FromAnAgent::Ask { .. })),
        "something was asked of the machine after the person said no: {after_the_refusal:?}"
    );
    let proposed = proposals(&door);
    assert_eq!(
        proposed.len(),
        2,
        "a change was proposed again: {proposed:?}"
    );
    assert!(
        the_last_question(&door).contains(
            "The person said no, so this was not done: Rename april.pdf to april-final.pdf."
        ),
        "the model was not shown that the person said no"
    );
}

/// **The bound on rounds begins again after each answer**: a request with more
/// changes than one turn's rounds still reaches its last change, because each
/// answer is the person's own act.
#[test]
fn the_bound_begins_again_after_each_answer() {
    let rounds = HowFar::THE_ORDINARY.rounds();
    let changes = u64::try_from(rounds).unwrap() + 2;
    let mut answers = Vec::new();
    for number in 1..=changes {
        answers.push(the_model_said(RENAME_MARCH));
        answers.push(waiting_as(number, "Rename march.pdf to march-final.pdf"));
    }
    let mut door = ADoorThatAnswers {
        answers,
        asked: Vec::new(),
    };
    let mut happened = first(&mut door);
    let line = the_answer_line(&march_renamed()).unwrap();
    for _ in 1..changes {
        happened = on_after(&mut door, &happened, &line);
    }
    assert!(
        matches!(happened, WhatHappened::AChangeIsWaiting { number, .. } if number == changes),
        "{happened:?}"
    );
}

/// **A change that ran and cannot be undone, and one the daemon refused at the
/// approval, are each shown to the model as the daemon said them**, and the
/// turn goes on from there.
#[test]
fn every_answer_the_daemon_gives_is_shown_and_the_turn_goes_on() {
    let refused = alo_protocol::what_the_agent_says_back::the_refusal_line(
        "That question stood too long, so nothing was changed.",
    )
    .unwrap();
    let cannot = the_answer_line(&ToAPerson::DidAndCannotBeUndone(Done::of_a_file_verb(
        &Answer::Renamed("/home/anna/march-final.pdf".into()),
    )))
    .unwrap();
    for line in [cannot, refused] {
        let mut door = ADoorThatAnswers {
            answers: vec![
                the_model_said(RENAME_MARCH),
                waiting_as(7, "Rename march.pdf to march-final.pdf"),
                the_model_said(AN_ANSWER),
            ],
            asked: Vec::new(),
        };
        let happened = first(&mut door);
        let ended = on_after(&mut door, &happened, &line);
        assert!(
            matches!(ended, WhatHappened::ItWasAnsweredInWords { .. }),
            "{ended:?}"
        );
        let told = alo_agent::shown_after_an_answer::the_step_after_an_answer(
            &the_answer_told(&line).unwrap(),
            "Rename march.pdf to march-final.pdf",
        )
        .unwrap();
        assert!(
            the_last_question(&door).contains(&told),
            "the model was not shown {told}"
        );
        assert!(
            door.asked
                .iter()
                .all(|it| matches!(it, FromAnAgent::Ask { .. } | FromAnAgent::Propose { .. })),
            "the agent asked the machine for something other than a question or a proposal"
        );
    }
}

/// **Nothing is driven on from a turn with nothing waiting.**
#[test]
fn a_turn_with_nothing_waiting_does_not_go_on() {
    let mut door = ADoorThatAnswers {
        answers: vec![the_model_said(AN_ANSWER)],
        asked: Vec::new(),
    };
    let happened = first(&mut door);
    assert_eq!(happened.going_on_after(&ToAPerson::Declined), None);
}
