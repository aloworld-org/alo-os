//! **How the first declarations were fingerprinted**, until 2026-11-20: a verb
//! by what it is told and its whole declaration, a request by the raw text of
//! its definition (`{:?}`).
//!
//! Kept so that a base declared that way can still be compared with: a change
//! is read against its base **in the base's own scheme**
//! (`crate::its_own::its_own_against`), so the change that moved the scheme
//! owns only what it changed, and not every line the new scheme writes
//! differently. Nothing writes this scheme any more.

use alo_capability::Verb;
use alo_instructing::verb_as_told::as_told;
use serde_json::Value;

use crate::a_road::ARequest;
use crate::the_declaration::{fingerprint, the_branch_for};

/// One verb's line, as the first scheme wrote it.
#[must_use]
pub fn a_verbs_line(verb: &Verb, envelope: &Value) -> String {
    let branch = the_branch_for(verb.name(), envelope)
        .map(Value::to_string)
        .unwrap_or_default();
    fingerprint(&format!(
        "{}\n--- declared\n{verb:?}\n--- held to\n{branch}",
        as_told(verb)
    ))
}

/// One request's line, as the first scheme wrote it.
#[must_use]
pub fn a_requests_line(request: &ARequest) -> String {
    fingerprint(&format!("{request:?}"))
}
