//! The person's own question, on the wire and at both doors.
//!
//! Two doors read one envelope, so the tests that matter are the ones that
//! prove each door refuses what belongs to the other.

use alo_strings::{Filling, Said};

use crate::agent::FromAnAgent;
use crate::person::FromAPerson;
use crate::refusing::NotUnderstood;
use crate::testing::in_english;
use crate::to_a_person::ToAPerson;
use crate::words;

/// Any real sentence from this crate's own vocabulary.
fn a_sentence() -> Said {
    in_english().say(&words::NOT_READABLE.key(), &Filling::nothing())
}

fn theirs() -> FromAPerson {
    FromAPerson::TheirOwnQuestion {
        question: "what is the capital of Spain".to_owned(),
    }
}

/// It survives being written and read back, with the question unchanged.
#[test]
fn a_persons_own_question_goes_out_and_comes_back_as_itself() {
    let line = theirs().written().expect("an answer cannot fail to write");
    assert_eq!(
        FromAPerson::read(&line).expect("the person's own door reads it"),
        theirs()
    );
}

/// **The agent's door refuses it**, which is the test this message exists for.
///
/// An agent that could send this would have a road to a provider that nothing
/// shows and nothing records — the one road in this product that is the
/// person's alone.
#[test]
fn the_agents_door_refuses_a_question_asked_as_the_person() {
    let line = theirs().written().expect("an answer cannot fail to write");
    assert!(matches!(
        FromAnAgent::read(&line),
        Err(NotUnderstood::NotForAnAgent)
    ));
}

/// And it is not read as the agent's own question by accident.
///
/// The two are different variants on purpose (ADR 0125 marked conflating them
/// the part most likely to be got wrong), so a round trip must not turn one
/// into the other.
#[test]
fn an_agents_question_and_a_persons_are_never_read_for_each_other() {
    let agents = FromAnAgent::Ask {
        question: "what is the capital of Spain".to_owned(),
        answered: crate::Answered::InWords,
    };
    let line = agents.written().expect("an answer cannot fail to write");
    assert!(matches!(
        FromAPerson::read(&line),
        Err(NotUnderstood::NotForAPerson)
    ));
    assert_eq!(
        FromAnAgent::read(&line).expect("its own door reads it"),
        agents
    );
}

/// The answer survives the wire with its three fields, at the person's door.
#[test]
fn the_answer_reaches_the_person_with_where_it_came_from() {
    let said = a_sentence();
    let answered = ToAPerson::answered("Madrid.", &said, "gpt-4o");
    let line = answered.written().expect("an answer cannot fail to write");
    let back = ToAPerson::read(&line).expect("the person's door reads its own answer");
    match back {
        ToAPerson::Answered {
            text,
            came_from,
            model,
        } => {
            assert_eq!(text, "Madrid.");
            assert_eq!(came_from.text(), said.text());
            assert_eq!(model, "gpt-4o");
        }
        other => panic!("the answer came back as something else: {other:?}"),
    }
}
