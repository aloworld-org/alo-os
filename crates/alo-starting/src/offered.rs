//! The identity of a system the menu offers, made the same way on both sides of
//! the broker's door.
//!
//! [ADR 0066](../../../docs/decisions/0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md)
//! §2: the verb that changes which system a machine starts by default takes
//! **the identity of a system the menu already offers** — not a path, not a
//! string, not a loader. There are two systems on a machine installed alongside
//! Windows and there is nothing else to name, so an identity here is a digest
//! over a word this crate chooses for each of them and never over anything a
//! surface was handed.
//!
//! # Why an identity at all, for a choice between two
//!
//! Because the broker's argument has two shapes and neither of them is text
//! (`alo_broker::Argument`). A switch would fit the arithmetic — there are two
//! systems — and would read as *on* and *off* at the door and in the record,
//! where what a person approved was *start Windows* or *start alo OS*. An
//! identity says which system in the same shape every other verb on the list
//! already uses, and the record keeps a value that means exactly one system.
//!
//! # And it is the menu's word, not the firmware's
//!
//! [`crate::Entry`]'s identity is over what a **firmware** reported, which
//! changes from machine to machine. This one is over what the **menu** offers,
//! which is the same on every machine alo OS installed alongside Windows —
//! deliberately, because what is being named is *which of the two systems*
//! rather than *which entry on this computer*. The check that this machine has
//! that system at all is [`crate::TheLoader::offering`], made where the change
//! is made.

use alo_broker::Identity;

use crate::systems::System;

/// The bytes a system's identity is made over, before the system itself.
///
/// A version in the name, so that a change to what is digested is a change to
/// this word rather than a silent disagreement between the side that proposes a
/// change and the side that carries it out.
pub const AS_OFFERED: &[u8] = b"alo-starting system 1";

impl System {
    /// The word the menu knows this system by, which is what its identity is
    /// made over.
    ///
    /// Windows's is the identifier the generated menu gives its own entry
    /// ([`crate::THE_WINDOWS_ENTRY`]); alo OS's is a word of this crate's,
    /// because alo OS's own entries are the base's and their identifiers change
    /// with every update ([`crate::TheStartingChoice`] says why).
    #[must_use]
    pub const fn offered_as(self) -> &'static str {
        match self {
            Self::AloOs => "alo-os",
            Self::Windows => crate::systems::THE_WINDOWS_ENTRY,
        }
    }

    /// The bytes this system's identity is made over: the word above, a zero
    /// byte, and the word the menu knows it by.
    #[must_use]
    pub fn as_offered(self) -> Vec<u8> {
        let mut over = AS_OFFERED.to_vec();
        over.push(0);
        over.extend_from_slice(self.offered_as().as_bytes());
        over
    }
}

/// The identity of this system, as both sides of the door make one.
///
/// One function, called by the surface that proposes the change and by whatever
/// carries it out, so the two cannot drift.
#[must_use]
pub fn the_identity_of(system: System) -> Identity {
    Identity::of_what_was_reported(&system.as_offered())
}

/// Which system this identity is, if it is one of the two.
///
/// Nothing, for an identity that is neither — which is a refusal where it is
/// read and never a guess.
#[must_use]
pub fn the_system_named(identity: Identity) -> Option<System> {
    System::BOTH
        .into_iter()
        .find(|system| the_identity_of(*system) == identity)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Each system reads back from its own identity, and no other does.**
    #[test]
    fn each_system_reads_back_from_its_own_identity() {
        for system in System::BOTH {
            assert_eq!(the_system_named(the_identity_of(system)), Some(system));
            assert_ne!(the_identity_of(system), the_identity_of(system.other()));
        }
    }

    /// **An identity that is not one of the two is neither**, rather than the
    /// nearer of them. The identity of a start-up entry is the case that
    /// matters: the two verbs on this road take arguments of the same shape,
    /// and one made for the other must not be read as a system.
    #[test]
    fn an_identity_that_is_not_a_systems_is_neither() {
        for other in [
            Identity::of_what_was_reported(b"a start-up entry"),
            Identity::of_what_was_reported(b""),
            Identity::of_what_was_reported(System::AloOs.offered_as().as_bytes()),
            Identity::of_what_was_reported(AS_OFFERED),
        ] {
            assert_eq!(the_system_named(other), None);
        }
    }

    /// **The two words the menu knows the systems by are the menu's own**, and
    /// neither is empty or shared.
    #[test]
    fn the_words_the_menu_knows_them_by_are_its_own() {
        assert_eq!(
            System::Windows.offered_as(),
            crate::systems::THE_WINDOWS_ENTRY
        );
        assert_ne!(System::AloOs.offered_as(), System::Windows.offered_as());
        for system in System::BOTH {
            assert!(!system.offered_as().is_empty());
            assert!(!system.offered_as().contains(char::is_whitespace));
        }
    }

    /// **What is digested carries the word that versions it**, so a change to
    /// the shape is a change to that word rather than a quiet disagreement.
    #[test]
    fn what_is_digested_carries_the_word_that_versions_it() {
        for system in System::BOTH {
            let over = system.as_offered();
            assert!(over.starts_with(AS_OFFERED));
            assert!(over.ends_with(system.offered_as().as_bytes()));
            assert!(over.contains(&0));
        }
    }
}
