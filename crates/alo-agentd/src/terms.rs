//! What the service is told about this machine and does not decide.
//!
//! Everything `crate::Serving` is handed rather than decides, in one value:
//! the agent's name as the grants know it, how long a turn lasts and a change
//! waits, how long the record is kept, which egress rule is in force, and what
//! the person here has called the machines they paired with. Every one of
//! them is read off the machine's description or is the shell's, and none is
//! this crate's opinion.
//!
//! **One rule, stated once.** The egress rule an answer to a paired machine
//! leaves under is the same rule that says where a question may be answered:
//! `alo_egress::EgressPolicy` is made from the description's
//! `alo_models::SourcePolicy`, and a machine whose two rules could disagree
//! would be a machine nobody could describe.

use std::time::Duration;

use alo_corridor::Naming;
use alo_egress::EgressPolicy;
use alo_keeping::Keeping;
use alo_nearby::MachineId;

/// What a machine told the service about itself.
pub struct Terms<'a> {
    /// The agent this machine has, as the grants name it.
    pub for_agent: &'a str,
    /// How long a turn's own grant lasts — a local turn's and a remote
    /// one's alike, because both are turns on this machine.
    pub lasting: Duration,
    /// How long a change waits for an answer.
    pub standing: Duration,
    /// How long what happened on this machine is kept.
    pub keeping: Keeping,
    /// What may leave this machine, made from the one rule the description
    /// states.
    pub policy: EgressPolicy,
    /// What the person here called the machines they paired with.
    ///
    /// Asked at the moment and deciding nothing. A running machine answers
    /// with `crate::network::TheNetwork::names` — the names read back at start
    /// and given on the person's door since — and a test that is not about
    /// names hands in [`NoNameYet`].
    pub naming: &'a dyn Naming,
}

impl std::fmt::Debug for Terms<'_> {
    /// Written by hand because the naming is a trait object.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Terms")
            .field("for_agent", &self.for_agent)
            .field("lasting", &self.lasting)
            .field("standing", &self.standing)
            .field("keeping", &self.keeping)
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

/// Nobody has given any paired machine a name.
///
/// Every machine is then spoken of by its identity, which `alo_nearby::Origin`
/// does when the name is empty. Nothing a machine runs is handed this since the
/// person's door learned to name a machine (`crate::naming_machines`): the
/// service answers with `crate::names::TheNames`. It stays for a test that is
/// not about names, written where a reader can see what it is.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoNameYet;

impl Naming for NoNameYet {
    fn called(&self, _machine: &MachineId) -> Option<String> {
        None
    }
}
