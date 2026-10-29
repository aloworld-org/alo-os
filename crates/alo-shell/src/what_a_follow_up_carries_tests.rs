//! What a follow-up is sent with, built from a real task.

use alo_instructing::{AnEarlierTurn, TheAnswer};
use alo_models::InferenceSource;
use alo_tasks::{ATask, Act, Turn};

use super::the_conversation_so_far;

/// A task that listed, proposed three changes, and was answered once each way
/// — the third never answered.
fn a_task_that_proposed() -> ATask {
    let mut task = ATask::asked_for("list my invoices", InferenceSource::ThisMachine);
    task.and_then(Act::done(
        "listed /home/anna/Invoices: march.pdf, april.pdf",
    ));
    task.alo_says(Turn::from_alo("march.pdf and april.pdf"));
    task.is_asking("Move march.pdf to Paid");
    task.alo_says(Turn::alo_proposing("Move march.pdf to Paid"));
    let _ = task.allow_once();
    task.is_asking("Delete april.pdf");
    task.alo_says(Turn::alo_proposing("Delete april.pdf"));
    let _ = task.deny();
    task.alo_says(Turn::alo_proposing("Rename notes.txt"));
    task
}

/// **Exactly the earlier turns, in order, with what became of each change.**
#[test]
fn a_follow_up_carries_exactly_the_earlier_turns() {
    let earlier = the_conversation_so_far(&a_task_that_proposed());
    assert_eq!(
        earlier.turns(),
        [
            AnEarlierTurn::ThePersonSaid("list my invoices".to_owned()),
            AnEarlierTurn::AloAnswered("march.pdf and april.pdf".to_owned()),
            AnEarlierTurn::AloProposed {
                sentence: "Move march.pdf to Paid".to_owned(),
                answer: TheAnswer::Allowed,
            },
            AnEarlierTurn::AloProposed {
                sentence: "Delete april.pdf".to_owned(),
                answer: TheAnswer::Denied,
            },
            AnEarlierTurn::AloProposed {
                sentence: "Rename notes.txt".to_owned(),
                answer: TheAnswer::NotAnswered,
            },
        ]
    );
}

/// **What a verb read is not carried**: the activity's details are the
/// machine's account, not the conversation, and never travel with a follow-up.
#[test]
fn what_the_activity_holds_is_never_carried() {
    let earlier = the_conversation_so_far(&a_task_that_proposed());
    assert!(!earlier.told().contains("listed /home/anna/Invoices"));
    assert!(!earlier.written().contains("listed /home/anna/Invoices"));
}

/// **A held reply is not part of what came before**: it is the request when
/// it is sent, not something alo is told it already heard.
#[test]
fn a_held_reply_is_not_carried_as_earlier() {
    let mut task = a_task_that_proposed();
    task.hold("and the April one too");
    assert!(
        !the_conversation_so_far(&task)
            .told()
            .contains("the April one")
    );
}

/// **A long conversation is carried whole** — nothing is left out of it
/// (ADR 0137).
#[test]
fn a_long_conversation_is_carried_whole() {
    let mut task = ATask::asked_for("list everything", InferenceSource::ThisMachine);
    for _ in 0..20 {
        task.alo_says(Turn::from_alo("x".repeat(3000)));
        task.the_person_says("go on");
    }
    assert_eq!(
        the_conversation_so_far(&task).turns().len(),
        task.conversation().each().len()
    );
}
