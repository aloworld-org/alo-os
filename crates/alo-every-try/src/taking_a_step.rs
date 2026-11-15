//! **Whether what the model asked for is the step the road expects.**
//!
//! In the order the daemon would decide it: a command is not a verb; the verb
//! is the one expected; its arguments are ones the machine accepts, through
//! `Verbs::call`, the daemon's own validator; it came through the door its
//! effect takes; and each value the road decides is that value.

use alo_capability::{Call, Value, Verbs};
use alo_protocol::FromAnAgent;
use alo_strings::Strings;

use crate::a_road::Step;
use crate::went_wrong::WentWrong;

/// Take `step`, the `at`-th of its road (from 1), with what the model `asked`.
///
/// # Errors
/// [`WentWrong`], saying how it left the road.
pub fn taking(
    step: &Step,
    at: usize,
    asked: &FromAnAgent,
    verbs: &Verbs,
    strings: &Strings,
) -> Result<Call, WentWrong> {
    let named = match asked {
        FromAnAgent::Run { command } => {
            return Err(WentWrong::RanACommand {
                step: at,
                expected: step.verb().to_owned(),
                command: command.clone(),
            });
        }
        FromAnAgent::Ask { .. } => {
            return Err(WentWrong::AnotherVerb {
                step: at,
                expected: step.verb().to_owned(),
                asked: "a question to a model".to_owned(),
            });
        }
        FromAnAgent::Read { verb, .. } | FromAnAgent::Propose { verb, .. } => verb.trim(),
    };
    if named != step.verb() {
        return Err(WentWrong::AnotherVerb {
            step: at,
            expected: step.verb().to_owned(),
            asked: named.to_owned(),
        });
    }
    // **As the machine reads it**: an application to install named by its
    // name is the catalogue's identifier, by the turn's own function, so the
    // check holds the model to what the machine accepts and no more.
    let given = alo_turn::as_the_catalogue_names_it(named, asked.given()).map_err(|why| {
        WentWrong::ArgumentsRefused {
            step: at,
            verb: named.to_owned(),
            why: why.said(strings).into_text(),
        }
    })?;
    let call = verbs
        .call(named, &given)
        .map_err(|why| WentWrong::ArgumentsRefused {
            step: at,
            verb: named.to_owned(),
            why: why.said(strings).into_text(),
        })?;
    if call.waits_for_approval() != asked.waits_for_a_person() {
        return Err(WentWrong::TheWrongDoor {
            step: at,
            verb: named.to_owned(),
            door: if asked.waits_for_a_person() {
                "change"
            } else {
                "read"
            },
        });
    }
    for (argument, expected) in step.expected() {
        let given = call
            .value(argument)
            .map_or_else(|| "nothing".to_owned(), Value::describe);
        if &given != expected {
            return Err(WentWrong::AnotherValue {
                step: at,
                argument: (*argument).to_owned(),
                expected: expected.clone(),
                given,
            });
        }
    }
    Ok(call)
}

#[cfg(test)]
#[path = "taking_a_step_tests.rs"]
mod tests;
