//! Checks for a private installed-agent walk; none of these creates a grant.
//!
//! These validate observations only. Installed acceptance additionally requires
//! the actual confined agent connection, native Settings choices and approvals,
//! real application state, execution records and repeated model requests.

use std::collections::BTreeSet;

use alo_adapters::studio::STUDIO;
use alo_protocol::ToAnAgent;

pub use crate::studio_agent_identity::require_confinement;
pub use crate::studio_agent_sequence::run;

/// The five boundaries observed while the person changes native Settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Asking each time, before any application grant.
    AskBeforeGrant,
    /// Asking each time, with the person's explicit Studio grant.
    Granted,
    /// Asking each time, immediately after the person takes that grant back.
    Revoked,
    /// Full access, with no remaining explicit application grant.
    Full,
    /// Asking each time again, still without an explicit application grant.
    AskAgain,
}

/// Require precisely the Studio declarations permitted at this boundary.
///
/// Unrelated application families do not affect this check. Order is not
/// significant, but duplicate, missing and unknown Studio actions are refused.
///
/// # Errors
/// A non-discovery reply or a Studio action set different from this stage's.
pub fn require_discovery(stage: Stage, reply: &ToAnAgent) -> Result<(), &'static str> {
    let ToAnAgent::Capabilities { verbs } = reply else {
        return Err("the agent did not receive capability discovery");
    };
    let mut observed = BTreeSet::new();
    for name in verbs.iter().filter(|name| name.starts_with("studio.")) {
        if !observed.insert(name.clone()) {
            return Err("capability discovery repeats a Studio action");
        }
    }
    let expected: BTreeSet<String> = match stage {
        Stage::Granted | Stage::Full => STUDIO
            .verbs
            .iter()
            .map(|verb| format!("studio.{}", verb.name))
            .collect(),
        Stage::AskBeforeGrant | Stage::Revoked | Stage::AskAgain => BTreeSet::new(),
    };
    if observed != expected {
        return Err("Studio discovery differs from the chosen permission boundary");
    }
    Ok(())
}

/// Require an actual protocol refusal to a fixed, valid withdrawn operation.
///
/// # Errors
/// Success, pending approval, an unanswered operation, model prose and empty
/// discovery are all different from an execution refusal.
pub fn require_refused(reply: &ToAnAgent) -> Result<(), &'static str> {
    if matches!(reply, ToAnAgent::Refused(_)) {
        Ok(())
    } else {
        Err("the withdrawn operation was not explicitly refused")
    }
}
