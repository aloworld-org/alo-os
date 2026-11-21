//! **A call the machine turns away as it is written, answered as the machine
//! answers it.**
//!
//! A booted machine makes every request into a call through `Verbs::call`
//! before it decides anything else (`alo_turn::Turning::calling`, the one door
//! in), and a call that does not form — a verb not on the list, an argument
//! it does not take or that is missing, a text with nothing in it — is turned
//! away there, at both levels, before any card. The daemon answers the agent
//! with that refusal (`alo_agentd`'s `not_done`), and the agent's loop reads
//! it as a step and goes on. So a try that sends such a call gets the same
//! refusal here and goes on, rather than ending where a booted machine would
//! not have ended it.

use alo_capability::Verbs;
use alo_protocol::{FromAnAgent, ToAnAgent};
use alo_strings::Strings;

use crate::as_the_machine_names::AsTheMachineNames;

/// The machine's refusal of `asked`, where it is a verb's request that does
/// not form into a call as the machine forms one; [`None`] for a command, a
/// question, an answer, and every request that does form.
#[must_use]
pub fn turned_away(
    asked: &FromAnAgent,
    verbs: &Verbs,
    machine: &AsTheMachineNames,
    strings: &Strings,
) -> Option<ToAnAgent> {
    let (FromAnAgent::Read { verb, .. } | FromAnAgent::Propose { verb, .. }) = asked else {
        return None;
    };
    let verb = verb.trim();
    // **As the machine forms it**: an application named by its name is the
    // installed one that answers to it, an application to install is the
    // catalogue's identifier, and a publication's account is the machine's to
    // fill — in the turn's own order (`Turning::calling_by_name`).
    let given = machine.naming(verbs, verb, &asked.given());
    let given = alo_turn::as_the_catalogue_names_it(verb, given).ok()?;
    let given = alo_publishing::what_the_model_must_give(verb, &given);
    let why = verbs.call(verb, &given).err()?;
    Some(ToAnAgent::refused(&why.said(strings)))
}

#[cfg(test)]
mod tests {
    use super::turned_away;
    use crate::as_the_machine_names::AsTheMachineNames;
    use crate::testing::{proposing, read, reading, the_verbs, the_words};
    use alo_adapters::fallback_verbs::{
        ACTIVATE_CONTROL, APPLICATION, KIND, NAME, TEXT, TYPE_TEXT,
    };
    use alo_instructing::WhatItMayReach;

    fn asked(line: &str) -> Option<String> {
        let machine = AsTheMachineNames::of(&WhatItMayReach::nothing().at_full_access(true));
        turned_away(&read(line), &the_verbs(), &machine, &the_words())
            .map(|refused| refused.written().unwrap_or_default())
    }

    /// A text with nothing in it, a verb nobody declared and an argument the
    /// verb does not take are each turned away in the machine's words.
    #[test]
    fn a_call_that_does_not_form_is_turned_away() {
        let empty = proposing(
            TYPE_TEXT,
            &[
                (APPLICATION, "org.mozilla.firefox"),
                (NAME, "Reload"),
                (TEXT, ""),
            ],
        );
        let said = alo_capability::CallError::Argument(alo_capability::ArgError::Empty {
            argument: TEXT.to_owned(),
        })
        .said(&the_words())
        .into_text();
        assert!(
            asked(&empty).is_some_and(|refused| refused.contains(&said)),
            "{:?}",
            asked(&empty)
        );
        assert!(asked(&reading("accessible.click", &[(NAME, "Reload")])).is_some());
        assert!(
            asked(&proposing(
                ACTIVATE_CONTROL,
                &[
                    (APPLICATION, "org.mozilla.firefox"),
                    (NAME, "Reload"),
                    ("colour", "red")
                ],
            ))
            .is_some()
        );
    }

    /// **The refusal case**: a call that forms is not turned away, whatever
    /// the road expected.
    #[test]
    fn a_call_that_forms_and_an_answer_are_not_turned_away() {
        let press = proposing(
            ACTIVATE_CONTROL,
            &[
                (APPLICATION, "org.mozilla.firefox"),
                (KIND, "button"),
                (NAME, "Reload"),
            ],
        );
        assert_eq!(asked(&press), None);
    }
}
