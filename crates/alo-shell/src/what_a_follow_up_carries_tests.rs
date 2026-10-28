//! What a follow-up is sent with, built from a real task.

use alo_instructing::{AnEarlierTurn, TheAnswer};
use alo_models::InferenceSource;
use alo_tasks::{ATask, Act, Turn};

use super::the_conversation_before_the_reply;

/// A task that listed, proposed two changes, and was answered once each way —
/// and then the person replied.
fn a_task_with_a_reply() -> ATask {
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
    task.the_person_says("and the April one too");
    task
}

/// **Exactly the earlier turns, in order, with what became of each change** —
/// and not the reply, which travels as the request.
#[test]
fn a_follow_up_carries_exactly_the_earlier_turns() {
    let earlier = the_conversation_before_the_reply(&a_task_with_a_reply());
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
    assert!(!earlier.is_shortened());
}

/// **What a verb read is not carried**: the activity's details are the
/// machine's account, not the conversation, and never travel with a follow-up.
#[test]
fn what_the_activity_holds_is_never_carried() {
    let earlier = the_conversation_before_the_reply(&a_task_with_a_reply());
    let told = earlier.told();
    assert!(!told.contains("listed /home/anna/Invoices"), "{told}");
    assert!(!earlier.written().contains("listed /home/anna/Invoices"));
}

/// **A task with only its request has nothing before it** — which is what a
/// first request carries.
#[test]
fn a_task_with_only_its_request_has_nothing_before_it() {
    let task = ATask::asked_for("list my invoices", InferenceSource::ThisMachine);
    assert!(the_conversation_before_the_reply(&task).is_empty());
}

/// **A long conversation says it was shortened**, so the window can tell the
/// person and the agent can tell the model.
#[test]
fn a_conversation_over_the_bound_says_so() {
    let mut task = ATask::asked_for("list everything", InferenceSource::ThisMachine);
    for _ in 0..5 {
        task.alo_says(Turn::from_alo("x".repeat(3000)));
        task.the_person_says("go on");
    }
    let earlier = the_conversation_before_the_reply(&task);
    assert!(earlier.is_shortened());
    assert!(earlier.left_out() > 0);
}
