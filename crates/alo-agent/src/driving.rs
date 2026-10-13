//! **The loop**: a person's words, a model asked what to do, and the machine
//! doing it — until a change waits for the person, or the words are the answer.
//!
//! Everything this calls was already built and tested. What was missing was
//! this: `alo-agentd` answers an agent, `alo_turn::Turning` decides every step
//! correctly, `alo-instructing` writes what a model is shown, `alo-protocol`
//! carries both halves, and **nothing was the agent**. A person could type a
//! request and a window opened with nothing behind it.
//!
//! # One round
//!
//! 1. Show the model the turn so far — the instructions a turn shows, the verbs
//!    this machine really has, the request, and every step already taken.
//! 2. Ask for its next request, held to the envelope (ADR 0032).
//! 3. Read the answer as a line of the protocol.
//! 4. A **read** goes to the machine, and what came back becomes a step. Round
//!    again.
//! 5. A **propose** goes to the machine and the turn **ends**: a change waits
//!    for a person, and waiting for them is not something an agent does.
//! 6. A model that answered in words rather than asking for anything has
//!    finished the request.
//!
//! # Why a refusal is a step and not an end
//!
//! A read refused — a folder nobody granted, a file that is gone — is told back
//! to the model, and it may ask for something else. That is how a person works,
//! and the alternative is a turn that gives up on its first wrong guess.
//!
//! It is bounded by [`HowFar`] and by nothing else, which is deliberate: a
//! model that asks for the same forbidden folder eight times is eight correct
//! refusals in the record and a turn that ends saying it got nowhere. Counting
//! *kinds* of refusal here would be this crate second-guessing the capability
//! model, which has one opinion about each request and does not need another.
//!
//! # What this crate cannot do, by construction
//!
//! It cannot approve anything. [`TheDoor`] is typed to the agent's half of the
//! protocol, and the approval lives on the person's door, in another process
//! (`docs/contracts/daemon-protocol.md`). It also cannot ask a model anything
//! except through the machine: there is no provider client here, so **which
//! model answers and whether anything leaves is the daemon's to decide from the
//! person's settings**, and law 1's indicator is lit by it rather than by us.

use alo_capability::Verbs;
use alo_instructing::{AStep, ATurnInProgress, shown_to_a_turn_in_progress};
use alo_protocol::{Answered, FromAnAgent, ToAnAgent};

use crate::how_far::HowFar;
use crate::the_door::{NotAnswered, TheDoor};
use crate::what_happened::WhatHappened;

/// Drive one request to its end.
///
/// `verbs` is the list the machine really has, so the model is told about what
/// exists; this crate reads it and decides nothing from it.
pub fn driving(
    request: &str,
    verbs: &Verbs,
    door: &mut dyn TheDoor,
    how_far: HowFar,
) -> WhatHappened {
    let mut steps: Vec<AStep> = Vec::new();

    for _ in 0..how_far.rounds() {
        let shown = shown_to_a_turn_in_progress(verbs, ATurnInProgress::of(request, &steps));
        let asked = FromAnAgent::Ask {
            question: shown,
            answered: Answered::AsTheNextRequest,
        };
        let said = match door.asking(&asked) {
            Ok(ToAnAgent::Answered { text, .. }) => text,
            Ok(ToAnAgent::Refused(why)) => {
                return WhatHappened::ItWasRefused {
                    said: why.text().to_owned(),
                    after: steps,
                };
            }
            // We asked a question; an answer about a verb means this machine
            // answered something we did not ask. Its own fault, in English.
            Ok(ToAnAgent::Did(_) | ToAnAgent::Proposed(_)) => {
                return WhatHappened::TheMachineCouldNotBeAsked {
                    why: "a question was answered with what a verb did".to_owned(),
                    after: steps,
                };
            }
            Err(not_answered) => return could_not_be_asked(not_answered, steps),
        };

        // What the model wrote, as it wrote it. The model is shown this back
        // rather than a re-rendering of it — `alo_instructing::AStep` says why.
        let next = match FromAnAgent::read(&said) {
            Ok(next) => next,
            // Not a line of the protocol. The model has answered in words,
            // which for a request that needed no change is the whole answer.
            Err(_) => {
                return WhatHappened::ItWasAnsweredInWords { said, after: steps };
            }
        };

        match next {
            FromAnAgent::Read { .. } => match door.asking(&next) {
                Ok(answer) => {
                    let (line, refused) = as_a_step(&answer);
                    steps.push(AStep::of(&said, &line));
                    // A refusal is a step and the turn goes on — this module's
                    // header is the argument. The bound is what ends it.
                    let _ = refused;
                }
                Err(not_answered) => return could_not_be_asked(not_answered, steps),
            },
            FromAnAgent::Propose { .. } => match door.asking(&next) {
                Ok(ToAnAgent::Proposed(standing)) => {
                    return WhatHappened::AChangeIsWaiting {
                        number: standing.number(),
                        sentence: standing.sentence().text().to_owned(),
                        after: steps,
                    };
                }
                Ok(ToAnAgent::Refused(why)) => {
                    return WhatHappened::ItWasRefused {
                        said: why.text().to_owned(),
                        after: steps,
                    };
                }
                Ok(ToAnAgent::Did(_) | ToAnAgent::Answered { .. }) => {
                    return WhatHappened::TheMachineCouldNotBeAsked {
                        why: "a change was answered as though it had already run".to_owned(),
                        after: steps,
                    };
                }
                Err(not_answered) => return could_not_be_asked(not_answered, steps),
            },
            // The model asked for a question to be put to a model, which is
            // what we just did for it. Asking again would be the same round
            // with nothing added, so the turn ends rather than spending the
            // bound on a loop with no step in it.
            FromAnAgent::Ask { .. } => {
                return WhatHappened::ItWasAnsweredInWords { said, after: steps };
            }
        }
    }

    WhatHappened::ItRanOutOfRounds { after: steps }
}

/// One answer, as the line a model is shown and whether it was a refusal.
///
/// The line is what the daemon really said, re-serialised by the protocol's own
/// writer — the one place that decides how a message reads.
fn as_a_step(answer: &ToAnAgent) -> (String, bool) {
    let refused = matches!(answer, ToAnAgent::Refused(_));
    let line = answer
        .written()
        .unwrap_or_else(|_| "{\"format\":1,\"tells\":{}}".to_owned());
    (line, refused)
}

/// The door failed, and how far the turn had got.
fn could_not_be_asked(why: NotAnswered, after: Vec<AStep>) -> WhatHappened {
    WhatHappened::TheMachineCouldNotBeAsked {
        why: why.to_string(),
        after,
    }
}
