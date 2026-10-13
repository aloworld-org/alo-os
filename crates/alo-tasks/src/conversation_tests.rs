//! That who said a thing is carried, and that waiting is the last turn's
//! business.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use super::{TheConversation, Turn, WhoSaidIt};

/// **A conversation begins with the person.**
///
/// A task exists because somebody asked for something. One that began with alo
/// speaking would be the agent starting a conversation nobody invited.
#[test]
fn a_conversation_begins_with_the_request() {
    let talk = TheConversation::began_with("Tidy up my invoices folder");

    let request = talk.the_request().expect("a task has a request");
    assert_eq!(request.who(), &WhoSaidIt::ThePerson);
    assert_eq!(request.said(), "Tidy up my invoices folder");
    assert_eq!(talk.each().len(), 1);
}

/// **Who said a thing is carried, never inferred.**
///
/// A person has to be able to tell their words from the machine's at a glance
/// and without colour. A surface working it out from position or punctuation
/// gets it wrong somewhere, and where it does, somebody believes they said
/// something they did not.
#[test]
fn every_turn_carries_its_speaker() {
    let mut talk = TheConversation::began_with("Tidy up my invoices folder");
    talk.and_then(Turn::from_alo("There are 14 files, 3 of them duplicates."));
    talk.and_then(Turn::alo_asking("Shall I keep the newest of each?"));
    talk.and_then(Turn::from_the_person("Yes"));
    talk.and_then(Turn::alo_proposing(
        "Delete 3 duplicate files in Documents/invoices",
    ));

    let who: Vec<&WhoSaidIt> = talk.each().iter().map(Turn::who).collect();
    assert_eq!(
        who,
        vec![
            &WhoSaidIt::ThePerson,
            &WhoSaidIt::Alo,
            &WhoSaidIt::AloAsking,
            &WhoSaidIt::ThePerson,
            &WhoSaidIt::AloProposing,
        ],
        "a turn lost or changed its speaker"
    );
}

/// **A question and a proposal are both waiting; a reply is not.**
#[test]
fn a_question_and_a_proposal_wait_and_a_reply_does_not() {
    assert!(Turn::alo_asking("Shall I keep the newest?").is_waiting_for_the_person());
    assert!(Turn::alo_proposing("Delete 3 files").is_waiting_for_the_person());
    assert!(!Turn::from_alo("There are 14 files.").is_waiting_for_the_person());
    assert!(!Turn::from_the_person("Yes").is_waiting_for_the_person());
}

/// **A question the person answered is not still waiting.**
///
/// Only the last turn decides. A conversation that scanned for any unanswered
/// question would keep a task looking blocked for the rest of its life — and
/// *is this waiting on me* is the question somebody asks of a minimised task
/// from across the room.
#[test]
fn an_answered_question_stops_the_task_waiting() {
    let mut talk = TheConversation::began_with("Tidy up my invoices folder");
    talk.and_then(Turn::alo_asking("Shall I keep the newest of each?"));
    assert!(talk.is_waiting_for_the_person(), "the premise");

    talk.and_then(Turn::from_the_person("Yes"));

    assert!(
        !talk.is_waiting_for_the_person(),
        "a question the person answered left the task looking blocked"
    );

    // And a later proposal makes it wait again, so this is about the last turn
    // rather than about having ever been answered.
    talk.and_then(Turn::alo_proposing("Delete 3 duplicate files"));
    assert!(talk.is_waiting_for_the_person());
}

/// **A task alo is still working on is not waiting for the person.**
#[test]
fn a_task_still_working_is_not_waiting() {
    let mut talk = TheConversation::began_with("Install Blender");
    talk.and_then(Turn::from_alo("Downloading Blender 4.2…"));

    assert!(!talk.is_waiting_for_the_person());
}

/// **A proposal is a turn in the conversation**, which is the owner's design:
/// *show the specific proposed action in the conversation*. It is not a
/// dialogue over the top, so a person reads it in the order it happened.
#[test]
fn a_proposal_is_part_of_what_was_said() {
    let mut talk = TheConversation::began_with("Tidy up my invoices folder");
    talk.and_then(Turn::alo_proposing(
        "Delete 3 duplicate files in Documents/invoices",
    ));

    let last = talk.each().last().expect("the proposal is there");
    assert_eq!(last.who(), &WhoSaidIt::AloProposing);
    assert_eq!(
        last.said(),
        "Delete 3 duplicate files in Documents/invoices",
        "the proposal is not the sentence a person approves"
    );
}
