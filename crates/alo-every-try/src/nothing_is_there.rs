//! **A path where nothing is, answered as the machine answers it.**
//!
//! Before anybody is asked, at both levels, the machine refuses a change whose
//! path names nothing (`alo_turn`'s `the_sentence_is_about_something`, called
//! from `Turning::proposing_call` and from `a_change_at_full_access`), in
//! `alo_files::RealError::Nothing`'s words. The model reads that refusal and may
//! go on from it. So a try that names a path where nothing is gets the same
//! refusal here and goes on, rather than ending where a booted machine would
//! not have ended it.
//!
//! **Only where a request says what is there** (`ARequest::where_there_is`): a
//! request that says nothing is held to its road as it always was, because
//! this check cannot know what a machine holds that the request did not
//! declare. A folder that holds something declared is there too.

use std::path::{Path, PathBuf};

use alo_capability::{Value, Verbs};
use alo_protocol::{FromAnAgent, ToAnAgent};
use alo_strings::Strings;

use crate::as_the_machine_names::AsTheMachineNames;

/// The machine's refusal of `asked`, where it names a path that neither is in
/// `there` nor holds anything in it; [`None`] where `there` is empty, where the
/// call names no path, or where every path it names is there.
#[must_use]
pub fn nothing_is_at(
    there: &[PathBuf],
    asked: &FromAnAgent,
    verbs: &Verbs,
    machine: &AsTheMachineNames,
    strings: &Strings,
) -> Option<ToAnAgent> {
    if there.is_empty() {
        return None;
    }
    let (FromAnAgent::Read { verb, .. } | FromAnAgent::Propose { verb, .. }) = asked else {
        return None;
    };
    let verb = verb.trim();
    let given = machine.naming(verbs, verb, &asked.given());
    let call = verbs.call(verb, &given).ok()?;
    let missing = call.values().values().find_map(|value| match value {
        Value::Path(path) if !is_there(there, path) => Some(path.display().to_string()),
        _ => None,
    })?;
    let said = alo_files::RealError::Nothing { path: missing }.said(strings);
    Some(ToAnAgent::refused(&said))
}

/// Whether `path` is one of `there` or a folder that holds one.
fn is_there(there: &[PathBuf], path: &Path) -> bool {
    there.iter().any(|at| at.starts_with(path))
}

#[cfg(test)]
#[path = "nothing_is_there_tests.rs"]
mod tests;
