//! **A slider named as nothing in the window is, answered as the machine
//! answers it.**
//!
//! Since 2026-11-17 the machine reads the window before the card and refuses a
//! move of a slider or number box it does not show by that name, at both
//! levels, in the move's own words (`alo_turn`'s `the_control_to_set_is_there`,
//! `alo_adapters::not_there_to_set`). The model reads the refusal and may name
//! it again. So where a road's step says the window shows no other control of
//! its kind ([`crate::a_road::Step::the_only_control_of_its_kind`]), another
//! name is refused here in those words, and the try goes on.

use alo_adapters::NotSet;
use alo_adapters::fallback_verbs::{APPLICATION, NAME, SET_VALUE, VALUE};
use alo_capability::{Value, Verbs};
use alo_protocol::{FromAnAgent, ToAnAgent};
use alo_strings::Strings;

use crate::a_road::Step;
use crate::as_the_machine_names::AsTheMachineNames;
use crate::went_wrong::WentWrong;

/// The machine's refusal of `asked`, taken for `step`, where it moves the one
/// control of its kind by a name the window does not show; [`None`] for every
/// other way of leaving the road.
#[must_use]
pub fn no_such_control(
    step: &Step,
    wrong: &WentWrong,
    asked: &FromAnAgent,
    verbs: &Verbs,
    machine: &AsTheMachineNames,
    strings: &Strings,
) -> Option<ToAnAgent> {
    if step.verb() != SET_VALUE || !step.is_the_only_control_of_its_kind() {
        return None;
    }
    let WentWrong::AnotherValue { argument, .. } = wrong else {
        return None;
    };
    if argument != NAME {
        return None;
    }
    let given = machine.naming(verbs, SET_VALUE, &asked.given());
    let call = verbs.call(SET_VALUE, &given).ok()?;
    let (Some(Value::Application(application)), Some(Value::Name(name)), Some(Value::Count(value))) =
        (call.value(APPLICATION), call.value(NAME), call.value(VALUE))
    else {
        return None;
    };
    let said = NotSet::NoSuchControl.said(application, name, *value, strings);
    Some(ToAnAgent::refused(&said))
}
