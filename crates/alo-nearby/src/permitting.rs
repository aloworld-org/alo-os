//! What one machine may ask another for, which is a list a person can read to
//! the end.
//!
//! # Why it is an enum and not a set of strings
//!
//! ADR 0003: a pairing behaves like a grant — **enumerated**, visible,
//! revocable in one action, and expiring. *Enumerated* is the word doing the
//! work. A pairing that permitted *everything* would be a grant nobody could
//! read, and a pairing that permitted a list of strings would be a grant whose
//! contents nothing in this workspace could check.
//!
//! So what a pairing may permit is a closed enum, each arm of which has a
//! sentence in [`crate::words`]. Adding an arm means adding a sentence, in a
//! file a translator reads, which is the cost that keeps the list short.
//!
//! # Two arms, and what is deliberately not a third
//!
//! There is no arm for *run a verb on that machine*, and there never will be
//! one here. ADR 0003 is exact about it: **pairing lets A ask; it never lets A
//! act.** A verb arriving from a paired machine is evaluated against the
//! receiving machine's own grants, by the receiving machine's own person, and
//! is not something this list could ever grant on that machine's behalf. What
//! the pairing contributes to it is standing alone — [`crate::Origin`] is made
//! by asking whether a pairing exists, [`crate::Pairings::paired_with`], and
//! asks about no arm — and the grant that machine's person made on their own
//! machine is what decides.

use crate::words;

/// One thing a paired machine may ask this one for.
///
/// Both arms are questions. Neither is an action, and this module's own
/// documentation says why there is no arm that is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum MayAskIts {
    /// Put a question to the models this machine has.
    ///
    /// The promise this exists for: *a machine without a GPU discovers the one
    /// with it, and the agents just work.* It is still egress on the asking
    /// machine and the indicator still fires — permission here is what makes
    /// the departure wanted, not what makes it silent.
    Models,

    /// Reach a workspace this machine serves.
    ///
    /// *A self-hosted workspace on the network is discovered, not configured —
    /// no DNS step.* Discovery finds it; this is what says a person meant to
    /// use it.
    Workspace,
}

/// Every arm, for the surface that lists them and the tests that check each one
/// can be said.
///
/// Written out rather than derived, because a list that generated itself would
/// silently grow when somebody added an arm — and growing the list of what a
/// pairing may permit is exactly the change that should not be able to happen
/// quietly.
pub const EVERYTHING_A_PAIRING_MAY_PERMIT: [MayAskIts; 2] =
    [MayAskIts::Models, MayAskIts::Workspace];

impl MayAskIts {
    /// The sentence a person reads for this, in their own language.
    #[must_use]
    pub const fn word(self) -> alo_strings::Word {
        match self {
            Self::Models => words::MAY_ASK_ITS_MODELS,
            Self::Workspace => words::MAY_REACH_ITS_WORKSPACE,
        }
    }

    /// One arm as it is written on the wire: one lowercase word, so that a
    /// proposal carrying it spells it the same way on both machines.
    #[must_use]
    pub const fn said(self) -> &'static str {
        match self {
            Self::Models => "models",
            Self::Workspace => "workspace",
        }
    }

    /// An arm somebody else wrote on the wire, or nothing for a word that is
    /// not one — which is how a proposal asking for something this list does
    /// not have is refused rather than widened.
    #[must_use]
    pub fn read(said: &str) -> Option<Self> {
        EVERYTHING_A_PAIRING_MAY_PERMIT
            .into_iter()
            .find(|arm| arm.said() == said)
    }
}

#[cfg(test)]
mod tests {
    use super::{EVERYTHING_A_PAIRING_MAY_PERMIT, MayAskIts};

    /// **Every arm has a sentence**, which is what makes the list one a person
    /// can read rather than one a program can.
    #[test]
    fn everything_a_pairing_may_permit_can_be_said_to_a_person() {
        for may in EVERYTHING_A_PAIRING_MAY_PERMIT {
            assert!(!may.word().says().is_empty(), "{may:?} has no sentence");
        }
    }

    /// The list is the arms, and this is the test that fails when somebody adds
    /// one without adding it here — which is the whole point of writing it out.
    #[test]
    fn the_list_holds_every_arm_exactly_once() {
        let mut sorted = EVERYTHING_A_PAIRING_MAY_PERMIT.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), EVERYTHING_A_PAIRING_MAY_PERMIT.len());
        assert!(sorted.contains(&MayAskIts::Models));
        assert!(sorted.contains(&MayAskIts::Workspace));
    }

    /// Every arm is written on the wire as one lowercase word and read back
    /// as itself, and a word that is not an arm is read as nothing.
    #[test]
    fn every_arm_is_written_on_the_wire_and_read_back_and_nothing_else_is() {
        for may in EVERYTHING_A_PAIRING_MAY_PERMIT {
            assert_eq!(MayAskIts::read(may.said()), Some(may));
            assert!(may.said().chars().all(|c| c.is_ascii_lowercase()));
        }
        for not_one in ["", "Models", "everything", "verbs", "models "] {
            assert_eq!(MayAskIts::read(not_one), None, "`{not_one}` was read");
        }
    }

    /// Two arms is a list. One would be a flag wearing a list's clothes, and
    /// this test is here so that a later change back to one is a decision
    /// somebody makes rather than one that happens.
    #[test]
    fn what_a_pairing_may_permit_is_a_list_rather_than_a_switch() {
        assert!(EVERYTHING_A_PAIRING_MAY_PERMIT.len() > 1);
    }
}
