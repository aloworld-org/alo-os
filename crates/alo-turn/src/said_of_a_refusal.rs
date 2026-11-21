//! **What a turn says on the journal when a call is refused**: the verb, and
//! why in kind — never a path, a sentence or anything typed.
//!
//! Main's walk at 802ab80a (run 44884016105) saw opening a file in VSCodium and
//! in Ark refused before it ran, with the right identifiers (the agent's own
//! line, `alo_agent::the_refused_call_named`) and nothing anywhere saying why:
//! a refusal crosses to the agent as a sentence only, and the daemon journals
//! only what it carries out (`crate::said_in_an_application`). This says why,
//! where the call is stopped, by the refusal's **kind**:
//!
//! - an application asked for, by its identifier, as the turn names one;
//! - a path asked for, as *a path* and never by its name;
//! - a refusal worded where the path was resolved, by that kind alone, because
//!   its sentence may name the path.
//!
//! Written as the daemon's own stderr, which is this machine's journal, and
//! nowhere else.

use alo_capability::{Ask, NotAuthorised, NotGranted, Refused};

/// The line saying `refused` was refused, and why, in kind.
pub(crate) fn said(refused: &Refused) -> String {
    let verb = refused.call().verb().escape_debug().to_string();
    let why = match refused.why() {
        NotAuthorised::NotGranted(NotGranted::Never { wanted, .. }) => {
            format!("nothing granted it, for {}", asked(wanted))
        }
        NotAuthorised::NotGranted(NotGranted::Lapsed { wanted, .. }) => {
            format!("the grant that covered it lapsed, for {}", asked(wanted))
        }
        NotAuthorised::NotGranted(NotGranted::NoAgent { .. }) => {
            "there is no agent on this machine".to_owned()
        }
        NotAuthorised::NotGrantedElsewhere(_) => {
            "it was decided where the path was resolved".to_owned()
        }
        NotAuthorised::ChangeWaits { .. } => {
            "a change was offered where a read was expected".to_owned()
        }
    };
    format!(
        "{} {verb}: {why}",
        alo_acting::said_on_the_log::A_CALL_WAS_REFUSED
    )
}

/// What was asked for: an application by its identifier, quoted; a path as
/// *a path*, never by its name; a facility by its kind.
fn asked(wanted: &Ask) -> String {
    match wanted {
        Ask::Application(identifier) => format!("{identifier:?}"),
        Ask::Path(_) => "a path".to_owned(),
        Ask::Facility(facility) => format!("the facility {facility:?}"),
    }
}

#[cfg(test)]
#[path = "said_of_a_refusal_tests.rs"]
mod tests;
