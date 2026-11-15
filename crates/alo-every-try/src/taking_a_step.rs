//! **Whether what the model asked for is the step the road expects.**
//!
//! In the order the daemon would decide it: a command is not a verb; the verb
//! is the one expected; its arguments are ones the machine accepts, through
//! `Verbs::call`, the daemon's own validator; it came through the door its
//! effect takes; and each value the road decides is that value.

use alo_capability::{Call, Verbs};
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
    let _ = (asked, verbs, strings);
    Err(WentWrong::AnotherVerb {
        step: at,
        expected: step.verb().to_owned(),
        asked: "nothing".to_owned(),
    })
}

#[cfg(test)]
#[path = "taking_a_step_tests.rs"]
mod tests;
