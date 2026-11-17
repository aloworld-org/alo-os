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
use crate::as_the_machine_names::AsTheMachineNames;
use crate::went_wrong::WentWrong;

/// Take `step`, the `at`-th of its road (from 1), with what the model `asked`,
/// on `machine`.
///
/// # Errors
/// [`WentWrong`], saying how it left the road.
pub fn taking(
    step: &Step,
    at: usize,
    asked: &FromAnAgent,
    verbs: &Verbs,
    machine: &AsTheMachineNames,
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
                given: String::new(),
            });
        }
        FromAnAgent::Read { verb, .. } | FromAnAgent::Propose { verb, .. } => verb.trim(),
    };
    if named != step.verb() {
        return Err(WentWrong::AnotherVerb {
            step: at,
            expected: step.verb().to_owned(),
            asked: named.to_owned(),
            given: crate::went_wrong::as_said(&asked.given()),
        });
    }
    // **As the machine reads it**: an application named by its name is the
    // installed one that answers to it, and an application to install named by
    // its name is the catalogue's identifier — in that order, by the turn's own
    // functions (`Turning::calling_by_name`), so the check holds the model to
    // what the machine accepts and no more.
    let given = machine.naming(verbs, named, &asked.given());
    let given = alo_turn::as_the_catalogue_names_it(named, given).map_err(|why| {
        WentWrong::RefusedBeforeAsking {
            step: at,
            verb: named.to_owned(),
            said: Box::new(why.said(strings)),
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
    for (argument, place) in step.fields() {
        let given = call
            .value(argument)
            .map_or_else(|| "nothing".to_owned(), Value::describe);
        // **As the machine finds a field**: by the rule typing uses, so the
        // check takes every name the machine would type into, and no other.
        if !alo_adapters::places_for_text::is_meant_by(place, &given) {
            return Err(WentWrong::AnotherValue {
                step: at,
                argument: (*argument).to_owned(),
                expected: place.name.clone(),
                given,
            });
        }
    }
    Ok(call)
}

#[cfg(test)]
#[path = "taking_a_step_tests.rs"]
mod tests;
