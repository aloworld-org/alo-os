//! **A control named where nothing is named, answered as the machine answers
//! it**: refused before any card, in the machine's own words, by the machine's
//! own decision (`alo_adapters::where_nothing_is_named`) — and the try goes on.

use alo_capability::Verbs;
use alo_protocol::{FromAnAgent, ToAnAgent};
use alo_strings::Strings;

use crate::as_the_machine_names::AsTheMachineNames;

/// The machine's refusal of `asked`, where it names a control in an
/// application whose windows a read in this try found to name nothing.
#[must_use]
pub fn refused(
    asked: &FromAnAgent,
    verbs: &Verbs,
    machine: &AsTheMachineNames,
    where_nothing_is_named: &[String],
    strings: &Strings,
) -> Option<ToAnAgent> {
    let FromAnAgent::Propose { verb, .. } = asked else {
        return None;
    };
    let verb = verb.trim();
    let given = machine.naming(verbs, verb, &asked.given());
    let call = verbs.call(verb, &given).ok()?;
    alo_adapters::where_nothing_is_named::named_where_nothing_is_named(
        &call,
        where_nothing_is_named,
        strings,
    )
    .map(|said| ToAnAgent::refused(&said))
}

/// **The applications whose windows the request's machine shows naming
/// nothing**, as its scripted reads answer — what a named guess is checked
/// against when it arrives, with no read first, as the machine asks its tree.
#[must_use]
pub fn as_the_request_shows(request: &crate::a_road::ARequest) -> Vec<String> {
    let _ = request;
    Vec::new()
}
