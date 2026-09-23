//! A person's own choice in Settings about which system this machine starts —
//! through the same verb an agent's approved proposal would become.
//!
//! [ADR 0066](../../../docs/decisions/0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md)
//! §2: Settings does not write the loader's file, because a person is not root.
//! It asks the broker, exactly as it does for printers, the network and
//! updates. So a person picks one of the two systems and that becomes **the
//! broker's own verb** — the same verb, crossing the same door, written into
//! the same record. There is no second road into which system this machine
//! starts, which is what makes [`crate::starts_at_said`] answerable from one
//! place.
//!
//! The person's click is the approval. There is no turn behind it and no
//! proposal to number, so whatever issues its token issues it under
//! `alo_broker::BY_HAND`, and the broker's record says a person made this
//! change themselves.

use alo_broker::SystemVerb;

use crate::offered::the_identity_of;
use crate::systems::System;
use crate::wanted::Change;

/// The broker's verb for *this computer starts this system when nobody
/// chooses*.
///
/// Whether this machine offers that system at all is decided where the change
/// is made ([`crate::start_by_default`]) and never here: a surface that could
/// decide it would be a second answer to *is there a Windows on this computer*,
/// and the machine's own is the one that counts.
#[must_use]
pub fn to_start_by_default(system: System) -> SystemVerb {
    Change::StartByDefault.to(the_identity_of(system))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Each system is its own verb, and the verb is the one on the broker's
    /// list.**
    #[test]
    fn each_system_is_its_own_verb_on_the_brokers_list() {
        let alo = to_start_by_default(System::AloOs);
        let windows = to_start_by_default(System::Windows);
        assert_ne!(alo, windows);
        for verb in [alo, windows] {
            assert_eq!(verb.name(), "starting.default");
            assert_eq!(Change::of(&verb), Some(Change::StartByDefault));
            assert!(alo_broker::EVERY_NAME.contains(&verb.name()));
        }
    }

    /// **What a person picked is what the verb carries**, read back through the
    /// identity both sides of the door make the same way.
    #[test]
    fn what_a_person_picked_is_what_the_verb_carries() {
        for system in System::BOTH {
            assert_eq!(
                to_start_by_default(system),
                SystemVerb::StartByDefault(the_identity_of(system))
            );
        }
    }
}
