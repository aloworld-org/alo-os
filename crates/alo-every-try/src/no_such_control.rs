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
use alo_protocol::{Done, FromAnAgent, ToAnAgent};
use alo_strings::Strings;

use crate::a_road::{ARequest, Answered, Step};
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

/// **What `OneThingShown::is` says of a slider or a number box** in a window
/// read (`alo_protocol::done`).
const A_SLIDER: &str = "slider";

/// The machine's refusal of `asked`, a move of a slider or number box by a
/// name the request's own window shows no slider of, where its road is not a
/// move of one; [`None`] where the request reads no window of that
/// application, where a read of it was not read whole, where one of its reads
/// shows a slider of that name, and for every other request.
///
/// **Where the window is the request's own**: every read of the application
/// the request answers — on any of its roads, or aside of them — is what the
/// machine reads before the card (`alo_turn`'s `the_control_to_set_is_there`),
/// so a name none of them shows as a slider is refused there, at both levels,
/// in the move's own words. Runs 44793910551 and 44793268747: the model moved
/// the Reload button with `accessible.set_value` three times.
#[must_use]
pub fn not_a_slider_the_window_shows(
    request: &ARequest,
    asked: &FromAnAgent,
    verbs: &Verbs,
    machine: &AsTheMachineNames,
    strings: &Strings,
) -> Option<ToAnAgent> {
    let FromAnAgent::Propose { verb, .. } = asked else {
        return None;
    };
    if verb.trim() != SET_VALUE {
        return None;
    }
    let given = machine.naming(verbs, SET_VALUE, &asked.given());
    let call = verbs.call(SET_VALUE, &given).ok()?;
    let (Some(Value::Application(application)), Some(Value::Name(name)), Some(Value::Count(value))) =
        (call.value(APPLICATION), call.value(NAME), call.value(VALUE))
    else {
        return None;
    };
    let mut reads = request
        .road()
        .iter()
        .chain(request.other_roads().iter().flatten())
        .chain(request.aside())
        .filter_map(|step| match step.answered() {
            Answered::With(Done::WindowsShowed {
                application: of,
                windows,
                not_all_read,
                ..
            }) if of == application => Some((windows, *not_all_read)),
            _ => None,
        })
        .peekable();
    reads.peek()?;
    let mut shown_whole = true;
    let mut a_slider_of_that_name = false;
    for (windows, not_all_read) in reads {
        shown_whole &= !not_all_read;
        a_slider_of_that_name |= windows
            .iter()
            .flat_map(|window| &window.things)
            .any(|thing| thing.is == A_SLIDER && thing.name == *name);
    }
    if !shown_whole || a_slider_of_that_name {
        return None;
    }
    let said = NotSet::NoSuchControl.said(application, name, *value, strings);
    Some(ToAnAgent::refused(&said))
}
