//! Which of the two things a person wants done with a drive, before anybody
//! has asked which drive that is.

use alo_broker::{Identity, SystemVerb};

/// What a person wants done with a drive they plugged in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// Open what is on it, for the signed-in person.
    Open,
    /// Finish with it, so it can be unplugged without losing anything.
    FinishWith,
}

impl Change {
    /// The broker's verb for this, about the thing with this identity.
    ///
    /// The identity is the filesystem's for [`Change::Open`] and the drive's
    /// for [`Change::FinishWith`], which is what `alo_drives::Drive` digests
    /// each of them from. They are deliberately not the same identity: the same
    /// card in another reader is not the filesystem a person approved.
    #[must_use]
    pub const fn to(self, identity: Identity) -> SystemVerb {
        match self {
            Self::Open => SystemVerb::MountDrive(identity),
            Self::FinishWith => SystemVerb::EjectDrive(identity),
        }
    }

    /// Which of the two the broker's verb is, if it is one of the drives'.
    #[must_use]
    pub const fn of(verb: &SystemVerb) -> Option<Self> {
        match verb {
            SystemVerb::MountDrive(_) => Some(Self::Open),
            SystemVerb::EjectDrive(_) => Some(Self::FinishWith),
            SystemVerb::AddPrinter(_)
            | SystemVerb::RemovePrinter(_)
            | SystemVerb::SetDefaultPrinter(_)
            | SystemVerb::JoinNetwork(_)
            | SystemVerb::ForgetNetwork(_)
            | SystemVerb::SetRadio(_)
            | SystemVerb::SetProxy(_)
            | SystemVerb::ApplyStagedUpdate(_)
            | SystemVerb::RollBack(_)
            | SystemVerb::RestartIntoWindows(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alo_broker::Switch;

    /// Something reported, for a verb to hold.
    fn an_identity() -> Identity {
        Identity::of_what_was_reported(b"a stick")
    }

    /// **Both read back from their verb, and no other verb is one of these.**
    #[test]
    fn both_read_back_and_no_other_verb_is_a_drives() {
        for change in [Change::Open, Change::FinishWith] {
            assert_eq!(Change::of(&change.to(an_identity())), Some(change));
        }
        for verb in SystemVerb::one_of_each(an_identity(), Switch::On) {
            let is_a_drives = matches!(verb, SystemVerb::MountDrive(_) | SystemVerb::EjectDrive(_));
            assert_eq!(Change::of(&verb).is_some(), is_a_drives, "{}", verb.name());
        }
    }

    /// **Opening and finishing with are not the same verb**, so an approval of
    /// one is never redeemed as the other.
    #[test]
    fn opening_and_finishing_with_are_different_verbs() {
        assert_ne!(
            Change::Open.to(an_identity()),
            Change::FinishWith.to(an_identity())
        );
    }
}
