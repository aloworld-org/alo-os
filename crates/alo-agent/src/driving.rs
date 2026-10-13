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
//! # Why a refusal is a step and not an end, whichever door it came from
//!
//! A refusal — a folder nobody granted, a file that is not there — is told back
//! to the model, and it may ask for something else. That is how a person works,
//! and the alternative is a turn that gives up on its first wrong guess.
//!
//! **This now holds for a refused proposal as well as a refused read**, and it
//! used not to. While a refused proposal could only mean *the grants said no*,
//! ending the turn was right. Since
//! `alo_turn::the_sentence_is_about_something` it can also mean *there is
//! nothing at that path*, which is precisely what a model that guessed a
//! filename has to be told — and measured on 2026-10-13, ending there meant the
//! one refusal that could teach it was the one it never saw.
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
use alo_instructing::{AStep, ATurnInProgress, Instructions, shown_in_progress};
use alo_protocol::{Answered, FromAnAgent, ToAnAgent};

use crate::how_far::HowFar;
use crate::the_door::{NotAnswered, TheDoor};
use crate::what_happened::WhatHappened;

/// Drive one request to its end.
///
/// `verbs` is the list the machine really has, so the model is told about what
/// exists; this crate reads it and decides nothing from it.
///
/// `instructions` is **taken and not assumed**, because a grade is a statement
/// about a set of them: a caller with no opinion passes
/// `Instructions::SHOWN_TO_A_TURN`, which is the measured one, and a harness
/// comparing two passes each in turn. A loop that chose for itself would be a
/// second place deciding the one thing every grade is about.
pub fn driving(
    request: &str,
    verbs: &Verbs,
    door: &mut dyn TheDoor,
    how_far: HowFar,
    instructions: Instructions,
) -> WhatHappened {
    let mut steps: Vec<AStep> = Vec::new();

    for _ in 0..how_far.rounds() {
        let shown = shown_in_progress(instructions, verbs, ATurnInProgress::of(request, &steps));
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
            // Not a line of the protocol, and there are two of those.
            //
            // **Words** — *it is in /home/anna/Invoices* — are the whole answer
            // to a request that needed no change, and the turn is finished.
            //
            // **A request that tried and failed** is a different thing, and
            // ending the turn on it throws away a correctable mistake.
            // Measured on 2026-10-13: `gpt-4o-mini` wrote
            // `{"asks":{"find_in_folder":{"verb":"find_in_folder",…}}}` — the
            // verb's name where the door belongs — and the turn ended as though
            // the model had answered in prose. It had not; it had made a shape
            // error it could have been told about.
            //
            // Told apart by whether it looks like JSON at all, which is the
            // only distinction available here and is not a judgement about
            // content. What the model is shown is the protocol's own `Debug` of
            // its own complaint, because `alo_protocol::NotUnderstood` has no
            // `Display` on purpose and this loop may not reword it.
            Err(why) if said.trim_start().starts_with('{') => {
                steps.push(AStep::of(&said, &format!("not a request: {why:?}")));
                continue;
            }
            Err(_) => {
                return WhatHappened::ItWasAnsweredInWords { said, after: steps };
            }
        };

        match next {
            FromAnAgent::Read { .. } => match door.asking(&next) {
                Ok(answer) => {
                    // A refusal is a step and the turn goes on, whichever door
                    // it came from — this module's header is the argument, and
                    // the bound is what ends it.
                    steps.push(AStep::of(&said, &as_a_step(&answer)));
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
                // **A refused proposal is a step too, for the same reason a
                // refused read is**, and this arm used to end the turn.
                //
                // It was right while a refused proposal could only mean *the
                // grants said no*, which no amount of trying again fixes. Since
                // `alo_turn::the_sentence_is_about_something` it can also mean
                // *there is nothing at that path* — which is exactly what a
                // model that guessed a filename needs to be told, and it has
                // rounds left to use. Measured: `gpt-4o-mini` proposed
                // `March-invoice.pdf` in a folder holding `march.pdf`, and
                // ending here meant the one refusal that could teach it was the
                // one it never saw.
                //
                // **The loop does not tell the two apart**, and will not: which
                // refusals are worth another go is the capability model's
                // opinion and it has exactly one per request. A model that
                // proposes the same forbidden thing until the bound stops it is
                // a short turn and a record full of correct refusals, which is
                // the cheaper mistake.
                Ok(ToAnAgent::Refused(why)) => {
                    let line = ToAnAgent::Refused(why).written().unwrap_or_default();
                    steps.push(AStep::of(&said, &line));
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

/// One answer, as the line a model is shown.
///
/// What the daemon really said, re-serialised by the protocol's own writer —
/// the one place that decides how a message reads. **Whether it was a refusal
/// is not answered here**, because the loop no longer asks: a refusal is a step
/// either way.
fn as_a_step(answer: &ToAnAgent) -> String {
    answer
        .written()
        .unwrap_or_else(|_| "{\"format\":1,\"tells\":{}}".to_owned())
}

/// The door failed, and how far the turn had got.
fn could_not_be_asked(why: NotAnswered, after: Vec<AStep>) -> WhatHappened {
    WhatHappened::TheMachineCouldNotBeAsked {
        why: why.to_string(),
        after,
    }
}
