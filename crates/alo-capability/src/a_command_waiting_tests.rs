//! **A waiting command keeps the guarantees a waiting change keeps**: one
//! approval is one execution, a repeated or stale number finds nothing, a
//! question lapses, and a command and a change never answer for each other.

use super::*;
use crate::test_calls::{archiving_march, files, granting_both, hour, noon};
use crate::testing::in_english;

/// A command, ready to put to the person.
fn making_a_folder() -> ACommand {
    ACommand::of("mkdir -p /home/anna/demo").unwrap()
}

/// A list with one command waiting on it.
fn one_command_waiting() -> (Approvals, ProposalId) {
    let mut approvals = Approvals::default();
    let id = approvals
        .propose_a_command(making_a_folder(), &files(), noon(), hour())
        .unwrap();
    (approvals, id)
}

/// **One approval, one execution.** The second approval of the same number
/// finds nothing, because the first took it off the list.
#[test]
fn a_command_is_approved_once_and_never_again() {
    let (mut approvals, id) = one_command_waiting();
    let approved = approvals.approve_a_command(id, noon()).unwrap();
    assert_eq!(approved.id(), id);
    assert_eq!(approved.command(), &making_a_folder());
    assert_eq!(approved.grantee(), &files());
    assert_eq!(
        approvals.approve_a_command(id, noon()),
        Err(AnswerError::NothingWaiting {
            number: id.as_u64()
        })
    );
    assert!(approvals.is_empty());
}

/// **A command and a change share one counter and never an answer.** A number
/// a change waits under approves no command, a command's number approves no
/// change, and no number is handed out twice.
#[test]
fn a_command_and_a_change_share_numbers_and_never_an_answer() {
    let grants = granting_both();
    let mut approvals = Approvals::default();
    let change = approvals
        .propose(Proposal::checked(&archiving_march(), &files(), &grants, noon(), hour()).unwrap());
    let command = approvals
        .propose_a_command(making_a_folder(), &files(), noon(), hour())
        .unwrap();
    assert_ne!(change, command, "one number was handed out twice");
    assert!(command > change, "the command's number went backwards");

    assert!(matches!(
        approvals.approve_a_command(change, noon()),
        Err(AnswerError::NothingWaiting { .. })
    ));
    assert!(matches!(
        approvals.approve(command, noon()),
        Err(AnswerError::NothingWaiting { .. })
    ));
    assert_eq!(approvals.decline(command), None);
    assert!(approvals.decline_a_command(change).is_none());
    // Both still wait, because nothing answered either.
    assert_eq!(approvals.waiting_at(noon()).count(), 1);
    assert_eq!(approvals.commands_waiting_at(noon()).count(), 1);
}

/// **A question that stood too long is refused, and taken away as it is**,
/// and the refusal names the command so the person can ask again.
#[test]
fn a_command_that_waited_too_long_is_refused_and_gone() {
    let (mut approvals, id) = one_command_waiting();
    let later = noon() + hour();
    assert_eq!(approvals.commands_waiting_at(later).count(), 0);
    let lapsed = approvals.approve_a_command(id, later).unwrap_err();
    assert_eq!(
        lapsed,
        AnswerError::ACommandLapsed {
            command: making_a_folder()
        }
    );
    let said = lapsed.said(&in_english());
    assert!(said.text().contains("mkdir -p /home/anna/demo"), "{said:?}");
    assert!(!said.is_a_bug(), "{said:?}");
    assert!(approvals.command_of(id).is_none());
}

/// **A declined command is handed back once**, so the record can keep the no,
/// and cannot then be approved.
#[test]
fn a_declined_command_is_given_back_once_and_cannot_be_approved() {
    let (mut approvals, id) = one_command_waiting();
    let declined = approvals.decline_a_command(id).unwrap();
    assert_eq!(declined.command(), &making_a_folder());
    assert!(approvals.decline_a_command(id).is_none());
    assert!(approvals.approve_a_command(id, noon()).is_err());
}

/// **A question that stands for no time is never asked.**
#[test]
fn a_command_that_would_stand_for_no_time_is_never_put() {
    let mut approvals = Approvals::default();
    assert_eq!(
        approvals.propose_a_command(
            making_a_folder(),
            &files(),
            noon(),
            std::time::Duration::ZERO
        ),
        Err(ProposalError::NoTime)
    );
    assert!(approvals.is_empty());
}

/// What waits says how long is left, and the sentence is the command whole.
#[test]
fn a_waiting_command_says_what_it_is_and_how_long_is_left() {
    let (approvals, id) = one_command_waiting();
    let waiting = approvals.command_of(id).unwrap();
    assert_eq!(waiting.lapses_in(noon()), Some(hour()));
    assert_eq!(waiting.lapses_in(noon() + hour()), None);
    assert_eq!(
        waiting.sentence(&in_english()).text(),
        "run this command: mkdir -p /home/anna/demo"
    );
}
