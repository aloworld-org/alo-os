//! **Typing that saves, aimed at a one-line field, answered as the machine
//! answers it.**
//!
//! Since 2026-11-20 the machine reads the window before the card and refuses
//! `accessible.type_text` into a one-line field such as a search box, at both
//! levels, in words that name `accessible.type_without_saving`
//! (`alo_turn`'s `the_typing_that_saves_is_for_a_document`,
//! `alo_adapters::a_one_line_field_to_save`). The model reads the refusal and
//! may type without saving instead. So where a road's step types without
//! saving into a one-line field ([`crate::a_road::Step::a_one_line_field`]),
//! typing that saves into that same field is refused here in those words, and
//! the try goes on.

use alo_adapters::a_one_line_field_said;
use alo_adapters::fallback_verbs::{APPLICATION, FIELD, NAME, TYPE_TEXT, TYPE_WITHOUT_SAVING};
use alo_capability::{Value, Verbs};
use alo_protocol::{FromAnAgent, ToAnAgent};
use alo_strings::Strings;

use crate::a_road::Step;
use crate::as_the_machine_names::AsTheMachineNames;
use crate::went_wrong::WentWrong;

/// The machine's refusal of `asked`, taken for `step`, where it types and
/// saves into the one-line field the step types into without saving;
/// [`None`] for every other way of leaving the road.
#[must_use]
pub fn typing_that_saves_a_field(
    step: &Step,
    wrong: &WentWrong,
    asked: &FromAnAgent,
    verbs: &Verbs,
    machine: &AsTheMachineNames,
    strings: &Strings,
) -> Option<ToAnAgent> {
    if step.verb() != TYPE_WITHOUT_SAVING || !step.is_a_one_line_field() {
        return None;
    }
    let WentWrong::AnotherVerb { asked: verb, .. } = wrong else {
        return None;
    };
    if verb != TYPE_TEXT {
        return None;
    }
    let given = machine.naming(verbs, TYPE_TEXT, &asked.given());
    let call = verbs.call(TYPE_TEXT, &given).ok()?;
    let (Some(Value::Application(application)), Some(Value::Name(name))) =
        (call.value(APPLICATION), call.value(NAME))
    else {
        return None;
    };
    // **Only the field the step types into**, in the application it is in:
    // another name means another place, which the machine answers otherwise.
    let expected = |argument: &str| {
        step.expected()
            .iter()
            .find(|(named, _)| *named == argument)
            .map(|(_, is)| is.as_str())
    };
    if expected(APPLICATION) != Some(application.as_str()) || expected(FIELD) != Some(name.as_str())
    {
        return None;
    }
    let said = a_one_line_field_said(application, name, strings);
    Some(ToAnAgent::refused(&said))
}
