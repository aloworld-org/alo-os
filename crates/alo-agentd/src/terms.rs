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
    /// Asked at the moment and deciding nothing; the shell keeps the names,
    /// and until one does, [`NoNameYet`] answers that nobody has named
    /// anything.
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
/// The person's name for a paired machine is the shell's to keep, and the
/// shell is outside the local-network plan; until it keeps one, every machine
/// is spoken of by its identity, which `alo_nearby::Origin` does when the
/// name is empty. This is that absence written where a reader can see it
/// rather than a closure in `main`.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoNameYet;

impl Naming for NoNameYet {
    fn called(&self, _machine: &MachineId) -> Option<String> {
        None
    }
}
