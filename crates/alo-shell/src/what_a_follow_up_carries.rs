//! **What a follow-up carries**: the conversation in a task window, as alo is
//! sent it with the person's next words
//! ([ADR 0137](../../../docs/decisions/0137-a-follow-up-is-sent-with-the-conversation-so-far.md)).
//!
//! Read off the task at the moment the person pressed Enter, and at no other —
//! *context is offered, never watched*. Nothing here runs on a timer or on a
//! frame, and nothing is kept between one follow-up and the next: each is built
//! again from what the window holds.
//!
//! # Made of what the person saw, and nothing else
//!
//! Each turn of the conversation as the window draws it: what they asked, what
//! alo answered, and each change alo proposed with what became of it. **The
//! activity is not read**, and neither is anything a verb read during an
//! earlier turn — the shell never holds either as conversation. So a file's
//! contents travel with a follow-up only where the person's words or alo's
//! answer quoted them, which is a thing the person could see on the screen.

use alo_instructing::{AnEarlierTurn, TheAnswer, TheConversationSoFar};
use alo_tasks::{ATask, Answered, WhoSaidIt};

/// **The conversation in this task window, whole**, as alo is sent it before
/// the reply the person is sending now.
///
/// Read before the reply joins the conversation, so the reply is not in it: it
/// is the request, and it travels as one.
pub(crate) fn the_conversation_so_far(task: &ATask) -> TheConversationSoFar {
    let mut answers: Vec<&(String, Answered)> = task.answers_given().each().iter().collect();
    let earlier = task
        .conversation()
        .each()
        .iter()
        .map(|turn| match turn.who() {
            WhoSaidIt::ThePerson => AnEarlierTurn::ThePersonSaid(turn.said().to_owned()),
            WhoSaidIt::Alo | WhoSaidIt::AloAsking => {
                AnEarlierTurn::AloAnswered(turn.said().to_owned())
            }
            WhoSaidIt::AloProposing => AnEarlierTurn::AloProposed {
                sentence: turn.said().to_owned(),
                answer: what_became_of(turn.said(), &mut answers),
            },
        })
        .collect();
    TheConversationSoFar::of(earlier)
}

/// The answer given to this proposal: the first not yet matched to an earlier
/// proposal of the same sentence, so the same sentence proposed twice is
/// answered twice in order.
fn what_became_of(sentence: &str, answers: &mut Vec<&(String, Answered)>) -> TheAnswer {
    let Some(at) = answers.iter().position(|(said, _)| said == sentence) else {
        return TheAnswer::NotAnswered;
    };
    match answers.remove(at).1 {
        Answered::AllowedOnce => TheAnswer::Allowed,
        Answered::Denied => TheAnswer::Denied,
        Answered::NothingWasAsked => TheAnswer::NotAnswered,
    }
}

#[cfg(test)]
#[path = "what_a_follow_up_carries_tests.rs"]
mod tests;
